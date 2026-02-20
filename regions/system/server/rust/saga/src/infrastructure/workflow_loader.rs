use std::path::{Path, PathBuf};

use crate::domain::entity::workflow::WorkflowDefinition;

/// WorkflowLoader はディスク上のYAMLファイルからワークフロー定義を読み込む。
pub struct WorkflowLoader {
    workflow_dir: PathBuf,
}

impl WorkflowLoader {
    pub fn new(workflow_dir: impl Into<PathBuf>) -> Self {
        Self {
            workflow_dir: workflow_dir.into(),
        }
    }

    /// ディレクトリ内の全 .yaml/.yml ファイルを読み込み、WorkflowDefinition リストを返す。
    /// ディレクトリが存在しない場合は空のリストを返す（エラーにしない）。
    pub async fn load_all(&self) -> anyhow::Result<Vec<WorkflowDefinition>> {
        if !self.workflow_dir.exists() {
            tracing::warn!(
                dir = %self.workflow_dir.display(),
                "workflow directory does not exist, returning empty list"
            );
            return Ok(Vec::new());
        }

        let mut entries = tokio::fs::read_dir(&self.workflow_dir).await?;
        let mut workflows = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext != "yaml" && ext != "yml" {
                continue;
            }

            match self.load_file(&path).await {
                Ok(wf) => {
                    tracing::info!(
                        file = %path.display(),
                        name = %wf.name,
                        "loaded workflow definition"
                    );
                    workflows.push(wf);
                }
                Err(e) => {
                    tracing::warn!(
                        file = %path.display(),
                        error = %e,
                        "failed to load workflow definition, skipping"
                    );
                }
            }
        }

        Ok(workflows)
    }

    /// 指定ファイルを読み込み、WorkflowDefinition を返す。
    pub async fn load_file(&self, path: &Path) -> anyhow::Result<WorkflowDefinition> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            anyhow::anyhow!("failed to read file {}: {}", path.display(), e)
        })?;
        let wf = WorkflowDefinition::from_yaml(&content).map_err(|e| {
            anyhow::anyhow!("failed to parse workflow from {}: {}", path.display(), e)
        })?;
        Ok(wf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_temp_dir() -> TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    const VALID_YAML: &str = r#"
name: order-fulfillment
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
    compensate: InventoryService.Release
    timeout_secs: 30
  - name: process-payment
    service: payment-service
    method: PaymentService.Charge
    timeout_secs: 60
"#;

    const ANOTHER_VALID_YAML: &str = r#"
name: refund-workflow
steps:
  - name: reverse-payment
    service: payment-service
    method: PaymentService.Refund
    timeout_secs: 30
"#;

    const INVALID_YAML: &str = r#"
name: ""
steps: []
"#;

    // ---- load_file のテスト ----

    #[tokio::test]
    async fn test_load_file_success() {
        let dir = make_temp_dir();
        let file_path = dir.path().join("order.yaml");
        fs::write(&file_path, VALID_YAML).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let wf = loader.load_file(&file_path).await.unwrap();

        assert_eq!(wf.name, "order-fulfillment");
        assert_eq!(wf.steps.len(), 2);
        assert_eq!(wf.steps[0].name, "reserve-inventory");
        assert_eq!(wf.steps[1].name, "process-payment");
    }

    #[tokio::test]
    async fn test_load_file_not_found() {
        let dir = make_temp_dir();
        let loader = WorkflowLoader::new(dir.path());
        let result = loader.load_file(&dir.path().join("nonexistent.yaml")).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("failed to read file"), "unexpected error: {}", msg);
    }

    #[tokio::test]
    async fn test_load_file_invalid_yaml() {
        let dir = make_temp_dir();
        let file_path = dir.path().join("bad.yaml");
        fs::write(&file_path, INVALID_YAML).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let result = loader.load_file(&file_path).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("failed to parse workflow"),
            "unexpected error: {}",
            msg
        );
    }

    // ---- load_all のテスト ----

    #[tokio::test]
    async fn test_load_all_empty_dir() {
        let dir = make_temp_dir();
        let loader = WorkflowLoader::new(dir.path());
        let workflows = loader.load_all().await.unwrap();
        assert!(workflows.is_empty());
    }

    #[tokio::test]
    async fn test_load_all_nonexistent_dir() {
        // 存在しないディレクトリはエラーではなく空のリストを返す
        let loader = WorkflowLoader::new("/tmp/k1s0-saga-test-nonexistent-dir-xyz");
        let workflows = loader.load_all().await.unwrap();
        assert!(workflows.is_empty());
    }

    #[tokio::test]
    async fn test_load_all_single_yaml() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("order.yaml"), VALID_YAML).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let workflows = loader.load_all().await.unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "order-fulfillment");
    }

    #[tokio::test]
    async fn test_load_all_multiple_yaml_files() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("order.yaml"), VALID_YAML).unwrap();
        fs::write(dir.path().join("refund.yml"), ANOTHER_VALID_YAML).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let mut workflows = loader.load_all().await.unwrap();
        workflows.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(workflows.len(), 2);
        let names: Vec<&str> = workflows.iter().map(|w| w.name.as_str()).collect();
        assert!(names.contains(&"order-fulfillment"));
        assert!(names.contains(&"refund-workflow"));
    }

    #[tokio::test]
    async fn test_load_all_ignores_non_yaml_files() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("order.yaml"), VALID_YAML).unwrap();
        fs::write(dir.path().join("readme.txt"), "some text").unwrap();
        fs::write(dir.path().join("data.json"), r#"{"key": "value"}"#).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let workflows = loader.load_all().await.unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "order-fulfillment");
    }

    #[tokio::test]
    async fn test_load_all_skips_invalid_yaml_continues_loading() {
        // 無効なYAMLファイルがあっても、有効なものはロードし続ける
        let dir = make_temp_dir();
        fs::write(dir.path().join("valid.yaml"), VALID_YAML).unwrap();
        fs::write(dir.path().join("invalid.yaml"), INVALID_YAML).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let workflows = loader.load_all().await.unwrap();

        // 有効な1件だけロードされる
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "order-fulfillment");
    }

    #[tokio::test]
    async fn test_load_all_yml_extension() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("refund.yml"), ANOTHER_VALID_YAML).unwrap();

        let loader = WorkflowLoader::new(dir.path());
        let workflows = loader.load_all().await.unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "refund-workflow");
    }
}
