"""devcontainer設計.md の仕様準拠テスト。

.devcontainer/devcontainer.json の内容がドキュメントと一致するかを検証する。
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class TestDevcontainerConfig:
    """devcontainer設計.md: devcontainer.json の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / ".devcontainer" / "devcontainer.json"
        assert config_path.exists(), "devcontainer.json が存在しません"
        # JSONC (comments) を処理するため、コメント行を除去してパース
        content = config_path.read_text(encoding="utf-8")
        lines = []
        for line in content.splitlines():
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            # インラインコメントは JSON パーサーが処理できないので除去
            if "//" in line and not stripped.startswith('"'):
                line = line[: line.index("//")]
            lines.append(line)
        self.config = json.loads("\n".join(lines))

    def test_name(self) -> None:
        assert self.config["name"] == "k1s0"

    def test_workspace_folder(self) -> None:
        assert self.config["workspaceFolder"] == "/workspace"

    def test_service(self) -> None:
        assert self.config["service"] == "devcontainer"

    def test_docker_compose_files(self) -> None:
        assert self.config["dockerComposeFile"] == [
            "../docker-compose.yaml",
            "docker-compose.extend.yaml",
        ]

    def test_go_version(self) -> None:
        features = self.config["features"]
        assert features["ghcr.io/devcontainers/features/go:1"]["version"] == "1.23"

    def test_rust_version(self) -> None:
        features = self.config["features"]
        assert features["ghcr.io/devcontainers/features/rust:1"]["version"] == "1.82"

    def test_node_version(self) -> None:
        features = self.config["features"]
        assert features["ghcr.io/devcontainers/features/node:1"]["version"] == "22"

    def test_python_version(self) -> None:
        features = self.config["features"]
        assert features["ghcr.io/devcontainers/features/python:1"]["version"] == "3.12"

    def test_docker_in_docker(self) -> None:
        features = self.config["features"]
        assert "ghcr.io/devcontainers/features/docker-in-docker:2" in features

    def test_kubectl_helm(self) -> None:
        features = self.config["features"]
        kubectl_helm = features["ghcr.io/devcontainers/features/kubectl-helm-minikube:1"]
        assert kubectl_helm["helm"] == "3.16"
        assert kubectl_helm["minikube"] == "none"

    def test_vscode_extensions(self) -> None:
        extensions = self.config["customizations"]["vscode"]["extensions"]
        expected = [
            "golang.go",
            "rust-lang.rust-analyzer",
            "dbaeumer.vscode-eslint",
            "esbenp.prettier-vscode",
            "Dart-Code.dart-code",
            "Dart-Code.flutter",
            "ms-python.python",
            "charliermarsh.ruff",
            "ms-azuretools.vscode-docker",
            "redhat.vscode-yaml",
            "42Crunch.vscode-openapi",
            "zxh404.vscode-proto3",
            "GraphQL.vscode-graphql",
            "eamodio.gitlens",
        ]
        for ext in expected:
            assert ext in extensions, f"拡張機能 {ext} が存在しません"

    def test_forward_ports(self) -> None:
        ports = self.config["forwardPorts"]
        expected_ports = [
            8080,
            50051,
            3000,
            5173,
            5432,
            3306,
            6379,
            6380,
            9092,
            8081,
            16686,
            4317,
            4318,
            3100,
            9090,
            3200,
            8090,
            8200,
            8180,
        ]
        for port in expected_ports:
            assert port in ports, f"ポート {port} がフォワードされていません"

    def test_post_create_command(self) -> None:
        assert self.config["postCreateCommand"] == "bash .devcontainer/post-create.sh"

    def test_remote_user(self) -> None:
        assert self.config["remoteUser"] == "vscode"

    def test_format_on_save(self) -> None:
        settings = self.config["customizations"]["vscode"]["settings"]
        assert settings["editor.formatOnSave"] is True

    def test_code_actions_on_save(self) -> None:
        """devcontainer設計.md: codeActionsOnSave の設定検証。"""
        settings = self.config["customizations"]["vscode"]["settings"]
        actions = settings["editor.codeActionsOnSave"]
        assert actions["source.fixAll"] == "explicit"
        assert actions["source.organizeImports"] == "explicit"

    def test_go_default_formatter(self) -> None:
        """devcontainer設計.md: Go の defaultFormatter 設定。"""
        settings = self.config["customizations"]["vscode"]["settings"]
        assert settings["[go]"]["editor.defaultFormatter"] == "golang.go"

    def test_rust_default_formatter(self) -> None:
        """devcontainer設計.md: Rust の defaultFormatter 設定。"""
        settings = self.config["customizations"]["vscode"]["settings"]
        assert settings["[rust]"]["editor.defaultFormatter"] == "rust-lang.rust-analyzer"

    def test_typescript_default_formatter(self) -> None:
        """devcontainer設計.md: TypeScript の defaultFormatter 設定。"""
        settings = self.config["customizations"]["vscode"]["settings"]
        assert (
            settings["[typescript][typescriptreact]"]["editor.defaultFormatter"]
            == "esbenp.prettier-vscode"
        )

    def test_dart_default_formatter(self) -> None:
        """devcontainer設計.md: Dart の defaultFormatter 設定。"""
        settings = self.config["customizations"]["vscode"]["settings"]
        assert settings["[dart]"]["editor.defaultFormatter"] == "Dart-Code.dart-code"

    def test_python_default_formatter(self) -> None:
        """devcontainer設計.md: Python の defaultFormatter 設定。"""
        settings = self.config["customizations"]["vscode"]["settings"]
        assert settings["[python]"]["editor.defaultFormatter"] == "charliermarsh.ruff"


