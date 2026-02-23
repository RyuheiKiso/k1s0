use std::collections::HashMap;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SecretValue {
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SecretVersion {
    pub version: i64,
    pub value: SecretValue,
    pub created_at: DateTime<Utc>,
    pub destroyed: bool,
}

#[derive(Debug, Clone)]
pub struct Secret {
    pub path: String,
    pub current_version: i64,
    pub versions: Vec<SecretVersion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Secret {
    pub fn new(path: String, data: HashMap<String, String>) -> Self {
        let now = Utc::now();
        let version = SecretVersion {
            version: 1,
            value: SecretValue { data },
            created_at: now,
            destroyed: false,
        };
        Self {
            path,
            current_version: 1,
            versions: vec![version],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn get_version(&self, version: Option<i64>) -> Option<&SecretVersion> {
        let v = version.unwrap_or(self.current_version);
        self.versions
            .iter()
            .find(|sv| sv.version == v && !sv.destroyed)
    }

    pub fn update(mut self, data: HashMap<String, String>) -> Self {
        let new_version = self.current_version + 1;
        let now = Utc::now();
        self.versions.push(SecretVersion {
            version: new_version,
            value: SecretValue { data },
            created_at: now,
            destroyed: false,
        });
        self.current_version = new_version;
        self.updated_at = now;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_secret() {
        let data = HashMap::from([("key".to_string(), "value".to_string())]);
        let secret = Secret::new("app/db/password".to_string(), data.clone());

        assert_eq!(secret.path, "app/db/password");
        assert_eq!(secret.current_version, 1);
        assert_eq!(secret.versions.len(), 1);
        assert_eq!(secret.versions[0].version, 1);
        assert_eq!(secret.versions[0].value.data, data);
        assert!(!secret.versions[0].destroyed);
    }

    #[test]
    fn test_get_version_current() {
        let data = HashMap::from([("key".to_string(), "value".to_string())]);
        let secret = Secret::new("app/db/password".to_string(), data);

        let v = secret.get_version(None);
        assert!(v.is_some());
        assert_eq!(v.unwrap().version, 1);
    }

    #[test]
    fn test_get_version_specific() {
        let data = HashMap::from([("key".to_string(), "v1".to_string())]);
        let secret = Secret::new("path".to_string(), data);
        let data2 = HashMap::from([("key".to_string(), "v2".to_string())]);
        let secret = secret.update(data2);

        let v1 = secret.get_version(Some(1));
        assert!(v1.is_some());
        assert_eq!(v1.unwrap().value.data["key"], "v1");

        let v2 = secret.get_version(Some(2));
        assert!(v2.is_some());
        assert_eq!(v2.unwrap().value.data["key"], "v2");
    }

    #[test]
    fn test_get_version_destroyed() {
        let data = HashMap::from([("key".to_string(), "value".to_string())]);
        let mut secret = Secret::new("path".to_string(), data);
        secret.versions[0].destroyed = true;

        let v = secret.get_version(Some(1));
        assert!(v.is_none());
    }

    #[test]
    fn test_update_increments_version() {
        let data = HashMap::from([("key".to_string(), "v1".to_string())]);
        let secret = Secret::new("path".to_string(), data);

        let data2 = HashMap::from([("key".to_string(), "v2".to_string())]);
        let secret = secret.update(data2);

        assert_eq!(secret.current_version, 2);
        assert_eq!(secret.versions.len(), 2);
        assert_eq!(secret.versions[1].value.data["key"], "v2");
    }
}
