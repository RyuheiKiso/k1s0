# RBAC 設計 ガイド

> **仕様**: テーブル定義・APIスキーマは [RBAC設計.md](./RBAC設計.md) を参照。

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

- [RBAC設計.md](./RBAC設計.md) -- 仕様（テーブル・マトリクス定義）
- [認証認可設計.md](./認証認可設計.md) -- 基本方針・技術スタック
- [認証設計.md](./認証設計.md) -- OAuth 2.0 / OIDC 実装
- [JWT設計.md](JWT設計.md) -- JWT 公開鍵ローテーション
- [サービス間認証設計.md](./サービス間認証設計.md) -- mTLS 設計
- [Vault設計.md](../../infrastructure/security/Vault設計.md) -- シークレット管理
