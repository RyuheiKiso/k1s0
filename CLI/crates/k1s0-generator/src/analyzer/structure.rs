//! ディレクトリ構造分析

use std::path::Path;

use super::types::{DetectedProjectType, StructureAnalysis};

/// Clean Architecture の層名
const LAYER_NAMES: &[&str] = &["domain", "application", "infrastructure", "presentation"];

/// 層名のエイリアス（検出に使用）
const LAYER_ALIASES: &[(&str, &[&str])] = &[
    ("domain", &["domain", "domains", "entities", "model", "models", "core"]),
    ("application", &["application", "app", "usecase", "usecases", "use_cases", "services"]),
    ("infrastructure", &["infrastructure", "infra", "adapter", "adapters", "persistence", "repository", "repositories", "external"]),
    ("presentation", &["presentation", "api", "handler", "handlers", "controller", "controllers", "rest", "grpc", "web", "ui"]),
];

/// 不要ファイルのパターン
const UNNECESSARY_FILES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.development",
    ".env.production",
    ".env.staging",
    ".env.test",
    ".env.example",
];

/// プロジェクト構造を分析する
pub fn analyze_structure(path: &Path, project_type: &DetectedProjectType) -> StructureAnalysis {
    let (required_dirs, required_files) = required_for_type(project_type);
    let src_prefix = source_prefix(project_type);

    let mut existing_dirs = Vec::new();
    let mut missing_dirs = Vec::new();
    for dir in &required_dirs {
        if path.join(dir).exists() {
            existing_dirs.push((*dir).to_string());
        } else {
            missing_dirs.push((*dir).to_string());
        }
    }

    let mut existing_files = Vec::new();
    let mut missing_files = Vec::new();
    for file in &required_files {
        if path.join(file).exists() {
            existing_files.push((*file).to_string());
        } else {
            missing_files.push((*file).to_string());
        }
    }

    // Clean Architecture 層の検出
    let mut detected_layers = Vec::new();
    let mut missing_layers = Vec::new();

    for layer in LAYER_NAMES {
        if detect_layer(path, src_prefix, layer) {
            detected_layers.push((*layer).to_string());
        } else {
            missing_layers.push((*layer).to_string());
        }
    }

    // 不要ファイルの検出
    let _unnecessary: Vec<String> = UNNECESSARY_FILES
        .iter()
        .filter(|f| path.join(f).exists())
        .map(|f| (*f).to_string())
        .collect();

    StructureAnalysis {
        existing_dirs,
        missing_dirs,
        existing_files,
        missing_files,
        detected_layers,
        missing_layers,
    }
}

/// 構造スコアを計算する (0-100)
pub fn calculate_structure_score(analysis: &StructureAnalysis) -> u32 {
    let total_dirs = analysis.existing_dirs.len() + analysis.missing_dirs.len();
    let total_files = analysis.existing_files.len() + analysis.missing_files.len();
    let total_layers = analysis.detected_layers.len() + analysis.missing_layers.len();

    if total_dirs + total_files + total_layers == 0 {
        return 0;
    }

    // ディレクトリ: 30%, ファイル: 30%, 層: 40%
    let dir_score = if total_dirs > 0 {
        (analysis.existing_dirs.len() as f64 / total_dirs as f64) * 30.0
    } else {
        30.0
    };

    let file_score = if total_files > 0 {
        (analysis.existing_files.len() as f64 / total_files as f64) * 30.0
    } else {
        30.0
    };

    let layer_score = if total_layers > 0 {
        (analysis.detected_layers.len() as f64 / total_layers as f64) * 40.0
    } else {
        40.0
    };

    let score = dir_score + file_score + layer_score;
    score.round().min(100.0) as u32
}

fn source_prefix(project_type: &DetectedProjectType) -> &'static str {
    match project_type {
        DetectedProjectType::BackendRust => "src",
        DetectedProjectType::BackendGo => "internal",
        DetectedProjectType::BackendCsharp => "src",
        DetectedProjectType::BackendPython => "src",
        DetectedProjectType::FrontendReact => "src",
        DetectedProjectType::FrontendFlutter => "lib/src",
        DetectedProjectType::Unknown => "src",
    }
}

fn detect_layer(path: &Path, src_prefix: &str, layer: &str) -> bool {
    // まずは正規名でチェック
    if path.join(src_prefix).join(layer).exists() {
        return true;
    }

    // エイリアスでチェック
    if let Some((_, aliases)) = LAYER_ALIASES.iter().find(|(name, _)| *name == layer) {
        for alias in *aliases {
            if path.join(src_prefix).join(alias).exists() {
                return true;
            }
        }
    }

    false
}

