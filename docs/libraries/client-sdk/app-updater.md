# k1s0-app-updater ライブラリ設計

## 概要

アプリ更新管理クライアントライブラリ。system-app-registry-server（REST API）と連携し、アプリバージョン確認・強制アップデート判定・SHA-256 チェックサム検証を統一インターフェースで提供する。Go / Rust / TypeScript はバージョンチェックとチェックサム検証に特化し、Dart はさらに定期チェック・ストアURL管理・アップデートダイアログ UI を含む。

**配置先**: `regions/system/library/{go,rust,typescript,dart}/app-updater/`（Dart は `app_updater`）

## 公開 API

### Go

| 型 | 種別 | 説明 |
|----|------|------|
| `AppUpdater` | interface | バージョン取得・アップデートチェックインターフェース |
| `AppRegistryAppUpdater` | struct | app-registry-server 連携実装 |
| `InMemoryAppUpdater` | struct | テスト用インメモリ実装 |
| `AppUpdaterConfig` | struct | ServerURL・AppID・Platform・Arch・Timeout 等の設定 |
| `AppVersionInfo` | struct | LatestVersion・MinimumVersion・Mandatory・ReleaseNotes・PublishedAt |
| `UpdateCheckResult` | struct | CurrentVersion・LatestVersion・MinimumVersion・UpdateType・ReleaseNotes |
| `UpdateType` | const iota | `None`・`Optional`・`Mandatory` |
| `DownloadArtifactInfo` | struct | URL・Checksum・Size・ExpiresAt |
| `AppUpdaterError` | struct | Code・Message を持つ基本エラー型 |
| `ConnectionError` | struct | 接続エラー（`CONNECTION_ERROR`） |
| `InvalidConfigError` | struct | 設定エラー（`INVALID_CONFIG`） |
| `ParseError` | struct | パースエラー（`PARSE_ERROR`） |
| `UnauthorizedError` | struct | 認証エラー（`UNAUTHORIZED`） |
| `AppNotFoundError` | struct | アプリ未検出エラー（`APP_NOT_FOUND`） |
| `VersionNotFoundError` | struct | バージョン未検出エラー（`VERSION_NOT_FOUND`） |
| `ChecksumError` | struct | チェックサムエラー（`CHECKSUM_ERROR`） |
| `CompareVersions` | func | バージョン文字列比較（セマンティック） |
| `DetermineUpdateType` | func | アップデート種別判定 |
| `CalculateChecksum` | func | SHA-256 チェックサム計算 |
| `VerifyChecksum` | func | チェックサム検証（bool 返却） |
| `VerifyChecksumOrError` | func | チェックサム検証（エラー返却） |

### Rust

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `AppUpdater` | トレイト | バージョン取得・アップデートチェックインターフェース |
| `AppRegistryAppUpdater` | 構造体 | app-registry-server 連携実装 |
| `InMemoryAppUpdater` | 構造体 | テスト用インメモリ実装（`Arc<RwLock<>>`） |
| `AppUpdaterConfig` | 構造体 | server_url・app_id・platform・arch・timeout 等の設定 |
| `AppVersionInfo` | 構造体 | latest_version・minimum_version・mandatory・release_notes・published_at |
| `UpdateCheckResult` | 構造体 | current_version・latest_version・minimum_version・update_type・release_notes |
| `UpdateType` | enum | `None`・`Optional`・`Mandatory` |
| `DownloadArtifactInfo` | 構造体 | url・checksum・size・expires_at |
| `AppUpdaterError` | enum | `Connection`・`InvalidConfig`・`Parse`・`Unauthorized`・`AppNotFound`・`VersionNotFound`・`Checksum`・`Io` |
| `ChecksumVerifier` | 構造体 | SHA-256 チェックサム計算・検証 |
| `compare_versions` | 関数 | バージョン文字列比較（`Ordering` 返却） |
| `determine_update_type` | 関数 | アップデート種別判定 |

### TypeScript

| 型 | 種別 | 説明 |
|----|------|------|
| `AppUpdater` | interface | バージョン取得・アップデートチェックインターフェース |
| `AppRegistryAppUpdater` | class | app-registry-server 連携実装 |
| `InMemoryAppUpdater` | class | テスト用インメモリ実装 |
| `AppUpdaterConfig` | interface | serverUrl・appId・platform・arch・timeout 等の設定 |
| `AppVersionInfo` | interface | latestVersion・minimumVersion・mandatory・releaseNotes・publishedAt |
| `UpdateCheckResult` | interface | currentVersion・latestVersion・minimumVersion・updateType・releaseNotes |
| `UpdateType` | type | `'none'` \| `'optional'` \| `'mandatory'` |
| `DownloadArtifactInfo` | interface | url・checksum・size・expiresAt |
| `AppUpdaterError` | class | code プロパティ付き基本エラークラス |
| `ConnectionError` | class | 接続エラー（`CONNECTION_ERROR`） |
| `InvalidConfigError` | class | 設定エラー（`INVALID_CONFIG`） |
| `ParseError` | class | パースエラー（`PARSE_ERROR`） |
| `UnauthorizedError` | class | 認証エラー（`UNAUTHORIZED`） |
| `AppNotFoundError` | class | アプリ未検出エラー（`APP_NOT_FOUND`） |
| `VersionNotFoundError` | class | バージョン未検出エラー（`VERSION_NOT_FOUND`） |
| `ChecksumError` | class | チェックサムエラー（`CHECKSUM_ERROR`） |
| `ChecksumVerifier` | class | SHA-256 チェックサム計算・検証（static メソッド） |
| `compareVersions` | function | バージョン文字列比較 |
| `determineUpdateType` | function | アップデート種別判定 |