class TestDevcontainerExtendCompose:
    """devcontainer設計.md: docker-compose.extend.yaml の検証。"""

    def test_extend_compose_exists(self) -> None:
        path = ROOT / ".devcontainer" / "docker-compose.extend.yaml"
        assert path.exists(), "docker-compose.extend.yaml が存在しません"

    def test_devcontainer_service(self) -> None:
        path = ROOT / ".devcontainer" / "docker-compose.extend.yaml"
        import yaml  # type: ignore[import-untyped]

        with open(path, encoding="utf-8") as f:
            config = yaml.safe_load(f)
        svc = config["services"]["devcontainer"]
        assert "mcr.microsoft.com/devcontainers" in svc["image"]
        assert svc["command"] == "sleep infinity"


class TestPostCreateScript:
    """devcontainer設計.md: post-create.sh の検証。"""

    def test_post_create_script_exists(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        assert script.exists(), "post-create.sh が存在しません"

    def test_post_create_installs_go_tools(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "goimports" in content
        assert "golangci-lint" in content
        assert "protoc-gen-go" in content
        assert "protoc-gen-go-grpc" in content
        assert "oapi-codegen" in content

    def test_post_create_installs_rust_components(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "rustup component add clippy rustfmt" in content

    def test_post_create_installs_flutter(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "3.24.0" in content
        assert "flutter precache" in content

    def test_post_create_installs_python_deps(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "pip install -r e2e/requirements.txt" in content

    def test_post_create_installs_precommit(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "pre-commit install" in content

    def test_post_create_installs_buf(self) -> None:
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "buf" in content

    def test_post_create_installs_protobuf_compiler(self) -> None:
        """devcontainer設計.md: protobuf-compiler がインストールされること。"""
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert "protobuf-compiler" in content


class TestDevcontainerSparseCheckout:
    """devcontainer設計.md: sparse-checkout 連携テスト。"""

    def test_sparse_checkout_docs(self) -> None:
        """devcontainer設計.md: sparse-checkout 連携がドキュメントに記載されていること。"""
        doc = (ROOT / "docs" / "devcontainer設計.md").read_text(encoding="utf-8")
        assert "sparse-checkout" in doc


class TestDevcontainerRustVersionSync:
    """devcontainer設計.md: Rust バージョン同期テスト。"""

    def test_rust_version_matches_docker_image_strategy(self) -> None:
        """devcontainer設計.md: devcontainer の Rust バージョンが Dockerイメージ戦略.md と同期。"""
        config_path = ROOT / ".devcontainer" / "devcontainer.json"
        content = config_path.read_text(encoding="utf-8")
        lines = []
        for line in content.splitlines():
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            if "//" in line and not stripped.startswith('"'):
                line = line[: line.index("//")]
            lines.append(line)
        config = json.loads("\n".join(lines))
        rust_version = config["features"]["ghcr.io/devcontainers/features/rust:1"]["version"]
        # Dockerイメージ戦略.md で rust:1.82-bookworm
        assert rust_version == "1.82"


class TestDevcontainerFlutterVersionSync:
    """devcontainer設計.md: Flutter バージョン同期テスト。"""

    def test_flutter_version_matches_ci(self) -> None:
        """devcontainer設計.md: post-create.sh の Flutter バージョンが 3.24.0 であること。"""
        script = ROOT / ".devcontainer" / "post-create.sh"
        content = script.read_text(encoding="utf-8")
        assert 'FLUTTER_VERSION="3.24.0"' in content


class TestDevcontainerGolangciLintSettings:
    """devcontainer設計.md: golangci-lint VSCode settings テスト。"""

    def test_golangci_lint_in_settings(self) -> None:
        """devcontainer設計.md: go.lintTool が golangci-lint に設定されていること。"""
        config_path = ROOT / ".devcontainer" / "devcontainer.json"
        content = config_path.read_text(encoding="utf-8")
        lines = []
        for line in content.splitlines():
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            if "//" in line and not stripped.startswith('"'):
                line = line[: line.index("//")]
            lines.append(line)
        config = json.loads("\n".join(lines))
        settings = config["customizations"]["vscode"]["settings"]
        assert settings["go.lintTool"] == "golangci-lint"
        assert settings["go.lintFlags"] == ["--fast"]


class TestDevcontainerExtendDependsOn:
    """devcontainer設計.md: docker-compose.extend depends_on テスト。"""

    def test_extend_depends_on_postgres_redis(self) -> None:
        """devcontainer設計.md: devcontainer が postgres と redis に依存していること。"""
        path = ROOT / ".devcontainer" / "docker-compose.extend.yaml"
        import yaml as _yaml  # type: ignore[import-untyped]

        with open(path, encoding="utf-8") as f:
            config = _yaml.safe_load(f)
        deps = config["services"]["devcontainer"]["depends_on"]
        if isinstance(deps, list):
            assert "postgres" in deps
            assert "redis" in deps
        else:
            assert "postgres" in deps
            assert "redis" in deps
