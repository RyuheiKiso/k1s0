use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    System,
    Business,
    Service,
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
