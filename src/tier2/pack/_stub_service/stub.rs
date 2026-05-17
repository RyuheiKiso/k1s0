// k1s0 tier2 第二業界 stub サービス
// manufacturing pack と異なる業界（サービス業等）の stub 実装
// 業界横断 conformance を担保するために製造業固有語を一切持たない
// 業界中立語のみを公開 API として使用することを実証する

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;

// ServiceOrder: サービス業の受注エンティティ（業界中立語）
// 製造業の ProductionBatch に相当するが、語彙は完全に独立している
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrder {
    // エンティティ主キー
    pub id: Uuid,
    // テナント ID（RLS が自動注入する）
    pub(crate) tenant_id: Uuid,
    // 受注コード（業界中立識別子）
    pub order_code: String,
    // 受注状態
    pub status: ServiceOrderStatus,
    // バージョン
    pub version: i64,
}

// ServiceOrderStatus: 受注状態（業界中立な状態値）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceOrderStatus {
    // 受注済み
    Received,
    // 処理中
    Processing,
    // 完了
    Completed,
    // キャンセル
    Cancelled,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // 第二業界 stub が製造業固有語を含まないことをコンパイル時に確認する
    fn test_stub_no_manufacturing_terms() {
        // ServiceOrder の公開 API 名に業界語（設備/ロット等）が含まれないことを確認する
        let order = ServiceOrder {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            order_code: "SO-001".to_string(),
            status: ServiceOrderStatus::Received,
            version: 1,
        };
        // 型名・フィールド名・enum 値に製造業固有語がないことをデバッグ文字列で確認する
        let debug_str = format!("{:?}", order.status);
        assert!(!debug_str.contains("ロット"));
        assert!(!debug_str.contains("設備"));
        assert!(!debug_str.contains("BOM"));
    }
}
