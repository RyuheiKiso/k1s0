# system/client 設計

## 位置づけ

`system/client` は UI を持たない **共通 SDK パッケージ** である。
エンドユーザーに公開する画面アプリケーションではなく、business/service 層の client アプリが
共通して必要とする認証・API クライアント・共通コンポーネント・ルーティングガードを提供する。

> **「アプリ」ではなく「ライブラリ」** — system/client を直接デプロイするのではなく、
> business/service client が依存パッケージとして import して使用する。

## 提供機能一覧

| 機能カテゴリ        | Flutter（system_client）                       | React（system-client）                        |
| ------------------- | ---------------------------------------------- | --------------------------------------------- |
| 認証状態管理        | `AuthState` (freezed) + Riverpod Provider      | `AuthContext` + `AuthProvider` + `useAuth`    |
| API クライアント    | Dio factory（Cookie / CSRF ヘッダー設定済み）  | Axios factory（withCredentials / CSRF 対応）  |
| ルーティングガード  | GoRouter の redirect（未認証 → /login）        | `ProtectedRoute`（未認証 → /login）           |
| 共通 Widget/Component | `AppButton`, `AppScaffold`, `LoadingIndicator` | `AppButton`, `LoadingSpinner`, `ErrorBoundary` |
| Config 管理         | `ConfigInterpreter` + `ConfigEditorPage`       | `ConfigInterpreter` + `ConfigEditorPage` + `useConfigEditor` |
| Navigation          | `NavigationInterpreter`（GoRouter 生成）       | `NavigationInterpreter`（ResolvedRoute 生成） + `NavigationDevTools` |

## Flutter パッケージ設計（system_client）

### パッケージ名

`system_client`

### 配置パス

`regions/system/client/flutter/system_client/`

### ディレクトリ構成

```
system_client/
├── pubspec.yaml
├── analysis_options.yaml
├── lib/
│   ├── system_client.dart           # barrel export
│   └── src/
│       ├── auth/
│       │   ├── auth_state.dart       # freezed: AuthState（unauthenticated / authenticated）
│       │   └── auth_provider.dart    # Riverpod: authProvider, loginProvider, logoutProvider
│       ├── http/
│       │   └── api_client.dart       # Dio factory（baseUrl, Cookie, CSRF headers）
│       ├── routing/
│       │   └── auth_guard.dart       # GoRouter redirect（未認証時 /login へ）
│       ├── config/
│       │   ├── config_interpreter.dart  # サーバーからスキーマ+値を取得して ConfigData を生成
│       │   ├── config_editor_page.dart  # 設定値を編集する StatefulWidget ページ
│       │   ├── config_types.dart        # ConfigFieldType, ConfigFieldSchema, ConfigEditorSchema 等
│       │   └── widgets/
│       │       ├── category_nav.dart       # カテゴリ選択ナビゲーション
│       │       ├── config_field_list.dart  # フィールド一覧の描画（型に応じた Widget 振り分け）
│       │       └── fields/
│       │           ├── string_field.dart   # 文字列入力フィールド
│       │           ├── integer_field.dart  # 整数・小数入力フィールド（min/max 対応）
│       │           ├── boolean_field.dart  # トグルスイッチ
│       │           └── enum_field.dart     # ドロップダウン選択
│       ├── navigation/
│       │   ├── navigation_interpreter.dart  # navigation.yaml / API レスポンスから GoRouter を生成
│       │   └── navigation_types.dart        # NavigationRoute, NavigationGuard, NavigationParam 等
│       └── widgets/
│           ├── app_button.dart        # 共通ボタン
│           ├── app_scaffold.dart      # 共通スキャフォールド
│           └── loading_indicator.dart # ローディング表示
└── test/
    ├── auth/
    │   └── auth_provider_test.dart
    ├── http/
    │   └── api_client_test.dart
    ├── routing/
    │   └── auth_guard_test.dart
    ├── config/
    │   └── config_interpreter_test.dart
    └── navigation/
        └── navigation_interpreter_test.dart
```

### 主要な依存パッケージ

| パッケージ          | 用途                               |
| ------------------- | ---------------------------------- |
| `flutter_riverpod`  | 状態管理（認証状態 Provider）      |
| `go_router`         | ルーティング・ガード               |
| `dio`               | HTTP クライアント                  |
| `freezed_annotation`| 不変データクラス（AuthState）      |
| `yaml`              | navigation.yaml のパース           |

## React パッケージ設計（system-client）

### パッケージ名

`system-client`

### 配置パス

`regions/system/client/react/system-client/`

### ディレクトリ構成

