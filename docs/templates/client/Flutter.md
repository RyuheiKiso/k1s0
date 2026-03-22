# テンプレート仕様 — Flutter

本ドキュメントは、[テンプレート仕様-クライアント](クライアント.md) から分割された Flutter テンプレートの詳細仕様である。

## 概要

k1s0 CLI の `ひな形生成 → client → flutter` で使用するテンプレートファイル群を定義する。テンプレートエンジン **Tera** の構文でパラメータ化されており、CLI の対話フローで収集した情報をもとに実用的なプロジェクトスケルトンを生成する。

| フレームワーク | 言語       | 用途               | テンプレートパス                  |
| -------------- | ---------- | ------------------ | --------------------------------- |
| Flutter        | Dart       | Web / Mobile / Desktop | `CLI/templates/client/flutter/`   |

### 配置制約

- **system/client は共通 SDK として配置** — `system/client/flutter/system_client` は UI を持たない共有ライブラリパッケージであり、直接ユーザーに公開する画面アプリではない（[system-client設計](../../servers/_common/client.md) 参照）
- エンドユーザー向けのアプリ（画面）は **business** および **service** Tier のみに配置する

### 認証方式

クライアントは BFF（Backend for Frontend）経由の **HttpOnly Cookie** 方式で認証を行う（[認証認可設計](../../architecture/auth/認証認可設計.md) D-013 参照）。テンプレートの API クライアント設定は Cookie ベースの認証に対応する。

## 参照マップ

| テンプレートファイル                        | 参照ドキュメント                                  | 該当セクション                               |
| ------------------------------------------- | ------------------------------------------------- | -------------------------------------------- |
| `dio_client.dart`                           | [認証認可設計](../../architecture/auth/認証認可設計.md)                    | D-013 BFF + HttpOnly Cookie                  |
| `Dockerfile`                                | [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md)        | ベースイメージ一覧・マルチステージビルド      |
| `nginx.conf`                                | [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md)        | Flutter クライアント                         |
| `analysis_options.yaml`                     | [コーディング規約](../../architecture/conventions/コーディング規約.md)            | Dart ツール・設定                            |
| `pubspec.yaml`（変数展開）                  | [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) | テンプレート変数一覧・フィルタ               |
| `test/widget_test.dart`                      | [コーディング規約](../../architecture/conventions/コーディング規約.md)            | Widget テスト（flutter_test）                |
| `README.md`                                  | ---                                                | プロジェクト概要・セットアップ手順           |

---

## Tier 別配置パス

### business Tier

```
regions/business/{domain}/client/flutter/{service_name}/
```

例:

| domain       | service_name       | 配置パス                                                   |
| ------------ | ------------------ | ---------------------------------------------------------- |
| `fa`         | `asset-manager`   | `regions/business/fa/client/flutter/asset-manager/`        |

### service Tier

```
regions/service/{service_name}/client/flutter/
```

例:

| service_name | 配置パス                                      |
| ------------ | --------------------------------------------- |
| `board`      | `regions/service/board/client/flutter/`       |

---

## Flutter テンプレート

テンプレートファイルは `CLI/templates/client/flutter/` に配置する。以下に各ファイルの完全なスケルトンコードを示す。

### pubspec.yaml

`CLI/templates/client/flutter/pubspec.yaml.tera`

```yaml
name: {{ service_name_snake }}
description: {{ service_name }} client application
version: 0.1.0
publish_to: none

environment:
  sdk: ">=3.5.0 <4.0.0"
  flutter: ">=3.24.0"

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.6.0
  go_router: ^14.6.0
  freezed_annotation: ^2.4.0
  json_annotation: ^4.9.0
  dio: ^5.7.0
  yaml: ^3.1.2
  # システム共通クライアント SDK（認証・API クライアント・CSRF 管理）
  system_client:
    path: ../../../../../system/client/flutter/system_client

dev_dependencies:
  flutter_test:
    sdk: flutter
  build_runner: ^2.4.0
  freezed: ^2.5.0
  json_serializable: ^6.8.0
  mocktail: ^1.0.0
  flutter_lints: ^5.0.0

flutter:
  assets:
    - config/config.yaml
    - config/config.development.yaml
    - config/config.staging.yaml
    - config/config.production.yaml
```

### analysis_options.yaml

`CLI/templates/client/flutter/analysis_options.yaml.tera`

```yaml
include: package:flutter_lints/flutter.yaml

linter:
  rules:
    - prefer_const_constructors
    - prefer_const_declarations
    - avoid_print
    - prefer_single_quotes
    - always_declare_return_types
    - annotate_overrides
    - avoid_empty_else
    - prefer_final_fields
    - sort_constructors_first
    - unawaited_futures
    - unnecessary_brace_in_string_interps

analyzer:
  errors:
    missing_return: error
    dead_code: warning
  exclude:
    - "**/*.g.dart"
    - "**/*.freezed.dart"
```

### lib/main.dart

