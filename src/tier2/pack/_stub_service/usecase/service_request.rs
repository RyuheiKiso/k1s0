// k1s0 tier2 _stub_service pack — ServiceRequest ユースケース（業界中立 stub）
// 第二業界（サービス業等）のユースケース実装サンプル
// manufacturing pack と完全に分離した語彙を使用する（業界語は一切含まない）
// テナント識別子は TenantContext 経由でのみ取得する（API 引数経由は禁止）
// このファイルは業界横断 conformance の参照実装として機能する

// シリアライズライブラリのインポート
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート（集約 ID / 依頼者 ID に使用する）
use uuid::Uuid;
// エラーハンドリングライブラリ
use anyhow::Result;

// --- ドメイン値型 ---

// RequestStatus: サービス依頼の状態（業界中立語）
// ServiceOrder（stub.rs）の ServiceOrderStatus とは独立した型として定義する
// この状態型はユースケース層のコマンド処理結果を表現する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStatus {
    // 依頼受付済み（初期状態）
    Accepted,
    // 処理開始済み（担当割当後）
    InProgress,
    // 完了（依頼者への納品完了）
    Fulfilled,
    // 依頼取消（取消申請が承認された）
    Cancelled,
}

impl RequestStatus {
    // RequestStatus を文字列に変換する（JSON ペイロードに埋め込む際に使用する）
    pub fn as_str(&self) -> &'static str {
        // 各 enum 値を業界中立語の文字列にマッピングする
        match self {
            // 受付済み
            Self::Accepted => "Accepted",
            // 処理中
            Self::InProgress => "InProgress",
            // 完了
            Self::Fulfilled => "Fulfilled",
            // 取消
            Self::Cancelled => "Cancelled",
        }
    }
}

// StateChange: ユースケース実行結果として emit する状態変化の型
// manufacturing の StateChange と構造は同じだが、型は独立している（pack 間の依存禁止）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    // 状態変化の種別（ドメインイベント名。業界中立語を使用する）
    pub event_type: String,
    // 状態変化の対象集約 ID
    pub aggregate_id: Uuid,
    // 状態変化のペイロード（JSON 形式で記録する）
    pub payload: serde_json::Value,
}

// --- コマンド型 ---

// ServiceRequestCommand: サービス依頼受付コマンド（業界中立 stub）
// フィールド名に業界固有語を使わない（order / request 等の中立語を使用する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequestCommand {
    // サービス依頼の集約 ID（UUID v4）
    pub aggregate_id: Uuid,
    // 依頼識別コード（業界中立な番号体系）
    pub request_code: String,
    // 依頼内容の要約（業界中立語で表現する）
    pub description: String,
    // 依頼者識別子（Keycloak subject / actor_id に対応する）
    pub requestor_id: String,
    // 優先度レベル（0 = 通常, 1 = 優先, 2 = 緊急）
    pub priority: u8,
}

// ServiceRequestCommand の検証（コマンド受付前に不変条件を確認する）
impl ServiceRequestCommand {
    // コマンドの不変条件を検証する（違反時はエラーを返す）
    pub fn validate(&self) -> Result<()> {
        // request_code が空でないことを確認する
        if self.request_code.trim().is_empty() {
            // request_code が空の場合はエラーを返す
            return Err(anyhow::anyhow!("request_code は空にできません"));
        }
        // requestor_id が空でないことを確認する
        if self.requestor_id.trim().is_empty() {
            // requestor_id が空の場合はエラーを返す
            return Err(anyhow::anyhow!("requestor_id は空にできません"));
        }
        // priority は 0-2 の範囲内でなければならない
        if self.priority > 2 {
            // 範囲外の priority はエラー
            return Err(anyhow::anyhow!(
                "priority は 0-2 の範囲でなければなりません: {}",
                self.priority
            ));
        }
        // 全検証通過
        Ok(())
    }
}

// --- ユースケース実行関数 ---

