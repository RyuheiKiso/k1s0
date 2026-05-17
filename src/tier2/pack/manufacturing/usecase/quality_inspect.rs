// k1s0 tier2 manufacturing pack — QualityRecord 検査ユースケース
// vocabulary.yaml の neutral_term: QualityRecord（業界語「検査結果」を隠蔽する）
// 公開 API / 公開型名に業界固有語（検査結果 / 不良票 / ロット 等）を使ってはならない
// テナント識別子は TenantContext 経由でのみ取得する（API 引数経由は禁止）

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート（集約 ID と検査者 ID に使用する）
use uuid::Uuid;
// エラーハンドリングライブラリ
use anyhow::Result;

// production_start の StateChange を再使用する（同一 pack 内の型を共有する）
use super::production_start::StateChange;

// --- ドメイン値型 ---

// InspectionOutcome: 品質検査の判定結果（業界中立語）
// vocabulary.yaml: "検査結果" → neutral_term: "QualityRecord"
// neutral_term の判定値を enum で表現する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionOutcome {
    // 合格（品質基準を満たす）
    Passed,
    // 不合格（品質基準を満たさない）
    // 不合格時は NonconformanceReport（vocabulary.yaml の neutral_term）が後続処理で生成される
    Failed,
    // 条件付き合格（追加処理が必要、例: 再検査 / 条件付きリリース）
    ConditionalPass,
}

impl InspectionOutcome {
    // InspectionOutcome を文字列に変換する（JSON ペイロードに埋め込む際に使用する）
    pub fn as_str(&self) -> &'static str {
        // 各 enum 値を業界中立語の文字列にマッピングする
        match self {
            // 合格
            Self::Passed => "Passed",
            // 不合格
            Self::Failed => "Failed",
            // 条件付き合格
            Self::ConditionalPass => "ConditionalPass",
        }
    }

    // 不合格かどうかを判定する（NonconformanceReport 生成の判断に使用する）
    pub fn is_nonconformance(&self) -> bool {
        // Failed の場合のみ true を返す（ConditionalPass は別フローで処理する）
        matches!(self, Self::Failed)
    }
}

// --- コマンド型 ---

// QualityInspectCommand: 品質検査実施コマンド
// 公開 API 名は QualityRecord（業界語「検査結果」は vocabulary.yaml でマップ済み）
// inspector_id は UUID で検査者を識別する（業界語「検査員」を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectCommand {
    // 検査対象の集約 ID（ProductionBatch の ID）
    pub aggregate_id: Uuid,
    // 品質検査の判定結果（業界中立語: InspectionOutcome）
    pub result: InspectionOutcome,
    // 検査実施者の識別子（Keycloak subject / actor_id に対応する）
    pub inspector_id: String,
}

// QualityInspectCommand の検証（コマンド受付前に不変条件を確認する）
impl QualityInspectCommand {
    // コマンドの不変条件を検証する（違反時はエラーを返す）
    pub fn validate(&self) -> Result<()> {
        // inspector_id が空でないことを確認する
        if self.inspector_id.trim().is_empty() {
            // inspector_id が空の場合はエラーを返す
            return Err(anyhow::anyhow!("inspector_id は空にできません"));
        }
        // 全検証通過
        Ok(())
    }
}

// --- ユースケース実行関数 ---

