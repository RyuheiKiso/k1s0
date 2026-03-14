import 'dart:async';

import 'package:package_info_plus/package_info_plus.dart';
import 'package:url_launcher/url_launcher.dart';

import 'config.dart';
import 'error.dart';
import 'model.dart';
import 'platform_detector.dart';
import 'registry_api_client.dart';

/// アプリアップデーターのインターフェース。
/// 各プラットフォーム向けの実装が準拠すべき契約を定義する。
abstract class AppUpdater {
  /// サーバーからバージョン情報を取得する。
  Future<AppVersionInfo> fetchVersionInfo();

  /// アップデートの有無を確認する。
  Future<UpdateCheckResult> checkForUpdate();

  /// 定期的なアップデート確認を開始する。
  /// [onUpdateAvailable] はアップデートが検出されるたびに呼び出される。
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  });

  /// 定期的なアップデート確認を停止する。
  void stopPeriodicCheck();

  /// ストアURLを返す。プラットフォームが非対応の場合は `null` を返す。
  String? getStoreUrl();

  /// ストアを開く。成功した場合は `true` を返す。
  Future<bool> openStore();

  /// リソースを解放する。タイマーなど保持するリソースをクリーンアップする。
  void dispose();
}

/// レジストリサーバーを使用する [AppUpdater] の実装。
/// [AppUpdaterConfig] を受け取り、HTTPクライアント経由でバージョン情報を管理する。
class AppRegistryAppUpdater implements AppUpdater {
  /// アップデーターの設定情報。
  final AppUpdaterConfig _config;

  /// レジストリAPIへのHTTPクライアント。
  final RegistryApiClient _client;

  /// 現在のアプリバージョンを取得するプロバイダー関数。
  final Future<String> Function() _currentVersionProvider;

  /// ストアURLを開く関数。テスト時にモック差し替えが可能。
  final Future<bool> Function(Uri uri) _launchUrl;

  /// 定期チェック用のタイマー。stopPeriodicCheck で破棄される。
  Timer? _periodicTimer;

  /// [AppRegistryAppUpdater] を生成する。
  /// [client] を省略した場合は設定から自動生成したクライアントを使用する。
  AppRegistryAppUpdater(
    this._config, {
    RegistryApiClient? client,
  })  : _client = client ??
            RegistryApiClient(
              baseUrl: _config.serverUrl,
              client: _config.httpClient,
              timeout: _config.timeout,
              tokenProvider: _config.tokenProvider,
            ),
        _currentVersionProvider =
            _config.currentVersionProvider ?? _defaultCurrentVersionProvider,
        _launchUrl = _config.urlLauncher ?? launchUrl {
    // 設定値のバリデーションをコンストラクタ内で即時実行する。
    _validateConfig(_config);
  }

  @override
  Future<AppVersionInfo> fetchVersionInfo() async {
    // プラットフォームとアーキテクチャを解決してAPIに渡す。
    final platform = _resolvePlatform();
    final arch = _resolveArch();

    final latest = await _client.getLatestVersion(
      _config.appId,
      platform: platform,
      arch: arch,
    );
    final versions = await _client.listVersions(_config.appId);

    // 必須バージョン一覧を公開日時の降順でソートし、最も新しいものを最低バージョンとする。
    final mandatoryVersions =
        versions.where((version) => version.mandatory).toList()
          ..sort((left, right) {
            final leftPublishedAt =
                left.publishedAt ?? DateTime.fromMillisecondsSinceEpoch(0);
            final rightPublishedAt =
                right.publishedAt ?? DateTime.fromMillisecondsSinceEpoch(0);
            return rightPublishedAt.compareTo(leftPublishedAt);
          });

    // 必須バージョンが存在する場合はその最新版を最低バージョンとして使用する。
    // 存在しない場合は最新バージョン自体が必須かどうかで判断し、非必須なら '0.0.0' を使用する。
    final minimumVersion = mandatoryVersions.isNotEmpty
        ? mandatoryVersions.first.version
        : latest.mandatory
            ? latest.version
            : '0.0.0';

    return AppVersionInfo(
      latestVersion: latest.version,
      minimumVersion: minimumVersion,
      releaseNotes: latest.releaseNotes,
      mandatory: latest.mandatory,
      storeUrl: getStoreUrl(),
      publishedAt: latest.publishedAt,
      platform: latest.platform,
      arch: latest.arch,
      sizeBytes: latest.sizeBytes,
      checksumSha256: latest.checksumSha256,
      downloadUrl: latest.downloadUrl,
    );
  }

  /// 指定バージョンのダウンロード情報を取得する。省略時は最新バージョンを使用。
  /// [version] が null の場合は fetchVersionInfo を呼び出して最新バージョンを解決する。
  Future<DownloadArtifactInfo> fetchDownloadInfo({
    String? version,
    String? platform,
    String? arch,
  }) async {
    // バージョン未指定の場合のみ fetchVersionInfo を呼び出してバージョンを解決する。
    final resolvedInfo = version == null ? await fetchVersionInfo() : null;
    return _client.getDownloadInfo(
      _config.appId,
      version ?? resolvedInfo!.latestVersion,
      platform: platform ?? _resolvePlatform(),
      arch: arch ?? _resolveArch(),
    );
  }

  @override
  Future<UpdateCheckResult> checkForUpdate() async {
    // 現在のバージョンとサーバーの情報を取得してアップデート種別を判定する。
    final currentVersion = await _currentVersionProvider();
    final versionInfo = await fetchVersionInfo();
    final updateType = determineUpdateType(
      currentVersion: currentVersion,
      versionInfo: versionInfo,
    );

    return UpdateCheckResult(
      type: updateType,
      currentVersion: currentVersion,
      versionInfo: versionInfo,
    );
  }

