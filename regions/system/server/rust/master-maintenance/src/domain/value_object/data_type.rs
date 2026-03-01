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
