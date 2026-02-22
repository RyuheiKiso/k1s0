"""設定ファイル読み込み"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml
from pydantic import ValidationError

from .exceptions import ConfigError, ConfigErrorCodes
from .merger import deep_merge
from .models import AppConfig


def _read_yaml(path: Path) -> dict[str, Any]:
    """YAML ファイルを読み込む。"""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as e:
        raise ConfigError(
            code=ConfigErrorCodes.READ_FILE,
            message=f"Failed to read config file: {path}",
            cause=e,
        ) from e
    try:
        data: dict[str, Any] = yaml.safe_load(text) or {}
    except yaml.YAMLError as e:
        raise ConfigError(
            code=ConfigErrorCodes.PARSE_YAML,
            message=f"Failed to parse YAML: {path}",
            cause=e,
        ) from e
    return data


def load(base_path: Path, env_path: Path | None = None) -> AppConfig:
    """設定ファイルを読み込んで AppConfig を返す。

    base_path: ベース設定ファイルパス（必須）
    env_path: 環境別設定ファイルパス（オプション）。存在する場合はベースにマージ。
    """
    data = _read_yaml(base_path)
    if env_path is not None and env_path.exists():
        env_data = _read_yaml(env_path)
        data = deep_merge(data, env_data)
    try:
        return AppConfig.model_validate(data)
    except ValidationError as e:
        raise ConfigError(
            code=ConfigErrorCodes.VALIDATION,
            message=f"Config validation failed: {e}",
            cause=e,
        ) from e
