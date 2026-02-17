use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::collections::HashMap;
use tera::{Result, Tera, Value};

/// Tera テンプレートエンジンにカスタムフィルタを登録する。
///
/// 登録されるフィルタ:
/// - `snake_case`: スネークケースに変換
/// - `pascal_case`: パスカルケースに変換
/// - `camel_case`: キャメルケースに変換
/// - `kebab_case`: ケバブケースに変換
/// - `upper_case`: 全て大文字に変換
/// - `lower_case`: 全て小文字に変換
pub fn register_filters(tera: &mut Tera) {
    tera.register_filter("snake_case", to_snake_case);
    tera.register_filter("pascal_case", to_pascal_case);
    tera.register_filter("camel_case", to_camel_case);
    tera.register_filter("kebab_case", to_kebab_case);
    tera.register_filter("upper_case", to_upper_case);
    tera.register_filter("lower_case", to_lower_case);
}

/// 文字列値を Value から取り出すヘルパー。
fn extract_str<'a>(value: &'a Value, filter_name: &str) -> Result<&'a str> {
    value
        .as_str()
        .ok_or_else(|| tera::Error::msg(format!("{} フィルタは文字列に対してのみ使用できます", filter_name)))
}

/// スネークケースに変換するフィルタ。
/// 例: "order-api" -> "order_api"
fn to_snake_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = extract_str(value, "snake_case")?;
    Ok(Value::String(s.to_snake_case()))
}

/// パスカルケースに変換するフィルタ。
/// 例: "order-api" -> "OrderApi"
fn to_pascal_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = extract_str(value, "pascal_case")?;
    Ok(Value::String(s.to_pascal_case()))
}

/// キャメルケースに変換するフィルタ。
/// 例: "order-api" -> "orderApi"
fn to_camel_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = extract_str(value, "camel_case")?;
    Ok(Value::String(s.to_lower_camel_case()))
}

/// ケバブケースに変換するフィルタ。
/// 例: "order_api" -> "order-api"
fn to_kebab_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = extract_str(value, "kebab_case")?;
    Ok(Value::String(s.to_kebab_case()))
}

/// 全て大文字に変換するフィルタ。
/// 例: "order-api" -> "ORDER-API"
fn to_upper_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = extract_str(value, "upper_case")?;
    Ok(Value::String(s.to_uppercase()))
}

