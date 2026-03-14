use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Baggage(HashMap<String, String>);

impl Baggage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    pub fn to_header(&self) -> String {
        let mut pairs: Vec<_> = self.0.iter().map(|(k, v)| format!("{k}={v}")).collect();
        pairs.sort();
        pairs.join(",")
    }

    pub fn from_header(s: &str) -> Self {
        let mut b = Self::new();
        for item in s.split(',') {
            if let Some((k, v)) = item.split_once('=') {
                b.set(k.trim(), v.trim());
            }
        }
        b
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Baggage にキーと値をセットし正しく取得できることを確認する。
    #[test]
    fn test_set_and_get() {
        let mut baggage = Baggage::new();
        baggage.set("user_id", "123");
        assert_eq!(baggage.get("user_id"), Some("123"));
        assert_eq!(baggage.get("missing"), None);
    }

    // 単一エントリを持つ Baggage がヘッダー文字列に正しく変換されることを確認する。
    #[test]
    fn test_to_header() {
        let mut baggage = Baggage::new();
        baggage.set("key1", "val1");
        let header = baggage.to_header();
        assert_eq!(header, "key1=val1");
    }

    // 複数エントリのヘッダー変換でキーがソートされて出力されることを確認する。
    #[test]
    fn test_to_header_multiple_sorted() {
        let mut baggage = Baggage::new();
        baggage.set("b", "2");
        baggage.set("a", "1");
        let header = baggage.to_header();
        assert_eq!(header, "a=1,b=2");
    }

    // ヘッダー文字列を解析して Baggage のエントリが正しく復元されることを確認する。
    #[test]
    fn test_from_header() {
        let baggage = Baggage::from_header("key1=val1,key2=val2");
        assert_eq!(baggage.get("key1"), Some("val1"));
        assert_eq!(baggage.get("key2"), Some("val2"));
        assert_eq!(baggage.len(), 2);
    }

    // 空文字列を解析した場合に Baggage が空になることを確認する。
    #[test]
    fn test_from_header_empty() {
        let baggage = Baggage::from_header("");
        assert!(baggage.is_empty());
    }

    // キーと値の前後にスペースがある場合もトリムして正しく解析されることを確認する。
    #[test]
    fn test_from_header_with_spaces() {
        let baggage = Baggage::from_header("key1 = val1 , key2 = val2");
        assert_eq!(baggage.get("key1"), Some("val1"));
        assert_eq!(baggage.get("key2"), Some("val2"));
    }

    // Baggage をヘッダーに変換し再解析した結果が元の値と一致することを確認する。
    #[test]
    fn test_roundtrip() {
        let mut original = Baggage::new();
        original.set("tenant", "acme");
        let header = original.to_header();
        let parsed = Baggage::from_header(&header);
        assert_eq!(parsed.get("tenant"), Some("acme"));
    }

    // 空の Baggage は is_empty が true で len が 0 であり、エントリ追加後に変化することを確認する。
    #[test]
    fn test_is_empty_and_len() {
        let mut baggage = Baggage::new();
        assert!(baggage.is_empty());
        assert_eq!(baggage.len(), 0);

        baggage.set("key", "val");
        assert!(!baggage.is_empty());
        assert_eq!(baggage.len(), 1);
    }

    // デフォルト構築した Baggage が空であることを確認する。
    #[test]
    fn test_default() {
        let baggage = Baggage::default();
        assert!(baggage.is_empty());
    }
}
