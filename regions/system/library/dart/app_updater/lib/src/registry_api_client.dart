import 'dart:convert';

import 'package:http/http.dart' as http;

import 'error.dart';
import 'model.dart';

/// レジストリサーバーのHTTP APIクライアント。
/// バージョン情報の取得やダウンロードURLの解決を担当する。
class RegistryApiClient {
  /// サーバーのベースURL。
  /// すべてのAPIエンドポイントのプレフィックスとして使用される。
  final String baseUrl;

  /// HTTPリクエストのタイムアウト。
  /// 超過した場合は ConnectionError がスローされる。
  final Duration timeout;

  /// 実際のHTTP通信を行うクライアント。テスト時にモックを注入できる。
  final http.Client _client;

  /// アクセストークンを取得するプロバイダー。設定されている場合は Bearer 認証に使用する。
  final Future<String> Function()? _tokenProvider;

  /// [RegistryApiClient] を生成する。
  /// [client] を省略した場合はデフォルトの http.Client を使用する。
  RegistryApiClient({
    required this.baseUrl,
    http.Client? client,
    this.timeout = const Duration(seconds: 10),
    Future<String> Function()? tokenProvider,
  })  : _client = client ?? http.Client(),
        _tokenProvider = tokenProvider;

  /// 指定アプリの最新バージョン情報を取得する。
  /// [platform] と [arch] はクエリパラメーターとして付与され、
  /// プラットフォーム固有のアーティファクト情報を絞り込む。
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

  /// 指定アプリの全バージョン一覧を取得する。
  /// レスポンスの 'versions' フィールドに含まれる配列をパースして返す。
  Future<List<RegistryVersionInfo>> listVersions(String appId) async {
    final uri = Uri.parse('$baseUrl/api/v1/apps/$appId/versions');
    final response = await _request(uri);
    final body = _decodeMap(response.body);
    final versions = body['versions'];

    // versions フィールドがリストでない場合はパースエラーとして扱う。
    if (versions is! List) {
      throw const ParseError('The versions response body was malformed.');
    }

    return versions.map((item) {
      // リスト内の各要素が JSON オブジェクトであることを確認する。
      if (item is! Map<String, dynamic>) {
        throw const ParseError('A version entry was not a JSON object.');
      }
      return RegistryVersionInfo.fromJson(item);
    }).toList();
  }

  /// 指定バージョンのダウンロード情報を取得する。
  /// 署名付きダウンロードURL、チェックサム、ファイルサイズを返す。
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

    // レスポンスボディから必要フィールドを取り出して DownloadArtifactInfo を構築する。
    return DownloadArtifactInfo(
      downloadUrl: body['download_url'] as String,
      expiresIn: body['expires_in'] as int? ?? 0,
      checksumSha256: body['checksum_sha256'] as String,
      sizeBytes: body['size_bytes'] as int?,
    );
  }

  /// 認証ヘッダーを付与してGETリクエストを送信する共通メソッド。
  /// タイムアウト超過や接続失敗は ConnectionError に変換する。
  /// HTTPステータスコードに基づいて適切なエラーをスローする。
  Future<http.Response> _request(Uri uri) async {
    final headers = <String, String>{
      'Content-Type': 'application/json',
    };

    // トークンが取得できた場合は Authorization ヘッダーに付与する。
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

    // 2xx 系は正常レスポンスとして返す。
    if (response.statusCode >= 200 && response.statusCode < 300) {
      return response;
    }

    // エラーステータスコードを種別ごとに適切なエラークラスに変換する。
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

  /// レスポンスボディをJSONオブジェクトにデコードする。
  /// JSON 形式でない場合や、オブジェクトでない場合は ParseError をスローする。
  Map<String, dynamic> _decodeMap(String body) {
    try {
      final decoded = jsonDecode(body);
      if (decoded is Map<String, dynamic>) {
        return decoded;
      }
      throw const ParseError('The response body was not a JSON object.');
    } on AppUpdaterError {
      // AppUpdaterError はそのまま再スローする。
      rethrow;
    } on FormatException catch (error) {
      throw ParseError('Failed to parse JSON response: $error');
    }
  }
}

/// レジストリサーバーから取得したバージョン情報。
/// API レスポンスを直接マッピングした生データモデル。
class RegistryVersionInfo {
  /// アプリID。レジストリ上のアプリを識別する。
  final String appId;

  /// バージョン番号。セマンティックバージョニング形式（例: '1.2.3'）。
  final String version;

  /// 対象プラットフォーム。'ios', 'android', 'windows' などの文字列。
  final String platform;

  /// 対象アーキテクチャ。'amd64', 'arm64' などの文字列。
  final String arch;

  /// ファイルサイズ（バイト）。ダウンロード前のサイズ表示に使用する。
  final int? sizeBytes;

  /// SHA-256チェックサム。ダウンロード後のファイル整合性検証に使用する。
  final String checksumSha256;

  /// リリースノート。このバージョンの変更内容を説明するテキスト。
  final String? releaseNotes;

  /// 強制アップデートかどうか。true の場合はユーザーが更新を拒否できない。
  final bool mandatory;

  /// リリース日時。このバージョンがサーバーに公開された日時。
  final DateTime? publishedAt;

  /// ダウンロードURL。アーティファクトを取得するためのURL。
  final String? downloadUrl;

  /// [RegistryVersionInfo] を生成する。
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

  /// JSONマップから [RegistryVersionInfo] を生成する。
  /// APIレスポンスのJSONフィールドをDartの型に変換する。
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
      // published_at が null の場合は null を格納する。
      publishedAt: json['published_at'] == null
          ? null
          : DateTime.parse(json['published_at'] as String),
      downloadUrl: json['download_url'] as String?,
    );
  }
}
