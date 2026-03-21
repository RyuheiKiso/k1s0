use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Text,
    Integer,
    Decimal,
    Boolean,
    Date,
    Datetime,
    Uuid,
    Jsonb,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Integer => write!(f, "integer"),
            Self::Decimal => write!(f, "decimal"),
            Self::Boolean => write!(f, "boolean"),
            Self::Date => write!(f, "date"),
            Self::Datetime => write!(f, "datetime"),
            Self::Uuid => write!(f, "uuid"),
            Self::Jsonb => write!(f, "jsonb"),
        }
    }
}

impl DataType {
    pub fn to_sql_type(&self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Integer => "INTEGER",
            Self::Decimal => "NUMERIC",
            Self::Boolean => "BOOLEAN",
            Self::Date => "DATE",
            Self::Datetime => "TIMESTAMPTZ",
            Self::Uuid => "UUID",
            Self::Jsonb => "JSONB",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Display traitが小文字のデータ型名を出力する
    #[test]
    fn display_outputs_lowercase_name() {
        assert_eq!(DataType::Text.to_string(), "text");
        assert_eq!(DataType::Integer.to_string(), "integer");
        assert_eq!(DataType::Decimal.to_string(), "decimal");
        assert_eq!(DataType::Boolean.to_string(), "boolean");
    }

    /// to_sql_type が対応するSQL型名を返す
    #[test]
    fn to_sql_type_returns_sql_name() {
        assert_eq!(DataType::Text.to_sql_type(), "TEXT");
        assert_eq!(DataType::Integer.to_sql_type(), "INTEGER");
        assert_eq!(DataType::Uuid.to_sql_type(), "UUID");
        assert_eq!(DataType::Jsonb.to_sql_type(), "JSONB");
    }

    /// serde_json からのデシリアライズが正しく動作する
    #[test]
    fn deserialize_from_string() {
        let dt: DataType = serde_json::from_str("\"integer\"").unwrap();
        assert_eq!(dt, DataType::Integer);
        let dt2: DataType = serde_json::from_str("\"datetime\"").unwrap();
        assert_eq!(dt2, DataType::Datetime);
    }

    /// 全バリアントがto_sql_typeを持つ（網羅的チェック）
    #[test]
    fn all_variants_have_sql_type() {
        let all = [
            DataType::Text,
            DataType::Integer,
            DataType::Decimal,
            DataType::Boolean,
            DataType::Date,
            DataType::Datetime,
            DataType::Uuid,
            DataType::Jsonb,
        ];
        for dt in &all {
            assert!(!dt.to_sql_type().is_empty());
        }
    }
}
