// config.rs — k1s0 tier1 Library backend: Configuration/Feature Flag L2* trait
// backend 向け L2*（Configuration/Feature Flag カテゴリ）を定義する。
// L2*: 族内で共通の API + OSS 概念は Library 独自語彙に翻訳する。
// 公開 API に OSS 型（launchdarkly_server_sdk::Client 等）を露出しない。
// frontend 向けは src/frontend/config.rs で別途定義する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: 設定値のシリアライズに使用する
use serde::{Deserialize, Serialize};
// serde_json: 設定値を汎用 JSON 値として扱う
use serde_json::Value as JsonValue;
// AuthContext: 設定取得に認証コンテキストを伝播する
use crate::core::auth::AuthContext;

// ConfigValue は設定値を表す Library 独自型。
// OSS の Value 型（serde_json::Value 以外）は露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    // key: 設定キー（階層は "." で区切る; 例: "feature.dark_mode"）
    pub key: String,
    // value: 設定値（JSON 型で表現する; 文字列 / 数値 / bool / object を統一的に扱う）
    pub value: JsonValue,
    // scope: この設定値が有効なスコープ（"global" / "tenant" / "user" 等）
    pub scope: String,
    // version: 設定値のバージョン番号（変更追跡用）
    pub version: u64,
}

// FeatureFlag は機能フラグの評価結果を表す Library 独自型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    // flag_key: フラグキー（例: "new_checkout_flow"）
    pub flag_key: String,
    // enabled: フラグが有効かどうか（true = 機能 ON）
    pub enabled: bool,
    // variant: バリアント値（A/B テスト等で使用する; None は enabled のみ）
    pub variant: Option<String>,
    // reason: 評価理由（"TARGETING_MATCH" / "FALLTHROUGH" 等の Library 独自語彙）
    pub reason: String,
}

// EvaluationContext はフラグ評価時のコンテキストを表す struct。
// OSS の EvaluationContext（OpenFeature 等）とは独立した Library 独自型。
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    // user_key: フラグ評価対象のユーザーキー（AuthContext の subject_id と合わせる）
    pub user_key: String,
    // tenant_id: フラグ評価対象のテナント識別子
    pub tenant_id: String,
    // attributes: 追加属性（ロールアウト条件に使用する; key-value ペア）
    pub attributes: std::collections::HashMap<String, String>,
}

// ConfigStore は Configuration の L2* 抽象 trait。
// Consul KV / etcd / AWS Parameter Store 等を実装で切り替えられる。
// 公開 API に OSS 型を露出しない。
#[async_trait]
pub trait ConfigStore: Send + Sync {
    // get は設定キーを受け取り、ConfigValue を返す。
    // auth_ctx はテナント境界の保証に使用する。
    async fn get(&self, key: &str, auth_ctx: &AuthContext) -> Result<Option<ConfigValue>>;

    // get_all はプレフィックスに一致する設定値を全て返す。
    async fn get_all(&self, prefix: &str, auth_ctx: &AuthContext) -> Result<Vec<ConfigValue>>;

    // set は設定値を書き込む（バージョン番号は実装が採番する）。
    async fn set(
        &self,
        key: &str,
        value: JsonValue,
        scope: &str,
        auth_ctx: &AuthContext,
    ) -> Result<ConfigValue>;

    // watch は設定キーの変更を監視するコールバックを登録する。
    // 変更時に callback が ConfigValue を受け取る（OSS の watcher 型を露出しない）。
    async fn watch(
        &self,
        key: &str,
        callback: Box<dyn Fn(ConfigValue) + Send + Sync>,
        auth_ctx: &AuthContext,
    ) -> Result<String>;

    // unwatch は watch_id を受け取り、監視を解除する。
    async fn unwatch(&self, watch_id: &str) -> Result<()>;
}

// FeatureFlagClient は Feature Flag の L2* 抽象 trait。
// LaunchDarkly / Unleash / Flagsmith 等を実装で切り替えられる。
// 公開 API に OSS 型を露出しない。
#[async_trait]
pub trait FeatureFlagClient: Send + Sync {
    // evaluate はフラグキーと評価コンテキストを受け取り、FeatureFlag を返す。
    // 評価ルールは実装が保持し、呼び出し元には露出しない。
    async fn evaluate(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
    ) -> Result<FeatureFlag>;

    // evaluate_all は全フラグを評価して一括返却する（初期ロードに使用する）。
    async fn evaluate_all(
        &self,
        ctx: &EvaluationContext,
    ) -> Result<Vec<FeatureFlag>>;

    // flush は評価イベントをバックエンドに送信する（analytics に使用する）。
    async fn flush(&self) -> Result<()>;
}
