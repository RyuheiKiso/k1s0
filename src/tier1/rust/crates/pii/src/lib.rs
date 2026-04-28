// 本ファイルは k1s0-tier1-pii の library エントリポイント。
//
// crate は bin（t1-pii Pod）と lib の両方をビルドする。lib 側は単体テスト
// や将来の Go ファサードからの直接 link 用途で再利用できるよう、masker
// module を public 公開する。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-009（t1-pii Pod、純関数 / ステートレス）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md

// PII 検出 / マスキングの中核ロジック。
pub mod masker;
