// k1s0-tier2-integration クレートのルートモジュール（設計方針 16 / 外部システム統合）
// Webhook 配信および OAuth Token Exchange を公開 API として提供する

// Webhook 配信モジュール（HMAC-SHA256 署名付き）
pub mod webhook;
// OAuth Token Exchange モジュール（RFC 8693 準拠）
pub mod oauth_token_exchange;

// 公開型の再エクスポート（tier2-integration 公開 API 表面を最小化する）
pub use webhook::WebhookDelivery;
// OAuthTokenExchange と TokenExchangeResponse を公開する
pub use oauth_token_exchange::{OAuthTokenExchange, TokenExchangeResponse};
