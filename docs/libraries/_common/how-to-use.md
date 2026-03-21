# ライブラリ How-to ガイド

「すぐ使える」コード例集。設計の詳細は各ライブラリのドキュメントを参照すること。

各ライブラリの配置パスは以下の通り:

| 言語 | 配置パス |
|------|---------|
| Go | `regions/system/library/go/{lib-name}/` |
| Rust | `regions/system/library/rust/{lib-name}/` |
| TypeScript | `regions/system/library/typescript/{lib-name}/` |
| Dart | `regions/system/library/dart/{lib-name}/` |

---

## 1. 認証ライブラリ（auth）

**やりたいこと**: JWT トークンを検証してクレーム（ユーザー情報・ロール）を取得する。
**ライブラリ**: Go → `authlib`パッケージ / Rust → `k1s0-auth` クレート / TypeScript → `@k1s0/auth` / Dart → `k1s0_auth`
**詳細設計**: [auth.md](../auth-security/auth.md)

### Go（サーバー側: JWT 検証）

```go
package main

import (
    "context"
    "net/http"
    "time"

    // authlib パッケージをインポートする
    authlib "github.com/k1s0-platform/system-library-go-auth"
)

func main() {
    // JWKS 検証器を生成する（cacheTTL は JWKS の再取得間隔）
    verifier := authlib.NewJWKSVerifier(
        "http://localhost:8180/realms/k1s0/protocol/openid-connect/certs",
        "http://localhost:8180/realms/k1s0",
        "k1s0-api",
        5*time.Minute,
    )

    // HTTP ハンドラでトークンを検証してクレームを取得する
    http.HandleFunc("/api/v1/orders", func(w http.ResponseWriter, r *http.Request) {
        // Bearer トークンを取得する
        tokenString := r.Header.Get("Authorization")
        // "Bearer " プレフィックスを除去する
        if len(tokenString) > 7 {
            tokenString = tokenString[7:]
        }

        claims, err := verifier.VerifyToken(r.Context(), tokenString)
        if err != nil {
            http.Error(w, "Unauthorized", http.StatusUnauthorized)
            return
        }

        // クレームからユーザー情報を取得する
        userID := claims.Sub
        username := claims.Username

        // RBAC チェック: "orders" リソースに対して "read" アクションを許可するか確認する
        if !authlib.CheckPermission(claims, "orders", "read") {
            http.Error(w, "Forbidden", http.StatusForbidden)
            return
        }

        _ = userID
        _ = username
        // ここでビジネスロジックを実行する
    })
}
```

**ミドルウェアとして使う場合（Gin）**:

```go
import (
    authlib "github.com/k1s0-platform/system-library-go-auth"
)

// ルーターに認証ミドルウェアを設定する
router.Use(authlib.AuthMiddleware(verifier))

// ハンドラ内でコンテキストからクレームを取得する
func OrderHandler(c *gin.Context) {
    claims := authlib.GetClaimsFromContext(c)
    if claims == nil {
        c.AbortWithStatus(http.StatusUnauthorized)
        return
    }
    userID := claims.Sub
    _ = userID
}
```

### Rust（サーバー側: JWT 検証）

```rust
use std::time::Duration;
// k1s0-auth クレートをインポートする
use k1s0_auth::{JwksVerifier, check_permission, AuthError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JWKS 検証器を生成する（HTTP クライアント構築エラーを Result で返す）
    let verifier = JwksVerifier::new(
        "http://localhost:8180/realms/k1s0/protocol/openid-connect/certs",
        "http://localhost:8180/realms/k1s0",
        "k1s0-api",
        Duration::from_secs(300),
    )?;

    // トークンを検証してクレームを取得する
    let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";
    let claims = verifier.verify_token(token).await?;

    // クレームからユーザー情報を取得する
    let user_id = &claims.sub;
    let username = claims.preferred_username.as_deref().unwrap_or("unknown");

    // RBAC チェック: "orders" リソースに対して "read" アクションを確認する
    if !check_permission(&claims, "orders", "read") {
        return Err(AuthError::PermissionDenied.into());
    }

    println!("user_id={user_id}, username={username}");
    Ok(())
}
```

**axum ミドルウェアとして使う場合**:

```rust
use k1s0_auth::middleware::{auth_middleware, require_permission};
use axum::{Router, middleware};
use std::sync::Arc;

let verifier = Arc::new(JwksVerifier::new(/* ... */)?);

let app = Router::new()
    .route("/api/v1/orders", get(order_handler))
    // 認証ミドルウェアを追加する
    .layer(middleware::from_fn_with_state(verifier, auth_middleware));
```

