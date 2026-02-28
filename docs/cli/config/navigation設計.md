# Navigation 設計

k1s0 におけるクライアントナビゲーション（画面遷移）の統一設計を定義する。
React・Flutter のいずれのフレームワークでも同一の `navigation.yaml` から
ナビゲーションを制御し、開発者体験（DX）と保守性を両立する。

> **設計思想** — Netflix の Server-Driven UI アプローチを参考に、k1s0 の既存インフラ
>（proto・CLI・system-client）と統合した **k1s0 版 SDUI（Server-Driven Navigation）**。
> ナビゲーション定義はサーバーが持ち、クライアントは「解釈エンジン」として動作する。

---

## 解決する課題

| 課題 | 従来のアプローチ | 本設計 |
| ---- | --------------- | ------ |
| 開発者ごとに遷移実装が異なる | コードレビューで対応（限界あり） | `navigation.yaml` を唯一の定義源とする |
| フレームワーク追加時の再実装 | React / Flutter それぞれで実装 | NavigationInterpreter のアダプターを追加するだけ |
| 認証ガードの実装漏れ | 個々の開発者の判断に依存 | YAML でガードを宣言、実装は SDK が担う |
| ローカル開発にサーバーが必要 | 常にサーバーを起動 | Local-first モードで YAML を直接読む |
| ルート ID の文字列タイポ | runtime エラー | CLI 生成の型定義でコンパイルエラー |

---

## アーキテクチャ全体像

<img src="diagrams/navigation-architecture.svg" width="900" />

---

## navigation.yaml（開発者インターフェース）

開発者が編集する唯一のファイル。`$schema` を指定することで VS Code / IntelliJ が
補完・バリデーションを提供する。

```yaml
# $schema: ./navigation-schema.json
version: 1

guards:
  - id: auth_required
    type: auth_required
    redirect_to: /login

  - id: admin_only
    type: role_required
    roles: [admin]
    redirect_to: /forbidden

routes:
  - id: root
    path: /
    redirect_to: /dashboard

  - id: login
    path: /login
    component_id: LoginPage
    guards: []

  - id: dashboard
    path: /dashboard
    component_id: DashboardPage
    guards: [auth_required]
    transition: fade

  - id: users
    path: /users
    component_id: UsersPage
    guards: [auth_required, admin_only]
    transition: slide
    children:
      - id: user_detail
        path: :id
        component_id: UserDetailPage
        guards: [auth_required]
        params:
          - name: id
            type: string

  - id: forbidden
    path: /forbidden
    component_id: ForbiddenPage
    guards: []

  - id: not_found
    path: '*'
    component_id: NotFoundPage
    guards: []
```

### フィールド定義

#### routes

| フィールド | 型 | 必須 | 説明 |
| ---------- | -- | ---- | ---- |
| `id` | string | ✅ | ルートの識別子。英小文字とアンダースコアのみ |
| `path` | string | ✅ | URL パス。パラメーターは `:name` 形式 |
| `component_id` | string | ✅ | クライアント側コンポーネントの識別子 |
| `guards` | string[] | — | 適用するガードの ID リスト。省略時は空配列 |
| `transition` | enum | — | 遷移アニメーション（`fade` / `slide` / `modal`）。省略時は即切り替え |
| `redirect_to` | string | — | このルートへのアクセス時に別パスへリダイレクト（`component_id` と排他） |
| `children` | Route[] | — | ネストされた子ルート |
| `params` | Param[] | — | パスパラメーターの型定義 |

#### guards

| フィールド | 型 | 必須 | 説明 |
| ---------- | -- | ---- | ---- |
| `id` | string | ✅ | ガードの識別子 |
| `type` | enum | ✅ | `auth_required` / `role_required` / `redirect_if_authenticated` |
| `redirect_to` | string | ✅ | ガード失敗時のリダイレクト先 |
| `roles` | string[] | — | `role_required` 時のみ指定。RBAC ロール名 |

---

## 実装方式

**REST API のみ**で実装する。gRPC は将来検討。

