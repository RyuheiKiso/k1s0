use std::process::Command;

use crate::application::port::RegionCheckout;
use crate::domain::region::{ProjectType, Region};
use crate::domain::workspace::WorkspacePath;

pub struct GitSparseCheckout;

impl RegionCheckout for GitSparseCheckout {
    fn setup(
        &self,
        workspace: &WorkspacePath,
        region: &Region,
        project_type: Option<&ProjectType>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let targets = region.checkout_targets(project_type);
        let output = Command::new("git")
            .arg("sparse-checkout")
            .arg("set")
            .args(targets)
            .current_dir(workspace.to_string_lossy().as_str())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git sparse-checkout set failed: {stderr}").into());
        }
        Ok(())
    }
}
