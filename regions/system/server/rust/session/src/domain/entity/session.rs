use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub metadata: HashMap<String, String>,
}

impl Session {
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    pub fn refresh(&mut self, new_expires_at: DateTime<Utc>) {
        self.expires_at = new_expires_at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_session(expires_at: DateTime<Utc>, revoked: bool) -> Session {
        Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            token: "tok-1".to_string(),
            expires_at,
            created_at: Utc::now(),
            revoked,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_is_valid_active_session() {
        let s = make_session(Utc::now() + Duration::hours(1), false);
        assert!(s.is_valid());
    }

    #[test]
    fn test_is_valid_revoked() {
        let s = make_session(Utc::now() + Duration::hours(1), true);
        assert!(!s.is_valid());
    }

    #[test]
    fn test_is_valid_expired() {
        let s = make_session(Utc::now() - Duration::hours(1), false);
        assert!(!s.is_valid());
    }

    #[test]
    fn test_revoke() {
        let mut s = make_session(Utc::now() + Duration::hours(1), false);
        assert!(s.is_valid());
        s.revoke();
        assert!(!s.is_valid());
        assert!(s.revoked);
    }

    #[test]
    fn test_refresh() {
        let mut s = make_session(Utc::now() - Duration::hours(1), false);
        assert!(s.is_expired());
        let new_exp = Utc::now() + Duration::hours(2);
        s.refresh(new_exp);
        assert_eq!(s.expires_at, new_exp);
        assert!(!s.is_expired());
    }
}
