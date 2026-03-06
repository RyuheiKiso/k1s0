use std::path::{Path, PathBuf};

use crate::config::Tier;

/// Build the output path for a generated server.
///
/// Pattern: `{base}/regions/{tier}/server/rust/{name}/`
pub fn build_output_path(base: &Path, tier: Tier, name: &str) -> PathBuf {
    base.join("regions")
        .join(tier.as_str())
        .join("server")
        .join("rust")
        .join(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn system_tier() {
        let p = build_output_path(Path::new("/repo"), Tier::System, "auth");
        assert_eq!(p, PathBuf::from("/repo/regions/system/server/rust/auth"));
    }

    #[test]
    fn business_tier() {
        let p = build_output_path(Path::new("/repo"), Tier::Business, "order");
        assert_eq!(
            p,
            PathBuf::from("/repo/regions/business/server/rust/order")
        );
    }

    #[test]
    fn service_tier() {
        let p = build_output_path(Path::new("/repo"), Tier::Service, "web-app");
        assert_eq!(
            p,
            PathBuf::from("/repo/regions/service/server/rust/web-app")
        );
    }
}