`navigation.yaml` の内容をサーバーが JSON レスポンスとして返す。
クライアントは `GET /api/v1/navigation` エンドポイントから NavigationResponse を取得し、
各フレームワークのルーターに変換する。

---

## 実装状況

| コンポーネント | 状態 |
|---|---|
| auth-server: GET /api/v1/navigation | 実装済み |
| Flutter: NavigationInterpreter | 実装済み |
| React: NavigationInterpreter | 実装済み |
| navigation.yaml 仕様 | 定義済み |
| CLI route-types/component-registry 生成 | 未実装 |

---

## system server エンドポイント

`regions/system/server/rust/` の認証サーバーに Navigation エンドポイントを追加する。

```
GET /api/v1/navigation
Authorization: Bearer <token>  （省略時は公開ルートのみ返す）

Response: 200 OK
Content-Type: application/json

{
  "routes": [...],
  "guards": [...]
}
```

### サーバー側の処理フロー

<img src="diagrams/navigation-server-flow.svg" width="600" />

---

## system-client SDK（NavigationInterpreter）

`system-client` パッケージに `NavigationInterpreter` を追加する。
クライアントアプリは起動時に1回 `GET /api/v1/navigation` を呼び出し、
返ってきた定義をフレームワークのルーターに変換する。

### React

```typescript
// system-client/src/navigation/NavigationInterpreter.ts

export interface NavigationConfig {
  /** 本番: サーバーからフェッチ / 開発: YAML を直接読む */
  mode: 'remote' | 'local';
  remoteUrl?: string;
  localConfigPath?: string;
  componentRegistry: ComponentRegistry;
}

export class NavigationInterpreter {
  constructor(private config: NavigationConfig) {}

  async build(): Promise<ReturnType<typeof createRouter>> {
    const nav = await this.fetchNavigation();
    return this.buildTanStackRouter(nav);
  }

  private async fetchNavigation(): Promise<NavigationResponse> {
    if (this.config.mode === 'local') {
      // development: public/navigation.yaml を直接読む（サーバー不要）
      const res = await fetch(this.config.localConfigPath ?? '/navigation.yaml');
      return parseYaml(await res.text());
    }
    const res = await fetch(this.config.remoteUrl ?? '/api/v1/navigation');
    return res.json();
  }

  private buildTanStackRouter(nav: NavigationResponse) {
    const routes = nav.routes.map((r) => this.buildRoute(r, nav.guards));
    return createRouter({ routeTree: rootRoute.addChildren(routes) });
  }
}
```

```typescript
// business/service client での使用例
import { NavigationInterpreter } from 'system-client';
import { componentRegistry } from './__generated__/component-registry';

const interpreter = new NavigationInterpreter({
  mode: import.meta.env.DEV ? 'local' : 'remote',
  remoteUrl: '/api/v1/navigation',
  localConfigPath: '/navigation.yaml',
  componentRegistry,
});

export const router = await interpreter.build();
```

### Flutter

```dart
// system_client/lib/src/navigation/navigation_interpreter.dart

class NavigationInterpreter {
  const NavigationInterpreter({
    required this.mode,
    required this.componentRegistry,
    this.remoteUrl = '/api/v1/navigation',
    this.localConfigAsset = 'assets/navigation.yaml',
  });

  final NavigationMode mode;
  final ComponentRegistry componentRegistry;
  final String remoteUrl;
  final String localConfigAsset;

  Future<GoRouter> build() async {
    final nav = await _fetchNavigation();
    return _buildGoRouter(nav);
  }

  Future<NavigationResponse> _fetchNavigation() async {
    if (mode == NavigationMode.local) {
      // development: assets/navigation.yaml を直接読む
      final yaml = await rootBundle.loadString(localConfigAsset);
      return NavigationResponse.fromYaml(yaml);
    }
    final res = await http.get(Uri.parse(remoteUrl));
    return NavigationResponse.fromJson(jsonDecode(res.body));
  }
}

enum NavigationMode { remote, local }
```


---

## Local-first 開発モード

サーバーを起動しなくても UI 開発ができる。`navigation.yaml` を各フレームワークの
`public` / `assets` ディレクトリに配置し、SDK が直接読み込む。

