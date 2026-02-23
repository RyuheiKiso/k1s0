use uuid::Uuid;

/// テスト用フィクスチャビルダー。
pub struct FixtureBuilder;

impl FixtureBuilder {
    /// ランダム UUID を生成する。
    pub fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// ランダムなテスト用メールアドレスを生成する。
    pub fn email() -> String {
        format!("test-{}@example.com", &Uuid::new_v4().to_string()[..8])
    }

    /// ランダムなテスト用ユーザー名を生成する。
    pub fn name() -> String {
        format!("user-{}", &Uuid::new_v4().to_string()[..8])
    }

    /// 指定範囲のランダム整数を生成する。
    pub fn int(min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u64;
        let random_bytes = Uuid::new_v4();
        let bytes = random_bytes.as_bytes();
        let val = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        min + (val % range) as i64
    }

    /// テスト用テナント ID を生成する。
    pub fn tenant_id() -> String {
        format!("tenant-{}", &Uuid::new_v4().to_string()[..8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_format() {
        let id = FixtureBuilder::uuid();
        assert_eq!(id.len(), 36);
        assert!(id.contains('-'));
    }

    #[test]
    fn test_email_format() {
        let email = FixtureBuilder::email();
        assert!(email.contains('@'));
        assert!(email.ends_with("@example.com"));
    }

    #[test]
    fn test_name_prefix() {
        let name = FixtureBuilder::name();
        assert!(name.starts_with("user-"));
    }

    #[test]
    fn test_int_range() {
        for _ in 0..100 {
            let val = FixtureBuilder::int(10, 20);
            assert!(val >= 10 && val < 20);
        }
    }

    #[test]
    fn test_int_same_min_max() {
        assert_eq!(FixtureBuilder::int(5, 5), 5);
    }

    #[test]
    fn test_tenant_id_format() {
        let tid = FixtureBuilder::tenant_id();
        assert!(tid.starts_with("tenant-"));
    }

    #[test]
    fn test_uniqueness() {
        let a = FixtureBuilder::uuid();
        let b = FixtureBuilder::uuid();
        assert_ne!(a, b);
    }
}
