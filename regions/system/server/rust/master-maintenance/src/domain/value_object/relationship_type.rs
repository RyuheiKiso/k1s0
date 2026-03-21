use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OneToOne => write!(f, "one_to_one"),
            Self::OneToMany => write!(f, "one_to_many"),
            Self::ManyToOne => write!(f, "many_to_one"),
            Self::ManyToMany => write!(f, "many_to_many"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Display traitが小文字のスネークケースを出力する
    #[test]
    fn display_outputs_snake_case() {
        assert_eq!(RelationshipType::OneToOne.to_string(), "one_to_one");
        assert_eq!(RelationshipType::OneToMany.to_string(), "one_to_many");
        assert_eq!(RelationshipType::ManyToOne.to_string(), "many_to_one");
        assert_eq!(RelationshipType::ManyToMany.to_string(), "many_to_many");
    }

    /// serdeのデシリアライズがsnake_caseに対応している
    #[test]
    fn deserialize_from_snake_case() {
        let rt: RelationshipType = serde_json::from_str("\"one_to_many\"").unwrap();
        assert_eq!(rt, RelationshipType::OneToMany);
    }

    /// PartialEqによる等値比較が機能する
    #[test]
    fn equality_comparison() {
        assert_eq!(RelationshipType::OneToOne, RelationshipType::OneToOne);
        assert_ne!(RelationshipType::OneToMany, RelationshipType::ManyToOne);
    }
}
