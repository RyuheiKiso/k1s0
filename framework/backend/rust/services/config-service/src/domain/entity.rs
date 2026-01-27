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

    // ========================================
    // SettingValueType Tests
    // ========================================

    #[test]
    fn test_setting_value_type_as_str() {
        assert_eq!(SettingValueType::String.as_str(), "string");
        assert_eq!(SettingValueType::Int.as_str(), "int");
        assert_eq!(SettingValueType::Bool.as_str(), "bool");
        assert_eq!(SettingValueType::Json.as_str(), "json");
    }

    #[test]
    fn test_setting_value_type_from_str_valid() {
        assert_eq!(SettingValueType::from_str("string"), Some(SettingValueType::String));
        assert_eq!(SettingValueType::from_str("int"), Some(SettingValueType::Int));
        assert_eq!(SettingValueType::from_str("bool"), Some(SettingValueType::Bool));
        assert_eq!(SettingValueType::from_str("json"), Some(SettingValueType::Json));
    }

    #[test]
    fn test_setting_value_type_from_str_invalid() {
        assert_eq!(SettingValueType::from_str("unknown"), None);
        assert_eq!(SettingValueType::from_str("STRING"), None); // Case sensitive
        assert_eq!(SettingValueType::from_str(""), None);
        assert_eq!(SettingValueType::from_str("integer"), None);
    }

    #[test]
    fn test_setting_value_type_default() {
        let default = SettingValueType::default();
        assert_eq!(default, SettingValueType::String);
    }

    #[test]
    fn test_setting_value_type_roundtrip() {
        for vt in [SettingValueType::String, SettingValueType::Int, SettingValueType::Bool, SettingValueType::Json] {
            let str_repr = vt.as_str();
            let parsed = SettingValueType::from_str(str_repr);
            assert_eq!(parsed, Some(vt));
        }
    }

    #[test]
    fn test_setting_value_type_eq() {
        assert_eq!(SettingValueType::String, SettingValueType::String);
        assert_ne!(SettingValueType::String, SettingValueType::Int);
    }

    #[test]
    fn test_setting_value_type_clone() {
        let vt = SettingValueType::Json;
        let cloned = vt.clone();
        assert_eq!(vt, cloned);
    }

    // ========================================
    // Setting Tests
    // ========================================

    #[test]
    fn test_setting_new() {
        let setting = Setting::new(1, "my-service", "dev", "feature.enabled", "true");

        assert_eq!(setting.id, 1);
        assert_eq!(setting.service_name, "my-service");
        assert_eq!(setting.env, "dev");
        assert_eq!(setting.key, "feature.enabled");
        assert_eq!(setting.value, "true");
        assert_eq!(setting.value_type, SettingValueType::String);
        assert!(setting.description.is_none());
        assert!(setting.is_active);
    }

    #[test]
    fn test_setting_new_with_string_types() {
        let setting = Setting::new(
            1,
            String::from("service"),
            String::from("prod"),
            String::from("key"),
            String::from("value"),
        );
        assert_eq!(setting.service_name, "service");
    }

    #[test]
    fn test_setting_with_value_type() {
        let setting = Setting::new(1, "service", "dev", "timeout", "30")
            .with_value_type(SettingValueType::Int);
        assert_eq!(setting.value_type, SettingValueType::Int);
    }

    #[test]
    fn test_setting_with_description() {
        let setting = Setting::new(1, "service", "dev", "key", "value")
            .with_description("This is a description");
        assert_eq!(setting.description, Some("This is a description".to_string()));
    }

    #[test]
    fn test_setting_with_active() {
        let setting = Setting::new(1, "service", "dev", "key", "value")
            .with_active(false);
        assert!(!setting.is_active);
    }

    #[test]
    fn test_setting_builder_chain() {
        let setting = Setting::new(1, "service", "dev", "key", "42")
            .with_value_type(SettingValueType::Int)
            .with_description("Some setting")
            .with_active(true);

        assert_eq!(setting.value_type, SettingValueType::Int);
        assert_eq!(setting.description, Some("Some setting".to_string()));
        assert!(setting.is_active);
    }

    #[test]
    fn test_setting_timestamps_are_set() {
        let before = SystemTime::now();
        let setting = Setting::new(1, "service", "dev", "key", "value");
        let after = SystemTime::now();

        assert!(setting.created_at >= before);
        assert!(setting.created_at <= after);
        assert!(setting.updated_at >= before);
        assert!(setting.updated_at <= after);
    }

    // ========================================
    // Setting Value Conversion Tests
    // ========================================

    #[test]
    fn test_setting_as_bool_true_values() {
        let values = ["true", "TRUE", "True", "1", "yes", "YES"];
        for val in values {
            let setting = Setting::new(1, "service", "dev", "key", val);
            assert_eq!(setting.as_bool(), Some(true), "Failed for value: {}", val);
        }
    }

    #[test]
    fn test_setting_as_bool_false_values() {
        let values = ["false", "FALSE", "False", "0", "no", "NO"];
        for val in values {
            let setting = Setting::new(1, "service", "dev", "key", val);
            assert_eq!(setting.as_bool(), Some(false), "Failed for value: {}", val);
        }
    }

    #[test]
    fn test_setting_as_bool_invalid() {
        let values = ["invalid", "maybe", "2", "-1", "", "truee"];
        for val in values {
            let setting = Setting::new(1, "service", "dev", "key", val);
            assert_eq!(setting.as_bool(), None, "Should be None for value: {}", val);
        }
    }

    #[test]
    fn test_setting_as_int_valid() {
        let setting = Setting::new(1, "service", "dev", "key", "42");
        assert_eq!(setting.as_int(), Some(42));

        let setting = Setting::new(1, "service", "dev", "key", "-100");
        assert_eq!(setting.as_int(), Some(-100));

        let setting = Setting::new(1, "service", "dev", "key", "0");
        assert_eq!(setting.as_int(), Some(0));
    }

    #[test]
    fn test_setting_as_int_invalid() {
        let setting = Setting::new(1, "service", "dev", "key", "invalid");
        assert_eq!(setting.as_int(), None);

        let setting = Setting::new(1, "service", "dev", "key", "42.5");
        assert_eq!(setting.as_int(), None);

        let setting = Setting::new(1, "service", "dev", "key", "");
        assert_eq!(setting.as_int(), None);
    }

    #[test]
    fn test_setting_as_int_boundary() {
        let setting = Setting::new(1, "service", "dev", "key", &i64::MAX.to_string());
        assert_eq!(setting.as_int(), Some(i64::MAX));

        let setting = Setting::new(1, "service", "dev", "key", &i64::MIN.to_string());
        assert_eq!(setting.as_int(), Some(i64::MIN));
    }

    #[test]
    fn test_setting_as_string() {
        let setting = Setting::new(1, "service", "dev", "key", "some value");
        assert_eq!(setting.as_string(), "some value");

        let setting = Setting::new(1, "service", "dev", "key", "");
        assert_eq!(setting.as_string(), "");
    }

    // ========================================
    // SettingQuery Tests
    // ========================================

    #[test]
    fn test_setting_query_new() {
        let query = SettingQuery::new();
        assert!(query.service_name.is_none());
        assert!(query.env.is_none());
        assert!(query.key_prefix.is_none());
        assert!(query.page_size.is_none());
        assert!(query.page_token.is_none());
    }

    #[test]
    fn test_setting_query_default() {
        let query = SettingQuery::default();
        assert!(query.service_name.is_none());
    }

    #[test]
    fn test_setting_query_with_service_name() {
        let query = SettingQuery::new().with_service_name("my-service");
        assert_eq!(query.service_name, Some("my-service".to_string()));
    }

    #[test]
    fn test_setting_query_with_env() {
        let query = SettingQuery::new().with_env("prod");
        assert_eq!(query.env, Some("prod".to_string()));
    }

    #[test]
    fn test_setting_query_with_key_prefix() {
        let query = SettingQuery::new().with_key_prefix("feature.");
        assert_eq!(query.key_prefix, Some("feature.".to_string()));
    }

    #[test]
    fn test_setting_query_with_page_size() {
        let query = SettingQuery::new().with_page_size(50);
        assert_eq!(query.page_size, Some(50));
    }

    #[test]
    fn test_setting_query_with_page_token() {
        let query = SettingQuery::new().with_page_token("token123");
        assert_eq!(query.page_token, Some("token123".to_string()));
    }

    #[test]
    fn test_setting_query_builder_chain() {
        let query = SettingQuery::new()
            .with_service_name("my-service")
            .with_env("prod")
            .with_key_prefix("feature.")
            .with_page_size(50)
            .with_page_token("offset:100");

        assert_eq!(query.service_name, Some("my-service".to_string()));
        assert_eq!(query.env, Some("prod".to_string()));
        assert_eq!(query.key_prefix, Some("feature.".to_string()));
        assert_eq!(query.page_size, Some(50));
        assert_eq!(query.page_token, Some("offset:100".to_string()));
    }

    #[test]
    fn test_setting_query_clone() {
        let query = SettingQuery::new()
            .with_service_name("service")
            .with_env("dev");
        let cloned = query.clone();

        assert_eq!(query.service_name, cloned.service_name);
        assert_eq!(query.env, cloned.env);
    }

    // ========================================
    // SettingList Tests
    // ========================================

    #[test]
    fn test_setting_list_new_empty() {
        let list = SettingList::new(vec![]);
        assert!(list.settings.is_empty());
        assert!(list.next_page_token.is_none());
    }

    #[test]
    fn test_setting_list_new_with_settings() {
        let settings = vec![
            Setting::new(1, "svc", "dev", "key1", "val1"),
            Setting::new(2, "svc", "dev", "key2", "val2"),
        ];
        let list = SettingList::new(settings);

        assert_eq!(list.settings.len(), 2);
        assert!(list.next_page_token.is_none());
    }

    #[test]
    fn test_setting_list_with_next_page_token() {
        let list = SettingList::new(vec![])
            .with_next_page_token("offset:100");
        assert_eq!(list.next_page_token, Some("offset:100".to_string()));
    }

    #[test]
    fn test_setting_list_clone() {
        let settings = vec![Setting::new(1, "svc", "dev", "key", "val")];
        let list = SettingList::new(settings).with_next_page_token("token");
        let cloned = list.clone();

        assert_eq!(list.settings.len(), cloned.settings.len());
        assert_eq!(list.next_page_token, cloned.next_page_token);
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_setting_with_empty_strings() {
        let setting = Setting::new(1, "", "", "", "");
        assert_eq!(setting.service_name, "");
        assert_eq!(setting.env, "");
        assert_eq!(setting.key, "");
        assert_eq!(setting.value, "");
    }

    #[test]
    fn test_setting_with_unicode() {
        let setting = Setting::new(1, "サービス", "開発", "キー", "値");
        assert_eq!(setting.service_name, "サービス");
        assert_eq!(setting.env, "開発");
        assert_eq!(setting.key, "キー");
        assert_eq!(setting.value, "値");
    }

    #[test]
    fn test_setting_with_special_characters() {
        let setting = Setting::new(1, "my-service_v2", "dev-01", "key.sub.item", "{\"json\": true}");
        assert_eq!(setting.service_name, "my-service_v2");
        assert_eq!(setting.env, "dev-01");
        assert_eq!(setting.key, "key.sub.item");
        assert_eq!(setting.value, "{\"json\": true}");
    }

    #[test]
    fn test_setting_with_zero_id() {
        let setting = Setting::new(0, "service", "dev", "key", "value");
        assert_eq!(setting.id, 0);
    }

    #[test]
    fn test_setting_with_negative_id() {
        let setting = Setting::new(-1, "service", "dev", "key", "value");
        assert_eq!(setting.id, -1);
    }
}
