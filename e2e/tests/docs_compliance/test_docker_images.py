"""Dockerイメージ戦略.md の仕様準拠テスト。

Dockerfile テンプレートの内容がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"


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