```
開発時（NODE_ENV=development / kDebugMode / DEBUG）:
  navigation.yaml を直接読む → サーバー不要

本番時（NODE_ENV=production）:
  GET /api/v1/navigation → system server 経由
```

| フレームワーク | ローカルファイルの配置場所 |
| -------------- | ------------------------- |
| React | `public/navigation.yaml` |
| Flutter | `assets/navigation.yaml`（pubspec.yaml の `assets:` に追加） |

---

## CLI 生成物

### 1. route-types（型安全なルート ID）

```bash
k1s0 generate navigation --target react
k1s0 generate navigation --target flutter
```

**React 生成例（`src/navigation/__generated__/route-types.ts`）:**

```typescript
// このファイルは CLI が自動生成する。直接編集しないこと。
// navigation.yaml から生成: 2026-02-23

export const RouteIds = {
  ROOT:        'root',
  LOGIN:       'login',
  DASHBOARD:   'dashboard',
  USERS:       'users',
  USER_DETAIL: 'user_detail',
  FORBIDDEN:   'forbidden',
  NOT_FOUND:   'not_found',
} as const;

export type RouteId = typeof RouteIds[keyof typeof RouteIds];

// パスパラメーターの型定義
export type RouteParams = {
  root:        Record<string, never>;
  login:       Record<string, never>;
  dashboard:   Record<string, never>;
  users:       Record<string, never>;
  user_detail: { id: string };
  forbidden:   Record<string, never>;
  not_found:   Record<string, never>;
};

// 型安全なナビゲーション hook
export function useNav() {
  const navigate = useNavigate();
  return {
    navigate: <T extends RouteId>(
      to: T,
      params?: RouteParams[T],
    ) => navigate({ to: buildPath(to, params) }),
  };
}
```

**Flutter 生成例（`lib/navigation/__generated__/route_ids.dart`）:**

```dart
// このファイルは CLI が自動生成する。直接編集しないこと。
// navigation.yaml から生成: 2026-02-23

enum RouteId {
  root,
  login,
  dashboard,
  users,
  userDetail,
  forbidden,
  notFound;

  String get path => switch (this) {
    RouteId.root        => '/',
    RouteId.login       => '/login',
    RouteId.dashboard   => '/dashboard',
    RouteId.users       => '/users',
    RouteId.userDetail  => '/users/:id',
    RouteId.forbidden   => '/forbidden',
    RouteId.notFound    => '*',
  };
}

extension TypedNavigation on GoRouter {
  void goTo(RouteId id, {Map<String, String> params = const {}}) {
    go(_buildPath(id.path, params));
  }
}
```

### 2. component-registry（ID とコンポーネントの紐付け）

開発者が各アプリで一度だけ作成する。CLI が雛形を生成し、以降は手動で管理する。

```typescript
// src/navigation/__generated__/component-registry.ts（雛形）
import type { ComponentRegistry } from 'system-client';

// ⚠️ component_id が navigation.yaml に追加されたら、このファイルも更新すること
// `k1s0 validate navigation` で未登録の component_id を検出できる
export const componentRegistry: ComponentRegistry = {
  LoginPage:      () => import('../../pages/LoginPage'),
  DashboardPage:  () => import('../../pages/DashboardPage'),
  UsersPage:      () => import('../../pages/UsersPage'),
  UserDetailPage: () => import('../../pages/UserDetailPage'),
  ForbiddenPage:  () => import('../../pages/ForbiddenPage'),
  NotFoundPage:   () => import('../../pages/NotFoundPage'),
};
```

---

## CLI validate コマンド

```bash
$ k1s0 validate navigation

Checking navigation.yaml...

  ✅ スキーマバリデーション OK
  ✅ guard 参照の整合性 OK（全 guard_ids が定義済み）
  ✅ route ID の重複なし
  ❌ component_id 'SettingsPage' が component-registry に未登録
       → src/navigation/__generated__/component-registry.ts を更新してください

1 error found.
```

### チェック項目