  @override
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  }) {
    final interval = _config.checkInterval;
    // checkInterval が null の場合は定期チェックを行わない。
    if (interval == null) {
      return;
    }

    // 既存のタイマーがあればキャンセルしてから新規作成する。
    _periodicTimer?.cancel();
    _periodicTimer = Timer.periodic(interval, (_) async {
      try {
        final result = await checkForUpdate();
        // アップデートが存在する場合のみコールバックを呼び出す。
        if (result.needsUpdate) {
          onUpdateAvailable(result);
        }
      } on AppUpdaterError {
        // バックグラウンドポーリング中のエラーは無視する。
        // ユーザー操作を妨げないよう、次回チェックまで静かに待機する。
      }
    });
  }

  @override
  void stopPeriodicCheck() {
    // タイマーをキャンセルして null にリセットする。
    _periodicTimer?.cancel();
    _periodicTimer = null;
  }

  @override
  String? getStoreUrl() {
    // プラットフォームに応じてストアURLを返す。非対応プラットフォームは null を返す。
    final platform = (_config.platform ?? _safePlatform())?.toLowerCase();
    if (platform == null) {
      return null;
    }
    switch (platform) {
      case 'ios':
        return _config.iosStoreUrl;
      case 'android':
        return _config.androidStoreUrl;
      default:
        return null;
    }
  }

  @override
  Future<bool> openStore() async {
    // ストアURLが取得できない場合は false を返す。
    final storeUrl = getStoreUrl();
    if (storeUrl == null) {
      return false;
    }
    return _launchUrl(Uri.parse(storeUrl));
  }

  @override
  void dispose() {
    // 保持するすべてのリソースを解放する。
    stopPeriodicCheck();
  }

  /// 設定またはプラットフォーム自動検出からプラットフォーム文字列を解決する。
  String? _resolvePlatform() {
    return (_config.platform ?? _safePlatform())?.toLowerCase();
  }

  /// 未対応プラットフォームの場合に null を返す安全なプラットフォーム検出。
  /// [UnsupportedError] をキャッチして null を返すことでクラッシュを防ぐ。
  String? _safePlatform() {
    try {
      return PlatformDetector.currentPlatform;
    } on UnsupportedError {
      return null;
    }
  }

  /// 設定またはプラットフォーム自動検出からアーキテクチャ文字列を解決する。
  /// 設定に明示的な値がある場合はそちらを優先する。
  String? _resolveArch() {
    if (_config.arch != null && _config.arch!.isNotEmpty) {
      return _config.arch;
    }

    try {
      return PlatformDetector.currentArch;
    } on UnsupportedError {
      return null;
    }
  }
}

/// `package_info_plus` を使ってアプリのバージョンを取得するデフォルト実装。
/// [AppUpdaterConfig.currentVersionProvider] が未設定の場合に使用される。
Future<String> _defaultCurrentVersionProvider() async {
  final packageInfo = await PackageInfo.fromPlatform();
  return packageInfo.version;
}

/// 設定値のバリデーションを行い、不正な場合は [InvalidConfigError] をスローする。
/// serverUrl と appId は空文字を許可しない。
void _validateConfig(AppUpdaterConfig config) {
  if (config.serverUrl.trim().isEmpty) {
    throw const InvalidConfigError('serverUrl must not be empty.');
  }
  if (config.appId.trim().isEmpty) {
    throw const InvalidConfigError('appId must not be empty.');
  }
}

/// 現在のバージョンとサーバーのバージョン情報を比較してアップデートの種類を返す。
/// 現在バージョンが最低バージョンを下回るか、最新バージョンが必須フラグを持つ場合は mandatory を返す。
UpdateType determineUpdateType({
  required String currentVersion,
  required AppVersionInfo versionInfo,
}) {
  // 現在バージョンが最低バージョンを下回る、またはサーバーが強制アップデートを要求する場合。
  if (_compareVersions(currentVersion, versionInfo.minimumVersion) < 0 ||
      versionInfo.mandatory) {
    return UpdateType.mandatory;
  }

  // 現在バージョンが最新バージョンを下回る場合は任意アップデートとして扱う。
  if (_compareVersions(currentVersion, versionInfo.latestVersion) < 0) {
    return UpdateType.optional;
  }

  // 最新バージョン以上であればアップデート不要。
  return UpdateType.none;
}

/// 2つのバージョン文字列を比較する。
/// left < right の場合は負の値、left > right の場合は正の値、等しい場合は 0 を返す。
int _compareVersions(String left, String right) {
  final leftParts = _normalizeVersion(left);
  final rightParts = _normalizeVersion(right);
  // 長い方のセグメント数に合わせてループする。
  final length = leftParts.length > rightParts.length
      ? leftParts.length
      : rightParts.length;

  for (var index = 0; index < length; index += 1) {
    // セグメントが存在しない場合は 0 として扱う。
    final leftValue = index < leftParts.length ? leftParts[index] : 0;
    final rightValue = index < rightParts.length ? rightParts[index] : 0;
    if (leftValue != rightValue) {
      return leftValue.compareTo(rightValue);
    }
  }

  return 0;
}

/// バージョン文字列を数値セグメントのリストに変換する。
/// 例: '1.2.3' → [1, 2, 3]。数値以外の文字は除去してパースする。
List<int> _normalizeVersion(String version) {
  return version
      .split('.')
      .map((segment) =>
          int.tryParse(segment.replaceAll(RegExp(r'[^0-9]'), '')) ?? 0)
      .toList();
}
