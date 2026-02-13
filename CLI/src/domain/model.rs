use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Frontend,
    Backend,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    React,
    Flutter,
    RustAxum,
    GoGin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Database {
    PostgreSql,
    None,
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub project_type: ProjectType,
    pub template: Template,
    pub database: Database,
    pub path: PathBuf,
}

impl Template {
    pub fn is_compatible_with(&self, project_type: &ProjectType) -> bool {
        match (self, project_type) {
            (Template::React, ProjectType::Frontend) => true,
            (Template::Flutter, ProjectType::Frontend) => true,
            (Template::RustAxum, ProjectType::Backend) => true,
            (Template::GoGin, ProjectType::Backend) => true,
            _ => false,
        }
    }

    pub fn templates_for(project_type: &ProjectType) -> Vec<Template> {
        match project_type {
            ProjectType::Frontend => vec![Template::React, Template::Flutter],
            ProjectType::Backend => vec![Template::RustAxum, Template::GoGin],
        }
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Frontend => write!(f, "frontend"),
            ProjectType::Backend => write!(f, "backend"),
        }
    }
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Template::React => write!(f, "react"),
            Template::Flutter => write!(f, "flutter"),
            Template::RustAxum => write!(f, "rust"),
            Template::GoGin => write!(f, "go"),
        }
    }
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Database::PostgreSql => write!(f, "postgresql"),
            Database::None => write!(f, "none"),
        }
    }
}

impl std::str::FromStr for Template {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "react" => Ok(Template::React),
            "flutter" => Ok(Template::Flutter),
            "rust" | "rust-axum" | "rust_axum" => Ok(Template::RustAxum),
            "go" | "go-gin" | "go_gin" => Ok(Template::GoGin),
            _ => Err(format!("Unknown template: {s}")),
        }
    }
}

impl std::str::FromStr for Database {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" | "pg" => Ok(Database::PostgreSql),
            "none" | "" => Ok(Database::None),
            _ => Err(format!("Unknown database: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_creation() {
        let frontend = ProjectType::Frontend;
        let backend = ProjectType::Backend;
        assert_eq!(frontend, ProjectType::Frontend);
        assert_eq!(backend, ProjectType::Backend);
        assert_ne!(frontend, backend);
    }

    #[test]
    fn test_template_creation() {
        assert_eq!(Template::React, Template::React);
        assert_eq!(Template::Flutter, Template::Flutter);
        assert_eq!(Template::RustAxum, Template::RustAxum);
        assert_eq!(Template::GoGin, Template::GoGin);
    }

    #[test]
    fn test_database_creation() {
        assert_eq!(Database::PostgreSql, Database::PostgreSql);
        assert_eq!(Database::None, Database::None);
        assert_ne!(Database::PostgreSql, Database::None);
    }

    #[test]
    fn test_project_config_construction() {
        let config = ProjectConfig {
            name: "my-app".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::React,
            database: Database::None,
            path: PathBuf::from("/tmp/my-app"),
        };
        assert_eq!(config.name, "my-app");
        assert_eq!(config.project_type, ProjectType::Frontend);
        assert_eq!(config.template, Template::React);
        assert_eq!(config.database, Database::None);
        assert_eq!(config.path, PathBuf::from("/tmp/my-app"));
    }

    #[test]
    fn test_template_compatibility_frontend() {
        assert!(Template::React.is_compatible_with(&ProjectType::Frontend));
        assert!(Template::Flutter.is_compatible_with(&ProjectType::Frontend));
        assert!(!Template::RustAxum.is_compatible_with(&ProjectType::Frontend));
        assert!(!Template::GoGin.is_compatible_with(&ProjectType::Frontend));
    }

    #[test]
    fn test_template_compatibility_backend() {
        assert!(!Template::React.is_compatible_with(&ProjectType::Backend));
        assert!(!Template::Flutter.is_compatible_with(&ProjectType::Backend));
        assert!(Template::RustAxum.is_compatible_with(&ProjectType::Backend));
        assert!(Template::GoGin.is_compatible_with(&ProjectType::Backend));
    }

    #[test]
    fn test_templates_for_frontend() {
        let templates = Template::templates_for(&ProjectType::Frontend);
        assert_eq!(templates, vec![Template::React, Template::Flutter]);
    }

    #[test]
    fn test_templates_for_backend() {
        let templates = Template::templates_for(&ProjectType::Backend);
        assert_eq!(templates, vec![Template::RustAxum, Template::GoGin]);
    }

    #[test]
    fn test_template_from_str() {
        assert_eq!("react".parse::<Template>().unwrap(), Template::React);
        assert_eq!("flutter".parse::<Template>().unwrap(), Template::Flutter);
        assert_eq!("rust".parse::<Template>().unwrap(), Template::RustAxum);
        assert_eq!("rust-axum".parse::<Template>().unwrap(), Template::RustAxum);
        assert_eq!("go".parse::<Template>().unwrap(), Template::GoGin);
        assert_eq!("go-gin".parse::<Template>().unwrap(), Template::GoGin);
        assert!("unknown".parse::<Template>().is_err());
    }

    #[test]
    fn test_database_from_str() {
        assert_eq!("postgresql".parse::<Database>().unwrap(), Database::PostgreSql);
        assert_eq!("postgres".parse::<Database>().unwrap(), Database::PostgreSql);
        assert_eq!("pg".parse::<Database>().unwrap(), Database::PostgreSql);
        assert_eq!("none".parse::<Database>().unwrap(), Database::None);
        assert!("unknown".parse::<Database>().is_err());
    }

    #[test]
    fn test_display_traits() {
        assert_eq!(ProjectType::Frontend.to_string(), "frontend");
        assert_eq!(ProjectType::Backend.to_string(), "backend");
        assert_eq!(Template::React.to_string(), "react");
        assert_eq!(Template::RustAxum.to_string(), "rust");
        assert_eq!(Database::PostgreSql.to_string(), "postgresql");
        assert_eq!(Database::None.to_string(), "none");
    }
}
