import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:http/http.dart' as http;

import 'model.dart';
import 'client.dart';

// ---------------------------------------------------------------------------
// S3FileClient — S3 互換ストレージへの直接アクセス実装
// ---------------------------------------------------------------------------

/// S3 REST API に直接アクセスする [FileClient] 実装。
///
/// AWS Signature V4 を使用してリクエストに署名する。
/// プリサインド URL 生成にはクエリ文字列認証を使用する。
class S3FileClient implements FileClient {
  final String _endpoint;
  final String _bucket;
  final String _region;
  final String _accessKeyId;
  final String _secretAccessKey;
  final Duration _timeout;
  final http.Client _http;

  S3FileClient(
    FileClientConfig config, {
    http.Client? httpClient,
  })  : _endpoint = _requireField(config.s3Endpoint, 's3Endpoint'),
        _bucket = _requireField(config.bucket, 'bucket'),
        _region = _requireField(config.region, 'region'),
        _accessKeyId = _requireField(config.accessKeyId, 'accessKeyId'),
        _secretAccessKey =
            _requireField(config.secretAccessKey, 'secretAccessKey'),
        _timeout = config.timeout,
        _http = httpClient ?? http.Client();

  static String _requireField(String? value, String name) {
    if (value == null || value.isEmpty) {
      throw FileClientError('$name が設定されていません', 'INVALID_CONFIG');
    }
    return value;
  }

  // ---------------------------------------------------------------------------
  // FileClient 実装
  // ---------------------------------------------------------------------------

