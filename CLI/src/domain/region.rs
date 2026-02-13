use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    System,
    Business,
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Library,
    Service,
}

impl Region {
    pub fn checkout_targets(&self, project_type: Option<&ProjectType>) -> &[&str] {
        match self {
            Region::System => match project_type {
                Some(ProjectType::Library) => &["system-region/library"],
                Some(ProjectType::Service) => &["system-region/service"],
                None => &["system-region"],
            },
            Region::Business => &["system-region", "business-region"],
            Region::Service => &["system-region", "business-region", "service-region"],
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Region::System => write!(f, "システム共通領域"),
            Region::Business => write!(f, "部門固有領域"),
            Region::Service => write!(f, "業務固有領域"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_region_with_library_checks_out_library() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Library)),
            &["system-region/library"]
        );
    }

    #[test]
    fn system_region_with_service_checks_out_service() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Service)),
            &["system-region/service"]
        );
    }

    #[test]
    fn system_region_without_project_type_falls_back() {
        assert_eq!(
            Region::System.checkout_targets(None),
            &["system-region"]
        );
    }

    #[test]
    fn business_region_includes_system_dependency() {
        assert_eq!(
            Region::Business.checkout_targets(None),
            &["system-region", "business-region"]
        );
    }

    #[test]
    fn service_region_includes_all_dependencies() {
        assert_eq!(
            Region::Service.checkout_targets(None),
            &["system-region", "business-region", "service-region"]
        );
    }

    #[test]
    fn business_region_ignores_project_type() {
        assert_eq!(
            Region::Business.checkout_targets(Some(&ProjectType::Library)),
            &["system-region", "business-region"]
        );
    }

    #[test]
    fn service_region_ignores_project_type() {
        assert_eq!(
            Region::Service.checkout_targets(Some(&ProjectType::Service)),
            &["system-region", "business-region", "service-region"]
        );
    }

    #[test]
    fn display_system_region() {
        assert_eq!(Region::System.to_string(), "システム共通領域");
    }

    #[test]
    fn display_business_region() {
        assert_eq!(Region::Business.to_string(), "部門固有領域");
    }

    #[test]
    fn display_service_region() {
        assert_eq!(Region::Service.to_string(), "業務固有領域");
    }
}