### TypeScript（クライアント側: OAuth2 PKCE ログイン）

```typescript
import { AuthClient } from '@k1s0/auth';

// AuthClient を初期化する（Web クライアント向け OAuth2 PKCE フロー）
const authClient = new AuthClient({
  discoveryUrl: 'http://localhost:8180/realms/k1s0/.well-known/openid-configuration',
  clientId: 'k1s0-web',
  redirectUri: 'http://localhost:3000/callback',
  scopes: ['openid', 'profile', 'email'],
});

// ログイン（認可サーバーにリダイレクトする）
await authClient.login();

// 認可コールバックを処理してトークンを取得する
// （コールバックページで URL パラメータから code と state を取得して渡す）
const urlParams = new URLSearchParams(window.location.search);
const tokenSet = await authClient.handleCallback(
  urlParams.get('code')!,
  urlParams.get('state')!,
);

// アクセストークンを取得する（有効期限が切れている場合は自動リフレッシュ）
const accessToken = await authClient.getAccessToken();

// 認証状態を確認する
const isAuth = authClient.isAuthenticated();
```

### Dart（クライアント側: OAuth2 PKCE ログイン）

```dart
import 'package:k1s0_auth/k1s0_auth.dart';

// AuthClient を初期化する
final authClient = AuthClient(AuthConfig(
  discoveryUrl: 'http://localhost:8180/realms/k1s0/.well-known/openid-configuration',
  clientId: 'k1s0-mobile',
  redirectUri: 'com.example.k1s0://callback',
  scopes: ['openid', 'profile', 'email'],
));

// ログイン（認可フローを開始する）
await authClient.login();

// アクセストークンを取得する（自動リフレッシュ対応）
final accessToken = await authClient.getAccessToken();

// 認証状態を確認する
final isAuthenticated = authClient.isAuthenticated();
```

---

## 2. 設定ライブラリ（config）

**やりたいこと**: YAML 設定ファイルを読み込み、環境別ファイルをマージして設定オブジェクトを得る。
**ライブラリ**: Go → `config`パッケージ / Rust → `k1s0-config` クレート / TypeScript → `@k1s0/config` / Dart → `k1s0_config`
**詳細設計**: [config.md](../config/config.md)

### Go

```go
package main

import (
    "log"

    // config パッケージをインポートする
    config "github.com/k1s0-platform/system-library-go-config"
)

func main() {
    // ベース設定ファイルと環境別ファイルを読み込んでマージする
    // envPath は省略可能（本番環境では config.prod.yaml 等を指定する）
    cfg, err := config.Load(
        "configs/config.yaml",
        "configs/config.local.yaml", // 環境別ファイル（省略可能）
    )
    if err != nil {
        log.Fatalf("設定読み込み失敗: %v", err)
    }

    // バリデーションを実行する（必須フィールドの欠落等を検出する）
    if err := cfg.Validate(); err != nil {
        log.Fatalf("設定バリデーション失敗: %v", err)
    }

    // 設定値を使用する
    log.Printf("サービス起動: name=%s port=%d", cfg.App.Name, cfg.Server.Port)

    // Vault シークレットをマージする（本番環境で使用する）
    secrets := map[string]string{
        "database.password": "retrieved-from-vault",
    }
    cfg.MergeVaultSecrets(secrets)
}
```

### Rust

```rust
use k1s0_config::{load, validate, merge_vault_secrets};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ベース設定ファイルを読み込む（環境別ファイルは None または Some("path") で指定）
    let mut cfg = load("configs/config.yaml", Some("configs/config.local.yaml"))?;

    // バリデーションを実行する
    validate(&cfg)?;

    // 設定値を使用する
    println!("サービス起動: name={} port={}", cfg.app.name, cfg.server.port);

    // Vault シークレットをマージする
    let mut secrets = std::collections::HashMap::new();
    secrets.insert("database.password".to_string(), "retrieved-from-vault".to_string());
    merge_vault_secrets(&mut cfg, &secrets);

    Ok(())
}
```

### TypeScript

```typescript
import { load, validate, mergeVaultSecrets } from '@k1s0/config';

// ベース設定ファイルと環境別ファイルを読み込む
const config = load('configs/config.yaml', 'configs/config.local.yaml');

// Zod スキーマを使ってバリデーションを実行する（不正な設定は例外をスロー）
validate(config);

// 設定値を使用する
console.log(`サービス起動: name=${config.app.name} port=${config.server.port}`);

// Vault シークレットをマージする（元の config は変更せず新しいオブジェクトを返す）
const secureConfig = mergeVaultSecrets(config, {
  'database.password': 'retrieved-from-vault',
});
```

