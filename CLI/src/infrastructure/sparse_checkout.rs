use std::path::Path;
use std::process::Command;

use crate::application::port::RegionCheckout;
use crate::domain::region::{
    BusinessRegionName, ClientFramework, Language, ProjectType, Region, ServiceType,
};
use crate::domain::workspace::WorkspacePath;

pub struct GitSparseCheckout;

impl RegionCheckout for GitSparseCheckout {
    fn setup(
        &self,
        workspace: &WorkspacePath,
        region: &Region,
        project_type: Option<&ProjectType>,
        language: Option<&Language>,
        business_region_name: Option<&BusinessRegionName>,
        service_type: Option<&ServiceType>,
        client_framework: Option<&ClientFramework>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut targets = region.checkout_targets(
            project_type,
            language,
            business_region_name,
            service_type,
            client_framework,
        );
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
            let mut dir = Path::new(ws_path.as_str())
                .join("business-region")
                .join(name.as_str());
            if let Some(lang) = language {
                dir = dir.join(match lang {
                    Language::Rust => "rust",
                    Language::Go => "go",
                });
            }
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }

        Ok(())
    }
}