### Dart

| 型 | 種別 | 説明 |
|----|------|------|
| `AppUpdater` | abstract class | アプリ更新管理の抽象インターフェース（定期チェック・ストア連携含む） |
| `AppRegistryAppUpdater` | class | app-registry-server 連携実装 |
| `InMemoryAppUpdater` | class | テスト用インメモリ実装（現在の主実装） |
| `MockAppUpdater` | class | テスト用モック実装 |
| `AppUpdaterConfig` | class | サーバーURL・アプリID・チェック間隔等の設定 |
| `AppVersionInfo` | class | 最新バージョン・最小サポートバージョン・リリースノート・ストアURL |
| `UpdateCheckResult` | class | チェック結果（更新要否・強制/任意・バージョン情報） |
| `UpdateType` | enum | `none`・`optional`・`mandatory` |
| `AppUpdaterError` | class | 接続エラー・設定エラー・パースエラー等 |
| `ChecksumVerifier` | class | SHA-256 チェックサム計算・検証 |
| `UpdateDialog` | class | アップデートダイアログ UI ヘルパー（Dart 固有） |

## Counts

| 言語 | 公開関数/メソッド | 公開型 | エラー型/定数 |
|------|-----------------|--------|-------------|
| Go | 7 | 8 | 7 |
| Rust | 6 | 7 | 8 |
| TypeScript | 6 | 7 | 7 |
| Dart | 10 | 8 | 9 |
| **合計** | **29** | **30** | **31** |

## Go 実装

**配置先**: `regions/system/library/go/app-updater/`

**go.mod**:

```
module github.com/k1s0-platform/system-library-go-app-updater
go 1.23.0
```

**モジュール構成**:

```
app-updater/
├── app_updater.go          # AppUpdater interface・AppRegistryAppUpdater・InMemoryAppUpdater・models・errors
├── checksum_verifier.go    # CalculateChecksum・VerifyChecksum・VerifyChecksumOrError
├── app_updater_test.go
├── checksum_verifier_test.go
└── go.mod
```

**インターフェース**:

```go
type AppUpdater interface {
    FetchVersionInfo(ctx context.Context) (*AppVersionInfo, error)
    CheckForUpdate(ctx context.Context) (*UpdateCheckResult, error)
}
```

> **注記（Dart との差異）**: Go の `AppUpdater` は `FetchVersionInfo` と `CheckForUpdate` のみ。Dart の `startPeriodicCheck` / `stopPeriodicCheck` / `getStoreUrl` / `openStore` / `dispose` はクライアント UI 固有のためバックエンド向け Go 実装には含まない。

**使用例**:

```go
import appupdater "github.com/k1s0-platform/system-library-go-app-updater"

// テスト用
updater := appupdater.NewInMemoryAppUpdater(
    &appupdater.AppVersionInfo{
        LatestVersion:  "2.1.0",
        MinimumVersion: "2.0.0",
        ReleaseNotes:   "バグ修正とパフォーマンス改善",
    },
    "1.9.0",
)

result, err := updater.CheckForUpdate(ctx)
// result.UpdateType == appupdater.Mandatory
// result.IsMandatory() == true
```

## Rust 実装

**配置先**: `regions/system/library/rust/app-updater/`

**Cargo.toml**:

```toml
[package]
name = "k1s0-app-updater"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
reqwest = { version = "0.12", features = ["json"] }
sha2 = "0.10"
tokio = { version = "1", features = ["fs"] }
chrono = { version = "0.4", features = ["serde"] }
```

**依存追加**: `k1s0-app-updater = { path = "../../system/library/rust/app-updater" }`

**モジュール構成**:

```
app-updater/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # AppUpdater トレイト・AppRegistryAppUpdater・InMemoryAppUpdater
│   ├── model.rs        # UpdateType・AppVersionInfo・UpdateCheckResult・DownloadArtifactInfo
│   ├── error.rs        # AppUpdaterError
│   ├── config.rs       # AppUpdaterConfig
│   ├── checksum.rs     # ChecksumVerifier
│   └── version.rs      # compare_versions・determine_update_type
├── tests/
│   └── app_updater_test.rs
└── Cargo.toml
```

