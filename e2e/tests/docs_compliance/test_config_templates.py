"""環境別 config テンプレートの存在と内容検証テスト。

Server (Go/Rust) と BFF (Go/Rust) の dev/staging/prod 環境別
config テンプレートが正しく存在し、内容が適切かを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"

# Server templates
SERVER_GO_CONFIG = TEMPLATES / "server" / "go" / "config"
SERVER_RUST_CONFIG = TEMPLATES / "server" / "rust" / "config"

# BFF templates
BFF_GO_CONFIG = TEMPLATES / "bff" / "go" / "config"
BFF_RUST_CONFIG = TEMPLATES / "bff" / "rust" / "config"

ENVS = ["dev", "staging", "prod"]
KINDS = [
    ("server/go", SERVER_GO_CONFIG),
    ("server/rust", SERVER_RUST_CONFIG),
    ("bff/go", BFF_GO_CONFIG),
    ("bff/rust", BFF_RUST_CONFIG),
]


class TestConfigTemplateFilesExist:
    """環境別 config テンプレートファイルの存在確認。"""

    @pytest.mark.parametrize("kind,config_dir", KINDS)
    @pytest.mark.parametrize("env", ENVS)
    def test_env_config_exists(self, kind: str, config_dir: Path, env: str) -> None:
        path = config_dir / f"config.{env}.yaml.tera"
        assert path.exists(), f"{kind}/config/config.{env}.yaml.tera が存在しません"


class TestConfigTemplateEnvironmentField:
    """環境別 config テンプレートに正しい environment 値が含まれているかの検証。"""

    @pytest.mark.parametrize("kind,config_dir", KINDS)
    @pytest.mark.parametrize("env", ENVS)
    def test_has_environment_field(self, kind: str, config_dir: Path, env: str) -> None:
        path = config_dir / f"config.{env}.yaml.tera"
        content = path.read_text(encoding="utf-8")
        assert f'environment: "{env}"' in content, (
            f"{kind}/config.{env}.yaml.tera に environment: \"{env}\" が含まれていません"
        )


class TestConfigTemplateServiceName:
    """環境別 config テンプレートに service_name 変数が含まれているかの検証。"""

    @pytest.mark.parametrize("kind,config_dir", KINDS)
    @pytest.mark.parametrize("env", ENVS)
    def test_has_service_name_variable(self, kind: str, config_dir: Path, env: str) -> None:
        path = config_dir / f"config.{env}.yaml.tera"
        content = path.read_text(encoding="utf-8")
        assert "{{ service_name }}" in content, (
            f"{kind}/config.{env}.yaml.tera に {{{{ service_name }}}} が含まれていません"
        )


class TestConfigTemplateLogLevel:
    """環境別 config テンプレートのログレベルが環境に応じて適切かの検証。"""

    EXPECTED_LOG_LEVELS = {
        "dev": "debug",
        "staging": "info",
        "prod": "warn",
    }

    @pytest.mark.parametrize("kind,config_dir", KINDS)
    @pytest.mark.parametrize("env", ENVS)
    def test_log_level(self, kind: str, config_dir: Path, env: str) -> None:
        path = config_dir / f"config.{env}.yaml.tera"
        content = path.read_text(encoding="utf-8")
        expected = self.EXPECTED_LOG_LEVELS[env]
        assert f'level: "{expected}"' in content, (
            f"{kind}/config.{env}.yaml.tera のログレベルが '{expected}' ではありません"
        )


class TestConfigTemplateBaseConfig:
    """ベース config.yaml.tera の存在確認。"""

    @pytest.mark.parametrize("kind,config_dir", KINDS)
    def test_base_config_exists(self, kind: str, config_dir: Path) -> None:
        path = config_dir / "config.yaml.tera"
        assert path.exists(), f"{kind}/config/config.yaml.tera が存在しません"
