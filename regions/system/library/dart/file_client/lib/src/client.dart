import 'dart:convert';
import 'package:http/http.dart' as http;

import 'model.dart';

abstract class FileClient {
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  );
  Future<PresignedUrl> generateDownloadUrl(String path, Duration expiresIn);
  Future<void> delete(String path);
  Future<FileMetadata> getMetadata(String path);
  Future<List<FileMetadata>> list(String prefix);
  Future<void> copy(String src, String dst);
}

class InMemoryFileClient implements FileClient {
  final Map<String, FileMetadata> _files = {};

  List<FileMetadata> get storedFiles => List.unmodifiable(_files.values);

  @override
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  ) async {
    _files[path] = FileMetadata(
      path: path,
      sizeBytes: 0,
      contentType: contentType,
      etag: '',
      lastModified: DateTime.now(),
      tags: {},
    );
    return PresignedUrl(
      url: 'https://storage.example.com/upload/$path',
      method: 'PUT',
      expiresAt: DateTime.now().add(expiresIn),
      headers: {},
    );
  }

  @override
  Future<PresignedUrl> generateDownloadUrl(
    String path,
    Duration expiresIn,
  ) async {
    if (!_files.containsKey(path)) {
      throw FileClientError('File not found: $path', 'NOT_FOUND');
    }
    return PresignedUrl(
      url: 'https://storage.example.com/download/$path',
      method: 'GET',
      expiresAt: DateTime.now().add(expiresIn),
      headers: {},
    );
  }

  @override
  Future<void> delete(String path) async {
    if (!_files.containsKey(path)) {
      throw FileClientError('File not found: $path', 'NOT_FOUND');
    }
    _files.remove(path);
  }

  @override
  Future<FileMetadata> getMetadata(String path) async {
    final meta = _files[path];
    if (meta == null) {
      throw FileClientError('File not found: $path', 'NOT_FOUND');
    }
    return meta;
  }

  @override
  Future<List<FileMetadata>> list(String prefix) async {
    return _files.values.where((f) => f.path.startsWith(prefix)).toList();
  }

  @override
  Future<void> copy(String src, String dst) async {
    final source = _files[src];
    if (source == null) {
      throw FileClientError('File not found: $src', 'NOT_FOUND');
    }
    _files[dst] = FileMetadata(
      path: dst,
      sizeBytes: source.sizeBytes,
      contentType: source.contentType,
      etag: source.etag,
      lastModified: source.lastModified,
      tags: Map.of(source.tags),
    );
  }
}

// ---------------------------------------------------------------------------
// FileClientConfig — バックエンド設定
// ---------------------------------------------------------------------------

/// ファイルクライアントの設定。
class FileClientConfig {
  /// file-server モードのエンドポイント URL。
  final String? serverUrl;

  /// S3 互換ストレージの直接エンドポイント。
  final String? s3Endpoint;

  final String? bucket;
  final String? region;
  final String? accessKeyId;
  final String? secretAccessKey;

  /// リクエストタイムアウト。デフォルト 30 秒。
  final Duration timeout;

  const FileClientConfig({
    this.serverUrl,
    this.s3Endpoint,
    this.bucket,
    this.region,
    this.accessKeyId,
    this.secretAccessKey,
    this.timeout = const Duration(seconds: 30),
  });
}

// ---------------------------------------------------------------------------
// ServerFileClient — file-server 経由の HTTP 実装
// ---------------------------------------------------------------------------

/// file-server に HTTP で委譲する [FileClient] 実装。
class ServerFileClient implements FileClient {
  final String _baseUrl;
  final Duration _timeout;
  final http.Client _http;

  ServerFileClient(
    FileClientConfig config, {
    http.Client? httpClient,
  })  : _baseUrl = _normalizeUrl(config.serverUrl),
        _timeout = config.timeout,
        _http = httpClient ?? http.Client();

  static String _normalizeUrl(String? url) {
    if (url == null || url.isEmpty) {
      throw FileClientError('serverUrl が設定されていません', 'INVALID_CONFIG');
    }
    return url.endsWith('/') ? url.substring(0, url.length - 1) : url;
  }