**トレイト**:

```rust
#[async_trait]
pub trait AppUpdater: Send + Sync {
    async fn fetch_version_info(&self) -> Result<AppVersionInfo, AppUpdaterError>;
    async fn check_for_update(&self) -> Result<UpdateCheckResult, AppUpdaterError>;
}
```

**エラー型**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppUpdaterError {
    #[error("connection error: {0}")]
    Connection(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("app not found: {0}")]
    AppNotFound(String),
    #[error("version not found: {0}")]
    VersionNotFound(String),
    #[error("checksum mismatch: {0}")]
    Checksum(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

**使用例**:

```rust
use k1s0_app_updater::{
    AppVersionInfo, InMemoryAppUpdater, AppUpdater, UpdateType,
};

let updater = InMemoryAppUpdater::new(
    AppVersionInfo {
        latest_version: "2.1.0".to_string(),
        minimum_version: "2.0.0".to_string(),
        mandatory: false,
        release_notes: Some("バグ修正とパフォーマンス改善".to_string()),
        published_at: None,
    },
    "1.9.0".to_string(),
);

let result = updater.check_for_update().await?;
assert_eq!(result.update_type, UpdateType::Mandatory);
assert!(result.is_mandatory());
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/app-updater/`

**package.json**:

```json
{
  "name": "@k1s0/app-updater",
  "version": "0.1.0",
  "type": "module"
}
```

**モジュール構成**:

```
app-updater/
├── src/
│   ├── index.ts        # 公開 API（再エクスポート）
│   ├── client.ts       # AppUpdater interface・AppRegistryAppUpdater・InMemoryAppUpdater
│   ├── types.ts        # UpdateType・AppVersionInfo・UpdateCheckResult・DownloadArtifactInfo
│   ├── error.ts        # AppUpdaterError + サブクラス
│   ├── config.ts       # AppUpdaterConfig interface
│   ├── checksum.ts     # ChecksumVerifier（static メソッド）
│   └── version.ts      # compareVersions・determineUpdateType
├── __tests__/
│   ├── client.test.ts
│   └── checksum.test.ts
├── package.json
└── tsconfig.json
```

**インターフェース**:

```typescript
export interface AppUpdater {
  fetchVersionInfo(): Promise<AppVersionInfo>;
  checkForUpdate(): Promise<UpdateCheckResult>;
}
```

**使用例**:

```typescript
import { InMemoryAppUpdater } from '@k1s0/app-updater';

const updater = new InMemoryAppUpdater(
  {
    latestVersion: '2.1.0',
    minimumVersion: '2.0.0',
    mandatory: false,
    releaseNotes: 'バグ修正とパフォーマンス改善',
  },
  '1.9.0',
);

const result = await updater.checkForUpdate();
// result.updateType === 'mandatory'
```

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
│       ├── checksum_verifier.dart # ChecksumVerifier
│       ├── dialog.dart           # アップデートダイアログ UI ヘルパー
│       └── error.dart            # AppUpdaterError
├── test/
│   ├── app_updater_test.dart
│   ├── checksum_verifier_test.dart
│   ├── registry_api_client_test.dart
│   ├── platform_detector_test.dart
│   └── dialog_test.dart
└── pubspec.yaml
```

**抽象インターフェース**:

```dart
abstract class AppUpdater {
  Future<AppVersionInfo> fetchVersionInfo();
  Future<UpdateCheckResult> checkForUpdate();
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  });
  void stopPeriodicCheck();
  String? getStoreUrl();
  Future<bool> openStore();
  void dispose();
}
```

> **注記（Dart 固有 API）**: `startPeriodicCheck` / `stopPeriodicCheck` / `getStoreUrl` / `openStore` / `dispose` / `UpdateDialog` は Flutter クライアント UI に特化した Dart 固有の API。Go / Rust / TypeScript にはこれらに対応するメソッドはない。

**データモデル**:

```dart
class AppVersionInfo {
  final String latestVersion;
  final String minimumVersion;
  final String? releaseNotes;
  final bool mandatory;
  final String? storeUrl;
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

enum UpdateType { none, optional, mandatory }

class UpdateCheckResult {
  final UpdateType type;
  final String currentVersion;
  final AppVersionInfo versionInfo;

