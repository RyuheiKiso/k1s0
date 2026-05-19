// k1s0-tier2-integration クレートのルートモジュール（設計方針 16 / 外部システム統合）
// Webhook 配信 / OAuth Token Exchange / EDI X12 / Modbus / OPC-UA アダプタを提供する

// Webhook 配信モジュール（HMAC-SHA256 署名付き）
pub mod webhook;
// OAuth Token Exchange モジュール（RFC 8693 準拠）
pub mod oauth_token_exchange;
// ANSI X12 EDI 取引先電文アダプタ（850 発注トランザクション）
pub mod edi_x12;
// 検査機器 Modbus TCP アダプタ（holding register フレーム変換）
pub mod inspection_machine_modbus;
// SCADA OPC-UA アダプタ（DataChange notification 変換）
pub mod scada_opcua;

// 公開型の再エクスポート（tier2-integration 公開 API 表面を最小化する）
pub use webhook::WebhookDelivery;
// OAuthTokenExchange と TokenExchangeResponse を公開する
pub use oauth_token_exchange::{OAuthTokenExchange, TokenExchangeResponse};
// EDI X12 アダプタの公開型を再エクスポートする
pub use edi_x12::{X12Adapter, X12850PurchaseOrder, MappedPurchaseOrder};
// Modbus アダプタの公開型を再エクスポートする
pub use inspection_machine_modbus::{ModbusAdapter, ModbusHoldingRegisterFrame, MappedInspectionResult};
// OPC-UA アダプタの公開型を再エクスポートする
pub use scada_opcua::{OpcUaAdapter, OpcUaDataChangeNotification, MappedProductionProgress};
