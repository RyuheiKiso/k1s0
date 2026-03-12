# k1s0-app-updater ライブラリ設計

## 概要

Dart/Flutter 向けアプリ更新管理クライアントライブラリ。system-app-registry-server（REST API）と連携し、アプリバージョン確認・強制アップデート判定・プラットフォーム別ストアURL取得・アップデートダイアログ表示を統一インターフェースで提供する。Flutter アプリケーションに組み込み、ユーザーに適切なタイミングでアップデートを促す。

バックグラウンドでの定期バージョンチェックにも対応し、アプリがフォアグラウンドにある間は設定間隔で最新バージョンを確認する。強制アップデート（最小サポートバージョン未満）と任意アップデート（新バージョン利用可能）を区別し、それぞれに対応するダイアログ UI を表示する。

**配置先**: `regions/system/library/dart/app_updater/`

## 公開 API

| 型 | 種別 | 説明 |
|----|------|------|
| `AppUpdater` | abstract class | アプリ更新管理の抽象インターフェース |
| `AppRegistryAppUpdater` | class | app-registry-server 連携実装 |
| `InMemoryAppUpdater` | class | テスト用インメモリ実装（現在の主実装） |
| `MockAppUpdater` | class | テスト用モック実装 |
| `AppUpdaterConfig` | class | サーバーURL・アプリID・チェック間隔等の設定 |
| `AppVersionInfo` | class | 最新バージョン・最小サポートバージョン・リリースノート・ストアURL |
| `UpdateCheckResult` | class | チェック結果（更新要否・強制/任意・バージョン情報） |
| `UpdateType` | enum | `none`・`optional`・`mandatory` |
| `AppUpdaterError` | class | 接続エラー・設定エラー・パースエラー等 |

## Dart 実装

**pubspec.yaml**:

```yaml
name: k1s0_app_updater
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"
  flutter: ">=3.10.0"

dependencies:
  flutter:
    sdk: flutter
  http: ^1.2.0
  package_info_plus: ^8.0.0
  url_launcher: ^6.2.0
  meta: ^1.14.0
```

**依存追加** (`pubspec.yaml`):

```yaml
dependencies:
  k1s0_app_updater:
    path: ../../system/library/dart/app_updater
```

**モジュール構成**:

```
app_updater/
├── lib/
│   ├── app_updater.dart          # 公開 API（再エクスポート）
│   └── src/
│       ├── app_updater.dart      # AppUpdater (abstract)・AppRegistryAppUpdater
│       ├── in_memory.dart        # InMemoryAppUpdater
│       ├── mock.dart             # MockAppUpdater
│       ├── config.dart           # AppUpdaterConfig
│       ├── model.dart            # AppVersionInfo・UpdateCheckResult・UpdateType
│       ├── dialog.dart           # アップデートダイアログ UI ヘルパー
│       └── error.dart            # AppUpdaterError
├── test/
│   ├── app_updater_test.dart
│   ├── in_memory_test.dart
│   └── model_test.dart
└── pubspec.yaml
```

**データモデル**:

```dart
/// アプリバージョン情報（app-registry-server から取得）
class AppVersionInfo {
  /// 最新バージョン文字列（例: "2.1.0"）
  final String latestVersion;

  /// 最小サポートバージョン（これ未満は強制アップデート）
  final String minimumVersion;

  /// リリースノート
  final String? releaseNotes;

  /// 強制アップデートフラグ（サーバー側の mandatory フィールド）
  final bool mandatory;

  /// プラットフォーム別ストアURL（iOS App Store / Google Play）
  final String? storeUrl;

  /// 公開日時
  final DateTime? publishedAt;

  const AppVersionInfo({
    required this.latestVersion,
    required this.minimumVersion,
    this.releaseNotes,
    this.mandatory = false,
    this.storeUrl,
    this.publishedAt,
  });
}

/// アップデート種別
enum UpdateType {
  /// アップデート不要（現在バージョン >= 最新バージョン）
  none,

  /// 任意アップデート（現在バージョン >= 最小バージョン かつ < 最新バージョン）
  optional,

  /// 強制アップデート（現在バージョン < 最小バージョン、または mandatory フラグが true）
  mandatory,
}

/// アップデートチェック結果
class UpdateCheckResult {
  /// アップデート種別
  final UpdateType type;

  /// 現在のアプリバージョン
  final String currentVersion;

  /// サーバーから取得したバージョン情報
  final AppVersionInfo versionInfo;

  const UpdateCheckResult({
    required this.type,
    required this.currentVersion,
    required this.versionInfo,
  });

  /// アップデートが必要か
  bool get needsUpdate => type != UpdateType.none;

  /// 強制アップデートか
  bool get isMandatory => type == UpdateType.mandatory;
}
```

