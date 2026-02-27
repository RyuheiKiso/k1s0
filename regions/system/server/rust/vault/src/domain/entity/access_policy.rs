/// SPIFFE ID ベースのアクセスポリシー。
/// シークレットパスに対してどの SPIFFE ID がアクセス可能かを定義する。
#[derive(Debug, Clone)]
pub struct SpiffeAccessPolicy {
    pub id: uuid::Uuid,
    pub secret_path_pattern: String,
    pub allowed_spiffe_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SpiffeAccessPolicy {
    /// パスがこのポリシーのパターンに一致するか判定する。
    /// `*` は単一セグメント、`**` は複数セグメントにマッチ。
    pub fn matches_path(&self, path: &str) -> bool {
        Self::glob_match(&self.secret_path_pattern, path)
    }

    /// 指定された SPIFFE ID がこのポリシーで許可されているか判定する。
    pub fn is_allowed(&self, spiffe_id: &str) -> bool {
        self.allowed_spiffe_ids.iter().any(|id| id == spiffe_id)
    }

    fn glob_match(pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        Self::match_parts(&pattern_parts, &path_parts)
    }

    fn match_parts(pattern: &[&str], path: &[&str]) -> bool {
        if pattern.is_empty() && path.is_empty() {
            return true;
        }
        if pattern.is_empty() {
            return false;
        }
        if pattern[0] == "**" {
            for i in 0..=path.len() {
                if Self::match_parts(&pattern[1..], &path[i..]) {
                    return true;
                }
            }
            return false;
        }
        if path.is_empty() {
            return false;
        }
        if pattern[0] == "*" || pattern[0] == path[0] {
            return Self::match_parts(&pattern[1..], &path[1..]);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_policy(pattern: &str, spiffe_ids: &[&str]) -> SpiffeAccessPolicy {
        SpiffeAccessPolicy {
            id: uuid::Uuid::new_v4(),
            secret_path_pattern: pattern.to_string(),
            allowed_spiffe_ids: spiffe_ids.iter().map(|s| s.to_string()).collect(),
            created_at: Utc::now(),
        }
    }

    // --- matches_path tests ---

    #[test]
    fn test_exact_match() {
        let policy = make_policy("secrets/production/db-password", &[]);
        assert!(policy.matches_path("secrets/production/db-password"));
        assert!(!policy.matches_path("secrets/production/api-key"));
    }

    #[test]
    fn test_single_wildcard() {
        let policy = make_policy("secrets/production/*", &[]);
        assert!(policy.matches_path("secrets/production/db-password"));
        assert!(policy.matches_path("secrets/production/api-key"));
        assert!(!policy.matches_path("secrets/staging/db-password"));
        assert!(!policy.matches_path("secrets/production/nested/key"));
    }

    #[test]
    fn test_double_wildcard() {
        let policy = make_policy("secrets/**", &[]);
        assert!(policy.matches_path("secrets/production/db-password"));
        assert!(policy.matches_path("secrets/staging/api-key"));
        assert!(policy.matches_path("secrets/a/b/c/d"));
        assert!(!policy.matches_path("other/production/key"));
    }

    #[test]
    fn test_double_wildcard_at_start() {
        let policy = make_policy("**/db-password", &[]);
        assert!(policy.matches_path("secrets/production/db-password"));
        assert!(policy.matches_path("db-password"));
        assert!(!policy.matches_path("secrets/production/api-key"));
    }

    #[test]
    fn test_double_wildcard_in_middle() {
        let policy = make_policy("secrets/**/password", &[]);
        assert!(policy.matches_path("secrets/password"));
        assert!(policy.matches_path("secrets/production/password"));
        assert!(policy.matches_path("secrets/a/b/c/password"));
        assert!(!policy.matches_path("secrets/production/api-key"));
    }

    #[test]
    fn test_wildcard_no_match_empty() {
        let policy = make_policy("secrets/*", &[]);
        assert!(!policy.matches_path("secrets"));
    }

    #[test]
    fn test_empty_pattern_matches_empty_path() {
        let policy = make_policy("", &[]);
        assert!(policy.matches_path(""));
    }

    // --- is_allowed tests ---

    #[test]
    fn test_allowed_spiffe_id() {
        let policy = make_policy(
            "secrets/production/*",
            &[
                "spiffe://cluster/ns/default/sa/payment-service",
                "spiffe://cluster/ns/default/sa/order-service",
            ],
        );
        assert!(policy.is_allowed("spiffe://cluster/ns/default/sa/payment-service"));
        assert!(policy.is_allowed("spiffe://cluster/ns/default/sa/order-service"));
        assert!(!policy.is_allowed("spiffe://cluster/ns/default/sa/unknown-service"));
    }

    #[test]
    fn test_no_allowed_spiffe_ids() {
        let policy = make_policy("secrets/*", &[]);
        assert!(!policy.is_allowed("spiffe://cluster/ns/default/sa/any-service"));
    }
}