| チェック | 内容 |
| -------- | ---- |
| スキーマバリデーション | `navigation.yaml` が JSON Schema に準拠しているか |
| guard 参照の整合性 | `guard_ids` に存在しない guard を参照していないか |
| component_id の登録確認 | `component_id` が `component-registry` に登録済みか |
| route ID の重複 | 同一 ID が複数のルートで使われていないか |
| 循環リダイレクト検出 | `redirect_to` が循環参照になっていないか |

CI パイプラインへの統合:

```yaml
# .github/workflows/ci.yaml
- name: Validate navigation
  run: k1s0 validate navigation
```

---

## Navigation DevTools（開発時のみ）

`vite.config.ts` / `pubspec.yaml` にプラグインを追加するだけで有効化される。
本番ビルドでは自動的に除外される。

```typescript
// vite.config.ts
import { navigationDevTools } from 'system-client/devtools';

export default defineConfig({
  plugins: [navigationDevTools()],
});
```

画面右下にオーバーレイを表示:

| 項目 | 表示例 |
| ---- | ------ |
| 現在のルート | `/users/123` |
| Route ID | `user_detail` |
| ガード | `auth_required` |
| 遷移アニメーション | `slide` |
| コンポーネント | `UserDetailPage` |
| アクション | ルートツリーを見る / YAML を見る |

---

## ディレクトリ構成

```
regions/system/
├── client/
│   ├── react/
│   │   └── system-client/
│   │       └── src/
│   │           └── navigation/
│   │               ├── NavigationInterpreter.ts
│   │               ├── NavigationInterpreter.test.ts
│   │               ├── types.ts                    # NavigationResponse 型
│   │               └── devtools/                   # DevTools オーバーレイ
│   ├── flutter/
│   │   └── system_client/
│   │       └── lib/src/navigation/
│   │           ├── navigation_interpreter.dart
│   │           └── navigation_interpreter_test.dart
└── server/
    └── rust/
        └── auth-server/               # Navigation エンドポイントを追加
            └── src/
                └── adapter/
                    └── handler/
                        └── navigation.rs
```

---

## 実装ロードマップ

| Phase | 内容 | 優先度 | 状態 |
| ----- | ---- | ------ | ---- |
| 1 | system server に `GET /api/v1/navigation` を追加 | 高 | 実装済み |
| 2 | React `NavigationInterpreter` + Local-first モード | 高 | 実装済み |
| 3 | Flutter `NavigationInterpreter` + Local-first モード | 高 | 実装済み |
| 4 | CLI `generate navigation` コマンド（route-types 生成） | 高 | 未実装 |
| 5 | CLI `validate navigation` コマンド | 中 | 未実装 |
| 6 | JSON Schema 生成（IDE 補完用） | 中 | 未実装 |
| 7 | Navigation DevTools（React / Flutter） | 中 | 未実装 |
| 9 | ユーザーロール別ナビゲーションフィルタリング | 低 | 未実装 |

---

## 設計上の補足

- **後方互換性** — `navigation.yaml` のスキーマ変更は `version` フィールドで管理する。`component_id` を削除する場合は deprecated 扱いとし、1バージョン猶予を設ける
- **キャッシュ戦略** — クライアントは `NavigationResponse` をローカルストレージにキャッシュし、サーバーが落ちていても最後の定義で動作する
- **A/B テスト** — サーバーがユーザーセグメントに応じて異なる `NavigationResponse` を返すことで、クライアント変更なしに遷移を実験できる（Phase 10 以降）
- **navigation.yaml の配置** — `regions/system/server/rust/auth-server/config/navigation.yaml` に配置し、`config.yaml` と同様に環境別オーバーライドを許容する

---

## 関連ドキュメント

- [tier-architecture](../../architecture/overview/tier-architecture.md)
- [system-client設計](../../servers/_common/client.md)
- [proto設計](../../architecture/api/proto設計.md)
- [認証認可設計](../../architecture/auth/認証認可設計.md)
- [CLIフロー](../flow/CLIフロー.md)
- [テンプレート仕様-React](../../templates/client/React.md)
- [テンプレート仕様-Flutter](../../templates/client/Flutter.md)
- [コーディング規約](../../architecture/conventions/コーディング規約.md)
