pub mod react;
pub mod flutter;
pub mod rust_axum;
pub mod go_gin;
pub mod kubernetes;

use std::path::PathBuf;

use crate::domain::model::{Database, ProjectConfig, Template};

pub fn generate_template_files(config: &ProjectConfig) -> Vec<(PathBuf, String)> {
    let mut files = match &config.template {
        Template::React => react::generate(&config.name),
        Template::Flutter => flutter::generate(&config.name),
        Template::RustAxum => rust_axum::generate(&config.name),
        Template::GoGin => go_gin::generate(&config.name),
    };

    files.push((PathBuf::from("docker-compose.yml"), docker_compose(config)));
    files.extend(kubernetes::generate(config));

    if config.database == Database::PostgreSql {
        files.push((PathBuf::from("migrations/001_init.sql"), migration_init()));
    }

    files
}

fn docker_compose(config: &ProjectConfig) -> String {
    let port = match &config.template {
        Template::React | Template::Flutter => "80",
        Template::RustAxum => "3000",
        Template::GoGin => "8080",
    };

    if config.database == Database::PostgreSql {
        format!(
            r#"services:
  app:
    build: .
    ports:
      - "{port}:{port}"
    environment:
      DATABASE_URL: postgres://app:password@db:5432/app_db
    depends_on:
      - db
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
        )
    } else {
        format!(
            r#"services:
  app:
    build: .
    ports:
      - "{port}:{port}"
"#
        )
    }
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
        assert!(paths.contains(&PathBuf::from("Dockerfile")));
        assert!(paths.contains(&PathBuf::from(".dockerignore")));
        assert!(paths.contains(&PathBuf::from("docker-compose.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/namespace.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/deployment.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/service.yml")));
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
        assert!(paths.contains(&PathBuf::from("Dockerfile")));
        assert!(paths.contains(&PathBuf::from(".dockerignore")));
        assert!(paths.contains(&PathBuf::from("docker-compose.yml")));
        assert!(paths.contains(&PathBuf::from("migrations/001_init.sql")));
        assert!(paths.contains(&PathBuf::from("k8s/postgres-secret.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/postgres-statefulset.yml")));
    }

    #[test]
    fn test_backend_without_db_has_docker_compose() {
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
        assert!(paths.contains(&PathBuf::from("Dockerfile")));
        assert!(paths.contains(&PathBuf::from(".dockerignore")));
        assert!(paths.contains(&PathBuf::from("docker-compose.yml")));
        assert!(!paths.contains(&PathBuf::from("migrations/001_init.sql")));
        assert!(paths.contains(&PathBuf::from("k8s/namespace.yml")));
        assert!(!paths.contains(&PathBuf::from("k8s/postgres-secret.yml")));
    }

    #[test]
    fn test_docker_compose_with_db() {
        let config = ProjectConfig {
            name: "test-svc".to_string(),
            project_type: ProjectType::Backend,
            template: Template::RustAxum,
            database: Database::PostgreSql,
            path: PathBuf::from("/tmp/test-svc"),
        };
        let content = docker_compose(&config);
        assert!(content.contains("postgres:16"));
        assert!(content.contains("5432:5432"));
        assert!(content.contains("build: ."));
        assert!(content.contains("3000:3000"));
        assert!(content.contains("depends_on"));
        assert!(content.contains("DATABASE_URL"));
    }

    #[test]
    fn test_docker_compose_without_db() {
        let config = ProjectConfig {
            name: "test-svc".to_string(),
            project_type: ProjectType::Backend,
            template: Template::GoGin,
            database: Database::None,
            path: PathBuf::from("/tmp/test-svc"),
        };
        let content = docker_compose(&config);
        assert!(content.contains("build: ."));
        assert!(content.contains("8080:8080"));
        assert!(!content.contains("postgres"));
    }

    #[test]
    fn test_docker_compose_frontend_port() {
        let config = ProjectConfig {
            name: "test-app".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::React,
            database: Database::None,
            path: PathBuf::from("/tmp/test-app"),
        };
        let content = docker_compose(&config);
        assert!(content.contains("80:80"));
    }

    #[test]
    fn test_migration_content() {
        let content = migration_init();
        assert!(content.contains("CREATE TABLE"));
    }
}
