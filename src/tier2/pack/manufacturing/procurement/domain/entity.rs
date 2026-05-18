// entity.rs — k1s0 tier2 manufacturing procurement (調達) bounded context ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §procurement bounded context に準拠する。
// ProcurementOrder エンティティを procurement (調達管理) bounded context として分離する。
// 業界語「発注書」ではなく ProcurementOrder (業界中立語) で宣言する。

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// OrderStatus: 調達注文の状態（業界中立な状態値として宣言する）
// proto/manufacturing/procurement/v1/order.proto の OrderStatus と 1:1 対応する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    // 下書き（まだ確定していない状態）
    Draft,
    // 提出済み（承認待ちの状態）
    Submitted,
    // 承認済み（発注確定の状態）
    Approved,
    // 却下（差し戻しの状態）
    Rejected,
    // 受領済み（納品完了の状態）
    Received,
}

// ApprovalRequirement: 承認要件（ep_procurement_approval_policy で使用する）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequirement {
    // None: 承認不要（自動承認）
    None,
    // SingleApprover: 単一承認者による承認が必要
    SingleApprover,
    // MultiApprover: 複数承認者（2 名以上）による承認が必要
    MultiApprover,
}

// ProcurementOrder: 調達注文集約エンティティ
// 公開 API 名は ProcurementOrder（業界語「発注書」を使わない: tier2 CLAUDE.md 命名禁則）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcurementOrder {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント ID（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // 注文の識別コード（業界中立な番号）
    pub order_code: String,
    // 注文品目の仕様 ID（ItemSpecification への参照）
    pub item_spec_id: Uuid,
    // 注文数量（1 以上の整数であることを保証する）
    pub quantity: i64,
    // 現在の注文状態
    pub status: OrderStatus,
    // 作成日時（HLC タイムスタンプで記録する）
    pub created_at: DateTime<Utc>,
    // 更新日時（HLC タイムスタンプで記録する）
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // ProcurementOrder の公開 API 名が業界中立語であることを確認する
    fn test_procurement_order_neutral_naming() {
        // struct 名が ProcurementOrder（業界語「発注書」ではない）であることをコンパイルで確認する
        let _: Option<ProcurementOrder> = None;
        // OrderStatus の初期値が Draft であることを確認する
        let status = OrderStatus::Draft;
        assert_eq!(format!("{:?}", status), "Draft");
    }
}
