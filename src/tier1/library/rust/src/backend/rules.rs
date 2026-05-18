// rules.rs — k1s0 tier1 Library backend: Rule Engine L1+ trait
// backend 専用カテゴリ（frontend には提供しない）。
// L1+: OSS の全機能を表現 + tier1 横断要素（auth context 伝播 / retry / tracing）を強制。
// 公開 API に OSS 型（drools-rs / easy-rules 等）を露出しない。
// ルール評価は pure function として実装し、副作用は呼び出し元が担う設計。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: ルール定義 / ファクトのシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: ルール評価に認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// SpanContext: ルール評価に tracing コンテキストを伝播する
use crate::core::observability::SpanContext;

// RulePriority はルールの優先度を表す Library 独自型。
// 数値が大きいほど優先度が高い（OSS の Priority 型に依存しない）。
pub type RulePriority = i32;

// RuleMetadata はルールのメタデータを表す Library 独自型。
// 生のルール定義文字列は含まない（RuleDefinition 型で別管理する）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    // rule_id: ルールの一意識別子（UUID v7 形式）
    pub rule_id: String,
    // rule_name: ルールの名称（人間が読める識別子）
    pub rule_name: String,
    // rule_version: ルールのバージョン番号（変更追跡用）
    pub rule_version: u32,
    // priority: ルールの優先度（数値が大きいほど先に評価する）
    pub priority: RulePriority,
    // tenant_id: このルールが属するテナントの識別子（グローバルルールは "global"）
    pub tenant_id: String,
    // enabled: ルールが有効かどうか（false は評価対象外）
    pub enabled: bool,
    // description: ルールの説明文（日本語可）
    pub description: String,
}

// FactSet はルール評価に使用するファクト（入力データ）を表す Library 独自型。
// 生の DTO / ドメインオブジェクトを JSON に変換して渡す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSet {
    // facts: ファクトのマップ（キー: ファクト名 / 値: JSON）
    pub facts: std::collections::HashMap<String, serde_json::Value>,
    // context_labels: 評価コンテキストのラベル（ルールセット選択に使用する）
    pub context_labels: Vec<String>,
}

// FactSet のファクトリメソッド
impl FactSet {
    // new は空のファクトセットを生成する
    pub fn new() -> Self {
        // 空のファクトセットを返す
        Self {
            facts: std::collections::HashMap::new(),
            context_labels: vec![],
        }
    }

    // insert はファクトを追加する（既存キーは上書きする）
    pub fn insert(&mut self, key: impl Into<String>, value: serde_json::Value) {
        // ファクトマップにエントリを追加する
        self.facts.insert(key.into(), value);
    }
}

// RuleEvaluationResult はルール評価の 1 結果を表す Library 独自型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluationResult {
    // rule_id: 評価したルールの識別子
    pub rule_id: String,
    // rule_name: 評価したルールの名称
    pub rule_name: String,
    // fired: ルールが発火したかどうか（condition が true だったかどうか）
    pub fired: bool,
    // actions_applied: 発火時に適用したアクションのリスト
    pub actions_applied: Vec<String>,
    // output_facts: ルール適用後に変更 / 追加されたファクト
    pub output_facts: std::collections::HashMap<String, serde_json::Value>,
}

// RuleSetEvaluationResult はルールセット全体の評価結果を表す Library 独自型。
#[derive(Debug, Clone)]
pub struct RuleSetEvaluationResult {
    // results: 各ルールの評価結果リスト（priority 降順でソート済み）
    pub results: Vec<RuleEvaluationResult>,
    // final_facts: 全ルール適用後の最終ファクトセット
    pub final_facts: FactSet,
    // fired_count: 発火したルール数
    pub fired_count: u32,
    // total_evaluated: 評価したルール総数
    pub total_evaluated: u32,
}

// RuleEngine は Rule Engine の L1+ 抽象 trait。
// Drools / Easy Rules / custom engine 等を実装で切り替えられる。
// auth context 伝播 / tracing は強制する。
#[async_trait]
pub trait RuleEngine: Send + Sync {
    // evaluate はファクトセットに対してルールセットを評価する。
    // auth_ctx は tenant_id のルールセット選択と audit ログに使用する。
    // span_ctx は tracing コンテキストの伝播に使用する。
    async fn evaluate(
        &self,
        facts: FactSet,
        ruleset_name: &str,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
    ) -> Result<RuleSetEvaluationResult>;

    // evaluate_single は指定した 1 ルールのみを評価する（デバッグ用）。
    async fn evaluate_single(
        &self,
        facts: FactSet,
        rule_id: &str,
        auth_ctx: &AuthContext,
    ) -> Result<RuleEvaluationResult>;

    // get_rule はルールのメタデータを返す（定義文字列は返さない）。
    async fn get_rule(
        &self,
        rule_id: &str,
        auth_ctx: &AuthContext,
    ) -> Result<Option<RuleMetadata>>;

    // list_rules は tenant_id のルール一覧を返す（ページネーション対応）。
    async fn list_rules(
        &self,
        ruleset_name: &str,
        page_size: u32,
        cursor: Option<String>,
        auth_ctx: &AuthContext,
    ) -> Result<(Vec<RuleMetadata>, Option<String>)>;

    // reload はルールセットを最新定義でリロードする（hot-reload に使用する）。
    async fn reload(&self, ruleset_name: &str, auth_ctx: &AuthContext) -> Result<u32>;
}
