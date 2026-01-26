use super::super::*;

#[test]
fn test_contains_adr_reference() {
    // 有効な ADR 参照
    assert!(contains_adr_reference("ADR-001"));
    assert!(contains_adr_reference("ADR-123"));
    assert!(contains_adr_reference("adr-001"));
    assert!(contains_adr_reference("\"ADR-001\""));
    assert!(contains_adr_reference("// ADR-001: リトライポリシー"));
    assert!(contains_adr_reference("RetryConfig::enabled(\"ADR-001\")"));

    // 無効な ADR 参照
    assert!(!contains_adr_reference("ADR-01"));  // 2桁は不可
    assert!(!contains_adr_reference("ADR-"));    // 数字なし
    assert!(!contains_adr_reference("ADR"));     // ハイフンなし
    assert!(!contains_adr_reference("ADDR-001")); // 異なるプレフィックス
}

#[test]
fn test_parse_yaml_line() {
    // 正常なキー: 値
    assert_eq!(
        parse_yaml_line("key: value"),
        Some(("key".to_string(), "value".to_string()))
    );
    assert_eq!(
        parse_yaml_line("  password: secret123"),
        Some(("password".to_string(), "secret123".to_string()))
    );

    // 値が空
    assert_eq!(
        parse_yaml_line("token:"),
        Some(("token".to_string(), "".to_string()))
    );

    // コメント行
    assert_eq!(parse_yaml_line("# comment"), None);
    assert_eq!(parse_yaml_line("  # indented comment"), None);

    // 空行
    assert_eq!(parse_yaml_line(""), None);
    assert_eq!(parse_yaml_line("   "), None);

    // リスト項目
    assert_eq!(parse_yaml_line("- item"), None);
}
