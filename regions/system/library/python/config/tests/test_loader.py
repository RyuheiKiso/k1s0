"""設定ローダーのユニットテスト"""

from pathlib import Path

import pytest
from k1s0_config.exceptions import ConfigError, ConfigErrorCodes
from k1s0_config.loader import load


def test_load_minimal_config(tmp_path: Path) -> None:
    """最小設定ファイルの読み込み。"""
    config_file = tmp_path / "config.yaml"
    config_file.write_text("app:\n  name: test-service\n")
    config = load(config_file)
    assert config.app.name == "test-service"


def test_load_with_env_override(tmp_path: Path) -> None:
    """環境別設定のマージ確認。"""
    base_file = tmp_path / "base.yaml"
    base_file.write_text("app:\n  name: base\nserver:\n  port: 8080\n")
    env_file = tmp_path / "prod.yaml"
    env_file.write_text("server:\n  port: 9090\n")
    config = load(base_file, env_file)
    assert config.app.name == "base"
    assert config.server.port == 9090


def test_load_env_not_exists(tmp_path: Path) -> None:
    """env_path が存在しない場合は base のみ使用。"""
    base_file = tmp_path / "base.yaml"
    base_file.write_text("app:\n  name: fallback\n")
    config = load(base_file, tmp_path / "nonexistent.yaml")
    assert config.app.name == "fallback"


def test_load_file_not_found(tmp_path: Path) -> None:
    """存在しないファイルで ConfigError(READ_FILE_ERROR) が発生すること。"""
    with pytest.raises(ConfigError) as exc_info:
        load(tmp_path / "missing.yaml")
    assert exc_info.value.code == ConfigErrorCodes.READ_FILE


def test_load_invalid_yaml(tmp_path: Path) -> None:
    """不正 YAML で ConfigError(PARSE_YAML_ERROR) が発生すること。"""
    bad_file = tmp_path / "bad.yaml"
    bad_file.write_text("app: {invalid: yaml: content:\n")
    with pytest.raises(ConfigError) as exc_info:
        load(bad_file)
    assert exc_info.value.code == ConfigErrorCodes.PARSE_YAML


def test_load_validation_error(tmp_path: Path) -> None:
    """バリデーション失敗で ConfigError(VALIDATION_ERROR) が発生すること。"""
    bad_config = tmp_path / "bad_config.yaml"
    bad_config.write_text("server:\n  port: 99999\n")
    with pytest.raises(ConfigError) as exc_info:
        load(bad_config)
    assert exc_info.value.code == ConfigErrorCodes.VALIDATION
