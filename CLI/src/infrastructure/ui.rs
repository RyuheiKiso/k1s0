use console::style;

pub fn format_message(message: &str) -> String {
    if message.contains("保存しました") || message.contains("作成しました") {
        format!("{} {}", style("✓").green(), style(message).green())
    } else if message.contains("失敗") || message.contains("無効") {
        format!("{} {}", style("✗").red(), style(message).red())
    } else if message.contains("未設定") {
        format!("{} {}", style("!").yellow(), style(message).yellow())
    } else {
        format!("  {}", style(message).cyan())
    }
}

pub fn render_banner() {
    let line = style("━".repeat(40)).cyan();
    println!("{line}");
    println!("  {}", style("k1s0 - Platform CLI").cyan().bold());
    println!("  {}", style("多言語対応プラットフォーム開発ツール").cyan());
    println!("{line}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_message_success() {
        let result = format_message("保存しました: C:\\workspace");
        assert!(result.contains("✓"));
        assert!(result.contains("保存しました"));
    }

    #[test]
    fn format_message_error_failure() {
        let result = format_message("保存に失敗しました: error");
        assert!(result.contains("✗"));
        assert!(result.contains("失敗"));
    }

    #[test]
    fn format_message_error_invalid() {
        let result = format_message("無効なパスです");
        assert!(result.contains("✗"));
        assert!(result.contains("無効"));
    }

    #[test]
    fn format_message_warning() {
        let result = format_message("ワークスペースが未設定です");
        assert!(result.contains("!"));
        assert!(result.contains("未設定"));
    }

    #[test]
    fn format_message_info() {
        let result = format_message("ワークスペース: C:\\projects");
        assert!(!result.contains("✓"));
        assert!(!result.contains("✗"));
        assert!(!result.contains("!"));
        assert!(result.contains("ワークスペース"));
    }
}