### Dart

```dart
import 'package:k1s0_config/k1s0_config.dart';

Future<void> main() async {
    // ベース設定ファイルを読み込む（環境別ファイルは省略可能）
    final config = await loadConfig(
        basePath: 'configs/config.yaml',
        envPath: 'configs/config.local.yaml',
    );

    // バリデーションを実行する
    validateConfig(config);

    // 設定値を使用する
    print('サービス起動: name=${config.app.name} port=${config.server.port}');
}
```

> **注意**: Dart 実装の `loadConfig` / `validateConfig` 関数名は実装を確認して記載すること（設計書に Dart コード例なし）。

---

## 3. 可観測性ライブラリ（telemetry）

**やりたいこと**: OpenTelemetry トレーシングを初期化し、構造化ログにトレース ID を付与する。
**ライブラリ**: Go → `telemetry`パッケージ / Rust → `k1s0-telemetry` クレート / TypeScript → `@k1s0/telemetry` / Dart → `k1s0_telemetry`
**詳細設計**: [telemetry.md](../observability/telemetry.md)

### Go

```go
package main

import (
    "context"
    "log/slog"

    // telemetry パッケージをインポートする
    "github.com/k1s0-platform/system-library-go-telemetry"
)

func main() {
    ctx := context.Background()

    // OpenTelemetry プロバイダーとロガーを初期化する
    provider, err := telemetry.InitTelemetry(ctx, telemetry.TelemetryConfig{
        ServiceName:   "order-server",
        Version:       "1.0.0",
        Tier:          "business",
        Environment:   "dev",
        TraceEndpoint: "localhost:4317", // Jaeger OTLP gRPC エンドポイント
        SampleRate:    1.0,
        LogLevel:      "info",
        LogFormat:     "json",
    })
    if err != nil {
        panic(err)
    }
    // アプリケーション終了時にプロバイダーをシャットダウンする
    defer provider.Shutdown(ctx)

    // ロガーを取得する
    logger := provider.Logger()

    // HTTP ハンドラ内でトレース ID をログに付与する
    // （span context がある場合は trace_id / span_id が自動付与される）
    tracedLogger := telemetry.LogWithTrace(ctx, logger)
    tracedLogger.Info("注文リクエストを処理中", slog.String("order_id", "ord-123"))

    // Prometheus メトリクスを記録する
    m := telemetry.NewMetrics("order-server")
    m.HTTPRequestsTotal.With(map[string]string{
        "method": "GET", "path": "/api/v1/orders", "status": "200",
    }).Inc()
    m.HTTPRequestDuration.With(map[string]string{
        "method": "GET", "path": "/api/v1/orders",
    }).Observe(0.045)
}
```

### Rust

```rust
use k1s0_telemetry::{TelemetryConfig, init_telemetry, shutdown, Metrics};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenTelemetry とロガーを初期化する（tracing-subscriber が設定される）
    init_telemetry(&TelemetryConfig {
        service_name: "order-server".to_string(),
        version: "1.0.0".to_string(),
        tier: "business".to_string(),
        environment: "dev".to_string(),
        trace_endpoint: Some("http://localhost:4317".to_string()),
        sample_rate: 1.0,
        log_level: "info".to_string(),
        log_format: "json".to_string(),
    })?;

    // tracing マクロでログを出力する（trace_id は自動付与される）
    tracing::info!(order_id = "ord-123", "注文リクエストを処理中");

    // Prometheus メトリクスを記録する
    let metrics = Metrics::new("order-server");
    metrics.record_http_request("GET", "/api/v1/orders", "200");
    metrics.record_http_duration("GET", "/api/v1/orders", 0.045);

    // DB クエリ時間を記録する（Rust 拡張メトリクス）
    metrics.record_db_query_duration("find_order", "orders", 0.012);

    // アプリケーション終了時にシャットダウンする
    shutdown();
    Ok(())
}
```

**axum-layer feature を使ってミドルウェアを設定する場合**:

```rust
// Cargo.toml: k1s0-telemetry = { path = "...", features = ["axum-layer"] }
use k1s0_telemetry::middleware::MetricsLayer;
use axum::Router;

let metrics = Metrics::new("order-server");
let app = Router::new()
    .route("/api/v1/orders", get(order_handler))
    // HTTP メトリクスを自動計測する Tower Layer を追加する
    .layer(MetricsLayer::new(metrics));
```