**設定**:

```dart
class AppUpdaterConfig {
  /// app-registry-server の URL
  final String serverUrl;

  /// アプリ ID（app-registry-server に登録されたアプリ識別子）
  final String appId;

  /// プラットフォーム（"ios" / "android"）。省略時は自動検出
  final String? platform;

  /// バックグラウンドチェック間隔。省略時はバックグラウンドチェック無効
  final Duration? checkInterval;

  /// iOS App Store URL（storeUrl が未設定の場合のフォールバック）
  final String? iosStoreUrl;

  /// Google Play Store URL（storeUrl が未設定の場合のフォールバック）
  final String? androidStoreUrl;

  /// HTTP リクエストタイムアウト。デフォルト Duration(seconds: 10)
  final Duration timeout;

  /// 認証トークン取得コールバック（Bearer トークン）
  final Future<String> Function()? tokenProvider;

  const AppUpdaterConfig({
    required this.serverUrl,
    required this.appId,
    this.platform,
    this.checkInterval,
    this.iosStoreUrl,
    this.androidStoreUrl,
    this.timeout = const Duration(seconds: 10),
    this.tokenProvider,
  });
}
```

**抽象インターフェース**:

```dart
/// アプリ更新管理の抽象インターフェース
abstract class AppUpdater {
  /// 最新バージョン情報を取得する
  Future<AppVersionInfo> fetchVersionInfo();

  /// アップデートチェックを実行する（現在バージョンとの比較結果を返す）
  Future<UpdateCheckResult> checkForUpdate();

  /// バックグラウンド定期チェックを開始する
  /// [onUpdateAvailable] はアップデートが検出されたときに呼ばれるコールバック
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  });

  /// バックグラウンド定期チェックを停止する
  void stopPeriodicCheck();

  /// プラットフォームに応じたストアURLを取得する
  String? getStoreUrl();

  /// ストアURLを開く（url_launcher 使用）
  Future<bool> openStore();

  /// リソースを解放する
  void dispose();
}
```

**AppRegistryAppUpdater 実装**:

`AppRegistryAppUpdater` は app-registry-server の REST API（`GET /api/v1/apps/:id/latest`）と連携し、最新バージョン情報を取得する。`package_info_plus` パッケージで現在のアプリバージョンを自動取得し、セマンティックバージョニングで比較を行う。

```dart
class AppRegistryAppUpdater implements AppUpdater {
  final AppUpdaterConfig _config;
  Timer? _periodicTimer;

  AppRegistryAppUpdater(this._config);

  @override
  Future<AppVersionInfo> fetchVersionInfo() async {
    // app-registry-server の GET /api/v1/apps/:id/latest にリクエスト
    // platform クエリパラメータでフィルター
    // レスポンスから AppVersionInfo を構築
  }

  @override
  Future<UpdateCheckResult> checkForUpdate() async {
    // 1. package_info_plus で現在バージョンを取得
    // 2. fetchVersionInfo() でサーバーから最新情報を取得
    // 3. セマンティックバージョン比較で UpdateType を判定
    //    - currentVersion < minimumVersion → UpdateType.mandatory
    //    - mandatory フラグが true → UpdateType.mandatory
    //    - currentVersion < latestVersion → UpdateType.optional
    //    - それ以外 → UpdateType.none
  }

  @override
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  }) {
    final interval = _config.checkInterval;
    if (interval == null) return;

    _periodicTimer?.cancel();
    _periodicTimer = Timer.periodic(interval, (_) async {
      final result = await checkForUpdate();
      if (result.needsUpdate) {
        onUpdateAvailable(result);
      }
    });
  }

  @override
  void stopPeriodicCheck() {
    _periodicTimer?.cancel();
    _periodicTimer = null;
  }

  @override
  String? getStoreUrl() {
    // Platform.isIOS → _config.iosStoreUrl
    // Platform.isAndroid → _config.androidStoreUrl
  }

  @override
  Future<bool> openStore() async {
    final url = getStoreUrl();
    if (url == null) return false;
    return launchUrl(Uri.parse(url));
  }

  @override
  void dispose() {
    stopPeriodicCheck();
  }
}
```

