import 'dart:convert';

import 'package:http/http.dart' as http;

import 'errors.dart';
import 'models/app_version.dart';

class RegistryApiClient {
  final String baseUrl;
  final http.Client _client;
  final String? _authToken;

  RegistryApiClient({
    required this.baseUrl,
    http.Client? client,
    String? authToken,
  })  : _client = client ?? http.Client(),
        _authToken = authToken;

  Map<String, String> get _headers => {
        'Content-Type': 'application/json',
        if (_authToken != null) 'Authorization': 'Bearer $_authToken',
      };

  Future<AppVersion?> getLatestVersion(
    String appId,
    String platform,
    String arch,
  ) async {
    final uri = Uri.parse(
      '$baseUrl/apps/$appId/versions/latest'
      '?platform=$platform&arch=$arch',
    );

    final response = await _request('GET', uri);
    if (response.statusCode == 404) return null;
    _ensureSuccess(response);

    return AppVersion.fromJson(
      jsonDecode(response.body) as Map<String, dynamic>,
    );
  }

  Future<List<AppVersion>> listVersions(String appId) async {
    final uri = Uri.parse('$baseUrl/apps/$appId/versions');

    final response = await _request('GET', uri);
    _ensureSuccess(response);

    final list = jsonDecode(response.body) as List<dynamic>;
    return list
        .map((e) => AppVersion.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<String> getDownloadUrl(
    String appId,
    String version,
    String platform,
    String arch,
  ) async {
    final uri = Uri.parse(
      '$baseUrl/apps/$appId/versions/$version/download'
      '?platform=$platform&arch=$arch',
    );

    final response = await _request('GET', uri);
    _ensureSuccess(response);

    final body = jsonDecode(response.body) as Map<String, dynamic>;
    return body['download_url'] as String;
  }

  Future<http.Response> _request(String method, Uri uri) async {
    try {
      final request = http.Request(method, uri)..headers.addAll(_headers);
      final streamed = await _client.send(request);
      return http.Response.fromStream(streamed);
    } on Exception catch (e) {
      throw NetworkError('Request failed: $e');
    }
  }

  void _ensureSuccess(http.Response response) {
    if (response.statusCode >= 200 && response.statusCode < 300) return;
    throw NetworkError(
      'HTTP ${response.statusCode}: ${response.body}',
    );
  }
}
