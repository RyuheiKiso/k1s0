//! ツールバージョン要件定義
//!
//! 各ツールの最小バージョンとインストール情報を定義する。

/// ツール情報
#[derive(Debug, Clone)]
pub struct ToolRequirement {
    /// ツール名
    pub name: &'static str,
    /// 最小バージョン（None の場合はバージョンチェックなし）
    pub min_version: Option<&'static str>,
    /// 必須かどうか
    pub required: bool,
    /// カテゴリ
    pub category: ToolCategory,
    /// インストールURL
    pub install_url: &'static str,
    /// インストールコマンド（プラットフォーム別）
    pub install_commands: InstallCommands,
    /// 説明
    pub description: &'static str,
}

/// ツールカテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    /// Rust 関連
    Rust,
    /// Go 関連
    Go,
    /// Node.js 関連
    Node,
    /// Flutter 関連
    Flutter,
    /// Protocol Buffers 関連
    Proto,
    /// Docker 関連
    Docker,
}

impl ToolCategory {
    /// カテゴリ名を取得
    pub fn name(&self) -> &'static str {
        match self {
            ToolCategory::Rust => "Rust",
            ToolCategory::Go => "Go",
            ToolCategory::Node => "Node.js",
            ToolCategory::Flutter => "Flutter",
            ToolCategory::Proto => "Protocol Buffers",
            ToolCategory::Docker => "Docker",
        }
    }
}

/// プラットフォーム別インストールコマンド
#[derive(Debug, Clone)]
pub struct InstallCommands {
    pub windows: Option<&'static str>,
    pub macos: Option<&'static str>,
    pub linux: Option<&'static str>,
}

impl InstallCommands {
    /// 現在のプラットフォームのコマンドを取得
    pub fn current_platform(&self) -> Option<&'static str> {
        #[cfg(target_os = "windows")]
        {
            self.windows
        }
        #[cfg(target_os = "macos")]
        {
            self.macos
        }
        #[cfg(target_os = "linux")]
        {
            self.linux
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }
}

// --- 必須ツール ---

/// Rust 最小バージョン
pub const RUST_MIN_VERSION: &str = "1.85.0";

/// Rust ツール情報
pub const RUST: ToolRequirement = ToolRequirement {
    name: "rustc",
    min_version: Some(RUST_MIN_VERSION),
    required: true,
    category: ToolCategory::Rust,
    install_url: "https://rustup.rs/",
    install_commands: InstallCommands {
        windows: Some("winget install Rustlang.Rustup"),
        macos: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"),
        linux: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"),
    },
    description: "Rust コンパイラ（CLI本体、backend-rust に必要）",
};

/// Cargo ツール情報
pub const CARGO: ToolRequirement = ToolRequirement {
    name: "cargo",
    min_version: None, // rustc と同じバージョン
    required: true,
    category: ToolCategory::Rust,
    install_url: "https://rustup.rs/",
    install_commands: InstallCommands {
        windows: Some("winget install Rustlang.Rustup"),
        macos: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"),
        linux: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"),
    },
    description: "Rust パッケージマネージャ",
};

/// Node.js 最小バージョン
pub const NODE_MIN_VERSION: &str = "20.0.0";

/// Node.js ツール情報
pub const NODE: ToolRequirement = ToolRequirement {
    name: "node",
    min_version: Some(NODE_MIN_VERSION),
    required: true,
    category: ToolCategory::Node,
    install_url: "https://nodejs.org/",
    install_commands: InstallCommands {
        windows: Some("winget install OpenJS.NodeJS.LTS"),
        macos: Some("brew install node@20"),
        linux: Some("curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && sudo apt-get install -y nodejs"),
    },
    description: "Node.js ランタイム（frontend-react に必要）",
};

/// pnpm 最小バージョン
pub const PNPM_MIN_VERSION: &str = "9.15.4";

/// pnpm ツール情報
pub const PNPM: ToolRequirement = ToolRequirement {
    name: "pnpm",
    min_version: Some(PNPM_MIN_VERSION),
    required: true,
    category: ToolCategory::Node,
    install_url: "https://pnpm.io/installation",
    install_commands: InstallCommands {
        windows: Some("npm install -g pnpm"),
        macos: Some("npm install -g pnpm"),
        linux: Some("npm install -g pnpm"),
    },
    description: "Node.js パッケージマネージャ",
};

// --- オプションツール ---

/// Go 最小バージョン
pub const GO_MIN_VERSION: &str = "1.21.0";

/// Go ツール情報
pub const GO: ToolRequirement = ToolRequirement {
    name: "go",
    min_version: Some(GO_MIN_VERSION),
    required: false,
    category: ToolCategory::Go,
    install_url: "https://go.dev/dl/",
    install_commands: InstallCommands {
        windows: Some("winget install GoLang.Go"),
        macos: Some("brew install go"),
        linux: Some("sudo apt install golang-go"),
    },
    description: "Go コンパイラ（backend-go に必要）",
};

/// golangci-lint 最小バージョン
pub const GOLANGCI_LINT_MIN_VERSION: &str = "1.55.0";

/// golangci-lint ツール情報
pub const GOLANGCI_LINT: ToolRequirement = ToolRequirement {
    name: "golangci-lint",
    min_version: Some(GOLANGCI_LINT_MIN_VERSION),
    required: false,
    category: ToolCategory::Go,
    install_url: "https://golangci-lint.run/welcome/install/",
    install_commands: InstallCommands {
        windows: Some("go install github.com/golangci-lint/golangci-lint/cmd/golangci-lint@latest"),
        macos: Some("brew install golangci-lint"),
        linux: Some("curl -sSfL https://raw.githubusercontent.com/golangci-lint/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin"),
    },
    description: "Go リンター（backend-go に推奨）",
};