**アップデートダイアログ UI ヘルパー**:

```dart
/// アップデートダイアログを表示するユーティリティ
class UpdateDialog {
  /// アップデートチェック結果に基づいてダイアログを表示する
  /// - [mandatory]: 閉じるボタンなし、ストアへの遷移のみ
  /// - [optional]: 「後で」ボタンあり、スキップ可能
  static Future<void> show({
    required BuildContext context,
    required UpdateCheckResult result,
    required AppUpdater updater,
    String? title,
    String? message,
  }) async {
    if (!result.needsUpdate) return;

    await showDialog(
      context: context,
      barrierDismissible: !result.isMandatory,
      builder: (context) => AlertDialog(
        title: Text(title ?? (result.isMandatory ? '更新が必要です' : '新しいバージョンがあります')),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(message ??
                'バージョン ${result.versionInfo.latestVersion} が利用可能です。'
                '（現在: ${result.currentVersion}）'),
            if (result.versionInfo.releaseNotes != null) ...[
              const SizedBox(height: 8),
              Text(result.versionInfo.releaseNotes!,
                  style: Theme.of(context).textTheme.bodySmall),
            ],
          ],
        ),
        actions: [
          if (!result.isMandatory)
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('後で'),
            ),
          TextButton(
            onPressed: () async {
              await updater.openStore();
            },
            child: const Text('更新する'),
          ),
        ],
      ),
    );
  }
}
```

**InMemoryAppUpdater（テスト用実装 -- 現在の主実装）**:

`InMemoryAppUpdater` はサーバー通信を行わず、コンストラクタで渡された `AppVersionInfo` を返すテスト用実装。`AppRegistryAppUpdater` の実装が完了するまでの主実装としても機能する。

```dart
class InMemoryAppUpdater implements AppUpdater {
  final AppVersionInfo _versionInfo;
  final String _currentVersion;
  Timer? _periodicTimer;

  InMemoryAppUpdater({
    required AppVersionInfo versionInfo,
    required String currentVersion,
  })  : _versionInfo = versionInfo,
        _currentVersion = currentVersion;

  @override
  Future<AppVersionInfo> fetchVersionInfo() async => _versionInfo;

  @override
  Future<UpdateCheckResult> checkForUpdate() async {
    // _currentVersion と _versionInfo を比較して UpdateCheckResult を返す
  }

  /// テスト補助 API: バージョン情報を更新する
  void setVersionInfo(AppVersionInfo info);

  /// テスト補助 API: 現在バージョンを更新する
  void setCurrentVersion(String version);

  // startPeriodicCheck / stopPeriodicCheck / getStoreUrl / openStore / dispose は実装済み
}
```

使用例:

```dart
import 'package:k1s0_app_updater/app_updater.dart';

// テスト用
final updater = InMemoryAppUpdater(
  versionInfo: AppVersionInfo(
    latestVersion: '2.1.0',
    minimumVersion: '2.0.0',
    releaseNotes: 'バグ修正とパフォーマンス改善',
  ),
  currentVersion: '1.9.0',
);

final result = await updater.checkForUpdate();
expect(result.type, UpdateType.mandatory);
expect(result.isMandatory, isTrue);
```

**MockAppUpdater（テスト用モック実装）**:

`MockAppUpdater` は `AppUpdater` abstract class を実装したモッククラス。各メソッドはコールバックで動作をオーバーライドできる。