  Future<http.Response> _doRequest(
    String method,
    String path, {
    Map<String, dynamic>? body,
  }) async {
    final uri = Uri.parse('$_baseUrl$path');
    final headers = body != null
        ? {'Content-Type': 'application/json'}
        : <String, String>{};
    final encodedBody = body != null ? jsonEncode(body) : null;

    final Future<http.Response> req;
    switch (method) {
      case 'POST':
        req = _http.post(uri, headers: headers, body: encodedBody);
        break;
      case 'DELETE':
        req = _http.delete(uri, headers: headers);
        break;
      case 'GET':
        req = _http.get(uri, headers: headers);
        break;
      default:
        throw FileClientError('Unknown HTTP method: $method', 'INTERNAL');
    }

    final resp = await req.timeout(_timeout);

    if (resp.statusCode == 404) {
      throw FileClientError(resp.body.isNotEmpty ? resp.body : path, 'NOT_FOUND');
    }
    if (resp.statusCode == 401 || resp.statusCode == 403) {
      throw FileClientError(resp.body, 'UNAUTHORIZED');
    }
    if (resp.statusCode >= 300) {
      throw FileClientError('HTTP ${resp.statusCode}: ${resp.body}', 'INTERNAL');
    }
    return resp;
  }

  @override
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  ) async {
    final resp = await _doRequest('POST', '/api/v1/files/upload-url', body: {
      'path': path,
      'content_type': contentType,
      'expires_in_secs': expiresIn.inSeconds,
    });
    final data = jsonDecode(resp.body) as Map<String, dynamic>;
    return PresignedUrl(
      url: data['url'] as String,
      method: data['method'] as String,
      expiresAt: DateTime.parse(data['expires_at'] as String),
      headers: (data['headers'] as Map<String, dynamic>? ?? {})
          .map((k, v) => MapEntry(k, v.toString())),
    );
  }

  @override
  Future<PresignedUrl> generateDownloadUrl(
    String path,
    Duration expiresIn,
  ) async {
    final resp = await _doRequest('POST', '/api/v1/files/download-url', body: {
      'path': path,
      'expires_in_secs': expiresIn.inSeconds,
    });
    final data = jsonDecode(resp.body) as Map<String, dynamic>;
    return PresignedUrl(
      url: data['url'] as String,
      method: data['method'] as String,
      expiresAt: DateTime.parse(data['expires_at'] as String),
      headers: (data['headers'] as Map<String, dynamic>? ?? {})
          .map((k, v) => MapEntry(k, v.toString())),
    );
  }

  @override
  Future<void> delete(String path) async {
    await _doRequest('DELETE', '/api/v1/files/${Uri.encodeComponent(path)}');
  }

  @override
  Future<FileMetadata> getMetadata(String path) async {
    final resp = await _doRequest(
      'GET',
      '/api/v1/files/${Uri.encodeComponent(path)}/metadata',
    );
    final data = jsonDecode(resp.body) as Map<String, dynamic>;
    return FileMetadata(
      path: data['path'] as String,
      sizeBytes: data['size_bytes'] as int,
      contentType: data['content_type'] as String,
      etag: data['etag'] as String,
      lastModified: DateTime.parse(data['last_modified'] as String),
      tags: (data['tags'] as Map<String, dynamic>? ?? {})
          .map((k, v) => MapEntry(k, v.toString())),
    );
  }

  @override
  Future<List<FileMetadata>> list(String prefix) async {
    final encoded = Uri.encodeQueryComponent(prefix);
    final resp = await _doRequest('GET', '/api/v1/files?prefix=$encoded');
    final list = jsonDecode(resp.body) as List<dynamic>;
    return list.map((item) {
      final data = item as Map<String, dynamic>;
      return FileMetadata(
        path: data['path'] as String,
        sizeBytes: data['size_bytes'] as int,
        contentType: data['content_type'] as String,
        etag: data['etag'] as String,
        lastModified: DateTime.parse(data['last_modified'] as String),
        tags: (data['tags'] as Map<String, dynamic>? ?? {})
            .map((k, v) => MapEntry(k, v.toString())),
      );
    }).toList();
  }

  @override
  Future<void> copy(String src, String dst) async {
    await _doRequest('POST', '/api/v1/files/copy', body: {'src': src, 'dst': dst});
  }
}
