// entity.rs — k1s0 tier2 _stub_service FA bounded context ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §汎用 FA bounded context に準拠する
// _stub_service の FA bounded context は業界固有語を含まない汎用語彙のみ使用する
// manufacturing pack の fa/domain/entity.rs とは語彙を完全に分離する（pack 間依存禁止）

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// OperationalUnitStatus: 操作単位の状態（業界中立な状態値として宣言する）
// どの業界でも利用できる汎用的な操作状態を定義する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalUnitStatus {
    // 稼働中（正常稼働）
    Running,
    // 停止中（電源断・計画停止）
    Stopped,
    // 一時停止中（保守作業・点検中）
    Suspended,
}

// OperationalUnit: 汎用操作単位集約エンティティ（業界語を含まない FA bounded context）
// 公開 API 名は OperationalUnit（特定業界の設備・装置・機器などの業界語を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalUnit {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント識別子（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // 操作単位の識別コード（業界中立な番号）
    pub unit_code: String,
    // 操作単位の現在の状態
    pub status: OperationalUnitStatus,
    // 作成日時（HLC タイムスタンプで記録する）
    pub created_at: DateTime<Utc>,
    // 更新日時（HLC タイムスタンプで記録する）
    pub updated_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

// CapacityLimit: 操作単位の容量上限制約（ep_capacity_constraint_provider で使用する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityLimit {
    // 制約対象の操作単位識別子
    pub unit_id: Uuid,
    // 同時実行可能なバッチの最大数
    pub max_concurrent_batches: u32,
    // 制約の有効開始日時（HLC タイムスタンプ）
    pub valid_from: DateTime<Utc>,
    // 制約の有効終了日時（HLC タイムスタンプ: None の場合は無期限）
    pub valid_until: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // OperationalUnit の公開 API 名が業界中立語であることを確認する
    fn test_operational_unit_neutral_naming() {
        // struct 名が OperationalUnit（業界語ではない）であることをコンパイルで確認する
        let _: Option<OperationalUnit> = None;
        // OperationalUnitStatus の enum 値も業界中立語であることを確認する
        let status = OperationalUnitStatus::Running;
        // Running が正しい文字列を返すことを確認する
        assert_eq!(format!("{:?}", status), "Running");
    }
}
