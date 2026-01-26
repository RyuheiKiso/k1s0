/// サービスの種別ごとの必須ファイル定義
#[derive(Debug, Clone)]
pub struct RequiredFiles {
    /// 必須ディレクトリ
    pub directories: Vec<&'static str>,
    /// 必須ファイル
    pub files: Vec<&'static str>,
}

impl RequiredFiles {
    /// backend-rust の必須ファイル
    pub fn backend_rust() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
                "src/presentation",
                "config",
                "deploy/base",
                "deploy/overlays/dev",
                "deploy/overlays/stg",
                "deploy/overlays/prod",
            ],
            files: vec![
                "Cargo.toml",
                "README.md",
                "src/main.rs",
                "src/domain/mod.rs",
                "src/application/mod.rs",
                "src/infrastructure/mod.rs",
                "src/presentation/mod.rs",
                "config/default.yaml",
                "config/dev.yaml",
                "config/stg.yaml",
                "config/prod.yaml",
                "buf.yaml",
            ],
        }
    }

    /// backend-go の必須ファイル
    pub fn backend_go() -> Self {
        Self {
            directories: vec![
                "internal/domain",
                "internal/application",
                "internal/infrastructure",
                "internal/presentation",
                "config",
            ],
            files: vec![
                "go.mod",
                "README.md",
                "cmd/main.go",
                "config/default.yaml",
            ],
        }
    }

    /// frontend-react の必須ファイル
    pub fn frontend_react() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
                "src/presentation",
                "src/pages",
                "src/components/layout",
                "config",
            ],
            files: vec![
                "package.json",
                "README.md",
                "src/main.tsx",
                "src/App.tsx",
                "config/default.yaml",
            ],
        }
    }

    /// frontend-flutter の必須ファイル
    pub fn frontend_flutter() -> Self {
        Self {
            directories: vec![
                "lib/src/domain",
                "lib/src/application",
                "lib/src/infrastructure",
                "lib/src/presentation",
                "config",
            ],
            files: vec![
                "pubspec.yaml",
                "README.md",
                "lib/main.dart",
                "config/default.yaml",
            ],
        }
    }

    /// テンプレート名から必須ファイルを取得
    pub fn from_template_name(name: &str) -> Option<Self> {
        match name {
            "backend-rust" => Some(Self::backend_rust()),
            "backend-go" => Some(Self::backend_go()),
            "frontend-react" => Some(Self::frontend_react()),
            "frontend-flutter" => Some(Self::frontend_flutter()),
            _ => None,
        }
    }
}
