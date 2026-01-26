/// ADR 参照パターン（ADR-XXX）が含まれているかチェック
pub(crate) fn contains_adr_reference(text: &str) -> bool {
    // ADR- または adr- を検索
    let text_upper = text.to_uppercase();
    if let Some(pos) = text_upper.find("ADR-") {
        // ADR- の後に少なくとも3桁の数字が続くかチェック
        let after_adr = &text[pos + 4..];
        let digit_count = after_adr.chars().take_while(|c| c.is_ascii_digit()).count();
        return digit_count >= 3;
    }
    false
}

/// YAML の行をパースしてキーと値を取得
pub(crate) fn parse_yaml_line(line: &str) -> Option<(String, String)> {
    // インデントを除去
    let trimmed = line.trim_start();

    // コメントや空行はスキップ
    if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.starts_with('-') {
        return None;
    }

    // キー: 値 の形式を探す
    if let Some(colon_pos) = trimmed.find(':') {
        let key = trimmed[..colon_pos].trim();
        let value = if colon_pos + 1 < trimmed.len() {
            trimmed[colon_pos + 1..].trim()
        } else {
            ""
        };

        // キーが空でなければ返す
        if !key.is_empty() {
            return Some((key.to_string(), value.to_string()));
        }
    }

    None
}
