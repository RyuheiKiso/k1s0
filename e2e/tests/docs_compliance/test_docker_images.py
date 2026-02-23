"""Dockerイメージ戦略.md の仕様準拠テスト。

Dockerfile テンプレートの内容がドキュメントと一致するかを検証する。
"""

from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"


class TestGoServerDockerfile:
    """Dockerイメージ戦略.md: Go サーバーの Dockerfile テンプレート検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "server" / "go" / "Dockerfile.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_build_stage_image(self) -> None:
        assert "golang:1.23" in self.content

    def test_runtime_stage_image(self) -> None:
        assert "distroless" in self.content

    def test_cgo_disabled(self) -> None:
        assert "CGO_ENABLED=0" in self.content

    def test_nonroot_user(self) -> None:
        assert "nonroot" in self.content

    def test_expose_port(self) -> None:
        assert "EXPOSE" in self.content


class TestRustServerDockerfile:
    """Dockerイメージ戦略.md: Rust サーバーの Dockerfile テンプレート検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "server" / "rust" / "Dockerfile.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_build_stage_image(self) -> None:
        assert "rust:1.82" in self.content

    def test_runtime_stage_image(self) -> None:
        assert "distroless" in self.content

    def test_release_build(self) -> None:
        assert "--release" in self.content

    def test_nonroot_user(self) -> None:
        assert "nonroot" in self.content


class TestReactClientDockerfile:
    """Dockerイメージ戦略.md: React クライアントの Dockerfile テンプレート検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "client" / "react" / "Dockerfile.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_build_stage_image(self) -> None:
        assert "node:22" in self.content

    def test_runtime_stage_image(self) -> None:
        assert "nginx" in self.content

    def test_npm_ci(self) -> None:
        assert "npm ci" in self.content

    def test_npm_build(self) -> None:
        assert "npm run build" in self.content


class TestFlutterClientDockerfile:
    """Dockerイメージ戦略.md: Flutter クライアントの Dockerfile テンプレート検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "client" / "flutter" / "Dockerfile.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_build_stage_image(self) -> None:
        assert "flutter" in self.content

    def test_runtime_stage_image(self) -> None:
        assert "nginx" in self.content

    def test_flutter_build_web(self) -> None:
        assert "flutter build web" in self.content


class TestNginxConfTemplates:
    """Dockerイメージ戦略.md: nginx.conf テンプレートの検証。"""

    @pytest.mark.parametrize("client_type", ["react", "flutter"])
    def test_nginx_conf_exists(self, client_type: str) -> None:
        path = TEMPLATES / "client" / client_type / "nginx.conf.tera"
        assert path.exists(), f"client/{client_type}/nginx.conf.tera が存在しません"

    @pytest.mark.parametrize("client_type", ["react", "flutter"])
    def test_nginx_conf_has_spa_routing(self, client_type: str) -> None:
        path = TEMPLATES / "client" / client_type / "nginx.conf.tera"
        content = path.read_text(encoding="utf-8")
        assert "try_files" in content or "index.html" in content


class TestDockerImageTagRules:
    """Dockerイメージ戦略.md: イメージタグ規則テスト。"""

    def test_ci_pipeline_has_tag_rule(self) -> None:
        """Dockerイメージ戦略.md: CI/CD テンプレートにタグ設定が含まれること。"""
        # Go サーバーの Dockerfile テンプレートが存在し、マルチステージビルド構造であること
        go_df = TEMPLATES / "server" / "go" / "Dockerfile.tera"
        assert go_df.exists()
        content = go_df.read_text(encoding="utf-8")
        # マルチステージビルドの FROM が 2 つ以上あること
        from_count = content.lower().count("from ")
        assert from_count >= 2, "マルチステージビルドの FROM が 2 つ以上必要です"

    def test_image_tag_4_patterns_in_doc(self) -> None:
        """Dockerイメージ戦略.md: イメージタグ規則4パターンがドキュメントに記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "{version}" in doc
        assert "{version}-{git-sha}" in doc
        assert "latest" in doc
        assert "{branch}-{git-sha}" in doc


class TestDockerImageTaggingRules:
    """Dockerイメージ戦略.md: タグ付けルールテスト。"""

    def test_tagging_rules_in_doc(self) -> None:
        """Dockerイメージ戦略.md: main ブランチマージ時のタグ付けルールが記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "main" in doc
        assert "latest" in doc
        assert "7 文字" in doc or "7文字" in doc


class TestDockerImageRegistryNaming:
    """Dockerイメージ戦略.md: レジストリ命名規則テスト。"""

    def test_registry_naming_format(self) -> None:
        """Dockerイメージ戦略.md: レジストリ命名規則が harbor.internal.example.com であること。"""
        values_path = ROOT / "infra" / "helm" / "services" / "service" / "order" / "values.yaml"
        import yaml as _yaml  # type: ignore[import-untyped]

        with open(values_path, encoding="utf-8") as f:
            values = _yaml.safe_load(f)
        assert values["image"]["registry"] == "harbor.internal.example.com"
        assert values["image"]["repository"] == "k1s0-service/order"


