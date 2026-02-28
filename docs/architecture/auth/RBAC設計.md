# RBAC 設計

D-005: アプリケーションレベル RBAC。Role/Permission/Resource モデル、Tier 別ロール定義、パーミッションマトリクスを定義する。

元ドキュメント: [認証認可設計.md](./認証認可設計.md)

---

## D-005: アプリケーションレベル RBAC

### Role → Permission → Resource モデル

```
User ──(has)──▶ Role ──(grants)──▶ Permission ──(on)──▶ Resource
```

- **Role**: ユーザーに割り当てる役割
- **Permission**: 操作の種類（`read`, `write`, `delete`, `admin`）
- **Resource**: 操作対象のリソース（`orders`, `ledger`, `users`）

### Tier 別ロール定義

#### system Tier ロール

| ロール              | 説明                           | Permission                          |
| ------------------- | ------------------------------ | ----------------------------------- |
| `sys_admin`         | システム全体の管理者           | すべてのリソースに対する全権限      |
| `sys_operator`      | システム運用担当               | 監視・ログ閲覧・設定変更           |
| `sys_auditor`       | 監査担当                       | 全リソースの読み取り専用           |

#### business Tier ロール

| ロール                 | 説明                          | Permission                          |
| ---------------------- | ----------------------------- | ----------------------------------- |
| `biz_{domain}_admin`   | 領域管理者                    | 領域内の全リソースに対する全権限    |
| `biz_{domain}_manager` | 領域マネージャー              | 領域内リソースの読み書き           |
| `biz_{domain}_viewer`  | 領域閲覧者                    | 領域内リソースの読み取り専用       |

#### service Tier ロール

| ロール                 | 説明                          | Permission                          |
| ---------------------- | ----------------------------- | ----------------------------------- |
| `svc_{service}_admin`  | サービス管理者                | サービス内の全リソースに対する全権限 |
| `svc_{service}_user`   | サービス利用者                | サービスの通常操作                  |
| `svc_{service}_viewer` | サービス閲覧者                | サービスリソースの読み取り専用      |

### パーミッションマトリクス（D-005）

#### system Tier パーミッションマトリクス

| ロール           | users | auth_config | audit_logs | api_gateway | vault_secrets | monitoring |
| ---------------- | ----- | ----------- | ---------- | ----------- | ------------- | ---------- |
| `sys_admin`      | CRUD  | CRUD        | R          | CRUD        | CRUD          | CRUD       |
| `sys_operator`   | R     | RU          | R          | R           | R             | RU         |
| `sys_auditor`    | R     | R           | R          | R           | ---           | R          |

#### business Tier パーミッションマトリクス（例: accounting 領域）

| ロール                      | ledger | journal_entries | reports | master_data | approvals |
| --------------------------- | ------ | --------------- | ------- | ----------- | --------- |
| `biz_accounting_admin`      | CRUD   | CRUD            | CRUD    | CRUD        | CRUD      |
| `biz_accounting_manager`    | RU     | CRU             | CR      | RU          | CRU       |
| `biz_accounting_viewer`     | R      | R               | R       | R           | R         |

#### service Tier パーミッションマトリクス（例: order サービス）

| ロール                | orders | order_items | shipments | payments |
| --------------------- | ------ | ----------- | --------- | -------- |
| `svc_order_admin`     | CRUD   | CRUD        | CRUD      | CRUD     |
| `svc_order_user`      | CRU    | CRU         | R         | CR       |
| `svc_order_viewer`    | R      | R           | R         | R        |

**凡例**: C = Create, R = Read, U = Update, D = Delete, --- = アクセス不可

#### パーミッション解決ルール

1. **ロールは Keycloak の `realm_access.roles` と `resource_access` から取得する**
2. 複数ロールを持つユーザーは、各ロールのパーミッションの **和集合** が適用される
3. 明示的に付与されていないパーミッションは **拒否（deny by default）**
4. `sys_admin` は全 Tier の全リソースにアクセス可能（スーパーユーザー）
5. Tier をまたぐアクセスは `tier_access` Claim で制御する

#### `tier_access` Claim の検証

二重検証により防御を多層化する。

| 検証レイヤー           | 実装箇所                | 目的                             |
| ---------------------- | ----------------------- | -------------------------------- |
| Mesh レベル            | Istio AuthorizationPolicy | インフラレベルでの一次防御       |
| アプリケーションレベル | 各サービスのミドルウェア  | アプリケーションレベルでの二次防御 |

**Mesh レベル検証（Istio AuthorizationPolicy）**

```yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: require-tier-access
  namespace: k1s0-business
spec:
  action: ALLOW
  rules:
    - when:
        - key: request.auth.claims[tier_access]
          values: ["business"]
```

**アプリケーションレベル検証（ミドルウェア）**

検証ロジック:
1. JWT の `tier_access` 配列を取得する
2. リクエスト先サービスが所属する Tier を特定する（サービス設定で定義）
3. サービスの Tier が `tier_access` 配列に含まれるかチェックする
4. 含まれていない場合は `403 Forbidden` を返却する

#### 新規サービス追加時のルール

新しいサービスを追加する際は、以下の3ロールを必ず定義する。

