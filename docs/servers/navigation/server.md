# system-navigation-server 設計

クライアントアプリのルーティング・ガード設定を提供するナビゲーション管理サービス。gRPC で認証済みユーザーのロールに応じたルート定義とルートガードを返す。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | navigation/read |
| sys_operator 以上 | navigation/write |
| sys_admin のみ | navigation/admin |

> 現行実装のルート公開判定は `guard.roles` ベースで行う。上記の resource/action RBAC は管理 API 向けの設計上の表現であり、`GetNavigation` の判定ロジック自体は resource-action 評価を行わない。


| 機能 | 説明 |
| --- | --- |
| ナビゲーション設定配信 | クライアントアプリが必要とするルーティング定義をロールベースでフィルタリングして返す |
| ルートガード | AUTH_REQUIRED / ROLE_REQUIRED / REDIRECT_IF_AUTHENTICATED の3種のガードを定義 |
| 階層ルーティング | 子ルート（`children`）をサポートした再帰的なルート構造 |
| ページ遷移アニメーション | Fade / Slide / Modal の遷移アニメーション設定を含む |
| 未認証アクセス | `bearer_token` 省略時は公開ルートのみ返す |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | 技術 |
| --- | --- |
| プロトコル | gRPC（Proto3） |

### Proto 定義パス

`api/proto/k1s0/system/navigation/v1/navigation.proto`

---

## API 定義

### REST API エンドポイント

navigation-server の REST は運用用エンドポイントのみを提供する。  
ナビゲーション本体の取得 API は gRPC (`GetNavigation`) を正とする。

| Method | Path | Description |
| --- | --- | --- |
| GET | `/healthz` | ヘルスチェック |
| GET | `/readyz` | レディネスチェック |
| GET | `/metrics` | Prometheus メトリクス |

### gRPC サービス定義

### サービス: NavigationService

```protobuf
service NavigationService {
  rpc GetNavigation(GetNavigationRequest) returns (GetNavigationResponse);
}
```

---

### RPC 一覧

| RPC | リクエスト | レスポンス | 説明 |
| --- | --- | --- | --- |
| `GetNavigation` | `GetNavigationRequest` | `GetNavigationResponse` | ナビゲーション設定（ルート・ガード）を取得する |

---

### メッセージ定義

#### GetNavigationRequest

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `bearer_token` | string | 1 | JWT Bearer トークン。省略時は公開ルートのみ返す |

#### GetNavigationResponse

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `routes` | repeated Route | 1 | ルーティング定義一覧 |
| `guards` | repeated Guard | 2 | ルートガード定義一覧 |

#### Route

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `id` | string | 1 | ルート一意識別子 |
| `path` | string | 2 | URL パス（例: `/dashboard`, `/users/:id`） |
| `component_id` | optional string | 3 | フロントエンドコンポーネント識別子（省略可） |
| `guard_ids` | repeated string | 4 | 適用するガード ID のリスト |
| `children` | repeated Route | 5 | 子ルート（再帰的） |
| `transition` | TransitionConfig | 6 | ページ遷移アニメーション設定 |
| `params` | repeated Param | 7 | URLパラメータ定義 |
| `redirect_to` | string | 8 | リダイレクト先パス（リダイレクトルートの場合） |

#### Guard

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `id` | string | 1 | ガード一意識別子 |
| `type` | GuardType | 2 | ガード種別 |
| `redirect_to` | string | 3 | ガード失敗時のリダイレクト先 |
| `roles` | repeated string | 4 | 必要なロールリスト（ROLE_REQUIRED 時に使用） |

#### Param

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `name` | string | 1 | パラメータ名（例: `id`） |
| `type` | ParamType | 2 | パラメータの型 |

#### TransitionConfig

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `type` | TransitionType | 1 | アニメーション種別 |
| `duration_ms` | uint32 | 2 | アニメーション時間（ミリ秒） |

