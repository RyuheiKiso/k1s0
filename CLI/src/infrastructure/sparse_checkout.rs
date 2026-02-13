use std::path::Path;
use std::process::Command;

use crate::application::port::RegionCheckout;
use crate::domain::region::{BusinessRegionName, ProjectType, Region};
use crate::domain::workspace::WorkspacePath;

pub struct GitSparseCheckout;

impl RegionCheckout for GitSparseCheckout {
    fn setup(
        &self,
        workspace: &WorkspacePath,
        region: &Region,
        project_type: Option<&ProjectType>,
        business_region_name: Option<&BusinessRegionName>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut targets = region.checkout_targets(project_type, business_region_name);
        for keep in ["CLI", "docs"] {
            if !targets.iter().any(|t| t == keep) {
                targets.push(keep.to_string());
            }
        }
        let ws_path = workspace.to_string_lossy();
        let output = Command::new("git")
            .arg("sparse-checkout")
            .arg("set")
            .args(&targets)
            .current_dir(ws_path.as_str())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git sparse-checkout set failed: {stderr}").into());
        }

        if let (Region::Business, Some(name)) = (region, business_region_name) {
            let dir = Path::new(ws_path.as_str())
                .join("business-region")
                .join(name.as_str());
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }

        Ok(())
    }
}
