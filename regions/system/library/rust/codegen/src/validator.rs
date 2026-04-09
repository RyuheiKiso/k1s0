use std::path::Path;

#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[must_use]
pub fn validate_generated(output_dir: &Path) -> ValidationResult {
    let mut result = ValidationResult {
        errors: vec![],
        warnings: vec![],
    };

    if !output_dir.join("Cargo.toml").exists() {
        result.errors.push("Cargo.toml not found".into());
    }

    if !output_dir.join("src/main.rs").exists() {
        result.errors.push("src/main.rs not found".into());
    }

    if !output_dir.join("src/lib.rs").exists() {
        result.errors.push("src/lib.rs not found".into());
    }

    if !output_dir.join("src/error.rs").exists() {
        result.warnings.push("src/error.rs not found".into());
    }

    if !output_dir.join("config/config.yaml").exists() {
        result.warnings.push("config/config.yaml not found".into());
    }

    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // 空ディレクトリでバリデーションを実行するとエラーが返されることを確認する。
    #[test]
    fn empty_dir_has_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let result = validate_generated(tmp.path());
        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 3);
    }

    // 必須ファイルが揃ったディレクトリでバリデーションが成功することを確認する。
    #[test]
    fn valid_dir_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/main.rs"), "").unwrap();
        std::fs::write(dir.join("src/lib.rs"), "").unwrap();
        let result = validate_generated(dir);
        assert!(result.is_ok());
    }

    // main.rs が欠けている場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn missing_main_rs() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/lib.rs"), "").unwrap();
        let result = validate_generated(dir);
        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| e.contains("main.rs")));
    }

    // オプションファイルが欠けている場合に警告が返されることを確認する。
    #[test]
    fn warnings_for_optional_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/main.rs"), "").unwrap();
        std::fs::write(dir.join("src/lib.rs"), "").unwrap();
        let result = validate_generated(dir);
        assert!(result.is_ok());
        assert!(!result.warnings.is_empty());
    }
}