```dart
import 'package:k1s0_app_updater/app_updater.dart';

final mock = MockAppUpdater();

// スタブ応答を設定
mock.onCheckForUpdate = () async => UpdateCheckResult(
  type: UpdateType.optional,
  currentVersion: '2.0.0',
  versionInfo: AppVersionInfo(
    latestVersion: '2.1.0',
    minimumVersion: '1.5.0',
    storeUrl: 'https://play.google.com/store/apps/details?id=com.example',
  ),
);

final result = await mock.checkForUpdate();
expect(result.type, UpdateType.optional);
// 呼び出し履歴の確認
expect(mock.calls, contains('checkForUpdate'));
```

**エラー型**:

```dart
class AppUpdaterError implements Exception {
  final String message;
  final String code;

  const AppUpdaterError(this.message, this.code);

  @override
  String toString() => 'AppUpdaterError($code): $message';
}

/// AppUpdaterError のエラーコード一覧:
///  - CONNECTION_ERROR   : app-registry-server への接続エラー
///  - INVALID_CONFIG     : 設定エラー（serverUrl/appId 未設定等）
///  - PARSE_ERROR        : サーバーレスポンスのパースエラー
///  - UNAUTHORIZED       : 認証エラー（HTTP 401/403）
///  - APP_NOT_FOUND      : 指定されたアプリが app-registry-server に存在しない
///  - VERSION_NOT_FOUND  : バージョン情報が取得できない
///  - STORE_URL_UNAVAILABLE : ストアURLが設定されていない
```

**カバレッジ目標**: 85%以上

## app-registry-server 連携

`AppRegistryAppUpdater` は以下の app-registry-server エンドポイントを利用する。

| メソッド | エンドポイント | 用途 |
|---------|--------------|------|
| GET | `/api/v1/apps/:id/latest?platform={platform}` | 最新バージョン情報の取得 |

### レスポンスマッピング

app-registry-server のレスポンスフィールドから `AppVersionInfo` への変換:

| サーバーレスポンス | AppVersionInfo フィールド | 説明 |
|------------------|-------------------------|------|
| `version` | `latestVersion` | 最新バージョン文字列 |
| `release_notes` | `releaseNotes` | リリースノート |
| `mandatory` | `mandatory` | 強制アップデートフラグ |
| `published_at` | `publishedAt` | 公開日時 |

`minimumVersion` は app-registry-server のバージョン一覧（`GET /api/v1/apps/:id/versions`）から `mandatory == true` の最新バージョンを取得して決定する。`storeUrl` は `AppUpdaterConfig` の `iosStoreUrl` / `androidStoreUrl` から取得する（サーバー側では管理しない）。

### 認証

app-registry-server は Bearer トークン + RBAC を要求する。`AppUpdaterConfig.tokenProvider` で認証トークンの取得方法を指定する。

```dart
final updater = AppRegistryAppUpdater(
  AppUpdaterConfig(
    serverUrl: 'https://app-registry.example.com',
    appId: 'com.example.myapp',
    platform: 'android',
    checkInterval: const Duration(hours: 1),
    androidStoreUrl: 'https://play.google.com/store/apps/details?id=com.example.myapp',
    tokenProvider: () async {
      // authlib 等で取得した Bearer トークンを返す
      return authClient.getAccessToken();
    },
  ),
);
```

## Flutter アプリケーションへの統合例

### アプリ起動時のチェック

