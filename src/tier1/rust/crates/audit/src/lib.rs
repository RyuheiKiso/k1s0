// 本ファイルは k1s0-tier1-audit の library エントリポイント。
//
// crate は bin（t1-audit Pod）と lib の両方をビルドする。lib 側は単体テストや
// 将来 Postgres backend 追加時の interface 共有のため、store module を public 公開する。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、WORM 追記専用）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// audit_id の生成方針:
//   audit_id = sha256( prev_hash || canonical_event_bytes ) を 16 進文字列で表現する。
//   これにより各イベントは前のイベントとハッシュチェーンで連結され、
//   過去のイベント書き換えは末尾までの再計算が必要になる（WORM 改竄検知）。

// WORM ハッシュチェーンと in-memory store を提供する module。
pub mod store;
// Export RPC の chunk 整形ヘルパ（CSV / NDJSON / JSON 配列フォーマッタ + chunk 送出ループ）。
pub mod export;
// AuditService trait 実装本体。
pub mod server;