fn required_for_type(project_type: &DetectedProjectType) -> (Vec<&'static str>, Vec<&'static str>) {
    match project_type {
        DetectedProjectType::BackendRust => (
            vec![
                "src",
                "src/domain",
                "src/application",
                "src/infrastructure",
                "src/presentation",
                "config",
                "deploy",
            ],
            vec![
                "Cargo.toml",
                "src/main.rs",
                "config/default.yaml",
                ".k1s0/manifest.json",
                "Dockerfile",
                ".dockerignore",
                "docker-compose.yml",
            ],
        ),
        DetectedProjectType::BackendGo => (
            vec![
                "cmd",
                "internal/domain",
                "internal/application",
                "internal/infrastructure",
                "internal/presentation",
                "config",
                "deploy",
            ],
            vec![
                "go.mod",
                "config/default.yaml",
                ".k1s0/manifest.json",
                "Dockerfile",
                ".dockerignore",
                "docker-compose.yml",
            ],
        ),
        DetectedProjectType::BackendCsharp => (
            vec!["src", "config", "deploy"],
            vec![
                "config/default.yaml",
                ".k1s0/manifest.json",
            ],
        ),
        DetectedProjectType::BackendPython => (
            vec!["src", "config", "deploy"],
            vec![
                "pyproject.toml",
                "config/default.yaml",
                ".k1s0/manifest.json",
            ],
        ),
        DetectedProjectType::FrontendReact => (
            vec![
                "src",
                "src/domain",
                "src/application",
                "src/presentation",
                "public",
            ],
            vec![
                "package.json",
                "tsconfig.json",
                ".k1s0/manifest.json",
                "Dockerfile",
                ".dockerignore",
                "docker-compose.yml",
                "deploy/nginx.conf",
            ],
        ),
        DetectedProjectType::FrontendFlutter => (
            vec![
                "lib",
                "lib/src/domain",
                "lib/src/application",
                "lib/src/presentation",
            ],
            vec!["pubspec.yaml", ".k1s0/manifest.json"],
        ),
        DetectedProjectType::Unknown => (vec![], vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_analyze_empty_rust_project() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        fs::write(tmp.path().join("Cargo.toml"), "[package]").expect("write failed");

        let result = analyze_structure(tmp.path(), &DetectedProjectType::BackendRust);
        assert!(!result.missing_dirs.is_empty());
        assert!(!result.missing_files.is_empty());
    }

    #[test]
    fn test_analyze_full_rust_project() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();

        for dir in &[
            "src/domain",
            "src/application",
            "src/infrastructure",
            "src/presentation",
            "config",
            "deploy",
            ".k1s0",
        ] {
            fs::create_dir_all(p.join(dir)).expect("mkdir failed");
        }

        for file in &[
            "Cargo.toml",
            "src/main.rs",
            "config/default.yaml",
            ".k1s0/manifest.json",
            "Dockerfile",
            ".dockerignore",
            "docker-compose.yml",
        ] {
            fs::write(p.join(file), "").expect("write failed");
        }

        let result = analyze_structure(p, &DetectedProjectType::BackendRust);
        assert!(result.missing_dirs.is_empty());
        assert!(result.missing_files.is_empty());
        assert_eq!(result.detected_layers.len(), 4);
    }

    #[test]
    fn test_layer_alias_detection() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();

        // Use aliases instead of standard names
        fs::create_dir_all(p.join("src/core")).expect("mkdir failed");
        fs::create_dir_all(p.join("src/usecases")).expect("mkdir failed");
        fs::create_dir_all(p.join("src/adapters")).expect("mkdir failed");
        fs::create_dir_all(p.join("src/handlers")).expect("mkdir failed");

        let result = analyze_structure(p, &DetectedProjectType::BackendRust);
        assert_eq!(result.detected_layers.len(), 4);
    }

    #[test]
    fn test_structure_score() {
        let analysis = StructureAnalysis {
            existing_dirs: vec!["src".to_string()],
            missing_dirs: vec!["config".to_string()],
            existing_files: vec!["Cargo.toml".to_string()],
            missing_files: vec!["Dockerfile".to_string()],
            detected_layers: vec!["domain".to_string(), "application".to_string()],
            missing_layers: vec!["infrastructure".to_string(), "presentation".to_string()],
        };

        let score = calculate_structure_score(&analysis);
        assert!(score > 0);
        assert!(score < 100);
    }
}
