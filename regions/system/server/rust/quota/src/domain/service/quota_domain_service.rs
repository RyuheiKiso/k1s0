use crate::domain::entity::quota::{Period, SubjectType};

pub struct QuotaDomainService;

impl QuotaDomainService {
    pub fn parse_subject_type(raw: &str) -> Result<SubjectType, String> {
        SubjectType::from_str(raw).ok_or_else(|| {
            format!(
                "subject_type must be one of: tenant, user, api_key, got: {raw}"
            )
        })
    }

    pub fn parse_period(raw: &str) -> Result<Period, String> {
        Period::from_str(raw)
            .ok_or_else(|| format!("period must be one of: daily, monthly, got: {raw}"))
    }

    pub fn validate_limit(limit: u64) -> Result<(), String> {
        if limit == 0 {
            return Err("limit must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn validate_alert_threshold(alert_threshold_percent: Option<u8>) -> Result<(), String> {
        if let Some(threshold) = alert_threshold_percent {
            if threshold > 100 {
                return Err("alert_threshold_percent must be between 0 and 100".to_string());
            }
        }
        Ok(())
    }

    #[must_use] 
    pub fn usage(limit: u64, used: u64) -> (u64, f64) {
        let remaining = limit.saturating_sub(used);
        let usage_percent = if limit == 0 {
            100.0
        } else {
            (used as f64 / limit as f64) * 100.0
        };
        (remaining, usage_percent)
    }
}