// execute: サービス依頼受付ユースケースを実行する
// context: テナントセッションコンテキスト（テナント識別子は context 経由でのみ取得する）
// cmd: サービス依頼コマンド（集約 ID / 依頼コード / 説明 / 依頼者 ID / 優先度）
// 戻り値: 状態変化（Outbox 経由で tier1 に送信する）、失敗時はエラー
//
// 業界中立設計の証明:
//   - manufacturing pack 固有語（ロット / 品目 / 設備 / BOM / 検査結果）を一切使わない
//   - pub API に業界固有語が含まれないことは CI lint（vocabulary.yaml 参照）が検証する
pub fn execute(
    // テナントセッションコンテキスト（テナント識別子は context 経由でのみ取得する）
    context: &k1s0_tier2::TenantContext,
    // サービス依頼コマンド
    cmd: ServiceRequestCommand,
) -> Result<StateChange> {
    // コマンドの不変条件を検証する（失敗時は早期リターン）
    cmd.validate()?;
    // テナント ID は context 経由でのみ取得する（API 引数経由は禁止）
    let _tenant_id = context.tenant_id();
    // 状態変化のペイロードを構築する（業界中立語のフィールド名のみ使用する）
    let payload = serde_json::json!({
        // 依頼識別コード
        "request_code": cmd.request_code,
        // 依頼内容の要約
        "description": cmd.description,
        // 依頼者識別子
        "requestor_id": cmd.requestor_id,
        // 優先度レベル
        "priority": cmd.priority,
        // 初期状態: Accepted（依頼受付済み）
        "initial_status": RequestStatus::Accepted.as_str()
    });
    // StateChange を生成する（ドメインイベント名は業界中立語を使用する）
    let state_change = StateChange {
        // ドメインイベント名（manufacturing 固有語は含まない）
        event_type: "ServiceRequest.Accepted".to_string(),
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
    // 有効なコマンドはバリデーションが通ることを確認する
    fn test_validate_valid_command() {
        // 有効なサービス依頼コマンドを生成する
        let cmd = ServiceRequestCommand {
            // 集約 ID は UUID v4 で生成する
            aggregate_id: Uuid::new_v4(),
            // 有効な依頼コード
            request_code: "SR-001".to_string(),
            // 依頼内容の説明
            description: "システムメンテナンス依頼".to_string(),
            // 有効な依頼者 ID
            requestor_id: "user-abc-001".to_string(),
            // 通常優先度
            priority: 0,
        };
        // バリデーションが通ることを確認する
        assert!(cmd.validate().is_ok());
    }

    #[test]
    // 空の request_code はバリデーションエラーになることを確認する
    fn test_validate_empty_request_code() {
        // request_code が空のコマンドを生成する
        let cmd = ServiceRequestCommand {
            aggregate_id: Uuid::new_v4(),
            // 空の依頼コード（バリデーションエラーになるべき）
            request_code: "".to_string(),
            description: "テスト".to_string(),
            requestor_id: "user-001".to_string(),
            priority: 1,
        };
        // バリデーションがエラーになることを確認する
        assert!(cmd.validate().is_err());
    }

    #[test]
    // priority が範囲外（3 以上）の場合はバリデーションエラーになることを確認する
    fn test_validate_invalid_priority() {
        // priority が 3 のコマンドを生成する
        let cmd = ServiceRequestCommand {
            aggregate_id: Uuid::new_v4(),
            request_code: "SR-002".to_string(),
            description: "優先度テスト".to_string(),
            requestor_id: "user-002".to_string(),
            // 範囲外の優先度（0-2 のみ有効）
            priority: 3,
        };
        // バリデーションがエラーになることを確認する
        assert!(cmd.validate().is_err());
    }

    #[test]
    // RequestStatus が manufacturing 固有語を含まないことを確認する
    fn test_no_manufacturing_terms_in_status() {
        // 全ての RequestStatus 文字列を確認する
        let statuses = [
            RequestStatus::Accepted,
            RequestStatus::InProgress,
            RequestStatus::Fulfilled,
            RequestStatus::Cancelled,
        ];
        // 各ステータスの文字列表現に製造業固有語が含まれないことを確認する
        for status in &statuses {
            let s = status.as_str();
            // 製造業固有語「ロット」を含まないことを確認する
            assert!(!s.contains("ロット"), "ステータス '{}' に製造業固有語が含まれています", s);
            // 製造業固有語「設備」を含まないことを確認する
            assert!(!s.contains("設備"), "ステータス '{}' に製造業固有語が含まれています", s);
            // 製造業固有語「Batch」を含まないことを確認する（ProductionBatch との分離）
            assert!(!s.contains("Batch"), "ステータス '{}' に製造業固有語が含まれています", s);
        }
    }
}
