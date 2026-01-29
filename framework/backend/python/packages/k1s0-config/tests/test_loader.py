"""Tests for configuration loading."""

from __future__ import annotations

from pathlib import Path

import pytest

from k1s0_config.loader import load_config


@pytest.fixture()
def config_dir(tmp_path: Path) -> Path:
    """Create a temporary config directory with sample files."""
    cfg = tmp_path / "config"
    cfg.mkdir()
    (cfg / "default.yaml").write_text(
        "server:\n  host: localhost\n  port: 8080\ndatabase:\n  host: db\n  port: 5432\n"
    )
    (cfg / "dev.yaml").write_text("server:\n  port: 3000\n")
    return cfg


@pytest.fixture()
def secrets_dir(tmp_path: Path) -> Path:
    """Create a temporary secrets directory."""
    s = tmp_path / "secrets"
    s.mkdir()
    (s / "db_password").write_text("s3cret\n")
    return s


class TestLoadConfig:
    def test_load_default(self, config_dir: Path) -> None:
        cfg = load_config(config_dir=config_dir)
        assert cfg.get_str("server.host") == "localhost"
        assert cfg.get_int("server.port") == 8080

    def test_env_overlay(self, config_dir: Path) -> None:
        cfg = load_config(env="dev", config_dir=config_dir)
        assert cfg.get_int("server.port") == 3000
        assert cfg.get_str("server.host") == "localhost"  # from default

    def test_missing_default_raises(self, tmp_path: Path) -> None:
        with pytest.raises(FileNotFoundError):
            load_config(config_dir=tmp_path / "nonexistent")

    def test_secrets_resolution(self, config_dir: Path, secrets_dir: Path) -> None:
        (config_dir / "default.yaml").write_text(
            "database:\n  host: db\n  password_file: db_password\n"
        )
        cfg = load_config(config_dir=config_dir, secrets_dir=secrets_dir)
        assert cfg.get_str("database.password") == "s3cret"

    def test_get_section(self, config_dir: Path) -> None:
        cfg = load_config(config_dir=config_dir)
        db = cfg.get_section("database")
        assert db.get_str("host") == "db"

    def test_get_missing_key_returns_default(self, config_dir: Path) -> None:
        cfg = load_config(config_dir=config_dir)
        assert cfg.get("nonexistent", "fallback") == "fallback"
        assert cfg.get_int("nonexistent", 42) == 42
        assert cfg.get_bool("nonexistent", True) is True
