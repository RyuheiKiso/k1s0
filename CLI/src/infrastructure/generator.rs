use std::fs;

use crate::application::interactive::FileGenerator;
use crate::domain::model::ProjectConfig;
use crate::infrastructure::templates::generate_template_files;

pub struct FsGenerator;

impl FsGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl FileGenerator for FsGenerator {
    fn generate(&self, config: &ProjectConfig) -> Result<(), String> {
        let files = generate_template_files(config);

        for (relative_path, content) in &files {
            let full_path = config.path.join(relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
            }
            fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write file {}: {}", full_path.display(), e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{Database, ProjectType, Template};
    use tempfile::TempDir;

    #[test]
    fn test_fs_generator_creates_react_files() {
        let tmp = TempDir::new().unwrap();
        let project_path = tmp.path().join("my-app");

        let config = ProjectConfig {
            name: "my-app".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::React,
            database: Database::None,
            path: project_path.clone(),
        };

        let generator = FsGenerator::new();
        generator.generate(&config).unwrap();

        assert!(project_path.join("package.json").exists());
        assert!(project_path.join("src/App.tsx").exists());
        assert!(project_path.join("src/index.tsx").exists());
        assert!(project_path.join("src/App.test.tsx").exists());
        assert!(project_path.join("README.md").exists());
        assert!(project_path.join(".github/workflows/ci.yml").exists());
    }

    #[test]
    fn test_fs_generator_creates_rust_with_db_files() {
        let tmp = TempDir::new().unwrap();
        let project_path = tmp.path().join("my-svc");

        let config = ProjectConfig {
            name: "my-svc".to_string(),
            project_type: ProjectType::Backend,
            template: Template::RustAxum,
            database: Database::PostgreSql,
            path: project_path.clone(),
        };

        let generator = FsGenerator::new();
        generator.generate(&config).unwrap();

        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/main.rs").exists());
        assert!(project_path.join("src/lib.rs").exists());
        assert!(project_path.join("tests/health_check.rs").exists());
        assert!(project_path.join("docker-compose.yml").exists());
        assert!(project_path.join("migrations/001_init.sql").exists());
    }

    #[test]
    fn test_fs_generator_creates_flutter_files() {
        let tmp = TempDir::new().unwrap();
        let project_path = tmp.path().join("my-flutter");

        let config = ProjectConfig {
            name: "my-flutter".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::Flutter,
            database: Database::None,
            path: project_path.clone(),
        };

        let generator = FsGenerator::new();
        generator.generate(&config).unwrap();

        assert!(project_path.join("pubspec.yaml").exists());
        assert!(project_path.join("lib/main.dart").exists());
        assert!(project_path.join("test/widget_test.dart").exists());
    }

    #[test]
    fn test_fs_generator_creates_go_files() {
        let tmp = TempDir::new().unwrap();
        let project_path = tmp.path().join("my-go");

        let config = ProjectConfig {
            name: "my-go".to_string(),
            project_type: ProjectType::Backend,
            template: Template::GoGin,
            database: Database::None,
            path: project_path.clone(),
        };

        let generator = FsGenerator::new();
        generator.generate(&config).unwrap();

        assert!(project_path.join("go.mod").exists());
        assert!(project_path.join("main.go").exists());
        assert!(project_path.join("main_test.go").exists());
    }

    #[test]
    fn test_generated_file_content() {
        let tmp = TempDir::new().unwrap();
        let project_path = tmp.path().join("content-test");

        let config = ProjectConfig {
            name: "content-test".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::React,
            database: Database::None,
            path: project_path.clone(),
        };

        let generator = FsGenerator::new();
        generator.generate(&config).unwrap();

        let pkg = fs::read_to_string(project_path.join("package.json")).unwrap();
        assert!(pkg.contains("\"name\": \"content-test\""));
    }
}
