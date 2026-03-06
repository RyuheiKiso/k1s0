use crate::domain::entity::TenantStatus;

pub struct TenantDomainService;

impl TenantDomainService {
    pub fn can_activate(status: &TenantStatus) -> bool {
        matches!(status, TenantStatus::Suspended)
    }
}
