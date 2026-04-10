use crate::domain::entity::TenantStatus;

pub struct TenantDomainService;

impl TenantDomainService {
    #[must_use]
    pub fn can_activate(status: &TenantStatus) -> bool {
        matches!(status, TenantStatus::Provisioning | TenantStatus::Suspended)
    }
}
