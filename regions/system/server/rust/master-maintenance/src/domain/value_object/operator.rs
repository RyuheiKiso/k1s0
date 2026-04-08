use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Exists,
    NotExists,
    Regex,
    Between,
}

impl Operator {
    #[must_use] 
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::Exists => "EXISTS",
            Self::NotExists => "NOT EXISTS",
            Self::Regex => "~",
            Self::Between => "BETWEEN",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 比較演算子がSQL文字列に変換される
    #[test]
    fn comparison_operators_to_sql() {
        assert_eq!(Operator::Eq.to_sql(), "=");
        assert_eq!(Operator::Neq.to_sql(), "!=");
        assert_eq!(Operator::Gt.to_sql(), ">");
        assert_eq!(Operator::Lte.to_sql(), "<=");
    }

    /// IN/NOT IN演算子がSQL文字列に変換される
    #[test]
    fn in_operators_to_sql() {
        assert_eq!(Operator::In.to_sql(), "IN");
        assert_eq!(Operator::NotIn.to_sql(), "NOT IN");
    }

    /// 存在確認演算子がSQL文字列に変換される
    #[test]
    fn existence_operators_to_sql() {
        assert_eq!(Operator::Exists.to_sql(), "EXISTS");
        assert_eq!(Operator::NotExists.to_sql(), "NOT EXISTS");
    }

    /// serdeのデシリアライズがsnake_caseに対応している
    #[test]
    fn deserialize_from_snake_case() {
        let op: Operator = serde_json::from_str("\"not_in\"").unwrap();
        assert_eq!(op, Operator::NotIn);
        let op2: Operator = serde_json::from_str("\"not_exists\"").unwrap();
        assert_eq!(op2, Operator::NotExists);
    }
}
