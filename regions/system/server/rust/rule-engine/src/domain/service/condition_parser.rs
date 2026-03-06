use crate::domain::entity::condition::{Combinator, ConditionNode, Operator};

pub struct ConditionParser;

impl ConditionParser {
    pub fn parse(json: &serde_json::Value) -> Result<ConditionNode, String> {
        Self::parse_node(json)
    }

    fn parse_node(json: &serde_json::Value) -> Result<ConditionNode, String> {
        let obj = json
            .as_object()
            .ok_or_else(|| "condition node must be a JSON object".to_string())?;

        // Check for combinator (all/any/none)
        for key in &["all", "any", "none"] {
            if let Some(children_val) = obj.get(*key) {
                let combinator = match *key {
                    "all" => Combinator::All,
                    "any" => Combinator::Any,
                    "none" => Combinator::None,
                    _ => unreachable!(),
                };

                let children_arr = children_val
                    .as_array()
                    .ok_or_else(|| format!("'{}' must be an array", key))?;

                let mut children = Vec::with_capacity(children_arr.len());
                for child in children_arr {
                    children.push(Self::parse_node(child)?);
                }

                return Ok(ConditionNode {
                    combinator: Some(combinator),
                    children: Some(children),
                    field: None,
                    operator: None,
                    value: None,
                });
            }
        }

        // Leaf condition: field + operator + value
        let field = obj
            .get("field")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "leaf condition must have 'field' string".to_string())?
            .to_string();

        let operator_str = obj
            .get("operator")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "leaf condition must have 'operator' string".to_string())?;

        let operator = Self::parse_operator(operator_str)?;

        let value = obj.get("value").cloned();

        Ok(ConditionNode {
            combinator: None,
            children: None,
            field: Some(field),
            operator: Some(operator),
            value,
        })
    }

    fn parse_operator(s: &str) -> Result<Operator, String> {
        match s {
            "eq" => Ok(Operator::Eq),
            "ne" => Ok(Operator::Ne),
            "gt" => Ok(Operator::Gt),
            "gte" => Ok(Operator::Gte),
            "lt" => Ok(Operator::Lt),
            "lte" => Ok(Operator::Lte),
            "in" => Ok(Operator::In),
            "not_in" => Ok(Operator::NotIn),
            "contains" => Ok(Operator::Contains),
            "regex" => Ok(Operator::Regex),
            other => Err(format!("unknown operator: '{}'", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_leaf() {
        let json = serde_json::json!({
            "field": "status",
            "operator": "eq",
            "value": "active"
        });
        let node = ConditionParser::parse(&json).unwrap();
        assert!(node.combinator.is_none());
        assert_eq!(node.field.as_deref(), Some("status"));
        assert_eq!(node.operator, Some(Operator::Eq));
        assert_eq!(node.value, Some(serde_json::json!("active")));
    }

    #[test]
    fn parse_combinator_all() {
        let json = serde_json::json!({
            "all": [
                { "field": "category", "operator": "eq", "value": "food" },
                { "field": "is_takeout", "operator": "eq", "value": true }
            ]
        });
        let node = ConditionParser::parse(&json).unwrap();
        assert_eq!(node.combinator, Some(Combinator::All));
        assert_eq!(node.children.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn parse_nested_combinators() {
        let json = serde_json::json!({
            "any": [
                {
                    "all": [
                        { "field": "a", "operator": "gt", "value": 10 },
                        { "field": "b", "operator": "lt", "value": 5 }
                    ]
                },
                { "field": "c", "operator": "eq", "value": "x" }
            ]
        });
        let node = ConditionParser::parse(&json).unwrap();
        assert_eq!(node.combinator, Some(Combinator::Any));
        let children = node.children.unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].combinator, Some(Combinator::All));
    }

    #[test]
    fn parse_unknown_operator_fails() {
        let json = serde_json::json!({
            "field": "x",
            "operator": "between",
            "value": [1, 10]
        });
        let err = ConditionParser::parse(&json).unwrap_err();
        assert!(err.contains("unknown operator: 'between'"));
    }

    #[test]
    fn parse_missing_field_fails() {
        let json = serde_json::json!({
            "operator": "eq",
            "value": "x"
        });
        let err = ConditionParser::parse(&json).unwrap_err();
        assert!(err.contains("'field'"));
    }

    #[test]
    fn parse_in_operator() {
        let json = serde_json::json!({
            "field": "region",
            "operator": "in",
            "value": ["JP", "US"]
        });
        let node = ConditionParser::parse(&json).unwrap();
        assert_eq!(node.operator, Some(Operator::In));
    }

    #[test]
    fn parse_none_combinator() {
        let json = serde_json::json!({
            "none": [
                { "field": "status", "operator": "eq", "value": "deleted" }
            ]
        });
        let node = ConditionParser::parse(&json).unwrap();
        assert_eq!(node.combinator, Some(Combinator::None));
    }
}