/// buf 最小バージョン
pub const BUF_MIN_VERSION: &str = "1.28.0";

/// buf ツール情報
pub const BUF: ToolRequirement = ToolRequirement {
    name: "buf",
    min_version: Some(BUF_MIN_VERSION),
    required: false,
    category: ToolCategory::Proto,
    install_url: "https://buf.build/docs/installation",
    install_commands: InstallCommands {
        windows: Some("scoop install buf"),
        macos: Some("brew install bufbuild/buf/buf"),
        linux: Some("curl -sSL https://github.com/bufbuild/buf/releases/latest/download/buf-Linux-x86_64 -o /usr/local/bin/buf && chmod +x /usr/local/bin/buf"),
    },
    description: "Protocol Buffers ツール（gRPC サービスに必要）",
};

/// Flutter 最小バージョン
pub const FLUTTER_MIN_VERSION: &str = "3.16.0";

/// Flutter ツール情報
pub const FLUTTER: ToolRequirement = ToolRequirement {
    name: "flutter",
    min_version: Some(FLUTTER_MIN_VERSION),
    required: false,
    category: ToolCategory::Flutter,
    install_url: "https://flutter.dev/docs/get-started/install",
    install_commands: InstallCommands {
        windows: Some("winget install Google.Flutter"),
        macos: Some("brew install flutter"),
        linux: Some("snap install flutter --classic"),
    },
    description: "Flutter SDK（frontend-flutter に必要）",
};

/// Dart 最小バージョン
pub const DART_MIN_VERSION: &str = "3.2.0";

/// Dart ツール情報
pub const DART: ToolRequirement = ToolRequirement {
    name: "dart",
    min_version: Some(DART_MIN_VERSION),
    required: false,
    category: ToolCategory::Flutter,
    install_url: "https://dart.dev/get-dart",
    install_commands: InstallCommands {
        windows: Some("winget install Dart.Dart-SDK"),
        macos: Some("brew install dart"),
        linux: Some("sudo apt-get install dart"),
    },
    description: "Dart SDK（Flutter に含まれる）",
};

/// 全ての必須ツール
pub const REQUIRED_TOOLS: &[&ToolRequirement] = &[&RUST, &CARGO, &NODE, &PNPM];

/// Docker 最小バージョン
pub const DOCKER_MIN_VERSION: &str = "24.0.0";

/// Docker ツール情報
pub const DOCKER: ToolRequirement = ToolRequirement {
    name: "docker",
    min_version: Some(DOCKER_MIN_VERSION),
    required: false,
    category: ToolCategory::Docker,
    install_url: "https://docs.docker.com/get-docker/",
    install_commands: InstallCommands {
        windows: Some("winget install Docker.DockerDesktop"),
        macos: Some("brew install --cask docker"),
        linux: Some("curl -fsSL https://get.docker.com | sh"),
    },
    description: "コンテナランタイム（Docker サポートに必要）",
};

/// Docker Compose ツール情報
pub const DOCKER_COMPOSE: ToolRequirement = ToolRequirement {
    name: "docker compose",
    min_version: None,
    required: false,
    category: ToolCategory::Docker,
    install_url: "https://docs.docker.com/compose/install/",
    install_commands: InstallCommands {
        windows: Some("Docker Desktop に含まれます"),
        macos: Some("Docker Desktop に含まれます"),
        linux: Some("sudo apt-get install docker-compose-plugin"),
    },
    description: "Docker Compose v2（docker-compose サポートに必要）",
};

/// 全てのオプションツール
pub const OPTIONAL_TOOLS: &[&ToolRequirement] = &[&GO, &GOLANGCI_LINT, &BUF, &FLUTTER, &DART, &DOCKER, &DOCKER_COMPOSE];

/// 全てのツール
pub fn all_tools() -> Vec<&'static ToolRequirement> {
    let mut tools = Vec::new();
    tools.extend(REQUIRED_TOOLS.iter().copied());
    tools.extend(OPTIONAL_TOOLS.iter().copied());
    tools
}

/// サービスタイプに対応するツールカテゴリを取得
pub fn categories_for_service_type(name: &str) -> Vec<ToolCategory> {
    match name {
        "backend-rust" => vec![ToolCategory::Rust, ToolCategory::Docker],
        "backend-go" => vec![ToolCategory::Go, ToolCategory::Docker],
        "backend-python" => vec![ToolCategory::Docker],
        "backend-csharp" => vec![ToolCategory::Docker],
        "frontend-react" => vec![ToolCategory::Node, ToolCategory::Docker],
        "frontend-flutter" => vec![ToolCategory::Flutter],
        _ => vec![],
    }
}

/// カテゴリでフィルタされたツールを取得
pub fn tools_by_category(category: ToolCategory) -> Vec<&'static ToolRequirement> {
    all_tools()
        .into_iter()
        .filter(|t| t.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_for_service_type() {
        assert_eq!(categories_for_service_type("backend-rust"), vec![ToolCategory::Rust, ToolCategory::Docker]);
        assert_eq!(categories_for_service_type("backend-go"), vec![ToolCategory::Go, ToolCategory::Docker]);
        assert_eq!(categories_for_service_type("frontend-react"), vec![ToolCategory::Node, ToolCategory::Docker]);
        assert_eq!(categories_for_service_type("frontend-flutter"), vec![ToolCategory::Flutter]);
        assert_eq!(categories_for_service_type("backend-python"), vec![ToolCategory::Docker]);
        assert_eq!(categories_for_service_type("backend-csharp"), vec![ToolCategory::Docker]);
        assert!(categories_for_service_type("unknown").is_empty());
    }
}