> ドメインモデルでは `guards` を解決済み構造として保持し、proto では `guard_ids` で参照する。
> また `TransitionConfig` はドメイン側でフラット構造に正規化して扱う。

---

### Enum 定義

#### GuardType

| 値 | 番号 | 説明 |
| --- | --- | --- |
| `GUARD_TYPE_UNSPECIFIED` | 0 | 未指定 |
| `GUARD_TYPE_AUTH_REQUIRED` | 1 | 認証必須（未認証時はリダイレクト） |
| `GUARD_TYPE_ROLE_REQUIRED` | 2 | 指定ロール必須（権限不足時はリダイレクト） |
| `GUARD_TYPE_REDIRECT_IF_AUTHENTICATED` | 3 | 認証済み時はリダイレクト（ログインページ等） |

#### TransitionType

| 値 | 番号 | 説明 |
| --- | --- | --- |
| `TRANSITION_TYPE_UNSPECIFIED` | 0 | 未指定 |
| `TRANSITION_TYPE_FADE` | 1 | フェードアニメーション |
| `TRANSITION_TYPE_SLIDE` | 2 | スライドアニメーション |
| `TRANSITION_TYPE_MODAL` | 3 | モーダル遷移 |

#### ParamType

| 値 | 番号 | 説明 |
| --- | --- | --- |
| `PARAM_TYPE_UNSPECIFIED` | 0 | 未指定 |
| `PARAM_TYPE_STRING` | 1 | 文字列 |
| `PARAM_TYPE_INT` | 2 | 整数 |
| `PARAM_TYPE_UUID` | 3 | UUID |

---

## 動作仕様

### 認証なし（公開ルートのみ）

`bearer_token` を省略または空文字で送信した場合、ガードのないルートのみを返す。

```protobuf
// リクエスト例（未認証）
GetNavigationRequest {
  bearer_token: ""
}
```

### 認証あり（ロールベースフィルタリング）

`bearer_token` を指定した場合、サーバーはトークンからロールを抽出し、アクセス可能なルートおよびガード定義を返す。

```protobuf
// リクエスト例（認証済み）
GetNavigationRequest {
  bearer_token: "eyJhbGciOiJSUzI1NiJ9..."
}
```

---

## 設定

### app

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | アプリケーション名 |
| `version` | string | アプリケーションバージョン（デフォルト `0.1.0`） |
| `environment` | string | 実行環境（`dev` / `staging` / `production`） |

### auth

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `jwks_url` | string | JWT 検証に利用する JWKS URL |
| `issuer` | string | JWT `iss` 検証値 |
| `audience` | string | JWT `aud` 検証値 |
| `jwks_cache_ttl_secs` | uint64 | JWKS キャッシュ秒数 |

### server

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `host` | string | バインドアドレス（デフォルト `0.0.0.0`） |
| `port` | uint16 | REST ポート |
| `grpc_port` | uint16 | gRPC ポート |

### navigation

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `navigation_path` | string | ナビゲーション定義 YAML の読み込みパス（デフォルト `config/navigation.yaml`） |

**設定例**

```yaml
app:
  name: "k1s0-navigation-server"
  version: "0.1.0"
  environment: "dev"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"
  issuer: "https://auth.k1s0.example.com/realms/system"
  audience: "k1s0-system"
  jwks_cache_ttl_secs: 300

navigation:
  navigation_path: "/etc/k1s0/navigation/navigation.yaml"
```

---

## 実装状況

| 項目 | 状態 |
| --- | --- |
| Proto 定義 | 完了（`api/proto/k1s0/system/navigation/v1/navigation.proto`） |
| サーバー実装 | 完了（`regions/system/server/rust/navigation/`） |
| クライアント生成コード | 未生成 |

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- JWT・ロールモデル
- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- ロールベースアクセス制御
- [system-auth/server.md](../auth/server.md) -- 認証サーバー（トークン発行元）

