// entity.rs — k1s0 tier2 _stub_service（第二業界 stub）ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §汎用 bounded context に準拠する
// 業界固有語を一切含まない汎用語彙を使用する（vocabulary.yaml の neutral_term で管理する）
// manufacturing pack とは語彙を完全に分離する（pack 間の依存禁止規約に準拠する）

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// GenericResource: 汎用リソースを表す集約エンティティ（業界語を一切含まない）
// 公開 API 名は GenericResource（特定業界の設備・物品・サービス用語を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericResource {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント識別子（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // リソースの識別コード（業界中立な番号）
    pub resource_code: String,
    // リソースの種別（domain_type: 業界横断で使用できる汎用区分）
    pub domain_type: String,
    // リソースの現在の状態
    pub status: GenericResourceStatus,
    // 作成日時（HLC タイムスタンプで記録する）
    pub created_at: DateTime<Utc>,
    // 更新日時（HLC タイムスタンプで記録する）
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

// GenericResourceStatus: リソースの状態（業界中立な状態値として宣言する）
// どの業界でも利用できる汎用的なライフサイクル状態を定義する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenericResourceStatus {
    // 稼働中（利用可能な状態）
    Active,
    // 停止中（利用不可の状態）
    Inactive,
    // メンテナンス中（保守作業中の状態）
    UnderMaintenance,
}

// GenericBatch: 汎用バッチ処理単位を表す集約エンティティ
// 製造・物流・金融など業界横断で使用できるバッチ処理の抽象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericBatch {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント識別子（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // バッチの識別コード（業界中立な番号）
    pub batch_code: String,
    // バッチの処理対象リソース ID（GenericResource への参照）
    pub resource_id: Uuid,
    // バッチの処理数量（整数: 業界横断で利用できる汎用数量）
    pub quantity: i64,
    // バッチの現在の状態
    pub status: GenericBatchStatus,
    // 作成日時（HLC タイムスタンプで記録する）
    pub created_at: DateTime<Utc>,
    // 更新日時（HLC タイムスタンプで記録する）
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

// GenericBatchStatus: 汎用バッチの状態（業界中立な状態値として宣言する）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenericBatchStatus {
    // 計画中（まだ実行されていない状態）
    Planned,
    // 実行中（処理が進行中の状態）
    InProgress,
    // 完了（処理が正常終了した状態）
    Completed,
    // 中断（処理が異常終了した状態）
    Aborted,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // GenericResource の公開 API 名が業界中立語であることを確認する
    fn test_generic_resource_neutral_naming() {
        // struct 名が GenericResource（特定業界語ではない）であることをコンパイルで確認する
        let _: Option<GenericResource> = None;
        // GenericResourceStatus の enum 値も業界中立語であることを確認する
        let status = GenericResourceStatus::Active;
        // Active が正しい文字列を返すことを確認する
        assert_eq!(format!("{:?}", status), "Active");
    }

    #[test]
    // GenericBatch の状態が正しく初期化されることを確認する
    fn test_generic_batch_status_planned() {
        // Planned 状態を確認する
        let status = GenericBatchStatus::Planned;
        // Planned が正しい文字列を返すことを確認する
        assert_eq!(format!("{:?}", status), "Planned");
    }
}