- `svc_{service}_admin` --- サービス内の全リソースに対する全権限
- `svc_{service}_user` --- 通常業務に必要な操作権限
- `svc_{service}_viewer` --- 読み取り専用

リソースごとのパーミッションは上記マトリクスのフォーマットに従い、サービスの設計ドキュメントに記載する。

### `has_permission` パーミッション解決ロジック

JWT Claims ベースの静的解決によりDB ルックアップ不要で低レイテンシの認可判定を実現する。

#### 解決フロー

```
1. JWT の realm_access.roles と resource_access.{client}.roles からロールを取得
2. ロール → パーミッション変換テーブルをインメモリキャッシュから参照
3. 要求されたパーミッション（permission + resource）がロールに含まれるかチェック
4. 含まれていれば許可、含まれていなければ拒否（deny by default）
```

#### ロール → パーミッション変換テーブルのキャッシュ

| 項目               | 値                                                     |
| ------------------ | ------------------------------------------------------ |
| データソース       | Keycloak Admin API（ロール定義 + パーミッション定義） |
| キャッシュ方式     | アプリケーション起動時にフェッチし、インメモリに保持   |
| キャッシュ TTL     | 5 分                                                   |
| 更新方式           | TTL 満了後のリクエスト時にバックグラウンドで再フェッチ |
| フォールバック     | キャッシュ更新失敗時は既存キャッシュを継続使用         |

### ミドルウェアシグネチャ

**Go**

```go
func RequirePermission(permission, resource string) func(http.Handler) http.Handler
```

**Rust**

```rust
pub async fn require_permission(
    permission: &str,
    resource: &str,
    req: Request,
    next: Next,
) -> Result<Response, ErrorResponse>
```

---

## パーミッション解決の設計背景

パーミッション解決は **JWT Claims ベースの静的解決** を基本とし、DB ルックアップを不要とすることで低レイテンシの認可判定を実現する。

- **インメモリ判定**: パーミッション判定は毎回 DB ルックアップを行わず、メモリ上のロール → パーミッション変換テーブルで即座に解決する
- **JWT Claims 信頼**: Kong で JWT 署名検証済みであることを前提とし、Claims 内のロール情報を信頼する
- **Keycloak Admin API**: 起動時および 5 分間隔で `GET /admin/realms/k1s0/roles` と各ロールのコンポジットロール情報を取得し、変換テーブルを構築する

## `tier_access` Claim の設計背景

`tier_access` Claim はユーザーがアクセス可能な Tier の一覧を定義する。二重検証により防御を多層化する。

- **Mesh レベル（Istio AuthorizationPolicy）**: Istio の AuthorizationPolicy で JWT Claims の `tier_access` を検証し、Mesh レベルで不正アクセスを遮断する
- **アプリケーションレベル（ミドルウェア）**: 各サービスのミドルウェアでも `tier_access` を二重検証する。Istio の検証をバイパスされた場合の防御層として機能する

---

## Go ミドルウェア実装例

```go
// internal/adapter/middleware/rbac.go

type RBACMiddleware struct {
    requiredPermission string
    requiredResource   string
}

func RequirePermission(permission, resource string) func(http.Handler) http.Handler {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            // Kong から転送されたヘッダーからロール情報を取得
            roles := strings.Split(r.Header.Get("X-User-Roles"), ",")
            userID := r.Header.Get("X-User-Id")

            if userID == "" {
                WriteError(w, r, http.StatusUnauthorized, "SYS_AUTH_UNAUTHENTICATED", "認証が必要です")
                return
            }

            if !hasPermission(roles, permission, resource) {
                WriteError(w, r, http.StatusForbidden, "SYS_AUTH_FORBIDDEN", "この操作を実行する権限がありません")
                return
            }

            next.ServeHTTP(w, r)
        })
    }
}

// ルーティング例
mux.Handle("GET /api/v1/orders",
    RequirePermission("read", "orders")(orderHandler.List))
mux.Handle("POST /api/v1/orders",
    RequirePermission("write", "orders")(orderHandler.Create))
mux.Handle("DELETE /api/v1/orders/{id}",
    RequirePermission("delete", "orders")(orderHandler.Delete))
```

## Rust ミドルウェア実装例

```rust
// src/adapter/middleware/rbac.rs

use axum::{extract::Request, middleware::Next, response::Response};

pub async fn require_permission(
    permission: &str,
    resource: &str,
    req: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let user_id = req.headers()
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ErrorResponse::unauthenticated("認証が必要です"))?;

    let roles: Vec<&str> = req.headers()
        .get("X-User-Roles")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(',')
        .collect();

    if !has_permission(&roles, permission, resource) {
        return Err(ErrorResponse::forbidden("この操作を実行する権限がありません"));
    }

    Ok(next.run(req).await)
}
```

---

## 関連ドキュメント

- [認証認可設計.md](./認証認可設計.md) -- 基本方針・技術スタック
- [認証設計.md](./認証設計.md) -- OAuth 2.0 / OIDC 実装
- [JWT設計.md](JWT設計.md) -- JWT 公開鍵ローテーション
- [サービス間認証設計.md](./サービス間認証設計.md) -- mTLS 設計
- [Vault設計.md](../../infrastructure/security/Vault設計.md) -- シークレット管理
