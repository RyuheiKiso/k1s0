use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Dependency はサービス間の依存関係を表す。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Dependency {
    pub source_service_id: Uuid,
    pub target_service_id: Uuid,
    pub dependency_type: DependencyType,
    pub description: Option<String>,
}

/// DependencyType は依存関係の種類を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Runtime,
    Build,
    Optional,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Runtime => write!(f, "runtime"),
            DependencyType::Build => write!(f, "build"),
            DependencyType::Optional => write!(f, "optional"),
        }
    }
}

impl std::str::FromStr for DependencyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "runtime" => Ok(DependencyType::Runtime),
            "build" => Ok(DependencyType::Build),
            "optional" => Ok(DependencyType::Optional),
            _ => Err(format!("invalid dependency type: {}", s)),
        }
    }
}