### TypeScript

```typescript
import { initTelemetry, shutdown, createLogger } from '@k1s0/telemetry';

// OpenTelemetry SDK を初期化する
initTelemetry({
  serviceName: 'order-server',
  version: '1.0.0',
  tier: 'business',
  environment: 'dev',
  traceEndpoint: 'http://localhost:4317',
  sampleRate: 1.0,
  logLevel: 'info',
  logFormat: 'json',
});

// pino ロガーを生成する
// アクティブなスパンがある場合は trace_id / span_id が自動付与される（mixin）
const logger = createLogger({
  serviceName: 'order-server',
  version: '1.0.0',
  tier: 'business',
  environment: 'dev',
  logLevel: 'info',
});

logger.info({ orderId: 'ord-123' }, '注文リクエストを処理中');

// アプリケーション終了時にシャットダウンする
process.on('SIGTERM', async () => {
  await shutdown();
  process.exit(0);
});
```

### Dart

```dart
import 'package:k1s0_telemetry/k1s0_telemetry.dart';
import 'package:logging/logging.dart';

void main() {
    // テレメトリーを初期化する（ログ設定 + OpenTelemetry トレース）
    initTelemetry(TelemetryConfig(
        serviceName: 'order-client',
        version: '1.0.0',
        tier: 'service',
        environment: 'dev',
        traceEndpoint: 'http://localhost:4317',
        sampleRate: 1.0,
        logLevel: 'info',
        logFormat: 'json',
    ));

    // Logger でログを出力する（JSON 形式で標準出力に出力される）
    final logger = Logger('order-client');
    logger.info('注文クライアントを起動中');

    // アプリケーション終了時にシャットダウンする
    shutdown();
}
```

---

## 4. ページネーションライブラリ（pagination）

**やりたいこと**: 一覧取得 API でページネーション（オフセットまたはカーソルベース）を実装する。
**ライブラリ**: Go → `pagination`パッケージ / Rust → `k1s0-pagination` クレート / TypeScript → 実装を確認して記載 / Dart → 実装を確認して記載
**詳細設計**: [pagination.md](../data/pagination.md)

### Go（オフセットベースページネーション）

```go
import (
    // pagination パッケージをインポートする
    "github.com/k1s0-platform/system-library-go-pagination"
)

func ListOrdersHandler(w http.ResponseWriter, r *http.Request) {
    // クエリパラメータからページネーション情報を取得する
    pageStr := r.URL.Query().Get("page")
    perPageStr := r.URL.Query().Get("per_page")

    // バリデーション: per_page は 1〜100 の範囲であること
    perPage := uint32(20)
    if perPageStr != "" {
        v, _ := strconv.Atoi(perPageStr)
        if err := pagination.ValidatePerPage(uint32(v)); err != nil {
            http.Error(w, "per_page must be 1-100", http.StatusBadRequest)
            return
        }
        perPage = uint32(v)
    }

    page := uint32(1)
    if pageStr != "" {
        v, _ := strconv.Atoi(pageStr)
        page = uint32(v)
    }

    // PageRequest を生成する（デフォルト: page=1, perPage=20）
    req := pagination.NewPageRequest(page, perPage)

    // DB からデータを取得する（Offset() でオフセット値を取得する）
    orders, err := db.FetchOrders(ctx, req.Offset(), req.PerPage)
    if err != nil {
        http.Error(w, "Internal Server Error", http.StatusInternalServerError)
        return
    }
    total, _ := db.CountOrders(ctx)

    // PageResponse を生成する（TotalPages は自動計算される）
    resp := pagination.NewPageResponse(orders, total, req)
    meta := resp.Meta()

    json.NewEncoder(w).Encode(map[string]any{
        "items":       resp.Items,
        "total":       meta.Total,
        "page":        meta.Page,
        "per_page":    meta.PerPage,
        "total_pages": meta.TotalPages,
    })
}
```

**カーソルベースページネーション（Go）**:

```go
// カーソルをエンコードする（sort_key と id の 2 引数）
cursor := pagination.EncodeCursor("2024-01-15T10:30:00Z", "order-456")

// カーソルをデコードする
sortKey, id, err := pagination.DecodeCursor(cursor)
if err != nil {
    http.Error(w, "Invalid cursor", http.StatusBadRequest)
    return
}
_ = sortKey
_ = id
```

