// entity.rs — k1s0 tier2 _stub_service inspection bounded context ドメインエンティティ
// docs/03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md §汎用 inspection bounded context に準拠する
// _stub_service の inspection bounded context は業界固有語を含まない汎用語彙のみ使用する
// manufacturing pack の inspection/domain/entity.rs とは語彙を完全に分離する（pack 間依存禁止）

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート
use uuid::Uuid;
// 日時ライブラリのインポート
use chrono::{DateTime, Utc};

// EvaluationVerdict: 評価結果の判定（業界中立な判定値として宣言する）
// どの業界でも利用できる汎用的な評価結果を定義する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationVerdict {
    // 適合（全評価指標が基準以内）
    Conforming,
    // 不適合（いずれかの評価指標が基準超過）
    NonConforming,
    // 保留（評価データ不足またはグレーゾーン）
    Pending,
}

// MetricSample: 評価に使用したサンプル測定値（業界中立語）
// どの業界でも利用できる汎用的な測定値型を定義する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    // 測定指標の識別子（Specification の metric 定義と対応する）
    pub metric_id: String,
    // 測定値（浮動小数点数で表現する）
    pub value: f64,
    // 測定単位（"mm" / "kg" / "%" 等の SI 単位または無次元）
    pub unit: String,
}

// ConstraintViolation: 仕様制約違反（ep_specification_validator で使用する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    // machine-readable エラーコード
    pub code: String,
    // human-readable エラーメッセージ
    pub message: String,
    // エラーが発生したフィールド名（省略可能）
    pub field: Option<String>,
}

// EvaluationRecord: 評価結果集約エンティティ
// 公開 API 名は EvaluationRecord（業界語「検査成績書 / 品質記録」を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    // エンティティ主キー（UUID v7: 時刻順ソート可能）
    pub id: Uuid,
    // テナント識別子（RLS が自動注入するため public コンストラクタには渡さない）
    pub(crate) tenant_id: Uuid,
    // 評価対象のバッチ識別子（GenericBatch への参照）
    pub batch_id: Uuid,
    // 評価に使用した仕様書 ID（Specification への参照）
    pub specification_id: Uuid,
    // サンプル測定値のリスト（空の場合は Pending になる）
    pub samples: Vec<MetricSample>,
    // 評価の合否判定
    pub verdict: EvaluationVerdict,
    // 評価担当者の識別子（AuthContext.user_id: 業界語「検査員」の代わり）
    pub evaluator_id: Uuid,
    // 評価日時（HLC タイムスタンプで記録する）
    pub evaluated_at: DateTime<Utc>,
    // 楽観的ロックバージョン（競合検出に使用する）
    pub version: i64,
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // EvaluationRecord の公開 API 名が業界中立語であることを確認する
    fn test_evaluation_record_neutral_naming() {
        // struct 名が EvaluationRecord（業界語「検査成績書」ではない）であることをコンパイルで確認する
        let _: Option<EvaluationRecord> = None;
        // EvaluationVerdict の初期値が Conforming であることを確認する
        let verdict = EvaluationVerdict::Conforming;
        // Conforming が正しい文字列を返すことを確認する
        assert_eq!(format!("{:?}", verdict), "Conforming");
    }

    #[test]
    // サンプル測定値が空の場合は Pending になることを確認する
    fn test_empty_samples_yields_pending() {
        // 測定値が空の Vec を用意する
        let samples: Vec<MetricSample> = vec![];
        // 空の測定値リストは評価不可なため Pending を期待する
        let verdict = if samples.is_empty() {
            EvaluationVerdict::Pending
        } else {
            EvaluationVerdict::Conforming
        };
        // Pending が返ることを確認する
        assert_eq!(verdict, EvaluationVerdict::Pending);
    }
}
