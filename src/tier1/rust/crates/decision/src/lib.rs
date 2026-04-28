// 本ファイルは k1s0-tier1-decision の library エントリポイント。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-008（t1-decision Pod、ZEN Engine ベース JDM 評価）
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md

// JDM ルール registry と evaluator。
pub mod registry;