// execute: 品質検査ユースケースを実行する
// context: テナントセッションコンテキスト（テナント識別子 / 操作者 / 目的を含む）
//          テナント識別子は context 経由でのみ取得する（API 引数からの受け取りは禁止）
// cmd: 品質検査コマンド（集約 ID / 判定結果 / 検査者 ID）
// 戻り値: 状態変化（Outbox 経由で tier1 に送信する）、失敗時はエラー
//
// vocabulary.yaml 対応:
//   industry_term: "検査結果" → neutral_term: "QualityRecord"
//   industry_term: "不良票"   → neutral_term: "NonconformanceReport"
pub fn execute(
    // テナントセッションコンテキスト（テナント識別子は context 経由でのみ取得する）
    context: &k1s0_tier2::TenantContext,
    // 品質検査コマンド
    cmd: QualityInspectCommand,
) -> Result<StateChange> {
    // コマンドの不変条件を検証する（失敗時は早期リターン）
    cmd.validate()?;
    // テナント ID は context 経由でのみ取得する
    let _tenant_id = context.tenant_id();
    // 不合格の場合は NonconformanceReport の生成フラグを立てる
    // （実際の NonconformanceReport 生成は別ユースケースが担う）
    let nonconformance_required = cmd.result.is_nonconformance();
    // 状態変化のペイロードを構築する（業界中立語のフィールド名のみ使用する）
    let payload = serde_json::json!({
        // 検査判定結果（neutral_term: QualityRecord の status）
        "inspection_outcome": cmd.result.as_str(),
        // 検査者識別子
        "inspector_id": cmd.inspector_id,
        // 不適合報告（NonconformanceReport）の生成が必要かどうか
        "nonconformance_required": nonconformance_required,
        // ProductionBatch の状態遷移
        // 合格: InProgress → Completed、不合格: InProgress → Rejected
        "status_transition": if nonconformance_required {
            serde_json::json!({ "from": "InProgress", "to": "Rejected" })
        } else {
            serde_json::json!({ "from": "InProgress", "to": "Completed" })
        }
    });
    // StateChange を生成する（ドメインイベント名は業界中立語を使用する）
    let event_type = if nonconformance_required {
        // 不合格の場合のドメインイベント名
        "QualityRecord.InspectionFailed"
    } else {
        // 合格 / 条件付き合格の場合のドメインイベント名
        "QualityRecord.InspectionPassed"
    };
    // 状態変化オブジェクトを生成して返す
    let state_change = StateChange {
        // ドメインイベント名（業界中立語）
        event_type: event_type.to_string(),
        // 操作対象の集約 ID
        aggregate_id: cmd.aggregate_id,
        // ドメインイベントのペイロード
        payload,
    };
    // 状態変化を返す（呼び出し元が Outbox に書き込む責任を持つ）
    Ok(state_change)
}

// --- テスト ---

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // InspectionOutcome::Passed が正しい文字列を返すことを確認する
    fn test_inspection_outcome_passed_str() {
        // Passed の文字列表現を確認する
        let outcome = InspectionOutcome::Passed;
        assert_eq!(outcome.as_str(), "Passed");
        // 合格は非適合報告が不要であることを確認する
        assert!(!outcome.is_nonconformance());
    }

    #[test]
    // InspectionOutcome::Failed が非適合報告必要と判定されることを確認する
    fn test_inspection_outcome_failed_nonconformance() {
        // Failed の場合は is_nonconformance が true を返す
        let outcome = InspectionOutcome::Failed;
        assert!(outcome.is_nonconformance());
        // 文字列表現が正しいことを確認する
        assert_eq!(outcome.as_str(), "Failed");
    }

    #[test]
    // 空の inspector_id はバリデーションエラーになることを確認する
    fn test_validate_empty_inspector_id() {
        // inspector_id が空のコマンドを生成する
        let cmd = QualityInspectCommand {
            aggregate_id: Uuid::new_v4(),
            result: InspectionOutcome::Passed,
            // 空の検査者 ID（バリデーションエラーになるべき）
            inspector_id: "".to_string(),
        };
        // バリデーションがエラーになることを確認する
        assert!(cmd.validate().is_err());
    }

    #[test]
    // 有効なコマンドはバリデーションが通ることを確認する
    fn test_validate_valid_command() {
        // 有効なコマンドを生成する
        let cmd = QualityInspectCommand {
            aggregate_id: Uuid::new_v4(),
            result: InspectionOutcome::ConditionalPass,
            // 有効な検査者 ID
            inspector_id: "inspector-001".to_string(),
        };
        // バリデーションが通ることを確認する
        assert!(cmd.validate().is_ok());
    }
}
