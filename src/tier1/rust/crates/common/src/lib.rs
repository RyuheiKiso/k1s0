// 本ファイルは k1s0-tier1-common crate のライブラリエントリポイント。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md（§ 共通規約）
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//
// crate は Go 側 `src/tier1/go/internal/common/` の Rust 等価物。3 Pod
// （t1-decision / t1-audit / t1-pii）が共通で使う横断機能を集約する。
//
// module 構成:
//   - idempotency:  24h TTL 重複抑止 cache（共通規約 §「冪等性と再試行」）
//   - tenant:       TenantContext 検証 helper（NFR-E-AC-003）
//   - auth:         JWT 認証 interceptor（off / hmac / jwks の 3 mode）
//   - ratelimit:    token bucket 形式のテナント単位 rate limit
//   - observability: gRPC 共通 interceptor（tracing span + RED 計数）
//   - audit:        privileged RPC の自動 audit 発火 interceptor
//   - http_gateway: HTTP/JSON ↔ gRPC 等価ハンドラ
//   - interceptors: 全 interceptor をまとめる helper

// 重複抑止 cache。
pub mod idempotency;
// テナント context 検証。
pub mod tenant;
// gRPC 認証 interceptor。
pub mod auth;
// gRPC 観測性 interceptor。
pub mod observability;
// gRPC レートリミット interceptor。
pub mod ratelimit;
// gRPC 監査 interceptor。
pub mod audit;
// HTTP/JSON gateway 実装（axum）。
pub mod http_gateway;
