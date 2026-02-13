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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Go,
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
        language: Option<&Language>,
        business_region_name: Option<&BusinessRegionName>,
    ) -> Vec<String> {
        match self {
            Region::System => match project_type {
                Some(ProjectType::Library) => {
                    let sub = match language {
                        Some(Language::Rust) => "system-region/library/rust",
                        Some(Language::Go) => "system-region/library/go",
                        None => "system-region/library",
                    };
                    vec![sub.to_string()]
                }
                Some(ProjectType::Service) => {
                    let sub = match language {
                        Some(Language::Rust) => "system-region/service/rust",
                        Some(Language::Go) => "system-region/service/go",
                        None => "system-region/service",
                    };
                    vec![sub.to_string()]
                }
                None => vec!["system-region".to_string()],
            },
            Region::Business => {
                let br = match business_region_name {
                    Some(name) => match language {
                        Some(Language::Rust) => {
                            format!("business-region/{}/rust", name.as_str())
                        }
                        Some(Language::Go) => {
                            format!("business-region/{}/go", name.as_str())
                        }
                        None => format!("business-region/{}", name.as_str()),
                    },
                    None => "business-region".to_string(),
                };
                vec!["system-region".to_string(), br]
            }
            Region::Service => {
                let br = match business_region_name {
                    Some(name) => format!("business-region/{}", name.as_str()),
                    None => "business-region".to_string(),
                };
                vec![
                    "system-region".to_string(),
                    br,
                    "service-region".to_string(),
                ]
            }
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
    fn system_region_library_rust_checks_out_library_rust() {
        assert_eq!(
            Region::System.checkout_targets(
                Some(&ProjectType::Library),
                Some(&Language::Rust),
                None
            ),
            vec!["system-region/library/rust"]
        );
    }

    #[test]
    fn system_region_library_go_checks_out_library_go() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Library), Some(&Language::Go), None),
            vec!["system-region/library/go"]
        );
    }

    #[test]
    fn system_region_library_without_language_falls_back() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Library), None, None),
            vec!["system-region/library"]
        );
    }

    #[test]
    fn system_region_service_rust_checks_out_service_rust() {
        assert_eq!(
            Region::System.checkout_targets(
                Some(&ProjectType::Service),
                Some(&Language::Rust),
                None
            ),
            vec!["system-region/service/rust"]
        );
    }

    #[test]
    fn system_region_service_go_checks_out_service_go() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Service), Some(&Language::Go), None),
            vec!["system-region/service/go"]
        );
    }

    #[test]
    fn system_region_service_without_language_falls_back() {
        assert_eq!(
            Region::System.checkout_targets(Some(&ProjectType::Service), None, None),
            vec!["system-region/service"]
        );
    }

    #[test]
    fn system_region_without_project_type_falls_back() {
        assert_eq!(
            Region::System.checkout_targets(None, None, None),
            vec!["system-region"]
        );
    }

    #[test]
    fn business_region_without_name_uses_default() {
        assert_eq!(
            Region::Business.checkout_targets(None, None, None),
            vec!["system-region", "business-region"]
        );
    }

    #[test]
    fn business_region_with_name_uses_subdirectory() {
        let name = BusinessRegionName::new("sales").unwrap();
        assert_eq!(
            Region::Business.checkout_targets(None, None, Some(&name)),
            vec!["system-region", "business-region/sales"]
        );
    }

    #[test]
    fn business_region_with_name_and_rust() {
        let name = BusinessRegionName::new("sales").unwrap();
        assert_eq!(
            Region::Business.checkout_targets(None, Some(&Language::Rust), Some(&name)),
            vec!["system-region", "business-region/sales/rust"]
        );
    }

    #[test]
    fn business_region_with_name_and_go() {
        let name = BusinessRegionName::new("sales").unwrap();
        assert_eq!(
            Region::Business.checkout_targets(None, Some(&Language::Go), Some(&name)),
            vec!["system-region", "business-region/sales/go"]
        );
    }

    #[test]
    fn service_region_includes_all_dependencies() {
        assert_eq!(
            Region::Service.checkout_targets(None, None, None),
            vec!["system-region", "business-region", "service-region"]
        );
    }

    #[test]
    fn business_region_ignores_project_type() {
        assert_eq!(
            Region::Business.checkout_targets(Some(&ProjectType::Library), None, None),
            vec!["system-region", "business-region"]
        );
    }

    #[test]
    fn service_region_ignores_project_type() {
        assert_eq!(
            Region::Service.checkout_targets(Some(&ProjectType::Service), None, None),
            vec!["system-region", "business-region", "service-region"]
        );
    }

    #[test]
    fn service_region_with_business_name_uses_subdirectory() {
        let name = BusinessRegionName::new("sales").unwrap();
        assert_eq!(
            Region::Service.checkout_targets(None, None, Some(&name)),
            vec!["system-region", "business-region/sales", "service-region"]
        );
    }

    #[test]
    fn service_region_with_business_name_ignores_language() {
        let name = BusinessRegionName::new("hr").unwrap();
        assert_eq!(
            Region::Service.checkout_targets(None, Some(&Language::Rust), Some(&name)),
            vec!["system-region", "business-region/hr", "service-region"]
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
