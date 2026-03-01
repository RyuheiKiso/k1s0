# system-navigation-server 設計

クライアントアプリのルーティング・ガード設定を提供するナビゲーション管理サービス。gRPC で認証済みユーザーのロールに応じたルート定義とルートガードを返す。

## 概要

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

## API 定義（gRPC）

### サービス: NavigationService

```protobuf
service NavigationService {
  rpc GetNavigation(GetNavigationRequest) returns (NavigationResponse);
}
```

---

### RPC 一覧

| RPC | リクエスト | レスポンス | 説明 |
| --- | --- | --- | --- |
| `GetNavigation` | `GetNavigationRequest` | `NavigationResponse` | ナビゲーション設定（ルート・ガード）を取得する |

---

### メッセージ定義

#### GetNavigationRequest

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `bearer_token` | string | 1 | JWT Bearer トークン。省略時は公開ルートのみ返す |

#### NavigationResponse

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `routes` | repeated Route | 1 | ルーティング定義一覧 |
| `guards` | repeated Guard | 2 | ルートガード定義一覧 |

#### Route

| フィールド | 型 | フィールド番号 | 説明 |
| --- | --- | --- | --- |
| `id` | string | 1 | ルート一意識別子 |
| `path` | string | 2 | URL パス（例: `/dashboard`, `/users/:id`） |
| `component_id` | string | 3 | フロントエンドコンポーネント識別子 |
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
