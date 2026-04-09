use uuid::Uuid;

/// テスト用フィクスチャビルダー。
pub struct FixtureBuilder;

impl FixtureBuilder {
    /// ランダム UUID を生成する。
    #[must_use]
    pub fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// ランダムなテスト用メールアドレスを生成する。
    #[must_use]
    pub fn email() -> String {
        format!("test-{}@example.com", &Uuid::new_v4().to_string()[..8])
    }

    /// ランダムなテスト用ユーザー名を生成する。
    #[must_use]
    pub fn name() -> String {
        format!("user-{}", &Uuid::new_v4().to_string()[..8])
    }

    /// 指定範囲のランダム整数を生成する。
    #[must_use]
    pub fn int(min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        // LOW-008: 安全な型変換（オーバーフロー防止）
        let range = u64::try_from(max - min).unwrap_or(u64::MAX);
        let random_bytes = Uuid::new_v4();
        let bytes = random_bytes.as_bytes();
        let val = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        // LOW-008: 安全な型変換（オーバーフロー防止）
        min + i64::try_from(val % range).unwrap_or(i64::MAX)
    }

    /// テスト用テナント ID を生成する。
    #[must_use]
    pub fn tenant_id() -> String {
        format!("tenant-{}", &Uuid::new_v4().to_string()[..8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 生成された UUID が 36 文字でハイフンを含む形式であることを確認する。
    #[test]
    fn test_uuid_format() {
        let id = FixtureBuilder::uuid();
        assert_eq!(id.len(), 36);
        assert!(id.contains('-'));
    }

    // 生成されたメールアドレスが "@" と "@example.com" を含む形式であることを確認する。
    #[test]
    fn test_email_format() {
        let email = FixtureBuilder::email();
        assert!(email.contains('@'));
        assert!(email.ends_with("@example.com"));
    }

    // 生成されたユーザー名が "user-" で始まることを確認する。
    #[test]
    fn test_name_prefix() {
        let name = FixtureBuilder::name();
        assert!(name.starts_with("user-"));
    }

    // 生成された整数が指定した範囲 [10, 20) に収まることを繰り返し確認する。
    #[test]
    fn test_int_range() {
        for _ in 0..100 {
            let val = FixtureBuilder::int(10, 20);
            assert!((10..20).contains(&val));
        }
    }

    // min と max が等しい場合はその値をそのまま返すことを確認する。
    #[test]
    fn test_int_same_min_max() {
        assert_eq!(FixtureBuilder::int(5, 5), 5);
    }

    // 生成されたテナント ID が "tenant-" で始まることを確認する。
    #[test]
    fn test_tenant_id_format() {
        let tid = FixtureBuilder::tenant_id();
        assert!(tid.starts_with("tenant-"));
    }

    // 連続して生成した UUID が互いに異なることを確認する。
    #[test]
    fn test_uniqueness() {
        let a = FixtureBuilder::uuid();
        let b = FixtureBuilder::uuid();
        assert_ne!(a, b);
    }
}
