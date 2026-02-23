/// 名前バリデーション: [a-z0-9-]+, 先頭末尾ハイフン禁止
///
/// # Errors
/// 名前が無効な場合。
///
/// # Panics
/// 正規表現のコンパイルに失敗した場合（正規表現が静的に正しいため実際には発生しない）。
pub fn validate_name(name: &str) -> Result<(), String> {
    let re = regex::Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$").unwrap();
    if !re.is_match(name) {
        return Err("英小文字・ハイフン・数字のみ許可。先頭末尾のハイフンは禁止。".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("order").is_ok());
        assert!(validate_name("order-api").is_ok());
        assert!(validate_name("my-service-123").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("1").is_ok());
        assert!(validate_name("abc").is_ok());
        assert!(validate_name("a1b2c3").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_name("-order").is_err());
        assert!(validate_name("order-").is_err());
        assert!(validate_name("Order").is_err());
        assert!(validate_name("order_api").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name("UPPER").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name("dot.name").is_err());
        assert!(validate_name("-").is_err());
        assert!(validate_name("--").is_err());
    }
}
