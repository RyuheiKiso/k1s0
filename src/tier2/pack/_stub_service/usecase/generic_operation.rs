// generic_operation.rs — k1s0 tier2 _stub_service 汎用操作ユースケース
// vocabulary.yaml の neutral_term: GenericRecord（業界語「処理結果」を隠蔽する）
// 公開 API / 公開型名に業界固有語（検査結果 / 発注書 / 設備 等）を使ってはならない
// テナント識別子は TenantContext 経由でのみ取得する（API 引数経由は禁止）
// manufacturing pack の quality_inspect.rs とは意図的に語彙を分離する（pack 間依存禁止）

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート（集約 ID と操作者 ID に使用する）
use uuid::Uuid;
// エラーハンドリングライブラリ
use anyhow::Result;

// --- ドメイン値型 ---

// OperationOutcome: 汎用操作の結果（業界中立語）
// vocabulary.yaml: "処理結果" → neutral_term: "GenericRecord"
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationOutcome {
    // 成功（処理が正常完了した）
    Succeeded,
    // 失敗（処理が異常終了した）
    Failed,
    // 部分成功（一部の処理が完了した状態）
    PartialSuccess,
}

// OperationOutcome の文字列変換ヘルパー実装
impl OperationOutcome {
    // OperationOutcome を文字列に変換する（JSON ペイロードに埋め込む際に使用する）
    pub fn as_str(&self) -> &'static str {
        // 各 enum 値を業界中立語の文字列にマッピングする
        match self {
            // 成功
            Self::Succeeded => "Succeeded",
            // 失敗
            Self::Failed => "Failed",
            // 部分成功
            Self::PartialSuccess => "PartialSuccess",
        }
    }

    // 失敗かどうかを判定する（FollowUpRecord 生成の判断に使用する）
    pub fn is_failure(&self) -> bool {
        // Failed の場合のみ true を返す（PartialSuccess は別フローで処理する）
        matches!(self, Self::Failed)
    }
}

// --- コマンド型 ---

// GenericOperationCommand: 汎用操作実施コマンド
// 公開 API 名は GenericRecord（業界語「処理結果」は vocabulary.yaml でマップ済み）
// operator_id は UUID で操作者を識別する（業界語「担当者 / 作業者」を使わない）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericOperationCommand {
    // 操作対象の集約 ID（GenericResource の ID）
    pub aggregate_id: Uuid,
    // 汎用操作の結果（業界中立語: OperationOutcome）
    pub outcome: OperationOutcome,
    // 操作実施者の識別子（Keycloak subject / actor_id に対応する）
    pub operator_id: String,
    // 操作の種別コード（業界中立な汎用操作種別）
    pub operation_type: String,
}

// GenericOperationCommand の検証（コマンド受付前に不変条件を確認する）
impl GenericOperationCommand {
    // コマンドの不変条件を検証する（違反時はエラーを返す）
    pub fn validate(&self) -> Result<()> {
        // operator_id が空でないことを確認する
        if self.operator_id.trim().is_empty() {
            // operator_id が空の場合はエラーを返す
            return Err(anyhow::anyhow!("operator_id は空にできません"));
        }
        // operation_type が空でないことを確認する
        if self.operation_type.trim().is_empty() {
            // operation_type が空の場合はエラーを返す
            return Err(anyhow::anyhow!("operation_type は空にできません"));
        }
        // 全検証通過
        Ok(())
    }
}

// --- 状態変化型 ---

// StateTransition: ドメインイベントの状態遷移（Outbox 経由で tier1 に送信する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    // ドメインイベント名（業界中立語）
    pub event_type: String,
    // 操作対象の集約 ID
    pub aggregate_id: Uuid,
    // ドメインイベントのペイロード（JSON 形式）
    pub payload: serde_json::Value,
}

// --- ユースケース実行関数 ---

