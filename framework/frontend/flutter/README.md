# Framework Frontend Flutter

Flutter 共通パッケージ。

## ディレクトリ構成

```
flutter/
├── melos.yaml
├── analysis_options.yaml
├── pubspec.yaml
└── packages/
    ├── k1s0_config/         # YAML設定管理
    ├── k1s0_http/           # API通信クライアント
    ├── k1s0_auth/           # 認証クライアント
    ├── k1s0_observability/  # OTel/ログ/メトリクス
    ├── k1s0_ui/             # Design System
    └── k1s0_state/          # 状態管理
```

## パッケージ一覧

| パッケージ | 説明 |
|-----------|------|
| k1s0_config | YAML設定管理、Zodスキーマバリデーション、環境マージ |
| k1s0_http | Dioベース通信クライアント、OTel計測、ProblemDetails対応 |
| k1s0_auth | JWT/OIDC認証、SecureStorage、トークン自動更新 |
| k1s0_observability | 構造化ログ、分散トレース、メトリクス収集 |
| k1s0_ui | Material 3 Design System、共通ウィジェット、テーマ |
| k1s0_state | Riverpod状態管理、AsyncValueヘルパー、永続化 |

## セットアップ

### 前提条件

- Flutter 3.0+
- Dart 3.0+
- melos 3.0+

### インストール

```bash
# melos のインストール
dart pub global activate melos

# プロジェクトのブートストラップ
cd framework/frontend/flutter
melos bootstrap
```

## 使用方法

### アプリケーションへの追加

`pubspec.yaml` にパッケージを追加:

```yaml
dependencies:
  k1s0_config:
    path: ../framework/frontend/flutter/packages/k1s0_config
  k1s0_http:
    path: ../framework/frontend/flutter/packages/k1s0_http
  k1s0_auth:
    path: ../framework/frontend/flutter/packages/k1s0_auth
  k1s0_observability:
    path: ../framework/frontend/flutter/packages/k1s0_observability
  k1s0_ui:
    path: ../framework/frontend/flutter/packages/k1s0_ui
  k1s0_state:
    path: ../framework/frontend/flutter/packages/k1s0_state
```

### 基本的な使い方

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:k1s0_config/k1s0_config.dart';
import 'package:k1s0_auth/k1s0_auth.dart';
import 'package:k1s0_ui/k1s0_ui.dart';
import 'package:k1s0_state/k1s0_state.dart';
import 'package:k1s0_observability/k1s0_observability.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // 設定の読み込み
  final configLoader = ConfigLoader(
    defaultPath: 'assets/config/default.yaml',
    environment: 'dev',
  );
  final config = await configLoader.load();

  runApp(
    K1s0StateProvider(
      enableLogging: true,
      child: ConfigScope(
        config: config,
        child: const MyApp(),
      ),
    ),
  );
}

class MyApp extends ConsumerWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);

    return MaterialApp(
      theme: themeState.lightTheme,
      darkTheme: themeState.darkTheme,
      themeMode: themeState.themeMode,
      home: const AuthGuard(
        child: HomePage(),
        unauthenticatedBuilder: (context) => const LoginPage(),
      ),
    );
  }
}
```

## 開発コマンド

```bash
# 全パッケージの静的解析
melos run analyze

# 全パッケージのフォーマット
melos run format

# 全パッケージのテスト実行
melos run test

# コード生成（freezed, json_serializable）
melos run build:runner

# クリーンアップ
melos run clean
```

## パッケージ詳細

### k1s0_config

YAML設定ファイルの読み込み、バリデーション、環境別マージを提供。

```dart
// 設定の読み込み
final loader = ConfigLoader(
  defaultPath: 'assets/config/default.yaml',
  environment: 'production',
);
final config = await loader.load();

// 型安全なアクセス
print(config.api.baseUrl);
print(config.auth.clientId);
```

### k1s0_http

Dioベースの HTTP クライアント。トレース伝播、エラーハンドリング、リトライを提供。

```dart
final client = K1s0HttpClient(
  config: HttpClientConfig(
    baseUrl: 'https://api.example.com',
    timeout: Duration(seconds: 30),
  ),
);

final response = await client.get<User>('/users/123');
```

### k1s0_auth

JWT/OIDC 認証、トークン管理、認証ガードを提供。

```dart
// AuthProvider で認証状態を管理
final authState = ref.watch(authProvider);

// 認証が必要な画面をガード
AuthGuard(
  child: DashboardPage(),
  unauthenticatedBuilder: (context) => LoginPage(),
)

// ロールベースの認可
RequireRole(
  roles: ['admin'],
  child: AdminPanel(),
  fallback: AccessDenied(),
)
```

### k1s0_observability

構造化ログ、分散トレース、パフォーマンスメトリクスを提供。

```dart
final logger = ref.read(loggerProvider);
logger.info('ユーザーがログインしました', {
  'userId': user.id,
  'loginMethod': 'oauth',
});

final tracer = ref.read(tracerProvider);
await tracer.trace('fetch-user-data', () async {
  return await api.getUser(userId);
});
```

### k1s0_ui

Material 3 ベースの Design System。テーマ、共通ウィジェット、フォームを提供。

```dart
// ボタン
K1s0PrimaryButton(
  onPressed: () {},
  child: Text('Submit'),
)

// テキストフィールド
K1s0TextField(
  controller: controller,
  label: 'Email',
  validator: K1s0Validators.email,
)

// フィードバック
K1s0Snackbar.success(context, 'Operation completed!');
```

### k1s0_state

Riverpod 状態管理ユーティリティ。AsyncValue ヘルパー、永続化、グローバル状態を提供。

```dart
// AsyncValue の便利な拡張
final items = ref.watch(itemsProvider);
items.when2(
  data: (data) => ListView(...),
  loading: () => LoadingWidget(),
  error: (e, s) => ErrorWidget(e),
  refreshing: (data) => RefreshingWidget(data),
);

// 状態の永続化
final storage = await PreferencesStorage.create();
ref.read(userPreferencesProvider.notifier).initialize(storage);
```

## 依存関係

```
k1s0_config
  └── (standalone, yaml依存)

k1s0_http
  └── dio, k1s0_observability(optional)

k1s0_auth
  ├── flutter_secure_storage
  ├── jwt_decoder
  └── go_router(optional)

k1s0_observability
  └── (standalone)

k1s0_ui
  └── flutter_riverpod

k1s0_state
  ├── flutter_riverpod
  ├── shared_preferences
  └── hive_flutter
```

## ライセンス

MIT
