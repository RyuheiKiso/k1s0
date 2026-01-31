use crate::manifest::LayerType;

/// サービスの種別ごとの必須ファイル定義
#[derive(Debug, Clone)]
pub struct RequiredFiles {
    /// 必須ディレクトリ
    pub directories: Vec<&'static str>,
    /// 必須ファイル
    pub files: Vec<&'static str>,
}

impl RequiredFiles {
    /// backend-rust（feature 層）の必須ファイル
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

    /// backend-rust（domain 層）の必須ファイル
    pub fn backend_rust_domain() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
            ],
            files: vec![
                "Cargo.toml",
                "README.md",
                "src/lib.rs",
                "src/domain/mod.rs",
                "src/application/mod.rs",
                "src/infrastructure/mod.rs",
            ],
        }
    }

    /// backend-go（feature 層）の必須ファイル
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

    /// backend-go（domain 層）の必須ファイル
    pub fn backend_go_domain() -> Self {
        Self {
            directories: vec![
                "internal/domain",
                "internal/application",
                "internal/infrastructure",
            ],
            files: vec![
                "go.mod",
                "README.md",
            ],
        }
    }

    /// frontend-react（feature 層）の必須ファイル
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

    /// frontend-react（domain 層）の必須ファイル
    pub fn frontend_react_domain() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
            ],
            files: vec![
                "package.json",
                "README.md",
                "tsconfig.json",
            ],
        }
    }

    /// frontend-flutter（feature 層）の必須ファイル
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

    /// frontend-flutter（domain 層）の必須ファイル
    pub fn frontend_flutter_domain() -> Self {
        Self {
            directories: vec![
                "lib/src/domain",
                "lib/src/application",
            ],
            files: vec![
                "pubspec.yaml",
                "README.md",
            ],
        }
    }

    /// backend-python（feature 層）の必須ファイル
    pub fn backend_python() -> Self {
        Self {
            directories: vec![
                "src",
                "config",
                "deploy/base",
            ],
            files: vec![
                "pyproject.toml",
                "README.md",
                "config/default.yaml",
                "config/dev.yaml",
                "config/stg.yaml",
                "config/prod.yaml",
            ],
        }
    }

    /// backend-python（domain 層）の必須ファイル
    pub fn backend_python_domain() -> Self {
        Self {
            directories: vec![],
            files: vec![
                "pyproject.toml",
                "README.md",
            ],
        }
    }

    /// backend-csharp（feature 層）の必須ファイル
    pub fn backend_csharp() -> Self {
        Self {
            directories: vec![
                "src",
                "config",
                "deploy/base",
            ],
            files: vec![
                "README.md",
                "config/default.yaml",
                "config/dev.yaml",
                "config/stg.yaml",
                "config/prod.yaml",
                "buf.yaml",
            ],
        }
    }

    /// backend-csharp（domain 層）の必須ファイル
    pub fn backend_csharp_domain() -> Self {
        Self {
            directories: vec![],
            files: vec![
                "README.md",
            ],
        }
    }

    /// backend-kotlin（feature 層）の必須ファイル
    pub fn backend_kotlin() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
                "src/presentation",
                "config",
                "deploy/base",
            ],
            files: vec![
                "build.gradle.kts",
                "README.md",
                "config/default.yaml",
                "config/dev.yaml",
                "config/stg.yaml",
                "config/prod.yaml",
            ],
        }
    }

    /// backend-kotlin（domain 層）の必須ファイル
    pub fn backend_kotlin_domain() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
            ],
            files: vec![
                "build.gradle.kts",
                "README.md",
            ],
        }
    }

    /// frontend-android（feature 層）の必須ファイル
    pub fn frontend_android() -> Self {
        Self {
            directories: vec![
                "app/src/main/kotlin",
                "config",
            ],
            files: vec![
                "build.gradle.kts",
                "README.md",
                "config/default.yaml",
            ],
        }
    }

    /// frontend-android（domain 層）の必須ファイル
    pub fn frontend_android_domain() -> Self {
        Self {
            directories: vec![],
            files: vec![
                "build.gradle.kts",
                "README.md",
            ],
        }
    }

    /// テンプレート名とレイヤーから必須ファイルを取得
    pub fn from_template_and_layer(name: &str, layer: &LayerType) -> Option<Self> {
        match (name, layer) {
            ("backend-rust", LayerType::Domain) => Some(Self::backend_rust_domain()),
            ("backend-rust", _) => Some(Self::backend_rust()),
            ("backend-go", LayerType::Domain) => Some(Self::backend_go_domain()),
            ("backend-go", _) => Some(Self::backend_go()),
            ("backend-csharp", LayerType::Domain) => Some(Self::backend_csharp_domain()),
            ("backend-csharp", _) => Some(Self::backend_csharp()),
            ("backend-python", LayerType::Domain) => Some(Self::backend_python_domain()),
            ("backend-python", _) => Some(Self::backend_python()),
            ("frontend-react", LayerType::Domain) => Some(Self::frontend_react_domain()),
            ("frontend-react", _) => Some(Self::frontend_react()),
            ("frontend-flutter", LayerType::Domain) => Some(Self::frontend_flutter_domain()),
            ("frontend-flutter", _) => Some(Self::frontend_flutter()),
            ("backend-kotlin", LayerType::Domain) => Some(Self::backend_kotlin_domain()),
            ("backend-kotlin", _) => Some(Self::backend_kotlin()),
            ("frontend-android", LayerType::Domain) => Some(Self::frontend_android_domain()),
            ("frontend-android", _) => Some(Self::frontend_android()),
            _ => None,
        }
    }

    /// テンプレート名から必須ファイルを取得（後方互換、feature 層として扱う）
    pub fn from_template_name(name: &str) -> Option<Self> {
        Self::from_template_and_layer(name, &LayerType::Feature)
    }
}