/// 全て小文字に変換するフィルタ。
/// 例: "OrderApi" -> "orderapi"
fn to_lower_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = extract_str(value, "lower_case")?;
    Ok(Value::String(s.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // snake_case フィルタのテスト
    // =========================================================================

    #[test]
    fn test_snake_case_from_kebab() {
        let value = Value::String("order-api".to_string());
        let result = to_snake_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("order_api".to_string()));
    }

    #[test]
    fn test_snake_case_from_kebab_multi_segment() {
        let value = Value::String("user-auth-service".to_string());
        let result = to_snake_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("user_auth_service".to_string()));
    }

    #[test]
    fn test_snake_case_single_word() {
        let value = Value::String("inventory".to_string());
        let result = to_snake_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("inventory".to_string()));
    }

    #[test]
    fn test_snake_case_from_pascal() {
        let value = Value::String("MyProjectName".to_string());
        let result = to_snake_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("my_project_name".to_string()));
    }

    // =========================================================================
    // pascal_case フィルタのテスト
    // =========================================================================

    #[test]
    fn test_pascal_case_from_kebab() {
        let value = Value::String("order-api".to_string());
        let result = to_pascal_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("OrderApi".to_string()));
    }

    #[test]
    fn test_pascal_case_from_kebab_multi_segment() {
        let value = Value::String("user-auth-service".to_string());
        let result = to_pascal_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("UserAuthService".to_string()));
    }

    #[test]
    fn test_pascal_case_single_word() {
        let value = Value::String("inventory".to_string());
        let result = to_pascal_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("Inventory".to_string()));
    }

    #[test]
    fn test_pascal_case_from_snake() {
        let value = Value::String("my_project_name".to_string());
        let result = to_pascal_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("MyProjectName".to_string()));
    }

    // =========================================================================
    // camel_case フィルタのテスト
    // =========================================================================

    #[test]
    fn test_camel_case_from_kebab() {
        let value = Value::String("order-api".to_string());
        let result = to_camel_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("orderApi".to_string()));
    }

    #[test]
    fn test_camel_case_from_kebab_multi_segment() {
        let value = Value::String("user-auth-service".to_string());
        let result = to_camel_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("userAuthService".to_string()));
    }

    #[test]
    fn test_camel_case_single_word() {
        let value = Value::String("inventory".to_string());
        let result = to_camel_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("inventory".to_string()));
    }

    #[test]
    fn test_camel_case_from_pascal() {
        let value = Value::String("MyProjectName".to_string());
        let result = to_camel_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("myProjectName".to_string()));
    }

    // =========================================================================
    // kebab_case フィルタのテスト
    // =========================================================================

    #[test]
    fn test_kebab_case_from_snake() {
        let value = Value::String("order_api".to_string());
        let result = to_kebab_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("order-api".to_string()));
    }

    #[test]
    fn test_kebab_case_from_pascal() {
        let value = Value::String("MyProjectName".to_string());
        let result = to_kebab_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("my-project-name".to_string()));
    }

    #[test]
    fn test_kebab_case_single_word() {
        let value = Value::String("inventory".to_string());
        let result = to_kebab_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("inventory".to_string()));
    }

    #[test]
    fn test_kebab_case_from_camel() {
        let value = Value::String("orderApi".to_string());
        let result = to_kebab_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("order-api".to_string()));
    }

    // =========================================================================
    // upper_case フィルタのテスト
    // =========================================================================

    #[test]
    fn test_upper_case_from_kebab() {
        let value = Value::String("order-api".to_string());
        let result = to_upper_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("ORDER-API".to_string()));
    }

    #[test]
    fn test_upper_case_from_snake() {
        let value = Value::String("order_api".to_string());
        let result = to_upper_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("ORDER_API".to_string()));
    }

    #[test]
    fn test_upper_case_single_word() {
        let value = Value::String("inventory".to_string());
        let result = to_upper_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("INVENTORY".to_string()));
    }

    #[test]
    fn test_upper_case_from_mixed() {
        let value = Value::String("OrderApi".to_string());
        let result = to_upper_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("ORDERAPI".to_string()));
    }

    // =========================================================================
    // lower_case フィルタのテスト
    // =========================================================================

    #[test]
    fn test_lower_case_from_pascal() {
        let value = Value::String("OrderApi".to_string());
        let result = to_lower_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("orderapi".to_string()));
    }

    #[test]
    fn test_lower_case_from_upper() {
        let value = Value::String("ORDER-API".to_string());
        let result = to_lower_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("order-api".to_string()));
    }

    #[test]
    fn test_lower_case_already_lower() {
        let value = Value::String("inventory".to_string());
        let result = to_lower_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("inventory".to_string()));
    }

    #[test]
    fn test_lower_case_from_screaming_snake() {
        let value = Value::String("ORDER_API".to_string());
        let result = to_lower_case(&value, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("order_api".to_string()));
    }

    // =========================================================================
    // エラーケースのテスト
    // =========================================================================

    #[test]
    fn test_snake_case_with_non_string() {
        let value = Value::Number(serde_json::Number::from(42));
        let result = to_snake_case(&value, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_pascal_case_with_non_string() {
        let value = Value::Bool(true);
        let result = to_pascal_case(&value, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_camel_case_with_non_string() {
        let value = Value::Null;
        let result = to_camel_case(&value, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_kebab_case_with_non_string() {
        let value = Value::Number(serde_json::Number::from(0));
        let result = to_kebab_case(&value, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_upper_case_with_non_string() {
        let value = Value::Bool(false);
        let result = to_upper_case(&value, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_lower_case_with_non_string() {
        let value = Value::Number(serde_json::Number::from(99));
        let result = to_lower_case(&value, &HashMap::new());
        assert!(result.is_err());
    }

    // =========================================================================
    // register_filters 統合テスト
    // =========================================================================

    #[test]
    fn test_register_filters_in_tera() {
        let mut tera = Tera::default();
        register_filters(&mut tera);

        // snake_case フィルタが登録されていることをテンプレートレンダリングで検証
        tera.add_raw_template("test_snake", "{{ val | snake_case }}")
            .unwrap();
        let mut ctx = tera::Context::new();
        ctx.insert("val", "order-api");
        let result = tera.render("test_snake", &ctx).unwrap();
        assert_eq!(result, "order_api");

        // pascal_case フィルタが登録されていることを検証
        tera.add_raw_template("test_pascal", "{{ val | pascal_case }}")
            .unwrap();
        let result = tera.render("test_pascal", &ctx).unwrap();
        assert_eq!(result, "OrderApi");

        // camel_case フィルタが登録されていることを検証
        tera.add_raw_template("test_camel", "{{ val | camel_case }}")
            .unwrap();
        let result = tera.render("test_camel", &ctx).unwrap();
        assert_eq!(result, "orderApi");

        // kebab_case フィルタが登録されていることを検証
        tera.add_raw_template("test_kebab", "{{ val | kebab_case }}")
            .unwrap();
        ctx.insert("val", "order_api");
        let result = tera.render("test_kebab", &ctx).unwrap();
        assert_eq!(result, "order-api");

        // upper_case フィルタが登録されていることを検証
        tera.add_raw_template("test_upper", "{{ val | upper_case }}")
            .unwrap();
        ctx.insert("val", "order-api");
        let result = tera.render("test_upper", &ctx).unwrap();
        assert_eq!(result, "ORDER-API");

        // lower_case フィルタが登録されていることを検証
        tera.add_raw_template("test_lower", "{{ val | lower_case }}")
            .unwrap();
        ctx.insert("val", "OrderApi");
        let result = tera.render("test_lower", &ctx).unwrap();
        assert_eq!(result, "orderapi");
    }
}
