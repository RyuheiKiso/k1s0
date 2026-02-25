import 'query.dart';
import 'document.dart';
import 'error.dart';

abstract class SearchClient {
  Future<IndexResult> indexDocument(String index, IndexDocument doc);
  Future<BulkResult> bulkIndex(String index, List<IndexDocument> docs);
  Future<SearchResult<Map<String, dynamic>>> search(String index, SearchQuery query);
  Future<void> deleteDocument(String index, String id);
  Future<void> createIndex(String name, IndexMapping mapping);
}

class InMemorySearchClient implements SearchClient {
  final Map<String, List<IndexDocument>> _indexes = {};

  @override
  Future<void> createIndex(String name, IndexMapping mapping) async {
    _indexes[name] = [];
  }

  @override
  Future<IndexResult> indexDocument(String index, IndexDocument doc) async {
    final docs = _indexes[index];
    if (docs == null) {
      throw SearchError('Index not found: $index', SearchErrorCode.indexNotFound);
    }
    docs.add(doc);
    return IndexResult(id: doc.id, version: docs.length);
  }

  @override
  Future<BulkResult> bulkIndex(String index, List<IndexDocument> docs) async {
    final existing = _indexes[index];
    if (existing == null) {
      throw SearchError('Index not found: $index', SearchErrorCode.indexNotFound);
    }
    existing.addAll(docs);
    return BulkResult(successCount: docs.length, failedCount: 0, failures: []);
  }

  @override
  Future<SearchResult<Map<String, dynamic>>> search(String index, SearchQuery query) async {
    final docs = _indexes[index];
    if (docs == null) {
      throw SearchError('Index not found: $index', SearchErrorCode.indexNotFound);
    }

    var filtered = docs.where((doc) {
      if (query.query.isEmpty) return true;
      return doc.fields.values.any(
        (v) => v is String && v.contains(query.query),
      );
    }).toList();

    final start = query.page * query.size;
    final paged = filtered.skip(start).take(query.size).toList();

    final hits = paged
        .map((doc) => <String, dynamic>{'id': doc.id, ...doc.fields})
        .toList();

    final facets = <String, List<FacetBucket>>{};
    for (final f in query.facets) {
      facets[f] = [FacetBucket(value: 'default', count: hits.length)];
    }

    return SearchResult(
      hits: hits,
      total: filtered.length,
      facets: facets,
      tookMs: 1,
    );
  }

  @override
  Future<void> deleteDocument(String index, String id) async {
    _indexes[index]?.removeWhere((doc) => doc.id == id);
  }

  int documentCount(String index) => _indexes[index]?.length ?? 0;
}
