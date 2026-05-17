// k1s0 tier2 manufacturing pack — ProductionBatch 開始ユースケース
// vocabulary.yaml の neutral_term: ProductionBatch（業界語「ロット」を隠蔽する）
// 公開 API / 公開型名に業界固有語（ロット / 設備 / 品目 / 拠点 / BOM）を使ってはならない
// テナント識別子は AuthContext（TenantContext）経由でのみ取得する（API 引数経由は禁止）

// シリアライズライブラリのインポート（コマンド型の JSON 変換に使用する）
use serde::{Deserialize, Serialize};
// UUID ライブラリのインポート（集約 ID に使用する）
use uuid::Uuid;
// エラーハンドリングライブラリ（anyhow::Result でエラーを一元管理する）
use anyhow::Result;
// TenantContext のインポート（テナント識別子は Context 経由でのみ取得する）
// パスは tier2/rust の pub モジュールから参照する
// （pack コードは tier2/rust の crate として組み込まれる前提）

// --- ドメイン層への依存（ドメインイベント / 状態変化）---

// StateChange: ユースケース実行結果として emit する状態変化の型
// ドメインイベント名と集約 ID を持つ（Outbox 経由で tier1 に送信する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    // 状態変化の種別（ドメインイベント名。業界中立語を使用する）
    pub event_type: String,
    // 状態変化の対象集約 ID（ProductionBatch の ID）
    pub aggregate_id: Uuid,
    // 状態変化のペイロード（JSON 形式で記録する）
    pub payload: serde_json::Value,
}

// --- コマンド型 ---

// ProductionStartCommand: 生産バッチ開始コマンド
// 公開 API 名は ProductionBatch（業界語「ロット」は vocabulary.yaml でマップ済み）
// フィールド名にも業界語を使わない（batch_code は中立語）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStartCommand {
    // 生産バッチの集約 ID（UUID v4、既存集約の特定に使用する）
    pub aggregate_id: Uuid,
    // バッチ識別コード（業界中立な番号体系、vocabulary.yaml: batch_code = neutral_term）
    pub batch_code: String,
    // 生産数量（業界中立語。vocabulary.yaml: quantity）
    pub quantity: i64,
}

// ProductionStartCommand の検証（コマンド受付前に不変条件を確認する）
impl ProductionStartCommand {
    // コマンドの不変条件を検証する（違反時はエラーを返す）
    pub fn validate(&self) -> Result<()> {
        // batch_code が空でないことを確認する
        if self.batch_code.trim().is_empty() {
            // batch_code が空の場合はエラーを返す
            return Err(anyhow::anyhow!("batch_code は空にできません"));
        }
        // quantity が正の値であることを確認する（0 以下は業務上無効）
        if self.quantity <= 0 {
            // 非正の quantity はエラー
            return Err(anyhow::anyhow!("quantity は 1 以上でなければなりません: {}", self.quantity));
        }
        // 全検証通過
        Ok(())
    }
}

// --- ユースケース実行関数 ---

// execute: 生産バッチ開始ユースケースを実行する
// context: テナントセッションコンテキスト（テナント識別子 / 操作者 / 目的を含む）
//          API 引数から tenant_id を受け取らない（context 経由でのみ取得する）
// cmd: 生産バッチ開始コマンド（集約 ID / バッチコード / 数量）
// 戻り値: 状態変化（Outbox 経由で tier1 に送信する）、失敗時はエラー
//
// vocabulary.yaml 対応:
//   industry_term: "ロット" → neutral_term: "ProductionBatch"
//   industry_term: "設備"   → neutral_term: "ResourceUnit"（本ユースケースでは未使用）
pub fn execute(
    // テナントセッションコンテキスト（テナント識別子は context 経由でのみ取得する）
    context: &k1s0_tier2::TenantContext,
    // 生産バッチ開始コマンド
    cmd: ProductionStartCommand,
) -> Result<StateChange> {
    // コマンドの不変条件を検証する（失敗時は早期リターンする）
    cmd.validate()?;
    // テナント ID は context 経由でのみ取得する（API 引数経由は禁止）
    // テナント ID は StateChange のペイロードには含まない（監査ログは別途 Outbox が記録する）
    let _tenant_id = context.tenant_id();
    // 状態変化ペイロードを構築する（業界中立語のフィールド名を使用する）
    let payload = serde_json::json!({
        // バッチ識別コード（vocabulary.yaml: neutral_term = batch_code）
        "batch_code": cmd.batch_code,
        // 生産数量
        "quantity": cmd.quantity,
        // 状態遷移: Planned → InProgress（BatchStatus の neutral_term）
        "status_transition": { "from": "Planned", "to": "InProgress" }
    });
    // StateChange を生成する（Outbox 経由で tier1 の event bus に emit する）
    let state_change = StateChange {
        // ドメインイベント名（業界中立語で表現する）
        event_type: "ProductionBatch.Started".to_string(),
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
    // ProductionStartCommand の検証が正しく機能することを確認する
    fn test_validate_valid_command() {
        // 有効なコマンドを生成する
        let cmd = ProductionStartCommand {
            // 集約 ID は UUID v4 で生成する
            aggregate_id: Uuid::new_v4(),
            // 有効なバッチコード
            batch_code: "BATCH-001".to_string(),
            // 正の数量
            quantity: 100,
        };
        // 検証が通ることを確認する
        assert!(cmd.validate().is_ok());
    }

    #[test]
    // 空のバッチコードは検証エラーになることを確認する
    fn test_validate_empty_batch_code() {
        // batch_code が空のコマンドを生成する
        let cmd = ProductionStartCommand {
            aggregate_id: Uuid::new_v4(),
            // 空のバッチコード（検証エラーになるべき）
            batch_code: "".to_string(),
            quantity: 10,
        };
        // 検証がエラーになることを確認する
        assert!(cmd.validate().is_err());
    }

    #[test]
    // 0 以下の数量は検証エラーになることを確認する
    fn test_validate_non_positive_quantity() {
        // quantity が 0 のコマンドを生成する
        let cmd = ProductionStartCommand {
            aggregate_id: Uuid::new_v4(),
            batch_code: "BATCH-002".to_string(),
            // 0 は業務上無効（1 以上でなければならない）
            quantity: 0,
        };
        // 検証がエラーになることを確認する
        assert!(cmd.validate().is_err());
    }
}
