use crate::domain::entity::condition::{Combinator, ConditionNode, Operator};

pub struct ConditionEvaluator;

impl ConditionEvaluator {
    pub fn evaluate(
        condition: &ConditionNode,
        context: &serde_json::Value,
    ) -> Result<bool, String> {
        if let Some(ref combinator) = condition.combinator {
            let children = condition
                .children
                .as_ref()
                .ok_or_else(|| "combinator node must have children".to_string())?;

            match combinator {
                Combinator::All => {
                    for child in children {
                        if !Self::evaluate(child, context)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                Combinator::Any => {
                    for child in children {
                        if Self::evaluate(child, context)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                Combinator::None => {
                    for child in children {
                        if Self::evaluate(child, context)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
            }
        } else {
            // Leaf condition
            let field = condition
                .field
                .as_ref()
                .ok_or_else(|| "leaf condition must have 'field'".to_string())?;
            let operator = condition
                .operator
                .as_ref()
                .ok_or_else(|| "leaf condition must have 'operator'".to_string())?;

            let actual = Self::resolve_field(context, field);
            let expected = condition.value.as_ref();

            Self::evaluate_operator(operator, actual, expected)
        }
    }

    fn resolve_field<'a>(
        context: &'a serde_json::Value,
        path: &str,
    ) -> Option<&'a serde_json::Value> {
        let mut current = context;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        Some(current)
    }

    fn evaluate_operator(
        operator: &Operator,
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        match operator {
            Operator::Eq => Ok(actual == expected),
            Operator::Ne => Ok(actual != expected),
            Operator::Gt => Self::compare_numeric(actual, expected, |a, b| a > b),
            Operator::Gte => Self::compare_numeric(actual, expected, |a, b| a >= b),
            Operator::Lt => Self::compare_numeric(actual, expected, |a, b| a < b),
            Operator::Lte => Self::compare_numeric(actual, expected, |a, b| a <= b),
            Operator::In => Self::evaluate_in(actual, expected),
            Operator::NotIn => Self::evaluate_in(actual, expected).map(|v| !v),
            Operator::Contains => Self::evaluate_contains(actual, expected),
            Operator::Regex => Self::evaluate_regex(actual, expected),
        }
    }

    fn compare_numeric(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
        cmp: fn(f64, f64) -> bool,
    ) -> Result<bool, String> {
        let a = actual
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "actual value is not a number".to_string())?;
        let b = expected
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "expected value is not a number".to_string())?;
        Ok(cmp(a, b))
    }

    fn evaluate_in(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        let actual_val =
            actual.ok_or_else(|| "actual value is null for 'in' operator".to_string())?;
        let arr = expected
            .and_then(|v| v.as_array())
            .ok_or_else(|| "expected value for 'in' operator must be an array".to_string())?;
        Ok(arr.contains(actual_val))
    }

    fn evaluate_contains(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        let haystack = actual
            .and_then(|v| v.as_str())
            .ok_or_else(|| "actual value is not a string for 'contains'".to_string())?;
        let needle = expected
            .and_then(|v| v.as_str())
            .ok_or_else(|| "expected value is not a string for 'contains'".to_string())?;
        Ok(haystack.contains(needle))
    }

    fn evaluate_regex(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        let text = actual
            .and_then(|v| v.as_str())
            .ok_or_else(|| "actual value is not a string for 'regex'".to_string())?;
        let pattern = expected
            .and_then(|v| v.as_str())
            .ok_or_else(|| "expected value is not a string for 'regex'".to_string())?;
        let re = regex::Regex::new(pattern)
            .map_err(|e| format!("invalid regex pattern '{}': {}", pattern, e))?;
        Ok(re.is_match(text))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::service::condition_parser::ConditionParser;

    fn eval(condition_json: serde_json::Value, context: serde_json::Value) -> bool {
        let node = ConditionParser::parse(&condition_json).unwrap();
        ConditionEvaluator::evaluate(&node, &context).unwrap()
    }

    #[test]
    fn test_eq_match() {
        assert!(eval(
            serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
            serde_json::json!({"status": "active"}),
        ));
    }

    #[test]
    fn test_eq_no_match() {
        assert!(!eval(
            serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
            serde_json::json!({"status": "inactive"}),
        ));
    }

    #[test]
    fn test_ne() {
        assert!(eval(
            serde_json::json!({"field": "status", "operator": "ne", "value": "deleted"}),
            serde_json::json!({"status": "active"}),
        ));
    }

    #[test]
    fn test_gt() {
        assert!(eval(
            serde_json::json!({"field": "amount", "operator": "gt", "value": 100}),
            serde_json::json!({"amount": 200}),
        ));
    }

    #[test]
    fn test_gte() {
        assert!(eval(
            serde_json::json!({"field": "amount", "operator": "gte", "value": 100}),
            serde_json::json!({"amount": 100}),
        ));
    }

    #[test]
    fn test_lt() {
        assert!(eval(
            serde_json::json!({"field": "score", "operator": "lt", "value": 50}),
            serde_json::json!({"score": 30}),
        ));
    }

    #[test]
    fn test_lte() {
        assert!(eval(
            serde_json::json!({"field": "score", "operator": "lte", "value": 50}),
            serde_json::json!({"score": 50}),
        ));
    }

    #[test]
    fn test_in() {
        assert!(eval(
            serde_json::json!({"field": "region", "operator": "in", "value": ["JP", "US"]}),
            serde_json::json!({"region": "JP"}),
        ));
    }

    #[test]
    fn test_not_in() {
        assert!(eval(
            serde_json::json!({"field": "region", "operator": "not_in", "value": ["JP", "US"]}),
            serde_json::json!({"region": "UK"}),
        ));
    }

    #[test]
    fn test_contains() {
        assert!(eval(
            serde_json::json!({"field": "name", "operator": "contains", "value": "special"}),
            serde_json::json!({"name": "this is special item"}),
        ));
    }

    #[test]
    fn test_regex() {
        assert!(eval(
            serde_json::json!({"field": "code", "operator": "regex", "value": "^TAX-\\d{4}$"}),
            serde_json::json!({"code": "TAX-1234"}),
        ));
    }

    #[test]
    fn test_nested_field_dot_notation() {
        assert!(eval(
            serde_json::json!({"field": "item.category", "operator": "eq", "value": "food"}),
            serde_json::json!({"item": {"category": "food"}}),
        ));
    }

    #[test]
    fn test_all_combinator() {
        assert!(eval(
            serde_json::json!({
                "all": [
                    {"field": "item.category", "operator": "eq", "value": "food"},
                    {"field": "item.is_takeout", "operator": "eq", "value": true}
                ]
            }),
            serde_json::json!({"item": {"category": "food", "is_takeout": true}}),
        ));
    }

    #[test]
    fn test_all_combinator_fail() {
        assert!(!eval(
            serde_json::json!({
                "all": [
                    {"field": "item.category", "operator": "eq", "value": "food"},
                    {"field": "item.is_takeout", "operator": "eq", "value": true}
                ]
            }),
            serde_json::json!({"item": {"category": "food", "is_takeout": false}}),
        ));
    }

    #[test]
    fn test_any_combinator() {
        assert!(eval(
            serde_json::json!({
                "any": [
                    {"field": "status", "operator": "eq", "value": "active"},
                    {"field": "status", "operator": "eq", "value": "pending"}
                ]
            }),
            serde_json::json!({"status": "pending"}),
        ));
    }

    #[test]
    fn test_none_combinator() {
        assert!(eval(
            serde_json::json!({
                "none": [
                    {"field": "status", "operator": "eq", "value": "deleted"},
                    {"field": "status", "operator": "eq", "value": "banned"}
                ]
            }),
            serde_json::json!({"status": "active"}),
        ));
    }

    #[test]
    fn test_missing_field_returns_none() {
        assert!(!eval(
            serde_json::json!({"field": "nonexistent", "operator": "eq", "value": "x"}),
            serde_json::json!({"status": "active"}),
        ));
    }
}
