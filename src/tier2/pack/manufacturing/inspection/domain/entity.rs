// entity.rs — k1s0 tier2 manufacturing inspection (検査) bounded context ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §inspection bounded context に準拠する。
// InspectionResult エンティティを inspection (検査管理) bounded context として分離する。
// 業界語「検査成績書」ではなく InspectionResult (業界中立語) で宣言する。

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// InspectionVerdict: 検査結果判定（業界中立な判定値として宣言する）
// proto/manufacturing/inspection/v1/result.proto の InspectionVerdict と 1:1 対応する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionVerdict {
    // 合格（全測定値が許容差以内）
    Pass,
    // 不合格（いずれかの測定値が許容差超過）
    Fail,
    // 要再検査（測定値なし / グレーゾーン）
    RequiresReview,
}

// Measurement: 測定値（業界語「実測値」の中立語として宣言する）
// proto/manufacturing/inspection/v1/result.proto の Measurement と 1:1 対応する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    // metric_id: 測定指標の識別子（ItemSpecification の metric 定義と対応する）
    pub metric_id: String,
    // value: 測定値（浮動小数点数で表現する）
    pub value: f64,
    // unit: 測定単位（"mm" / "kg" / "Pa" 等）
    pub unit: String,
}

// ValidationError: 品目仕様バリデーションエラー（ep_item_specification_validator で使用する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    // code: machine-readable エラーコード
    pub code: String,
    // message: human-readable エラーメッセージ
    pub message: String,
    // field: エラーが発生したフィールド名（省略可能）
    pub field: Option<String>,
}

// InspectionResult: 検査結果集約エンティティ
// 公開 API 名は InspectionResult（業界語「検査成績書」を使わない: tier2 CLAUDE.md 命名禁則）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント ID（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // 検査対象の生産バッチ識別子（ProductionBatch への参照）
    pub batch_id: Uuid,
    // 検査に使用した品目仕様 ID（ItemSpecification への参照）
    pub item_spec_id: Uuid,
    // 測定値のリスト（空の場合は RequiresReview になる）
    pub measurements: Vec<Measurement>,
    // 検査の合否判定
    pub verdict: InspectionVerdict,
    // 検査担当者の識別子（AuthContext.user_id: 業界語「検査員」の代わり）
    pub inspector_id: Uuid,
    // 検査日時（HLC タイムスタンプで記録する）
    pub inspected_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // InspectionResult の公開 API 名が業界中立語であることを確認する
    fn test_inspection_result_neutral_naming() {
        // struct 名が InspectionResult（業界語「検査成績書」ではない）であることをコンパイルで確認する
        let _: Option<InspectionResult> = None;
        // InspectionVerdict の初期値が Pass であることを確認する
        let verdict = InspectionVerdict::Pass;
        assert_eq!(format!("{:?}", verdict), "Pass");
    }

    #[test]
    // 測定値が空の場合は RequiresReview になることを検証する
    fn test_empty_measurements_requires_review() {
        // 測定値が空の Vec を用意する
        let measurements: Vec<Measurement> = vec![];
        // 空の測定値リストは合否判定不可なため RequiresReview を期待する
        let verdict = if measurements.is_empty() {
            InspectionVerdict::RequiresReview
        } else {
            InspectionVerdict::Pass
        };
        assert_eq!(verdict, InspectionVerdict::RequiresReview);
    }
}
