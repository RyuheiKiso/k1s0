//! lib.rs — 製造業 pack stress test クレート エントリポイント
//! テナント容量適合仕様 09 §製造業 pack stress test 8 シナリオを提供する
//! このクレートは kind クラスタ上の統合テストを実行するために使用する

// fixture モジュール: テスト用フィクスチャデータ (テナント ID / domain オブジェクト)
pub mod fixture;
// scenarios モジュール: ストレステストシナリオ 8 件 (s01-s08)
pub mod scenarios;