```
system-client/
├── package.json
├── tsconfig.json
├── vite.config.ts               # library mode でビルド
├── src/
│   ├── index.ts                 # barrel export
│   ├── auth/
│   │   ├── AuthContext.tsx       # React Context 定義
│   │   ├── AuthProvider.tsx      # Provider 実装
│   │   ├── useAuth.ts            # useAuth hook
│   │   └── useAuth.test.tsx      # TDD テスト
│   ├── http/
│   │   ├── apiClient.ts          # Axios factory（withCredentials, CSRF interceptor）
│   │   └── apiClient.test.ts     # TDD テスト
│   ├── routing/
│   │   ├── ProtectedRoute.tsx    # 未認証時リダイレクト
│   │   └── ProtectedRoute.test.tsx
│   ├── config/
│   │   ├── ConfigInterpreter.ts       # スキーマ+値取得 → ConfigEditorConfig 生成
│   │   ├── ConfigInterpreter.test.ts
│   │   ├── ConfigEditorPage.tsx       # 設定エディタページコンポーネント
│   │   ├── ConfigEditorPage.test.tsx
│   │   ├── types.ts                   # ConfigFieldType, ConfigEditorSchema 等の型定義
│   │   ├── hooks/
│   │   │   ├── useConfigEditor.ts     # 設定エディタの状態管理 hook
│   │   │   └── useConfigEditor.test.ts
│   │   └── components/
│   │       ├── CategoryNav.tsx        # カテゴリ選択ナビゲーション
│   │       ├── ConfigFieldList.tsx    # フィールド一覧の描画
│   │       └── fields/
│   │           ├── StringField.tsx    # 文字列フィールド
│   │           ├── IntegerField.tsx   # 整数フィールド
│   │           ├── BooleanField.tsx   # 真偽値フィールド
│   │           ├── EnumField.tsx      # 列挙型フィールド
│   │           ├── ObjectField.tsx    # オブジェクトフィールド
│   │           └── ArrayField.tsx     # 配列フィールド
│   ├── navigation/
│   │   ├── NavigationInterpreter.ts       # navigation API / ローカル設定から ResolvedRoute を生成
│   │   ├── NavigationInterpreter.test.ts
│   │   ├── types.ts                       # NavigationRoute, NavigationGuard, ComponentRegistry 等
│   │   └── devtools/
│   │       └── NavigationDevTools.tsx     # 開発時のナビゲーション可視化ツール
│   └── components/
│       ├── AppButton.tsx
│       ├── AppButton.test.tsx
│       ├── LoadingSpinner.tsx
│       └── ErrorBoundary.tsx
└── tests/
    └── setup.ts                  # Vitest + Testing Library セットアップ
```

### 主要な依存パッケージ

| パッケージ                  | 用途                          |
| --------------------------- | ----------------------------- |
| `axios`                     | HTTP クライアント             |
| `@tanstack/react-router`    | ルーティング                  |
| `vitest`                    | テストランナー                |
| `@testing-library/react`    | コンポーネントテスト          |
| `msw`                       | API モック（テスト用）        |

## business/service client からの依存方法

### Flutter

```yaml
# business/service 側の pubspec.yaml
dependencies:
  system_client:
    path: ../../../../system/client/flutter/system_client
```

### React

```json
// business/service 側の package.json
{
  "dependencies": {
    "system-client": "file:../../../../system/client/react/system-client"
  }
}
```

## Config 管理機能

サービスの設定スキーマをサーバーから取得し、動的に設定エディタ UI を生成する機能。config-server の gRPC/REST API と連携して設定値の取得・更新を行う。

### アーキテクチャ

```
┌─────────────┐    GET /api/v1/config-schema/:service    ┌───────────────┐
│  Client App  │────────────────────────────────────────>│ config-server  │
│  (Flutter /  │    GET /api/v1/config/services/:service  │               │
│   React)     │<────────────────────────────────────────│               │
│              │    PUT /api/v1/config/:ns/:key           │               │
└─────────────┘                                          └───────────────┘
       │
       ▼
┌──────────────────┐
│ ConfigInterpreter │ スキーマ + 値を結合して UI データを生成
├──────────────────┤
│ ConfigEditorPage  │ カテゴリナビ + フィールド一覧のエディタ画面
└──────────────────┘
```

### ConfigInterpreter

サーバーから設定スキーマ（カテゴリ・フィールド定義）と現在の設定値を並列で取得し、UI に渡すデータ構造を生成する。

| 項目 | Flutter | React |
|------|---------|-------|
| クラス名 | `ConfigInterpreter` | `ConfigInterpreter` |
| HTTP クライアント | Dio | Axios |
| 入力 | `serviceName: String` | `serviceName: string` |
| 出力 | `ConfigData`（schema + values） | `ConfigEditorConfig`（schema + categories with fieldValues） |
| スキーマ取得 | `GET /api/v1/config-schema/:service` | `GET /api/v1/config-schema/:service` |
| 値取得 | `GET /api/v1/config/services/:service` | `GET /api/v1/config/services/:service` |

### ConfigEditorPage

設定値を編集するページコンポーネント。左ペインにカテゴリナビゲーション、右ペインにフィールド一覧を表示する。楽観的ロックによる競合検知（HTTP 409）に対応する。

