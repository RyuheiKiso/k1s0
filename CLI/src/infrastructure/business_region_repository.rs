use std::process::Command;

use crate::application::port::BusinessRegionRepository;
use crate::domain::workspace::WorkspacePath;

pub struct GitBusinessRegionRepository;

impl BusinessRegionRepository for GitBusinessRegionRepository {
    fn list(&self, workspace: &WorkspacePath) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["ls-tree", "--name-only", "HEAD", "business-region/"])
            .current_dir(workspace.to_string_lossy().as_str())
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let regions: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                line.strip_prefix("business-region/")
                    .map(|name| name.to_string())
            })
            .filter(|name| !name.is_empty())
            .collect();

        Ok(regions)
    }
}
