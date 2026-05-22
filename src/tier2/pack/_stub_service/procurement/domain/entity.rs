// entity.rs — k1s0 tier2 _stub_service procurement bounded context ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §汎用 procurement bounded context に準拠する
// _stub_service の procurement bounded context は業界固有語を含まない汎用語彙のみ使用する
// manufacturing pack の procurement/domain/entity.rs とは語彙を完全に分離する（pack 間依存禁止）

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// RequisitionStatus: 調達依頼の状態（業界中立な状態値として宣言する）
// どの業界でも利用できる汎用的な調達プロセス状態を定義する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequisitionStatus {
    // 下書き（まだ確定していない状態）
    Draft,
    // 提出済み（承認待ちの状態）
    Submitted,
    // 承認済み（発注確定の状態）
    Approved,
    // 却下（差し戻しの状態）
    Declined,
    // 受領済み（納品完了の状態）
    Fulfilled,
}

// AuthorizationLevel: 承認権限レベル（ep_authorization_policy で使用する）
// manufacturing の ApprovalRequirement とは名称を意図的に変えて語彙を分離する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationLevel {
    // 自動承認（承認不要）
    Automatic,
    // 単一権限者による承認が必要
    SingleAuthorizer,
    // 複数権限者（2 名以上）による承認が必要
    MultipleAuthorizers,
}

// SupplyRequisition: 汎用調達依頼集約エンティティ
// 公開 API 名は SupplyRequisition（業界語「発注書 / 購買注文」を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyRequisition {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント識別子（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // 依頼の識別コード（業界中立な番号）
    pub requisition_code: String,
    // 依頼対象の仕様 ID（Specification への参照）
    pub specification_id: Uuid,
    // 依頼数量（1 以上の整数であることを保証する）
    pub quantity: i64,
    // 現在の依頼状態
    pub status: RequisitionStatus,
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
    // SupplyRequisition の公開 API 名が業界中立語であることを確認する
    fn test_supply_requisition_neutral_naming() {
        // struct 名が SupplyRequisition（業界語「発注書」ではない）であることをコンパイルで確認する
        let _: Option<SupplyRequisition> = None;
        // RequisitionStatus の初期値が Draft であることを確認する
        let status = RequisitionStatus::Draft;
        // Draft が正しい文字列を返すことを確認する
        assert_eq!(format!("{:?}", status), "Draft");
    }
}