`CLI/templates/client/flutter/lib/main.dart.tera`

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:{{ service_name_snake }}/app/router.dart';
import 'package:{{ service_name_snake }}/config/app_config.dart';
import 'package:{{ service_name_snake }}/config/config_provider.dart';

/// アプリケーションのエントリポイント
/// YAML設定ファイルを読み込んでからProviderScopeにoverrideして起動する
void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  /// 環境変数APP_ENVから実行環境を取得する（デフォルトはdevelopment）
  const env = String.fromEnvironment('APP_ENV', defaultValue: 'development');

  /// YAML設定ファイルを読み込む
  final config = await AppConfig.load(env);

  runApp(
    ProviderScope(
      overrides: [appConfigProvider.overrideWithValue(config)],
      child: const MyApp(),
    ),
  );
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: '{{ service_name_pascal }}',
      theme: ThemeData(
        colorSchemeSeed: Colors.blue,
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorSchemeSeed: Colors.blue,
        useMaterial3: true,
        brightness: Brightness.dark,
      ),
      themeMode: ThemeMode.system,
      routerConfig: appRouter,
      debugShowCheckedModeBanner: false,
    );
  }
}
```

### lib/app/router.dart

`CLI/templates/client/flutter/lib/app/router.dart.tera`

```dart
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

/// アプリケーションルーター
///
/// go_router を使用してルーティングを定義する。
/// 新しい画面を追加する際は routes にルートを追加すること。
final GoRouter appRouter = GoRouter(
  initialLocation: '/',
  debugLogDiagnostics: true,
  routes: <RouteBase>[
    GoRoute(
      path: '/',
      name: 'home',
      builder: (BuildContext context, GoRouterState state) {
        return const HomeScreen();
      },
    ),
    // TODO: {{ service_name }} 固有のルートを追加
  ],
  errorBuilder: (BuildContext context, GoRouterState state) {
    return const NotFoundScreen();
  },
);

/// ホーム画面（スケルトン）
class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('{{ service_name_pascal }}'),
      ),
      body: const Center(
        child: Text('Welcome to {{ service_name_pascal }}'),
      ),
    );
  }
}

/// 404 画面
class NotFoundScreen extends StatelessWidget {
  const NotFoundScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Not Found'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text('ページが見つかりません'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () => context.go('/'),
              child: const Text('ホームに戻る'),
            ),
          ],
        ),
      ),
    );
  }
}
```

### lib/utils/dio_client.dart

`CLI/templates/client/flutter/lib/utils/dio_client.dart.tera`

> **重要:** CSRF トークンをレスポンスヘッダー（`x-csrf-token`）から取得してはならない。
> BFF は CSRF トークンを `/auth/session` の JSON ボディで返す。
> `system_client` の `ApiClient.create()` と `authProvider` 経由で正しく実装すること。

```dart
// system_client の ApiClient を使用して BFF の CSRF 契約に準拠する
// 旧 DioClient は x-csrf-token レスポンスヘッダーを読んでいたが誤りだった
// 本ファイルは後方互換のために DioClient.create を ApiClient.create へ委譲する
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

/// DioClient は非推奨。直接 ApiClient.create() を使用すること。
/// 既存コードとの互換性のために残しているが、内部は ApiClient.create() に委譲する。
class DioClient {
  /// 指定されたベースURLでDioインスタンスを生成する
  /// system_client の ApiClient.create() に委譲し、CSRF 契約を正しく実装する
  static Dio create({required String baseUrl, CsrfTokenProvider? csrfTokenProvider}) {
    return ApiClient.create(
      baseUrl: baseUrl,
      csrfTokenProvider: csrfTokenProvider,
    );
  }
}
```

Provider では `authProvider.notifier.csrfToken` を `csrfTokenProvider` に渡す:

```dart
final dioProvider = Provider<Dio>((ref) {
  final config = ref.watch(appConfigProvider);
  final authNotifier = ref.read(authProvider.notifier);
  return ApiClient.create(
    baseUrl: config.api.baseUrl,
    csrfTokenProvider: () async => authNotifier.csrfToken,
    sessionCookieInterceptor: kIsWeb ? null : SessionCookieInterceptor(),
  );
});
```

### test/widget_test.dart

`CLI/templates/client/flutter/test/widget_test.dart.tera`

Widget のスモークテスト。Flutter の慣習に従い `test/` ディレクトリに配置する。

```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:{{ service_name_snake }}/main.dart';

void main() {
  testWidgets('MyApp renders without crashing', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    expect(find.text('{{ service_name_pascal }}'), findsOneWidget);
  });

  testWidgets('HomeScreen displays welcome message', (WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: Center(
            child: Text('Welcome to {{ service_name_pascal }}'),
          ),
        ),
      ),
    );

    expect(find.text('Welcome to {{ service_name_pascal }}'), findsOneWidget);
  });
}
```

### Dockerfile

`CLI/templates/client/flutter/Dockerfile.tera`

```dockerfile
# ---- Build Stage ----
FROM ghcr.io/cirruslabs/flutter:3.24.0 AS build
WORKDIR /app

