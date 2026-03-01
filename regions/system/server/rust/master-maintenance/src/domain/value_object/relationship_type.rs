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
