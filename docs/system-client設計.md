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
│       └── widgets/
│           ├── app_button.dart        # 共通ボタン
│           ├── app_scaffold.dart      # 共通スキャフォールド
│           └── loading_indicator.dart # ローディング表示
└── test/
    ├── auth/
    │   └── auth_provider_test.dart
    ├── http/
    │   └── api_client_test.dart
    └── routing/
        └── auth_guard_test.dart
```

### 主要な依存パッケージ

| パッケージ          | 用途                               |
| ------------------- | ---------------------------------- |
| `flutter_riverpod`  | 状態管理（認証状態 Provider）      |
| `go_router`         | ルーティング・ガード               |
| `dio`               | HTTP クライアント                  |
| `freezed_annotation`| 不変データクラス（AuthState）      |

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

## 関連ドキュメント

- [tier-architecture](tier-architecture.md)
- [ディレクトリ構成図](ディレクトリ構成図.md)
- [認証認可設計](認証認可設計.md)
- [テンプレート仕様-Flutter](テンプレート仕様-Flutter.md)
- [テンプレート仕様-React](テンプレート仕様-React.md)