  @override
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  ) async {
    final url = _generatePresignedUrl(
      method: 'PUT',
      objectKey: path,
      expiresIn: expiresIn,
      additionalHeaders: {'content-type': contentType},
    );
    return PresignedUrl(
      url: url,
      method: 'PUT',
      expiresAt: DateTime.now().toUtc().add(expiresIn),
      headers: {'content-type': contentType},
    );
  }

  @override
  Future<PresignedUrl> generateDownloadUrl(
    String path,
    Duration expiresIn,
  ) async {
    final url = _generatePresignedUrl(
      method: 'GET',
      objectKey: path,
      expiresIn: expiresIn,
    );
    return PresignedUrl(
      url: url,
      method: 'GET',
      expiresAt: DateTime.now().toUtc().add(expiresIn),
      headers: {},
    );
  }

  @override
  Future<void> delete(String path) async {
    final now = DateTime.now().toUtc();
    final uri = _objectUri(path);
    final headers = _signRequest(
      method: 'DELETE',
      uri: uri,
      headers: {},
      payload: Uint8List(0),
      dateTime: now,
    );
    final resp = await _http
        .delete(uri, headers: headers)
        .timeout(_timeout);
    _checkResponse(resp, path);
  }

  @override
  Future<FileMetadata> getMetadata(String path) async {
    final now = DateTime.now().toUtc();
    final uri = _objectUri(path);
    final headers = _signRequest(
      method: 'HEAD',
      uri: uri,
      headers: {},
      payload: Uint8List(0),
      dateTime: now,
    );
    final resp = await _http
        .head(uri, headers: headers)
        .timeout(_timeout);
    _checkResponse(resp, path);

    final contentLength =
        int.tryParse(resp.headers['content-length'] ?? '0') ?? 0;
    final contentType =
        resp.headers['content-type'] ?? 'application/octet-stream';
    final etag = (resp.headers['etag'] ?? '').replaceAll('"', '');
    final lastModified = _parseLastModified(resp.headers['last-modified']);

    return FileMetadata(
      path: path,
      sizeBytes: contentLength,
      contentType: contentType,
      etag: etag,
      lastModified: lastModified,
      tags: {},
    );
  }

  @override
  Future<List<FileMetadata>> list(String prefix) async {
    final now = DateTime.now().toUtc();
    final uri = _bucketUri().replace(queryParameters: {
      'list-type': '2',
      'prefix': prefix,
    });
    final headers = _signRequest(
      method: 'GET',
      uri: uri,
      headers: {},
      payload: Uint8List(0),
      dateTime: now,
    );
    final resp = await _http
        .get(uri, headers: headers)
        .timeout(_timeout);
    _checkResponse(resp, prefix);

    return _parseListObjectsV2Response(resp.body);
  }

  @override
  Future<void> copy(String src, String dst) async {
    final now = DateTime.now().toUtc();
    final uri = _objectUri(dst);
    final copySource = '/$_bucket/${_encodeKey(src)}';
    final headers = _signRequest(
      method: 'PUT',
      uri: uri,
      headers: {'x-amz-copy-source': copySource},
      payload: Uint8List(0),
      dateTime: now,
    );
    final resp = await _http
        .put(uri, headers: headers)
        .timeout(_timeout);
    _checkResponse(resp, '$src -> $dst');
  }

  // ---------------------------------------------------------------------------
  // URI 構築
  // ---------------------------------------------------------------------------

  /// バケットルートの URI（パススタイル）。
  Uri _bucketUri() {
    final base = _endpoint.endsWith('/')
        ? _endpoint.substring(0, _endpoint.length - 1)
        : _endpoint;
    return Uri.parse('$base/$_bucket');
  }

  /// オブジェクトキーの URI（パススタイル）。
  Uri _objectUri(String key) {
    final base = _endpoint.endsWith('/')
        ? _endpoint.substring(0, _endpoint.length - 1)
        : _endpoint;
    return Uri.parse('$base/$_bucket/${_encodeKey(key)}');
  }

  /// オブジェクトキーの各セグメントを URI エンコードする。
  static String _encodeKey(String key) {
    return key
        .split('/')
        .map((segment) => Uri.encodeComponent(segment))
        .join('/');
  }

  // ---------------------------------------------------------------------------
  // レスポンス処理
  // ---------------------------------------------------------------------------

  void _checkResponse(http.Response resp, String context) {
    if (resp.statusCode == 404 || resp.statusCode == 403) {
      throw FileClientError(
        'Object not found: $context',
        'NOT_FOUND',
      );
    }
    if (resp.statusCode == 401) {
      throw FileClientError(
        'Unauthorized: $context',
        'UNAUTHORIZED',
      );
    }
    if (resp.statusCode >= 300) {
      throw FileClientError(
        'S3 error ${resp.statusCode}: ${resp.body}',
        'S3_ERROR',
      );
    }
  }

  DateTime _parseLastModified(String? value) {
    if (value == null || value.isEmpty) return DateTime.now().toUtc();
    try {
      return HttpDate.parse(value);
    } catch (_) {
      try {
        return DateTime.parse(value);
      } catch (_) {
        return DateTime.now().toUtc();
      }
    }
  }

  /// ListObjectsV2 XML レスポンスをパースする。
  List<FileMetadata> _parseListObjectsV2Response(String xml) {
    final results = <FileMetadata>[];
    // <Contents> ブロックを正規表現で抽出（軽量 XML パーサー不要）
    final contentsPattern = RegExp(
      r'<Contents>(.*?)</Contents>',
      dotAll: true,
    );
    for (final match in contentsPattern.allMatches(xml)) {
      final block = match.group(1)!;
      final key = _extractXmlValue(block, 'Key');
      final size = int.tryParse(_extractXmlValue(block, 'Size')) ?? 0;
      final etag = _extractXmlValue(block, 'ETag').replaceAll('"', '');
      final lastModifiedStr = _extractXmlValue(block, 'LastModified');
      final lastModified = lastModifiedStr.isNotEmpty
          ? DateTime.parse(lastModifiedStr)
          : DateTime.now().toUtc();

      results.add(FileMetadata(
        path: key,
        sizeBytes: size,
        contentType: 'application/octet-stream',
        etag: etag,
        lastModified: lastModified,
        tags: {},
      ));
    }
    return results;
  }

  static String _extractXmlValue(String xml, String tag) {
    final pattern = RegExp('<$tag>(.*?)</$tag>', dotAll: true);
    final match = pattern.firstMatch(xml);
    final raw = match?.group(1)?.trim() ?? '';
    return _decodeXmlEntities(raw);
  }

  /// XML エンティティをデコードする。
  static String _decodeXmlEntities(String text) {
    return text
        .replaceAll('&amp;', '&')
        .replaceAll('&lt;', '<')
        .replaceAll('&gt;', '>')
        .replaceAll('&quot;', '"')
        .replaceAll('&apos;', "'");
  }

  // ---------------------------------------------------------------------------
  // AWS Signature V4 — リクエスト署名
  // ---------------------------------------------------------------------------

  /// 通常の S3 リクエストに Authorization ヘッダーを付与して署名する。
  Map<String, String> _signRequest({
    required String method,
    required Uri uri,
    required Map<String, String> headers,
    required Uint8List payload,
    required DateTime dateTime,
  }) {
    final dateStamp = _dateStamp(dateTime);
    final amzDate = _amzDate(dateTime);
    final host = uri.host + (uri.hasPort ? ':${uri.port}' : '');

    final signedHeaders = Map<String, String>.from(headers);
    signedHeaders['host'] = host;
    signedHeaders['x-amz-date'] = amzDate;
    signedHeaders['x-amz-content-sha256'] = _hashPayload(payload);

    final sortedHeaderKeys = signedHeaders.keys.toList()
      ..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
    final canonicalHeaders = sortedHeaderKeys
        .map((k) => '${k.toLowerCase()}:${signedHeaders[k]!.trim()}')
        .join('\n');
    final signedHeaderNames =
        sortedHeaderKeys.map((k) => k.toLowerCase()).join(';');

    final canonicalQueryString = _canonicalQueryString(uri);

    final canonicalRequest = [
      method,
      _canonicalPath(uri),
      canonicalQueryString,
      '$canonicalHeaders\n',
      signedHeaderNames,
      _hashPayload(payload),
    ].join('\n');

    final credentialScope = '$dateStamp/$_region/s3/aws4_request';
    final stringToSign = [
      'AWS4-HMAC-SHA256',
      amzDate,
      credentialScope,
      _sha256Hex(utf8.encode(canonicalRequest)),
    ].join('\n');

    final signingKey = _deriveSigningKey(dateStamp);
    final signature = _hmacSha256Hex(signingKey, utf8.encode(stringToSign));

    final authorization =
        'AWS4-HMAC-SHA256 Credential=$_accessKeyId/$credentialScope, '
        'SignedHeaders=$signedHeaderNames, '
        'Signature=$signature';

    return {
      ...signedHeaders,
      'authorization': authorization,
    };
  }

  // ---------------------------------------------------------------------------
  // AWS Signature V4 — プリサインド URL 生成
  // ---------------------------------------------------------------------------

  /// S3 プリサインド URL をクエリ文字列認証で生成する。
  String _generatePresignedUrl({
    required String method,
    required String objectKey,
    required Duration expiresIn,
    Map<String, String> additionalHeaders = const {},
  }) {
    final now = DateTime.now().toUtc();
    final dateStamp = _dateStamp(now);
    final amzDate = _amzDate(now);
    final expiresSeconds = expiresIn.inSeconds;

    final uri = _objectUri(objectKey);
    final host = uri.host + (uri.hasPort ? ':${uri.port}' : '');

    // 署名対象ヘッダー（host は必須）
    final headersToSign = <String, String>{'host': host};
    for (final entry in additionalHeaders.entries) {
      headersToSign[entry.key.toLowerCase()] = entry.value;
    }
    final sortedHeaderKeys = headersToSign.keys.toList()..sort();
    final signedHeaderNames = sortedHeaderKeys.join(';');

    final credentialScope = '$dateStamp/$_region/s3/aws4_request';
    final credential = '$_accessKeyId/$credentialScope';

    // クエリパラメータ
    final queryParams = <String, String>{
      'X-Amz-Algorithm': 'AWS4-HMAC-SHA256',
      'X-Amz-Credential': credential,
      'X-Amz-Date': amzDate,
      'X-Amz-Expires': expiresSeconds.toString(),
      'X-Amz-SignedHeaders': signedHeaderNames,
    };

    // 既存のクエリパラメータとマージ
    final allParams = <String, String>{
      ...uri.queryParameters,
      ...queryParams,
    };

    // 正規クエリ文字列（ソート済み）
    final sortedParamKeys = allParams.keys.toList()..sort();
    final canonicalQueryString = sortedParamKeys
        .map((k) =>
            '${Uri.encodeQueryComponent(k)}=${Uri.encodeQueryComponent(allParams[k]!)}')
        .join('&');

    final canonicalHeaders = sortedHeaderKeys
        .map((k) => '${k.toLowerCase()}:${headersToSign[k]!.trim()}')
        .join('\n');

    // プリサインドではペイロードは UNSIGNED-PAYLOAD
    final canonicalRequest = [
      method,
      _canonicalPath(uri),
      canonicalQueryString,
      '$canonicalHeaders\n',
      signedHeaderNames,
      'UNSIGNED-PAYLOAD',
    ].join('\n');

    final stringToSign = [
      'AWS4-HMAC-SHA256',
      amzDate,
      credentialScope,
      _sha256Hex(utf8.encode(canonicalRequest)),
    ].join('\n');

    final signingKey = _deriveSigningKey(dateStamp);
    final signature = _hmacSha256Hex(signingKey, utf8.encode(stringToSign));

    // 最終 URL を構築
    final signedUri = uri.replace(queryParameters: {
      ...allParams,
      'X-Amz-Signature': signature,
    });
    return signedUri.toString();
  }

  // ---------------------------------------------------------------------------
  // 暗号ユーティリティ
  // ---------------------------------------------------------------------------

  static String _dateStamp(DateTime dt) {
    return '${dt.year.toString().padLeft(4, '0')}'
        '${dt.month.toString().padLeft(2, '0')}'
        '${dt.day.toString().padLeft(2, '0')}';
  }

  static String _amzDate(DateTime dt) {
    return '${_dateStamp(dt)}T'
        '${dt.hour.toString().padLeft(2, '0')}'
        '${dt.minute.toString().padLeft(2, '0')}'
        '${dt.second.toString().padLeft(2, '0')}Z';
  }

  static String _canonicalPath(Uri uri) {
    if (uri.path.isEmpty) return '/';
    // パスセグメントは既にエンコード済みなのでそのまま返す
    return uri.path;
  }

  static String _canonicalQueryString(Uri uri) {
    if (uri.queryParameters.isEmpty) return '';
    final sorted = uri.queryParameters.entries.toList()
      ..sort((a, b) => a.key.compareTo(b.key));
    return sorted
        .map((e) =>
            '${Uri.encodeQueryComponent(e.key)}=${Uri.encodeQueryComponent(e.value)}')
        .join('&');
  }

  static String _sha256Hex(List<int> data) {
    return sha256.convert(data).toString();
  }

  static String _hashPayload(Uint8List payload) {
    return _sha256Hex(payload);
  }

  /// AWS Signature V4 の署名キーを導出する。
  List<int> _deriveSigningKey(String dateStamp) {
    final kDate =
        _hmacSha256(utf8.encode('AWS4$_secretAccessKey'), utf8.encode(dateStamp));
    final kRegion = _hmacSha256(kDate, utf8.encode(_region));
    final kService = _hmacSha256(kRegion, utf8.encode('s3'));
    final kSigning = _hmacSha256(kService, utf8.encode('aws4_request'));
    return kSigning;
  }

  static List<int> _hmacSha256(List<int> key, List<int> data) {
    final hmac = Hmac(sha256, key);
    return hmac.convert(data).bytes;
  }

  static String _hmacSha256Hex(List<int> key, List<int> data) {
    final hmac = Hmac(sha256, key);
    return hmac.convert(data).toString();
  }
}

// ---------------------------------------------------------------------------
// HttpDate — HTTP 日付形式パーサー
// ---------------------------------------------------------------------------

/// RFC 7231 / RFC 850 / asctime 形式の HTTP 日付をパースするユーティリティ。
class HttpDate {
  static final _rfc1123 = RegExp(
    r'\w{3}, (\d{2}) (\w{3}) (\d{4}) (\d{2}):(\d{2}):(\d{2}) GMT',
  );

  static const _months = {
    'Jan': 1, 'Feb': 2, 'Mar': 3, 'Apr': 4,
    'May': 5, 'Jun': 6, 'Jul': 7, 'Aug': 8,
    'Sep': 9, 'Oct': 10, 'Nov': 11, 'Dec': 12,
  };

  static DateTime parse(String value) {
    final match = _rfc1123.firstMatch(value);
    if (match != null) {
      final day = int.parse(match.group(1)!);
      final month = _months[match.group(2)!]!;
      final year = int.parse(match.group(3)!);
      final hour = int.parse(match.group(4)!);
      final minute = int.parse(match.group(5)!);
      final second = int.parse(match.group(6)!);
      return DateTime.utc(year, month, day, hour, minute, second);
    }
    throw FormatException('Unsupported HTTP date format: $value');
  }
}