```dart
class MyApp extends StatefulWidget {
  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  late final AppUpdater _updater;

  @override
  void initState() {
    super.initState();
    _updater = AppRegistryAppUpdater(
      AppUpdaterConfig(
        serverUrl: 'https://app-registry.example.com',
        appId: 'com.example.myapp',
        checkInterval: const Duration(hours: 6),
        iosStoreUrl: 'https://apps.apple.com/app/id123456789',
        androidStoreUrl: 'https://play.google.com/store/apps/details?id=com.example.myapp',
        tokenProvider: () => authService.getToken(),
      ),
    );
  }

  @override
  void dispose() {
    _updater.dispose();
    super.dispose();
  }

  Future<void> _checkUpdate(BuildContext context) async {
    try {
      final result = await _updater.checkForUpdate();
      if (result.needsUpdate && context.mounted) {
        await UpdateDialog.show(
          context: context,
          result: result,
          updater: _updater,
        );
      }
    } on AppUpdaterError catch (e) {
      // 接続エラー等はサイレントに処理（ユーザー操作をブロックしない）
      debugPrint('Update check failed: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Builder(
        builder: (context) {
          // 初回表示時にチェック
          WidgetsBinding.instance.addPostFrameCallback((_) {
            _checkUpdate(context);
          });
          return const HomeScreen();
        },
      ),
    );
  }
}
```

### バックグラウンド定期チェック

```dart
// アプリ起動後にバックグラウンドチェックを開始
_updater.startPeriodicCheck(
  onUpdateAvailable: (result) {
    // GlobalKey<NavigatorState> 等でダイアログを表示
    if (navigatorKey.currentContext != null) {
      UpdateDialog.show(
        context: navigatorKey.currentContext!,
        result: result,
        updater: _updater,
      );
    }
  },
);

// アプリ終了時やログアウト時に停止
_updater.stopPeriodicCheck();
```

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト | バージョン比較ロジック・UpdateType 判定・レスポンスパース | flutter_test |
| モックテスト | MockAppUpdater によるウィジェットテスト | flutter_test + MockAppUpdater |
| HTTP モックテスト | app-registry-server レスポンスのモック | http_mock_adapter / mockito |
| ウィジェットテスト | UpdateDialog の表示・操作確認 | flutter_test + WidgetTester |
| 統合テスト | 実際の app-registry-server との結合テスト | integration_test |

### ユニットテスト例

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  group('UpdateCheckResult', () {
    test('現在バージョンが最小バージョン未満の場合は mandatory', () async {
      final updater = InMemoryAppUpdater(
        versionInfo: AppVersionInfo(
          latestVersion: '3.0.0',
          minimumVersion: '2.0.0',
        ),
        currentVersion: '1.5.0',
      );

      final result = await updater.checkForUpdate();
      expect(result.type, UpdateType.mandatory);
      expect(result.isMandatory, isTrue);
    });

    test('現在バージョンが最新未満だが最小以上の場合は optional', () async {
      final updater = InMemoryAppUpdater(
        versionInfo: AppVersionInfo(
          latestVersion: '2.1.0',
          minimumVersion: '1.0.0',
        ),
        currentVersion: '2.0.0',
      );

      final result = await updater.checkForUpdate();
      expect(result.type, UpdateType.optional);
    });

    test('現在バージョンが最新以上の場合は none', () async {
      final updater = InMemoryAppUpdater(
        versionInfo: AppVersionInfo(
          latestVersion: '2.0.0',
          minimumVersion: '1.0.0',
        ),
        currentVersion: '2.0.0',
      );

      final result = await updater.checkForUpdate();
      expect(result.type, UpdateType.none);
      expect(result.needsUpdate, isFalse);
    });

    test('mandatory フラグが true の場合は mandatory', () async {
      final updater = InMemoryAppUpdater(
        versionInfo: AppVersionInfo(
          latestVersion: '2.1.0',
          minimumVersion: '1.0.0',
          mandatory: true,
        ),
        currentVersion: '2.0.0',
      );

      final result = await updater.checkForUpdate();
      expect(result.type, UpdateType.mandatory);
    });
  });
}
```

**カバレッジ目標**: 85%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) -- ライブラリ一覧・テスト方針
- [system-app-registry-server設計](../../servers/app-registry/server.md) -- アプリレジストリサーバー（バージョン情報取得元）
- [system-library-authlib設計](../auth-security/authlib.md) -- JWT 認証ライブラリ（tokenProvider で利用）
- [system-library-session-client設計](session-client.md) -- セッション管理クライアント
- [アプリ配布基盤設計](../../infrastructure/distribution/アプリ配布基盤設計.md) -- アプリ配布基盤の全体設計
