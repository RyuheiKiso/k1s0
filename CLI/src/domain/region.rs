use std::fmt;

use super::error::BusinessRegionError;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessRegionName(String);

impl BusinessRegionName {
    pub fn new(name: &str) -> Result<Self, BusinessRegionError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(BusinessRegionError::EmptyName);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Region {
    pub fn checkout_targets(
        &self,
        project_type: Option<&ProjectType>,
        business_region_name: Option<&BusinessRegionName>,
    ) -> Vec<String> {
        match self {
            Region::System => match project_type {
                Some(ProjectType::Library) => vec!["system-region/library".to_string()],
                Some(ProjectType::Service) => vec!["system-region/service".to_string()],
                None => vec!["system-region".to_string()],
            },
            Region::Business => {
                let br = match business_region_name {
                    Some(name) => format!("business-region/{}", name.as_str()),
                    None => "business-region".to_string(),
                };
                vec!["system-region".to_string(), br]
            }
            Region::Service => vec![
                "system-region".to_string(),
                "business-region".to_string(),
                "service-region".to_string(),
            ],
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

    // --- BusinessRegionName tests ---

    #[test]
    fn business_region_name_rejects_empty() {
        assert_eq!(
            BusinessRegionName::new(""),
            Err(BusinessRegionError::EmptyName)
        );
    }

    #[test]
    fn business_region_name_rejects_whitespace_only() {
        assert_eq!(
            BusinessRegionName::new("   "),
            Err(BusinessRegionError::EmptyName)
        );
    }

    #[test]
    fn business_region_name_trims_whitespace() {
        let name = BusinessRegionName::new("  sales  ").unwrap();
        assert_eq!(name.as_str(), "sales");
    }

    #[test]
    fn business_region_name_accepts_valid_name() {
        let name = BusinessRegionName::new("hr-department").unwrap();
        assert_eq!(name.as_str(), "hr-department");
    }

    // --- checkout_targets tests ---

    #[test]
    fn system_region_with_library_checks_out_library() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Library), None),
            vec!["system-region/library"]
        );
    }

    #[test]
    fn system_region_with_service_checks_out_service() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Service), None),
            vec!["system-region/service"]
        );
    }

    #[test]
    fn system_region_without_project_type_falls_back() {
        assert_eq!(
            Region::System.checkout_targets(None, None),
            vec!["system-region"]
        );
    }

    #[test]
    fn business_region_without_name_uses_default() {
        assert_eq!(
            Region::Business.checkout_targets(None, None),
            vec!["system-region", "business-region"]
        );
    }

    #[test]
    fn business_region_with_name_uses_subdirectory() {
        let name = BusinessRegionName::new("sales").unwrap();
        assert_eq!(
            Region::Business.checkout_targets(None, Some(&name)),
            vec!["system-region", "business-region/sales"]
        );
    }

    #[test]
    fn service_region_includes_all_dependencies() {
        assert_eq!(
            Region::Service.checkout_targets(None, None),
            vec!["system-region", "business-region", "service-region"]
        );
    }

    #[test]
    fn business_region_ignores_project_type() {
        assert_eq!(
            Region::Business.checkout_targets(Some(&ProjectType::Library), None),
            vec!["system-region", "business-region"]
        );
    }

    #[test]
    fn service_region_ignores_project_type() {
        assert_eq!(
            Region::Service.checkout_targets(Some(&ProjectType::Service), None),
            vec!["system-region", "business-region", "service-region"]
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
