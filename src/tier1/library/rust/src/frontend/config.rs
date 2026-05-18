// config.rs — k1s0 tier1 Library frontend: Configuration/Feature Flag L2* thin wrapper
// frontend 向けの設定 / 機能フラグ thin wrapper を定義する。
// backend::config の型を再利用し、frontend 固有の動作（ローカルキャッシュ / watch）を追加する。
// backend 向けの ConfigStore（書き込み / watcher 登録）は含まない。
// frontend は読み取り専用の設定アクセスと機能フラグ評価のみを提供する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;

// backend::config の型を frontend 向けに re-export する
// （frontend も ConfigValue / FeatureFlag / EvaluationContext を使用するため）
pub use crate::backend::config::{ConfigValue, EvaluationContext, FeatureFlag};

// FrontendConfigSnapshot は frontend が保持する設定スナップショットを表す Library 独自型。
// 設定値を事前取得してローカルキャッシュに保持する（通信遅延を隠蔽する）。
#[derive(Debug, Clone)]
pub struct FrontendConfigSnapshot {
    // config_values: 設定値のマップ（キー: config キー / 値: ConfigValue）
    config_values: std::collections::HashMap<String, ConfigValue>,
    // feature_flags: 機能フラグのマップ（キー: フラグキー / 値: FeatureFlag）
    feature_flags: std::collections::HashMap<String, FeatureFlag>,
    // snapshot_version: スナップショットのバージョン番号（更新追跡用）
    pub snapshot_version: u64,
}

// FrontendConfigSnapshot のファクトリメソッドとアクセサ
impl FrontendConfigSnapshot {
    // new は設定値と機能フラグのリストからスナップショットを生成する
    pub fn new(
        config_values: Vec<ConfigValue>,
        feature_flags: Vec<FeatureFlag>,
        snapshot_version: u64,
    ) -> Self {
        // 設定値をマップに変換する
        let config_map = config_values
            .into_iter()
            .map(|v| (v.key.clone(), v))
            .collect();
        // 機能フラグをマップに変換する
        let flag_map = feature_flags
            .into_iter()
            .map(|f| (f.flag_key.clone(), f))
            .collect();
        // スナップショットを構築する
        Self {
            config_values: config_map,
            feature_flags: flag_map,
            snapshot_version,
        }
    }

    // get_config は設定キーを受け取り ConfigValue を返す
    pub fn get_config(&self, key: &str) -> Option<&ConfigValue> {
        // マップから設定値を取得する
        self.config_values.get(key)
    }

    // get_flag は機能フラグキーを受け取り FeatureFlag を返す
    pub fn get_flag(&self, flag_key: &str) -> Option<&FeatureFlag> {
        // マップから機能フラグを取得する
        self.feature_flags.get(flag_key)
    }

    // is_enabled は機能フラグキーを受け取り有効かどうかを返す（フラグが存在しない場合は false）
    pub fn is_enabled(&self, flag_key: &str) -> bool {
        // フラグが存在しない場合はデフォルト無効（safe default）
        self.feature_flags
            .get(flag_key)
            .map(|f| f.enabled)
            .unwrap_or(false)
    }
}

// FrontendConfigClient は frontend 向けの設定 / 機能フラグクライアント L2* trait。
// 読み取り専用のアクセスと定期的なスナップショット更新のみを提供する。
// backend 向けの ConfigStore（書き込み / watcher 登録）は含まない。
#[async_trait]
pub trait FrontendConfigClient: Send + Sync {
    // load_snapshot は現在の設定スナップショットをロードする（初期化時に呼び出す）。
    // EvaluationContext を受け取り、ユーザー固有のフラグ評価を行う。
    async fn load_snapshot(
        &self,
        eval_ctx: &EvaluationContext,
    ) -> Result<FrontendConfigSnapshot>;

    // refresh_snapshot は最新の設定スナップショットを取得して返す（定期更新用）。
    // 前回の snapshot_version より新しい場合のみ設定を返す（変更なしは None）。
    async fn refresh_snapshot(
        &self,
        current_version: u64,
        eval_ctx: &EvaluationContext,
    ) -> Result<Option<FrontendConfigSnapshot>>;

    // get_config は設定キーを受け取り ConfigValue を返す（ローカルキャッシュ経由）。
    // キャッシュミス時はネットワーク取得を行う。
    async fn get_config(
        &self,
        key: &str,
        eval_ctx: &EvaluationContext,
    ) -> Result<Option<ConfigValue>>;

    // is_enabled は機能フラグキーを受け取り有効かどうかを返す（ローカルキャッシュ経由）。
    // キャッシュミス時はデフォルト値（false）を返す（fail-safe）。
    async fn is_enabled(
        &self,
        flag_key: &str,
        eval_ctx: &EvaluationContext,
    ) -> Result<bool>;
}
