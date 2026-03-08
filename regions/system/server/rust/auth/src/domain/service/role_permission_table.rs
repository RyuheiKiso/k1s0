use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use tokio::sync::RwLock;

#[async_trait]
pub trait RolePermissionSource: Send + Sync {
    async fn fetch_role_permissions(&self) -> anyhow::Result<HashMap<String, Vec<String>>>;
}

#[derive(Clone)]
pub struct RolePermissionTable {
    source: Arc<dyn RolePermissionSource>,
    cache: Cache<String, Arc<Vec<String>>>,
    last_refresh_at: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl RolePermissionTable {
    pub fn new(source: Arc<dyn RolePermissionSource>, ttl_secs: u64, max_capacity: u64) -> Self {
        Self {
            source,
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
            last_refresh_at: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn sync_once(&self) -> anyhow::Result<()> {
        let table = self.source.fetch_role_permissions().await?;
        self.cache.invalidate_all();
        for (role, permissions) in table {
            self.cache.insert(role, Arc::new(permissions)).await;
        }
        *self.last_refresh_at.write().await = Some(Utc::now());
        Ok(())
    }

    pub fn start_background_refresh(self: Arc<Self>, refresh_interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(refresh_interval);
            loop {
                ticker.tick().await;
                if let Err(err) = self.sync_once().await {
                    tracing::warn!(
                        error = %err,
                        "failed to refresh keycloak role-permission table"
                    );
                }
            }
        });
    }

    /// Returns None when the table has not been refreshed yet.
    pub async fn check_permission(
        &self,
        roles: &[String],
        resource: &str,
        action: &str,
    ) -> Option<bool> {
        if self.last_refresh_at.read().await.is_none() {
            return None;
        }

        let resource = resource.to_ascii_lowercase();
        let action = action.to_ascii_lowercase();
        for role in roles {
            if let Some(permissions) = self.cache.get(role).await {
                if permissions_allow(permissions.as_ref(), &resource, &action) {
                    return Some(true);
                }
            }
        }
        Some(false)
    }

    #[allow(dead_code)]
    pub async fn last_refresh_at(&self) -> Option<DateTime<Utc>> {
        *self.last_refresh_at.read().await
    }
}

fn permissions_allow(permissions: &[String], resource: &str, action: &str) -> bool {
    permissions.iter().any(|permission| {
        let normalized = permission.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return false;
        }
        if normalized == "*" || normalized == "*:*" {
            return true;
        }

        let Some((perm_resource, perm_action)) = normalized.split_once(':') else {
            return false;
        };
        let resource_match = perm_resource == "*" || perm_resource == resource;
        let action_match = perm_action == "*" || perm_action == action;
        resource_match && action_match
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubSource {
        table: HashMap<String, Vec<String>>,
    }

    #[async_trait]
    impl RolePermissionSource for StubSource {
        async fn fetch_role_permissions(&self) -> anyhow::Result<HashMap<String, Vec<String>>> {
            Ok(self.table.clone())
        }
    }

    #[tokio::test]
    async fn test_check_permission_with_synced_table() {
        let source = Arc::new(StubSource {
            table: HashMap::from([
                ("sys_admin".to_string(), vec!["*:*".to_string()]),
                ("sys_operator".to_string(), vec!["users:read".to_string()]),
            ]),
        });
        let table = RolePermissionTable::new(source, 300, 100);
        table.sync_once().await.unwrap();

        assert_eq!(
            table
                .check_permission(&["sys_operator".to_string()], "users", "read")
                .await,
            Some(true)
        );
        assert_eq!(
            table
                .check_permission(&["sys_operator".to_string()], "users", "delete")
                .await,
            Some(false)
        );
        assert_eq!(
            table
                .check_permission(&["sys_admin".to_string()], "users", "delete")
                .await,
            Some(true)
        );
    }
}