| 項目 | Flutter | React |
|------|---------|-------|
| コンポーネント | `ConfigEditorPage`（StatefulWidget） | `ConfigEditorPage`（React Component） |
| 状態管理 | `setState` + `_dirtyValues` Map | `useConfigEditor` hook |
| 保存 | `PUT /api/v1/config/:ns/:key` | `useConfigEditor.save()` |
| 競合検知 | HTTP 409 → AlertDialog 表示 | HTTP 409 対応 |

### ConfigTypes（型定義）

| 型名 | 説明 |
|------|------|
| `ConfigFieldType` | フィールド種別（string, integer, float, boolean, enum, object, array） |
| `ConfigFieldSchema` | フィールド定義（key, label, type, min, max, options, pattern, unit, default） |
| `ConfigCategorySchema` | カテゴリ定義（id, label, icon, namespaces, fields） |
| `ConfigEditorSchema` | エディタスキーマ全体（service, namespace_prefix, categories） |

### Config Widget / Component 一覧

| Widget / Component | Flutter | React |
|-------------------|---------|-------|
| カテゴリナビ | `CategoryNav` | `CategoryNav` |
| フィールド一覧 | `ConfigFieldList` | `ConfigFieldList` |
| 文字列フィールド | `StringField`（fields/） | `StringField` |
| 整数フィールド | `IntegerField`（fields/） | `IntegerField` |
| 真偽値フィールド | `BooleanField`（fields/） | `BooleanField` |
| 列挙型フィールド | `EnumField`（fields/） | `EnumField` |
| オブジェクトフィールド | `StringField`（フォールバック） | `ObjectField` |
| 配列フィールド | `StringField`（フォールバック） | `ArrayField` |

---

## Navigation 機能

navigation.yaml またはサーバー API から取得したナビゲーション設定を解釈し、プラットフォーム固有のルーター設定を動的に生成する機能。

### アーキテクチャ

```
┌─────────────────┐    remote: GET /api/v1/navigation     ┌─────────────────┐
│   Client App     │──────────────────────────────────────>│ navigation API   │
│                  │    local: assets/navigation.yaml      │                  │
│                  │<──────────────────────────────────────│                  │
└─────────────────┘                                       └─────────────────┘
        │
        ▼
┌───────────────────────┐
│ NavigationInterpreter  │  設定を解釈してルーターを生成
├───────────────────────┤
│ Flutter: GoRouter      │  GoRoute ツリーを構築
│ React: ResolvedRoute[] │  コンポーネント解決 + ガードマッピング
└───────────────────────┘
```

### NavigationInterpreter

ナビゲーション設定（routes + guards）を取得し、プラットフォーム固有のルーター構造を生成する。

| 項目 | Flutter | React |
|------|---------|-------|
| クラス名 | `NavigationInterpreter` | `NavigationInterpreter` |
| モード | `NavigationMode.remote` / `NavigationMode.local` | `'remote'` / `'local'` |
| 入力 | `ComponentRegistry`（componentId → Widget Builder） | `ComponentRegistry`（componentId → Component / lazy import） |
| 出力 | `GoRouter` | `RouterResult`（ResolvedRoute[] + guards + raw） |
| ローカル設定 | `rootBundle.loadString` → YAML パース | `fetch` → JSON パース |
| リモート設定 | Dio GET → JSON パース | `fetch` GET → JSON パース |

### NavigationTypes（型定義）

| 型名 | 説明 |
|------|------|
| `GuardType` | ガード種別（auth_required, role_required, redirect_if_authenticated） |
| `TransitionType` | 画面遷移種別（fade, slide, modal） |
| `ParamType` | パラメータ型（string, int, uuid） |
| `NavigationParam` | ルートパラメータ定義（name + type） |
| `NavigationGuard` | ガード定義（id, type, redirect_to, roles） |
| `NavigationRoute` | ルート定義（id, path, component_id, guards, transition, redirect_to, children, params） |
| `NavigationResponse` | API レスポンス全体（routes + guards） |

### NavigationDevTools

React 版のみ提供する開発時のナビゲーション可視化ツール。ルートツリーとガード設定を視覚的に確認できる。

| 項目 | 値 |
|------|-----|
| 配置 | `react/system-client/src/navigation/devtools/NavigationDevTools.tsx` |
| 用途 | 開発時のルーティング設定デバッグ |
| 本番ビルド | tree-shaking により除外 |

---

## 関連ドキュメント

- [tier-architecture](../../architecture/overview/tier-architecture.md)
- [ディレクトリ構成図](../../architecture/overview/ディレクトリ構成図.md)
- [認証認可設計](../../architecture/auth/認証認可設計.md)
- [テンプレート仕様-Flutter](../../templates/client/Flutter.md)
- [テンプレート仕様-React](../../templates/client/React.md)
- [system-config-database設計](../config/database.md)
- [navigation設計](../../cli/config/navigation設計.md)
