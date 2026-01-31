# k1s0 Framework

開発基盤チームが提供する共通部品（crate/ライブラリ/パッケージ）および共通マイクロサービス。

## ディレクトリ構成

```
framework/
├── backend/
│   ├── rust/
│   │   ├── crates/           # 共通 crate 群（12 crates）
│   │   └── services/         # 共通マイクロサービス
│   │       ├── auth-service/
│   │       ├── config-service/
│   │       └── endpoint-service/
│   ├── go/                   # Go パッケージ群
│   ├── csharp/               # C# NuGet パッケージ群
│   ├── python/               # Python パッケージ群（uv）
│   └── kotlin/               # Kotlin パッケージ群（Gradle）
├── frontend/
│   ├── react/
│   │   └── packages/         # React 共通パッケージ
│   ├── flutter/
│   │   └── packages/         # Flutter 共通パッケージ
│   └── android/
│       └── packages/         # Android 共通パッケージ
└── database/
    └── table/                # 共通テーブル定義（DDL 正本）
```

## 提供物

### Backend Crates/Packages

全バックエンド言語（Rust, Go, C#, Python, Kotlin）で共通の12パッケージを提供。

| パッケージ | 説明 | Tier |
|-----------|------|------|
| k1s0-error | エラー表現の統一（RFC 7807） | 1 |
| k1s0-config | 設定読み込み（`--env`/`--config`/`--secrets-dir`） | 1 |
| k1s0-validation | 入力バリデーション | 1 |
| k1s0-observability | ログ/トレース/メトリクス（OpenTelemetry） | 2 |
| k1s0-grpc-server | gRPC サーバー共通基盤 | 2 |
| k1s0-grpc-client | gRPC クライアント共通 | 2 |
| k1s0-resilience | リトライ/サーキットブレーカー/タイムアウト/バルクヘッド | 2 |
| k1s0-health | ヘルスチェック（liveness/readiness/startup） | 2 |
| k1s0-db | DB 接続・トランザクション | 2 |
| k1s0-cache | Redis キャッシュ | 2 |
| k1s0-domain-event | ドメインイベント publish/subscribe/outbox | 2 |
| k1s0-auth | JWT/OIDC 認証・ポリシーベース認可 | 3 |

**Tier 依存ルール:**
- Tier 1: フレームワーク依存なし
- Tier 2: Tier 1 のみ依存可
- Tier 3: Tier 1 および Tier 2 に依存可

### 言語別の技術スタック

| 言語 | 主要技術 |
|------|---------|
| Rust | axum, tokio, tonic, SQLx |
| Go | gRPC-Go, go-playground/validator, zap |
| C# | ASP.NET Core 8.0, EF Core, StackExchange.Redis |
| Python | FastAPI, SQLAlchemy + asyncpg, grpcio, Pydantic |
| Kotlin | Ktor 3.x, Exposed + HikariCP, grpc-kotlin, Lettuce |

### Frontend Packages

**React（10 パッケージ）:**

| パッケージ | 説明 |
|-----------|------|
| @k1s0/navigation | 設定駆動ルーティング |
| @k1s0/config | YAML 設定管理 |
| @k1s0/api-client | HTTP/gRPC API クライアント |
| @k1s0/ui | Design System（MUI ベース）、DataTable、Form Generator |
| @k1s0/shell | AppShell（Header/Sidebar/Footer） |
| @k1s0/auth-client | 認証クライアント |
| @k1s0/observability | フロントエンド OTel/ログ |
| @k1s0/realtime | WebSocket/SSE クライアント |
| eslint-config-k1s0 | ESLint ルール |
| tsconfig-k1s0 | 共有 TypeScript 設定 |

**Flutter（8 パッケージ）:**

| パッケージ | 説明 |
|-----------|------|
| k1s0_navigation | 設定駆動ルーティング（go_router ベース） |
| k1s0_config | YAML 設定管理 |
| k1s0_http | HTTP クライアント（Dio ベース） |
| k1s0_ui | Design System（Material 3）、DataTable、Form Generator |
| k1s0_auth | 認証クライアント（JWT/OIDC） |
| k1s0_observability | 構造化ログ、トレース |
| k1s0_state | Riverpod 状態管理ユーティリティ |
| k1s0_realtime | WebSocket/SSE クライアント |

**Android（8 パッケージ）:**

| パッケージ | 説明 |
|-----------|------|
| k1s0-navigation | Navigation Compose ルーティング |
| k1s0-config | YAML 設定管理 |
| k1s0-http | Ktor Client HTTP |
| k1s0-ui | Material 3 Design System |
| k1s0-auth | JWT 認証クライアント |
| k1s0-observability | ログ/トレース |
| k1s0-state | ViewModel + StateFlow ユーティリティ |
| k1s0-realtime | WebSocket/SSE クライアント |

### 共通サービス

| サービス | 説明 | 所有テーブル |
|---------|------|-------------|
| auth-service | 認証・認可 | `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission` |
| config-service | 動的設定 | `fw_m_setting` |
| endpoint-service | エンドポイント管理 | `fw_m_endpoint` |

## 依存方向

```
feature → domain → framework   ✓（許可）
framework → domain             ✗（禁止）
framework → feature            ✗（禁止）
```

## 関連ドキュメント

- [Framework 設計書](../docs/design/framework.md)
- [規約: サービス構成](../docs/conventions/service-structure.md)
- [規約: 設定と秘密情報](../docs/conventions/config-and-secrets.md)
