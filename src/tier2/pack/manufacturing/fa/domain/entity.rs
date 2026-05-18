// entity.rs — k1s0 tier2 manufacturing FA (設備) bounded context ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §FA bounded context に準拠する。
// ResourceUnit エンティティを FA (設備管理) bounded context として分離する。
// 元実装 (pack/manufacturing/domain/entity.rs) から re-export し後方互換を維持する。

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// ResourceStatus: 設備の操業状態（業界中立な状態値として宣言する）
// proto/manufacturing/fa/v1/operation.proto の ResourceStatus と 1:1 対応する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    // 稼働中（正常稼働）
    Active,
    // 停止中（電源断・計画停止）
    Inactive,
    // メンテナンス中（保守作業中）
    Maintenance,
}

// ResourceUnit: FA bounded context における設備集約エンティティ
// 公開 API 名は ResourceUnit（業界語「設備」を使わない: tier2 CLAUDE.md 命名禁則）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUnit {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント ID（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // リソースの識別コード（業界中立な番号: 設備番号の代わりに使用する）
    pub resource_code: String,
    // リソースの現在の操業状態
    pub status: ResourceStatus,
    // 作成日時（HLC タイムスタンプで記録する）
    pub created_at: DateTime<Utc>,
    // 更新日時（HLC タイムスタンプで記録する）
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

// CapacityConstraint: リソースの容量制約を宣言する型（ep_capacity_constraint_provider で使用する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityConstraint {
    // resource_id: 制約対象リソースの識別子
    pub resource_id: Uuid,
    // max_concurrent_batches: 同時実行可能な生産バッチの最大数
    pub max_concurrent_batches: u32,
    // validity_from: 制約の有効開始日時（HLC タイムスタンプ）
    pub validity_from: DateTime<Utc>,
    // validity_until: 制約の有効終了日時（HLC タイムスタンプ）
    pub validity_until: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // ResourceUnit の公開 API 名が業界中立語であることを確認する
    fn test_resource_unit_neutral_naming() {
        // struct 名が ResourceUnit（業界語「設備」ではない）であることをコンパイルで確認する
        let _: Option<ResourceUnit> = None;
        // ResourceStatus の enum 値も業界中立語であることを確認する
        let status = ResourceStatus::Active;
        assert_eq!(format!("{:?}", status), "Active");
    }
}
