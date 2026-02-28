import 'dart:convert';
import 'dart:io';

import 'client.dart';
import 'document.dart';
import 'error.dart';
import 'query.dart';

/// GrpcSearchClient は search-server への HTTP クライアント実装。
/// HTTP/JSON API 経由で search-server と通信する。
class GrpcSearchClient implements SearchClient {
  final String _baseUrl;
  final HttpClient _httpClient;

  GrpcSearchClient(String serverUrl, {HttpClient? httpClient})
      : _baseUrl = _normalizeUrl(serverUrl),
        _httpClient = httpClient ?? HttpClient();

  static String _normalizeUrl(String url) {
    final normalized = url.startsWith('http://') || url.startsWith('https://')
        ? url
        : 'http://$url';
    return normalized.endsWith('/') ? normalized.substring(0, normalized.length - 1) : normalized;
  }

  /// gRPC チャネル相当のリソースを解放する（HTTP 実装では close のみ）。
  void close() {
    _httpClient.close();
  }

  @override
  Future<void> createIndex(String name, IndexMapping mapping) async {
    final uri = Uri.parse('$_baseUrl/api/v1/indexes/${Uri.encodeComponent(name)}');
    final body = jsonEncode({
      'name': name,
      'mapping': _mappingToJson(mapping),
    });

    final response = await _doRequest('PUT', uri, body: body);
    if (response.statusCode != 200 &&
        response.statusCode != 201 &&
        response.statusCode != 204) {
      final respBody = await _readBody(response);
      throw _parseError('create_index', response.statusCode, respBody);
    }
    await _drainBody(response);
  }

  @override
  Future<IndexResult> indexDocument(String index, IndexDocument doc) async {
    final uri = Uri.parse(
        '$_baseUrl/api/v1/indexes/${Uri.encodeComponent(index)}/documents');
    final body = jsonEncode({'id': doc.id, 'fields': doc.fields});

    final response = await _doRequest('POST', uri, body: body);
    final respBody = await _readBody(response);

    if (response.statusCode != 200 && response.statusCode != 201) {
      throw _parseError('index_document', response.statusCode, respBody);
    }

    final data = jsonDecode(respBody) as Map<String, dynamic>;
    return IndexResult(
      id: data['id'] as String,
      version: data['version'] as int,
    );
  }

  @override
  Future<BulkResult> bulkIndex(String index, List<IndexDocument> docs) async {
    final uri = Uri.parse(
        '$_baseUrl/api/v1/indexes/${Uri.encodeComponent(index)}/documents/_bulk');
    final body = jsonEncode({
      'documents': docs.map((d) => {'id': d.id, 'fields': d.fields}).toList(),
    });

    final response = await _doRequest('POST', uri, body: body);
    final respBody = await _readBody(response);

    if (response.statusCode != 200 && response.statusCode != 201) {
      throw _parseError('bulk_index', response.statusCode, respBody);
    }

    final data = jsonDecode(respBody) as Map<String, dynamic>;
    final rawFailures = (data['failures'] as List<dynamic>?) ?? [];
    final failures = rawFailures
        .cast<Map<String, dynamic>>()
        .map((f) => BulkFailure(
              id: f['id'] as String,
              error: f['error'] as String,
            ))
        .toList();

    return BulkResult(
      successCount: data['successCount'] as int? ?? 0,
      failedCount: data['failedCount'] as int? ?? 0,
      failures: failures,
    );
  }

  @override
  Future<SearchResult<Map<String, dynamic>>> search(
      String index, SearchQuery query) async {
    final uri = Uri.parse(
        '$_baseUrl/api/v1/indexes/${Uri.encodeComponent(index)}/_search');
    final body = jsonEncode(_queryToJson(query));

    final response = await _doRequest('POST', uri, body: body);
    final respBody = await _readBody(response);

    if (response.statusCode != 200) {
      throw _parseError('search', response.statusCode, respBody);
    }

    final data = jsonDecode(respBody) as Map<String, dynamic>;

    final hits = (data['hits'] as List<dynamic>? ?? [])
        .cast<Map<String, dynamic>>()
        .toList();

    final rawFacets = (data['facets'] as Map<String, dynamic>?) ?? {};
    final facets = rawFacets.map((k, v) {
      final buckets = (v as List<dynamic>)
          .cast<Map<String, dynamic>>()
          .map((b) => FacetBucket(
                value: b['value'] as String,
                count: b['count'] as int,
              ))
          .toList();
      return MapEntry(k, buckets);
    });

    return SearchResult(
      hits: hits,
      total: data['total'] as int? ?? 0,
      facets: facets,
      tookMs: data['tookMs'] as int? ?? 0,
    );
  }

  @override
  Future<void> deleteDocument(String index, String id) async {
    final uri = Uri.parse(
        '$_baseUrl/api/v1/indexes/${Uri.encodeComponent(index)}/documents/${Uri.encodeComponent(id)}');

    final response = await _doRequest('DELETE', uri);

    if (response.statusCode != 200 && response.statusCode != 204) {
      final respBody = await _readBody(response);
      throw _parseError('delete_document', response.statusCode, respBody);
    }
    await _drainBody(response);
  }

  // -- 内部ヘルパー --

  Future<HttpClientResponse> _doRequest(String method, Uri uri,
      {String? body}) async {
    final request = await _httpClient.openUrl(method, uri);
    request.headers.set(HttpHeaders.contentTypeHeader, 'application/json');
    if (body != null) {
      request.write(body);
    }
    return request.close();
  }

  Future<String> _readBody(HttpClientResponse response) async {
    return response.transform(utf8.decoder).join();
  }

  Future<void> _drainBody(HttpClientResponse response) async {
    await response.drain<void>();
  }

  SearchError _parseError(String op, int statusCode, String body) {
    switch (statusCode) {
      case 404:
        return SearchError(
            '$op: index not found: $body', SearchErrorCode.indexNotFound);
      case 400:
        return SearchError(
            '$op: invalid query: $body', SearchErrorCode.invalidQuery);
      case 408:
      case 504:
        return SearchError('$op: timeout', SearchErrorCode.timeout);
      default:
        return SearchError(
            '$op: server error (status=$statusCode): $body',
            SearchErrorCode.serverError);
    }
  }

  Map<String, dynamic> _mappingToJson(IndexMapping mapping) {
    return {
      'fields': mapping.fields.map(
        (k, v) => MapEntry(k, {'fieldType': v.fieldType, 'indexed': v.indexed}),
      ),
    };
  }

  Map<String, dynamic> _queryToJson(SearchQuery query) {
    return {
      'query': query.query,
      'filters': query.filters
          .map((f) => {
                'field': f.field,
                'operator': f.operator,
                'value': f.value,
                if (f.valueTo != null) 'valueTo': f.valueTo,
              })
          .toList(),
      'facets': query.facets,
      'page': query.page,
      'size': query.size,
    };
  }
}
