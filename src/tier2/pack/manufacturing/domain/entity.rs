// k1s0 tier2 manufacturing pack ドメインエンティティ骨格
// 公開 API は業界中立語（ResourceUnit / ProductionBatch 等）を使用する
// vocabulary.yaml の neutral_term を公開 API 名として採用する

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// ResourceUnit: 製造設備を表す集約エンティティ（業界語「設備」を neutral_term で隠蔽）
// 公開 API 名は ResourceUnit（業界語「設備」を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUnit {
    // エンティティ主キー（UUID v4）
    pub id: Uuid,
    // テナント ID（RLS が自動注入するため、公開コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // リソースの識別コード（業界中立な番号）
    pub resource_code: String,
    // リソースの状態（active / inactive / maintenance）
    pub status: ResourceStatus,
    // 作成日時
    pub created_at: DateTime<Utc>,
    // 更新日時
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン
    pub version: i64,
}

// ResourceStatus: リソースの操業状態（業界中立な状態値）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    // 稼働中
    Active,
    // 停止中
    Inactive,
    // メンテナンス中
    Maintenance,
}

// ProductionBatch: 生産バッチを表す集約エンティティ（業界語「ロット」を neutral_term で隠蔽）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionBatch {
    // エンティティ主キー
    pub id: Uuid,
    // テナント ID（RLS が自動注入する）
    pub(crate) tenant_id: Uuid,
    // バッチ識別コード
    pub batch_code: String,
    // 対象アイテム仕様の ID（ItemSpecification への参照）
    pub item_spec_id: Uuid,
    // バッチの生産数量
    pub quantity: i64,
    // バッチ状態
    pub status: BatchStatus,
    // 作成日時
    pub created_at: DateTime<Utc>,
    // 更新日時
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン
    pub version: i64,
}

// BatchStatus: 生産バッチの状態（業界中立な状態値）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    // 計画中
    Planned,
    // 生産中
    InProgress,
    // 完了
    Completed,
    // 不合格
    Rejected,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // ResourceUnit の公開 API 名が業界中立語であることを確認する
    fn test_resource_unit_neutral_naming() {
        // struct 名が ResourceUnit（業界語「設備」ではない）であることをコンパイルで確認
        let _unit: ResourceUnit;
        // BatchStatus の enum 値も業界中立語であることを確認する
        let status = BatchStatus::InProgress;
        assert_eq!(format!("{:?}", status), "InProgress");
    }
}