  bool get needsUpdate => type != UpdateType.none;
  bool get isMandatory => type == UpdateType.mandatory;
}
```

**設定**:

```dart
class AppUpdaterConfig {
  final String serverUrl;
  final String appId;
  final String? platform;
  final Duration? checkInterval;
  final String? iosStoreUrl;
  final String? androidStoreUrl;
  final Duration timeout;
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

**エラー型**:

```dart
class AppUpdaterError implements Exception {
  final String message;
  final String code;

  const AppUpdaterError(this.message, this.code);
}

/// エラーコード一覧:
///  - CONNECTION_ERROR   : app-registry-server への接続エラー
///  - INVALID_CONFIG     : 設定エラー（serverUrl/appId 未設定等）
///  - PARSE_ERROR        : サーバーレスポンスのパースエラー
///  - UNAUTHORIZED       : 認証エラー（HTTP 401/403）
///  - APP_NOT_FOUND      : 指定されたアプリが app-registry-server に存在しない
///  - VERSION_NOT_FOUND  : バージョン情報が取得できない
///  - STORE_URL_UNAVAILABLE : ストアURLが設定されていない
///  - CHECKSUM_ERROR     : チェックサム不一致
```

**使用例**:

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

## 言語間の差異

| 機能 | Go | Rust | TypeScript | Dart |
|------|:--:|:----:|:----------:|:----:|
| バージョンチェック | ○ | ○ | ○ | ○ |
| チェックサム検証 | ○ | ○ | ○ | ○ |
| 定期バックグラウンドチェック | - | - | - | ○ |
| ストアURL管理 | - | - | - | ○ |
| アップデートダイアログ UI | - | - | - | ○ |
| モック実装 | - | - | - | ○ |

> Go / Rust / TypeScript はサーバーサイド・CLI 向けのバージョンチェック + チェックサム検証に特化。Dart は Flutter デスクトップクライアント向けの完全なアプリ更新管理を提供。

## バージョン比較アルゴリズム

全言語で同一のセマンティックバージョン比較ロジックを実装:

1. バージョン文字列を `.` で分割
2. 各セグメントから非数字文字を除去（例: `"1.2.0-rc1"` → `[1, 2, 0]`）
3. 短い方をゼロで右パディング
4. 左から順にセグメントを比較

**アップデート種別判定**:

| 条件 | UpdateType |
|------|-----------|
| currentVersion < minimumVersion **または** mandatory フラグが true | `mandatory` |
| currentVersion < latestVersion | `optional` |
| それ以外 | `none` |

## ChecksumVerifier

全言語で SHA-256 によるファイルチェックサム検証を提供:

| メソッド | 説明 |
|---------|------|
| `calculate` / `CalculateChecksum` | ファイルの SHA-256 ハッシュを計算（小文字 hex 文字列） |
| `verify` / `VerifyChecksum` | 計算値と期待値を比較（大文字小文字無視、bool 返却） |
| `verify_or_error` / `VerifyChecksumOrError` / `verifyOrThrow` | 不一致時にエラーを返す |

## app-registry-server 連携

`AppRegistryAppUpdater` は以下の app-registry-server エンドポイントを利用する。

| メソッド | エンドポイント | 用途 |
|---------|--------------|------|
| GET | `/api/v1/apps/:id/versions/latest?platform={platform}&arch={arch}` | 最新バージョン情報の取得 |

### レスポンスマッピング

app-registry-server のレスポンスフィールドから `AppVersionInfo` への変換:

| サーバーレスポンス | AppVersionInfo フィールド | 説明 |
|------------------|-------------------------|------|
| `version` / `latest_version` | `latestVersion` | 最新バージョン文字列 |
| `release_notes` | `releaseNotes` | リリースノート |
| `mandatory` | `mandatory` | 強制アップデートフラグ |
| `published_at` | `publishedAt` | 公開日時 |

### 認証

app-registry-server は Bearer トークン + RBAC を要求する。Dart では `AppUpdaterConfig.tokenProvider` で認証トークンの取得方法を指定する。

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
      debugPrint('Update check failed: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Builder(
        builder: (context) {
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
_updater.startPeriodicCheck(
  onUpdateAvailable: (result) {
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
| ユニットテスト | バージョン比較ロジック・UpdateType 判定・チェックサム検証 | Go: testify / Rust: tokio::test / TS: vitest / Dart: flutter_test |
| InMemory テスト | InMemoryAppUpdater の動作検証 | 各言語のテストフレームワーク |
| HTTP モックテスト | app-registry-server レスポンスのモック | Dart: http_mock_adapter |
| ウィジェットテスト | UpdateDialog の表示・操作確認（Dart のみ） | flutter_test + WidgetTester |

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) -- ライブラリ一覧・テスト方針
- [system-app-registry-server設計](../../servers/app-registry/server.md) -- アプリレジストリサーバー（バージョン情報取得元）
- [system-library-authlib設計](../auth-security/authlib.md) -- JWT 認証ライブラリ（tokenProvider で利用）
- [system-library-session-client設計](session-client.md) -- セッション管理クライアント
- [アプリ配布基盤設計](../../infrastructure/distribution/アプリ配布基盤設計.md) -- アプリ配布基盤の全体設計