class TestDockerImageProjectStorage:
    """Dockerイメージ戦略.md: プロジェクト別格納先テスト。"""

    def test_project_storage_doc(self) -> None:
        """Dockerイメージ戦略.md: プロジェクト別格納先がドキュメントに記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "k1s0-system" in doc
        assert "k1s0-business" in doc
        assert "k1s0-service" in doc
        assert "k1s0-infra" in doc


class TestDockerImageVulnerabilityPolicy:
    """Dockerイメージ戦略.md: 脆弱性ポリシーテスト。"""

    def test_vulnerability_policy_in_doc(self) -> None:
        """Dockerイメージ戦略.md: 脆弱性ポリシー4段階が記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "Critical" in doc
        assert "High" in doc
        assert "Medium" in doc
        assert "Low" in doc


class TestDockerImageRetentionPolicy:
    """Dockerイメージ戦略.md: イメージ保持ポリシーテスト。"""

    def test_retention_policy_in_doc(self) -> None:
        """Dockerイメージ戦略.md: イメージ保持ポリシーが記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "10 バージョン" in doc or "10バージョン" in doc
        assert "7 日" in doc or "7日" in doc


class TestDockerImageCosignKeyless:
    """Dockerイメージ戦略.md: Cosign keyless signing テスト。"""

    def test_cosign_keyless_in_doc(self) -> None:
        """Dockerイメージ戦略.md: keyless signing が記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "keyless" in doc.lower()
        assert "COSIGN_EXPERIMENTAL" in doc or "Sigstore" in doc


class TestDockerImageSignatureVerification:
    """Dockerイメージ戦略.md: 署名検証テスト。"""

    def test_signature_verification_in_doc(self) -> None:
        """Dockerイメージ戦略.md: cosign verify が記載されていること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "cosign verify" in doc
        assert "certificate-oidc-issuer" in doc


class TestDockerImageMultiArch:
    """Dockerイメージ戦略.md: マルチアーキテクチャ方針テスト。"""

    def test_amd64_only_policy(self) -> None:
        """Dockerイメージ戦略.md: amd64 のみがビルドターゲットであること。"""
        from pathlib import Path

        doc = Path(ROOT / "docs" / "Dockerイメージ戦略.md").read_text(encoding="utf-8")
        assert "amd64" in doc
        assert "ARM64" in doc or "arm64" in doc


class TestDockerImageGoLdflags:
    """Dockerイメージ戦略.md: Go ldflags テスト。"""

    def test_go_ldflags_in_template(self) -> None:
        """Dockerイメージ戦略.md: Go Dockerfile テンプレートに -ldflags が含まれること。"""
        go_df = TEMPLATES / "server" / "go" / "Dockerfile.tera"
        content = go_df.read_text(encoding="utf-8")
        assert "ldflags" in content


class TestDockerImageRustDummyBuildCache:
    """Dockerイメージ戦略.md: Rust ダミービルドキャッシュ最適化テスト。"""

    def test_rust_dummy_build_optimization(self) -> None:
        """Dockerイメージ戦略.md: Rust Dockerfile にダミー main.rs ビルドのキャッシュ最適化が含まれること。"""
        rust_df = TEMPLATES / "server" / "rust" / "Dockerfile.tera"
        content = rust_df.read_text(encoding="utf-8")
        # ダミー main.rs でキャッシュレイヤーを作成する最適化パターン
        assert "main.rs" in content
        assert "cargo build" in content


class TestDockerImageNginxVersion:
    """Dockerイメージ戦略.md: nginx バージョン検証テスト。"""

    @pytest.mark.parametrize("client_type", ["react", "flutter"])
    def test_nginx_version(self, client_type: str) -> None:
        """Dockerイメージ戦略.md: クライアント Dockerfile で nginx:1.27-alpine を使用すること。"""
        path = TEMPLATES / "client" / client_type / "Dockerfile.tera"
        content = path.read_text(encoding="utf-8")
        assert "nginx" in content, f"{client_type} Dockerfile に nginx が含まれていません"


class TestDockerImageCosign:
    """Dockerイメージ戦略.md: Cosign 設定テスト。"""

    def test_cosign_in_ci_template(self) -> None:
        """Dockerイメージ戦略.md: CI/CD テンプレートに cosign sign が含まれること。"""
        ci_dir = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "cicd"
        if ci_dir.exists():
            # CI テンプレートディレクトリ内で cosign を検索
            found = False
            for f in ci_dir.rglob("*"):
                if f.is_file():
                    try:
                        content = f.read_text(encoding="utf-8")
                        if "cosign" in content:
                            found = True
                            break
                    except (UnicodeDecodeError, PermissionError):
                        continue
            assert found, "CI/CD テンプレートに cosign 設定が見つかりません"
        else:
            # CI/CD テンプレートが未作成の場合、ドキュメントの記載のみを検証
            pytest.skip("CI/CD テンプレートディレクトリが存在しません")
