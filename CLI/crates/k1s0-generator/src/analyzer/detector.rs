//! プロジェクトタイプの自動検出

use std::path::Path;

use super::types::DetectedProjectType;

/// プロジェクトタイプを自動検出する
///
/// ファイルの存在をチェックしてプロジェクトの種類を判別する。
pub fn detect_project_type(path: &Path) -> DetectedProjectType {
    // Rust: Cargo.toml
    if path.join("Cargo.toml").exists() {
        return DetectedProjectType::BackendRust;
    }

    // Go: go.mod
    if path.join("go.mod").exists() {
        return DetectedProjectType::BackendGo;
    }

    // C#: *.sln or *.csproj
    if has_extension_in_dir(path, "sln") || has_extension_in_dir(path, "csproj") {
        return DetectedProjectType::BackendCsharp;
    }

    // Python: pyproject.toml or setup.py
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return DetectedProjectType::BackendPython;
    }

    // Flutter: pubspec.yaml (check before React since both could have package.json)
    if path.join("pubspec.yaml").exists() {
        return DetectedProjectType::FrontendFlutter;
    }

    // React: package.json with react dependency
    if path.join("package.json").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if content.contains("\"react\"") {
                return DetectedProjectType::FrontendReact;
            }
        }
    }

    DetectedProjectType::Unknown
}

/// ディレクトリ直下に指定した拡張子のファイルがあるか
fn has_extension_in_dir(dir: &Path, ext: &str) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|e| e == ext)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(detect_project_type(tmp.path()), DetectedProjectType::BackendRust);
    }

    #[test]
    fn test_detect_go() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("go.mod"), "module example").unwrap();
        assert_eq!(detect_project_type(tmp.path()), DetectedProjectType::BackendGo);
    }

    #[test]
    fn test_detect_unknown() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_project_type(tmp.path()), DetectedProjectType::Unknown);
    }
}
