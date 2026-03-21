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

#[cfg(test)]
mod tests {
    use super::*;

    /// DependencyType の Display 変換テスト
    #[test]
    fn test_dependency_type_display() {
        assert_eq!(DependencyType::Runtime.to_string(), "runtime");
        assert_eq!(DependencyType::Build.to_string(), "build");
        assert_eq!(DependencyType::Optional.to_string(), "optional");
    }

    /// DependencyType の FromStr 正常変換テスト
    #[test]
    fn test_dependency_type_from_str_valid() {
        use std::str::FromStr;
        assert_eq!(DependencyType::from_str("runtime").unwrap(), DependencyType::Runtime);
        assert_eq!(DependencyType::from_str("build").unwrap(), DependencyType::Build);
        assert_eq!(DependencyType::from_str("optional").unwrap(), DependencyType::Optional);
    }

    /// DependencyType の FromStr は大文字小文字を区別しない
    #[test]
    fn test_dependency_type_from_str_case_insensitive() {
        use std::str::FromStr;
        assert_eq!(DependencyType::from_str("RUNTIME").unwrap(), DependencyType::Runtime);
        assert_eq!(DependencyType::from_str("Build").unwrap(), DependencyType::Build);
    }

    /// 不明な文字列は Err を返す
    #[test]
    fn test_dependency_type_from_str_invalid() {
        use std::str::FromStr;
        assert!(DependencyType::from_str("unknown").is_err());
    }
}
