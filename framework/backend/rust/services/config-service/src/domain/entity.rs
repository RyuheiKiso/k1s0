//! 設定エンティティ

use std::time::SystemTime;

/// 設定の値型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingValueType {
    /// 文字列
    String,
    /// 整数
    Int,
    /// 真偽値
    Bool,
    /// JSON
    Json,
}

impl SettingValueType {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Json => "json",
        }
    }

    /// 文字列から変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "string" => Some(Self::String),
            "int" => Some(Self::Int),
            "bool" => Some(Self::Bool),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

impl Default for SettingValueType {
    fn default() -> Self {
        Self::String
    }
}

/// 設定エンティティ
#[derive(Debug, Clone)]
pub struct Setting {
    /// 設定ID
    pub id: i64,
    /// サービス名スコープ
    pub service_name: String,
    /// 環境 (dev, stg, prod, default)
    pub env: String,
    /// 設定キー
    pub key: String,
    /// 値型
    pub value_type: SettingValueType,
    /// 設定値
    pub value: String,
    /// 説明
    pub description: Option<String>,
    /// ステータス (true: active, false: inactive)
    pub is_active: bool,
    /// 作成日時
    pub created_at: SystemTime,
    /// 更新日時
    pub updated_at: SystemTime,
}

impl Setting {
    /// 新しい設定を作成
    pub fn new(
        id: i64,
        service_name: impl Into<String>,
        env: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            service_name: service_name.into(),
            env: env.into(),
            key: key.into(),
            value_type: SettingValueType::String,
            value: value.into(),
            description: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// 値型を設定
    pub fn with_value_type(mut self, value_type: SettingValueType) -> Self {
        self.value_type = value_type;
        self
    }

    /// 説明を設定
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// アクティブ状態を設定
    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// 整数値として取得
    pub fn as_int(&self) -> Option<i64> {
        self.value.parse().ok()
    }

    /// 真偽値として取得
    pub fn as_bool(&self) -> Option<bool> {
        match self.value.to_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        }
    }

    /// 文字列値として取得
    pub fn as_string(&self) -> &str {
        &self.value
    }
}

/// 設定の一覧検索条件
#[derive(Debug, Clone, Default)]
pub struct SettingQuery {
    /// サービス名
    pub service_name: Option<String>,
    /// 環境
    pub env: Option<String>,
    /// キープレフィックス
    pub key_prefix: Option<String>,
    /// ページサイズ
    pub page_size: Option<u32>,
    /// ページトークン
    pub page_token: Option<String>,
}

impl SettingQuery {
    /// 新しいクエリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// サービス名を設定
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// 環境を設定
    pub fn with_env(mut self, env: impl Into<String>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// キープレフィックスを設定
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// ページサイズを設定
    pub fn with_page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// ページトークンを設定
    pub fn with_page_token(mut self, token: impl Into<String>) -> Self {
        self.page_token = Some(token.into());
        self
    }
}

/// 設定一覧の結果
#[derive(Debug, Clone)]
pub struct SettingList {
    /// 設定一覧
    pub settings: Vec<Setting>,
    /// 次ページのトークン
    pub next_page_token: Option<String>,
}

impl SettingList {
    /// 新しい結果を作成
    pub fn new(settings: Vec<Setting>) -> Self {
        Self {
            settings,
            next_page_token: None,
        }
    }

    /// 次ページトークンを設定
    pub fn with_next_page_token(mut self, token: impl Into<String>) -> Self {
        self.next_page_token = Some(token.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_value_type() {
        assert_eq!(SettingValueType::String.as_str(), "string");
        assert_eq!(SettingValueType::Int.as_str(), "int");
        assert_eq!(SettingValueType::Bool.as_str(), "bool");
        assert_eq!(SettingValueType::Json.as_str(), "json");

        assert_eq!(SettingValueType::from_str("string"), Some(SettingValueType::String));
        assert_eq!(SettingValueType::from_str("unknown"), None);
    }

    #[test]
    fn test_setting_new() {
        let setting = Setting::new(1, "my-service", "dev", "feature.enabled", "true");

        assert_eq!(setting.id, 1);
        assert_eq!(setting.service_name, "my-service");
        assert_eq!(setting.env, "dev");
        assert_eq!(setting.key, "feature.enabled");
        assert_eq!(setting.value, "true");
        assert!(setting.is_active);
    }

    #[test]
    fn test_setting_as_bool() {
        let setting = Setting::new(1, "service", "dev", "key", "true");
        assert_eq!(setting.as_bool(), Some(true));

        let setting = Setting::new(1, "service", "dev", "key", "false");
        assert_eq!(setting.as_bool(), Some(false));

        let setting = Setting::new(1, "service", "dev", "key", "invalid");
        assert_eq!(setting.as_bool(), None);
    }

    #[test]
    fn test_setting_as_int() {
        let setting = Setting::new(1, "service", "dev", "key", "42");
        assert_eq!(setting.as_int(), Some(42));

        let setting = Setting::new(1, "service", "dev", "key", "invalid");
        assert_eq!(setting.as_int(), None);
    }

    #[test]
    fn test_setting_query() {
        let query = SettingQuery::new()
            .with_service_name("my-service")
            .with_env("prod")
            .with_key_prefix("feature.")
            .with_page_size(50);

        assert_eq!(query.service_name, Some("my-service".to_string()));
        assert_eq!(query.env, Some("prod".to_string()));
        assert_eq!(query.key_prefix, Some("feature.".to_string()));
        assert_eq!(query.page_size, Some(50));
    }
}
