pub mod react;
pub mod flutter;
pub mod rust_axum;
pub mod go_gin;

use std::path::PathBuf;

use crate::domain::model::{Database, ProjectConfig, Template};

pub fn generate_template_files(config: &ProjectConfig) -> Vec<(PathBuf, String)> {
    let mut files = match &config.template {
        Template::React => react::generate(&config.name),
        Template::Flutter => flutter::generate(&config.name),
        Template::RustAxum => rust_axum::generate(&config.name),
        Template::GoGin => go_gin::generate(&config.name),
    };

    if config.database == Database::PostgreSql {
        files.extend(postgresql_files());
    }

    files
}

fn postgresql_files() -> Vec<(PathBuf, String)> {
    vec![
        (PathBuf::from("docker-compose.yml"), docker_compose()),
        (PathBuf::from("migrations/001_init.sql"), migration_init()),
    ]
}

fn docker_compose() -> String {
    r#"services:
  db:
    image: postgres:16
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: password
      POSTGRES_DB: app_db
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
"#
    .to_string()
}

fn migration_init() -> String {
    r#"-- Initial migration
-- Add your tables here

CREATE TABLE IF NOT EXISTS example (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{ProjectConfig, ProjectType};

    #[test]
    fn test_react_template_files() {
        let config = ProjectConfig {
            name: "test-app".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::React,
            database: Database::None,
            path: PathBuf::from("/tmp/test-app"),
        };

        let files = generate_template_files(&config);
        let paths: Vec<PathBuf> = files.iter().map(|(p, _)| p.clone()).collect();
        assert!(paths.contains(&PathBuf::from("package.json")));
        assert!(paths.contains(&PathBuf::from("src/App.tsx")));
        assert!(!paths.contains(&PathBuf::from("docker-compose.yml")));
    }

    #[test]
    fn test_backend_with_postgresql_adds_db_files() {
        let config = ProjectConfig {
            name: "test-svc".to_string(),
            project_type: ProjectType::Backend,
            template: Template::RustAxum,
            database: Database::PostgreSql,
            path: PathBuf::from("/tmp/test-svc"),
        };

        let files = generate_template_files(&config);
        let paths: Vec<PathBuf> = files.iter().map(|(p, _)| p.clone()).collect();
        assert!(paths.contains(&PathBuf::from("Cargo.toml")));
        assert!(paths.contains(&PathBuf::from("docker-compose.yml")));
        assert!(paths.contains(&PathBuf::from("migrations/001_init.sql")));
    }

    #[test]
    fn test_backend_without_db_no_db_files() {
        let config = ProjectConfig {
            name: "test-svc".to_string(),
            project_type: ProjectType::Backend,
            template: Template::GoGin,
            database: Database::None,
            path: PathBuf::from("/tmp/test-svc"),
        };

        let files = generate_template_files(&config);
        let paths: Vec<PathBuf> = files.iter().map(|(p, _)| p.clone()).collect();
        assert!(paths.contains(&PathBuf::from("go.mod")));
        assert!(!paths.contains(&PathBuf::from("docker-compose.yml")));
    }

    #[test]
    fn test_docker_compose_content() {
        let content = docker_compose();
        assert!(content.contains("postgres:16"));
        assert!(content.contains("5432:5432"));
    }

    #[test]
    fn test_migration_content() {
        let content = migration_init();
        assert!(content.contains("CREATE TABLE"));
    }
}
