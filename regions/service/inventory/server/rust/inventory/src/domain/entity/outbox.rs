/// Outbox イベントエンティティ。
///
/// k1s0-outbox ライブラリの OutboxEvent をそのまま再エクスポートする。
/// 各サービスで同一構造の OutboxEvent を重複定義していたのを共通化した。
pub use k1s0_outbox::OutboxEvent;