// execute: 汎用操作ユースケースを実行する
// context: テナントセッションコンテキスト（テナント識別子 / 操作者 / 目的を含む）
//          テナント識別子は context 経由でのみ取得する（API 引数からの受け取りは禁止）
// cmd: 汎用操作コマンド（集約 ID / 操作結果 / 操作者 ID / 操作種別）
// 戻り値: 状態遷移（Outbox 経由で tier1 に送信する）、失敗時はエラー
//
// vocabulary.yaml 対応:
//   industry_term: "処理結果" → neutral_term: "GenericRecord"
//   industry_term: "後続処理依頼" → neutral_term: "FollowUpRecord"
pub fn execute(
    // テナントセッションコンテキスト（テナント識別子は context 経由でのみ取得する）
    context: &k1s0_tier2::TenantContext,
    // 汎用操作コマンド
    cmd: GenericOperationCommand,
) -> Result<StateTransition> {
    // コマンドの不変条件を検証する（失敗時は早期リターン）
    cmd.validate()?;
    // テナント ID は context 経由でのみ取得する
    let _tenant_id = context.tenant_id();
    // 失敗の場合は FollowUpRecord の生成フラグを立てる
    // （実際の FollowUpRecord 生成は別ユースケースが担う）
    let followup_required = cmd.outcome.is_failure();
    // 状態変化のペイロードを構築する（業界中立語のフィールド名のみ使用する）
    let payload = serde_json::json!({
        // 操作結果（neutral_term: GenericRecord の status）
        "operation_outcome": cmd.outcome.as_str(),
        // 操作者識別子
        "operator_id": cmd.operator_id,
        // 操作種別
        "operation_type": cmd.operation_type,
        // 後続処理依頼（FollowUpRecord）の生成が必要かどうか
        "followup_required": followup_required,
        // GenericResource の状態遷移
        // 成功: InProgress → Completed、失敗: InProgress → Aborted
        "status_transition": if followup_required {
            serde_json::json!({ "from": "InProgress", "to": "Aborted" })
        } else {
            serde_json::json!({ "from": "InProgress", "to": "Completed" })
        }
    });
    // StateTransition を生成する（ドメインイベント名は業界中立語を使用する）
    let event_type = if followup_required {
        // 失敗の場合のドメインイベント名
        "GenericRecord.OperationFailed"
    } else {
        // 成功 / 部分成功の場合のドメインイベント名
        "GenericRecord.OperationSucceeded"
    };
    // 状態遷移オブジェクトを生成して返す
    let state_transition = StateTransition {
        // ドメインイベント名（業界中立語）
        event_type: event_type.to_string(),
        // 操作対象の集約 ID
        aggregate_id: cmd.aggregate_id,
        // ドメインイベントのペイロード
        payload,
    };
    // 状態遷移を返す（呼び出し元が Outbox に書き込む責任を持つ）
    Ok(state_transition)
}

// --- テスト ---

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // OperationOutcome::Succeeded が正しい文字列を返すことを確認する
    fn test_operation_outcome_succeeded_str() {
        // Succeeded の文字列表現を確認する
        let outcome = OperationOutcome::Succeeded;
        // 成功文字列が正しいことを確認する
        assert_eq!(outcome.as_str(), "Succeeded");
        // 成功は後続処理依頼が不要であることを確認する
        assert!(!outcome.is_failure());
    }

    #[test]
    // OperationOutcome::Failed が後続処理依頼必要と判定されることを確認する
    fn test_operation_outcome_failed_requires_followup() {
        // Failed の場合は is_failure が true を返すことを確認する
        let outcome = OperationOutcome::Failed;
        // 失敗は後続処理依頼が必要であることを確認する
        assert!(outcome.is_failure());
        // 文字列表現が正しいことを確認する
        assert_eq!(outcome.as_str(), "Failed");
    }

    #[test]
    // 空の operator_id はバリデーションエラーになることを確認する
    fn test_validate_empty_operator_id() {
        // operator_id が空のコマンドを生成する
        let cmd = GenericOperationCommand {
            aggregate_id: Uuid::new_v4(),
            outcome: OperationOutcome::Succeeded,
            // 空の操作者 ID（バリデーションエラーになるべき）
            operator_id: "".to_string(),
            // 操作種別
            operation_type: "generic_process".to_string(),
        };
        // バリデーションがエラーになることを確認する
        assert!(cmd.validate().is_err());
    }

    #[test]
    // 有効なコマンドはバリデーションが通ることを確認する
    fn test_validate_valid_command() {
        // 有効なコマンドを生成する
        let cmd = GenericOperationCommand {
            aggregate_id: Uuid::new_v4(),
            outcome: OperationOutcome::PartialSuccess,
            // 有効な操作者 ID
            operator_id: "operator-001".to_string(),
            // 有効な操作種別
            operation_type: "generic_batch_process".to_string(),
        };
        // バリデーションが通ることを確認する
        assert!(cmd.validate().is_ok());
    }
}