COPY pubspec.* ./
RUN flutter pub get

COPY . .
RUN flutter build web --release

# ---- Runtime Stage ----
FROM nginx:1.27-alpine
COPY --from=build /app/build/web /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

# nginx のデフォルトユーザーは root のため、非 root 実行に切り替える。
# helm設計.md の securityContext との整合については React クライアントと同様。
USER nginx
EXPOSE 8080
```

### nginx.conf

`CLI/templates/client/flutter/nginx.conf.tera`

Flutter クライアントは React と同じ nginx.conf を使用する。SPA ルーティング・gzip 圧縮・セキュリティヘッダーの設定は共通であるため、[テンプレート仕様-React](React.md) の nginx.conf をそのまま適用する。

### README.md

`CLI/templates/client/flutter/README.md.tera`

```markdown
# {{ service_name }}

{{ service_name_pascal }} クライアント（Flutter）。

## セットアップ

```bash
# 依存インストール
flutter pub get

# 開発サーバー起動（Web）
flutter run -d chrome

# テスト実行
flutter test

# ビルド（Web）
flutter build web --release
```

## ディレクトリ構成

```
.
├── lib/
│   ├── main.dart         # エントリポイント
│   ├── app/              # ルーティング
│   └── utils/            # API クライアント・ユーティリティ
├── test/                 # テスト
├── pubspec.yaml
├── analysis_options.yaml
├── Dockerfile
└── README.md
```

## 開発

- **状態管理**: Riverpod
- **ルーティング**: go_router
- **HTTP クライアント**: Dio
- **テスト**: flutter_test + mocktail
```

### lib/providers/service_provider.dart

`CLI/templates/client/flutter/lib/providers/service_provider.dart.tera`

Riverpod の AsyncNotifierProvider パターンを使用したサービスプロバイダー。fetchAll / create / delete の基本操作を提供する。

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:{{ service_name_snake }}/utils/dio_client.dart';

/// {{ service_name_pascal }} のデータモデル
class {{ service_name_pascal }}Item {
  final String id;
  final String name;
  final String? description;
  final String status;

  const {{ service_name_pascal }}Item({
    required this.id,
    required this.name,
    this.description,
    required this.status,
  });

  factory {{ service_name_pascal }}Item.fromJson(Map<String, dynamic> json) {
    return {{ service_name_pascal }}Item(
      id: json['id'] as String,
      name: json['name'] as String,
      description: json['description'] as String?,
      status: json['status'] as String,
    );
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'description': description,
    'status': status,
  };
}

/// {{ service_name_pascal }} 一覧の AsyncNotifierProvider
final {{ service_name_snake }}ListProvider =
    AsyncNotifierProvider<{{ service_name_pascal }}ListNotifier, List<{{ service_name_pascal }}Item>>(
  {{ service_name_pascal }}ListNotifier.new,
);

class {{ service_name_pascal }}ListNotifier extends AsyncNotifier<List<{{ service_name_pascal }}Item>> {
  @override
  Future<List<{{ service_name_pascal }}Item>> build() async {
    return fetchAll();
  }

  /// 全件取得
  Future<List<{{ service_name_pascal }}Item>> fetchAll() async {
    final response = await DioClient.instance.get<List<dynamic>>('/api/v1/{{ service_name }}');
    return (response.data ?? [])
        .map((json) => {{ service_name_pascal }}Item.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 新規作成
  Future<void> create({
    required String name,
    String? description,
  }) async {
    await DioClient.instance.post<Map<String, dynamic>>(
      '/api/v1/{{ service_name }}',
      data: {
        'name': name,
        'description': description,
      },
    );
    // 一覧を再取得して状態を更新
    ref.invalidateSelf();
  }

  /// 削除
  Future<void> delete(String id) async {
    await DioClient.instance.delete<void>('/api/v1/{{ service_name }}/$id');
    // 一覧を再取得して状態を更新
    ref.invalidateSelf();
  }
}
```

| パターン | 説明 |
|---|---|
| `AsyncNotifierProvider` | 非同期データの状態管理。`build()` で初期データ取得 |
| `fetchAll()` | 一覧取得。Dio クライアント経由で BFF/API を呼び出す |
| `create()` | 新規作成後、`ref.invalidateSelf()` で一覧を再取得 |
| `delete()` | 削除後、`ref.invalidateSelf()` で一覧を再取得 |

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [テンプレート仕様-React](React.md) --- React テンプレート
- [テンプレート仕様-サーバー](../server/サーバー.md) --- サーバーテンプレート
- [テンプレート仕様-ライブラリ](../codegen/ライブラリ.md) --- ライブラリテンプレート
- [テンプレート仕様-データベース](../data/データベース.md) --- データベーステンプレート
- [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) --- Docker ビルド戦略
- [認証認可設計](../../architecture/auth/認証認可設計.md) --- BFF + Cookie 認証
- [アプリ配布基盤設計](../../infrastructure/distribution/アプリ配布基盤設計.md) --- デスクトップアプリの配布・自動更新
