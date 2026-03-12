import 'dart:convert';

import 'package:http/http.dart' as http;

import 'error.dart';
import 'model.dart';

class RegistryApiClient {
  final String baseUrl;
  final Duration timeout;
  final http.Client _client;
  final Future<String> Function()? _tokenProvider;

  RegistryApiClient({
    required this.baseUrl,
    http.Client? client,
    this.timeout = const Duration(seconds: 10),
    Future<String> Function()? tokenProvider,
  })  : _client = client ?? http.Client(),
        _tokenProvider = tokenProvider;

  Future<RegistryVersionInfo> getLatestVersion(
    String appId, {
    String? platform,
    String? arch,
  }) async {
    final uri = Uri.parse(
      '$baseUrl/api/v1/apps/$appId/latest',
    ).replace(
      queryParameters: {
        if (platform != null && platform.isNotEmpty) 'platform': platform,
        if (arch != null && arch.isNotEmpty) 'arch': arch,
      },
    );

    final response = await _request(uri);
    return RegistryVersionInfo.fromJson(_decodeMap(response.body));
  }

  Future<List<RegistryVersionInfo>> listVersions(String appId) async {
    final uri = Uri.parse('$baseUrl/api/v1/apps/$appId/versions');
    final response = await _request(uri);
    final body = _decodeMap(response.body);
    final versions = body['versions'];

    if (versions is! List) {
      throw const ParseError('The versions response body was malformed.');
    }

    return versions.map((item) {
      if (item is! Map<String, dynamic>) {
        throw const ParseError('A version entry was not a JSON object.');
      }
      return RegistryVersionInfo.fromJson(item);
    }).toList();
  }

  Future<DownloadArtifactInfo> getDownloadInfo(
    String appId,
    String version, {
    String? platform,
    String? arch,
  }) async {
    final uri = Uri.parse(
      '$baseUrl/api/v1/apps/$appId/versions/$version/download',
    ).replace(
      queryParameters: {
        if (platform != null && platform.isNotEmpty) 'platform': platform,
        if (arch != null && arch.isNotEmpty) 'arch': arch,
      },
    );

    final response = await _request(uri);
    final body = _decodeMap(response.body);

    return DownloadArtifactInfo(
      downloadUrl: body['download_url'] as String,
      expiresIn: body['expires_in'] as int? ?? 0,
      checksumSha256: body['checksum_sha256'] as String,
      sizeBytes: body['size_bytes'] as int?,
    );
  }

  Future<http.Response> _request(Uri uri) async {
    final headers = <String, String>{
      'Content-Type': 'application/json',
    };

    final token = await _tokenProvider?.call();
    if (token != null && token.isNotEmpty) {
      headers['Authorization'] = 'Bearer $token';
    }

    late http.Response response;
    try {
      response = await _client.get(uri, headers: headers).timeout(timeout);
    } on Exception catch (error) {
      throw ConnectionError('Request failed: $error');
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return response;
    }

    switch (response.statusCode) {
      case 401:
      case 403:
        throw UnauthorizedError('Authorization failed for ${uri.path}.');
      case 404:
        throw VersionNotFoundError(
            'No version information was found for ${uri.path}.');
      default:
        throw ConnectionError('HTTP ${response.statusCode}: ${response.body}');
    }
  }

  Map<String, dynamic> _decodeMap(String body) {
    try {
      final decoded = jsonDecode(body);
      if (decoded is Map<String, dynamic>) {
        return decoded;
      }
      throw const ParseError('The response body was not a JSON object.');
    } on AppUpdaterError {
      rethrow;
    } on FormatException catch (error) {
      throw ParseError('Failed to parse JSON response: $error');
    }
  }
}

class RegistryVersionInfo {
  final String appId;
  final String version;
  final String platform;
  final String arch;
  final int? sizeBytes;
  final String checksumSha256;
  final String? releaseNotes;
  final bool mandatory;
  final DateTime? publishedAt;
  final String? downloadUrl;

  const RegistryVersionInfo({
    required this.appId,
    required this.version,
    required this.platform,
    required this.arch,
    required this.sizeBytes,
    required this.checksumSha256,
    required this.releaseNotes,
    required this.mandatory,
    required this.publishedAt,
    required this.downloadUrl,
  });

  factory RegistryVersionInfo.fromJson(Map<String, dynamic> json) {
    return RegistryVersionInfo(
      appId: json['app_id'] as String? ?? '',
      version: json['version'] as String,
      platform: json['platform'] as String,
      arch: json['arch'] as String,
      sizeBytes: json['size_bytes'] as int?,
      checksumSha256: json['checksum_sha256'] as String,
      releaseNotes: json['release_notes'] as String?,
      mandatory: json['mandatory'] as bool? ?? false,
      publishedAt: json['published_at'] == null
          ? null
          : DateTime.parse(json['published_at'] as String),
      downloadUrl: json['download_url'] as String?,
    );
  }
}