### Rust（オフセットベースページネーション）

```rust
use k1s0_pagination::{PageRequest, PageResponse, validate_per_page, encode_cursor, decode_cursor};

async fn list_orders(page: u64, per_page: u64) -> Result<PageResponse<Order>, AppError> {
    // per_page バリデーション（1〜100 の範囲）
    validate_per_page(per_page as u32)?;

    // PageRequest を生成する
    let req = PageRequest { page, per_page };

    // DB からデータを取得する
    let offset = req.offset(); // (page - 1) * per_page
    let orders = db.fetch_orders(offset, req.per_page).await?;
    let total = db.count_orders().await?;

    // PageResponse を生成する（total_pages は自動計算される）
    let response = PageResponse::new(orders, total, &req);
    let meta = response.meta(); // PaginationMeta

    println!("page={}/{}", meta.page, meta.total_pages);
    Ok(response)
}
```

**カーソルベースページネーション（Rust）**:

```rust
// カーソルをエンコードする（sort_key と id の 2 引数; base64url 形式）
let cursor = encode_cursor("2024-01-15T10:30:00Z", "order-456");

// カーソルをデコードする
let (sort_key, id) = decode_cursor(&cursor)?;
println!("sort_key={sort_key}, id={id}");
```

### TypeScript（オフセットベースページネーション）

```typescript
// TypeScript 実装のパッケージ名・関数名は
// regions/system/library/typescript/pagination/ の実装を確認して使用すること
import {
  createPageResponse,
  validatePerPage,
  defaultPageRequest,
  pageOffset,
} from '@k1s0/pagination';  // パッケージ名は実装を確認して記載

async function listOrders(page: number, perPage: number) {
  // per_page バリデーション（範囲外は PerPageValidationError をスロー）
  validatePerPage(perPage);

  // ページリクエストを構築する
  const req = { page, perPage };
  const offset = pageOffset(req); // (page - 1) * perPage

  // DB からデータを取得する
  const orders = await db.fetchOrders(offset, perPage);
  const total = await db.countOrders();

  // PageResponse を生成する
  const resp = createPageResponse(orders, total, req);
  const meta = resp.meta();

  return {
    items: resp.items,
    total: meta.total,
    page: meta.page,
    perPage: meta.perPage,
    totalPages: meta.totalPages,
  };
}
```

**カーソルベースページネーション（TypeScript）**:

```typescript
import { encodeCursor, decodeCursor } from '@k1s0/pagination';

// カーソルをエンコードする（base64url 形式）
const cursor = encodeCursor('2024-01-15T10:30:00Z', 'order-456');

// カーソルをデコードする
const { sortKey, id } = decodeCursor(cursor);
```

### Dart（オフセットベースページネーション）

```dart
import 'package:k1s0_pagination/k1s0_pagination.dart';

Future<Map<String, dynamic>> listOrders(int page, int perPage) async {
    // per_page バリデーション（範囲外は PerPageValidationException をスロー）
    validatePerPage(perPage);

    // PageRequest を生成する
    final req = PageRequest(page: page, perPage: perPage);
    final offset = req.offset; // (page - 1) * perPage

    // DB からデータを取得する
    final orders = await db.fetchOrders(offset, perPage);
    final total = await db.countOrders();

    // PageResponse を生成する
    final resp = PageResponse.create(orders, total, req);
    final meta = resp.meta;

    return {
        'items': resp.items,
        'total': meta.total,
        'page': meta.page,
        'per_page': meta.perPage,
        'total_pages': meta.totalPages,
    };
}
```

**カーソルベースページネーション（Dart）**:

```dart
// カーソルをエンコードする（base64url 形式）
final cursor = encodeCursor('2024-01-15T10:30:00Z', 'order-456');

// カーソルをデコードする（Record 型で返す）
final (:sortKey, :id) = decodeCursor(cursor);
```

---

## 関連ドキュメント

- [概要.md](./概要.md) — ライブラリ一覧と用途
- [共通実装パターン.md](./共通実装パターン.md) — ディレクトリ構成・Cargo.toml 依存追加方法
- [auth.md](../auth-security/auth.md) — 認証ライブラリ詳細設計
- [config.md](../config/config.md) — 設定ライブラリ詳細設計
- [telemetry.md](../observability/telemetry.md) — telemetry ライブラリ詳細設計
- [pagination.md](../data/pagination.md) — pagination ライブラリ詳細設計
