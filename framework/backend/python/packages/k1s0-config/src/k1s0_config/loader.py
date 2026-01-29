"""Configuration loader that merges default.yaml with environment-specific overlays."""

from __future__ import annotations

from pathlib import Path

import yaml

from k1s0_config.config import K1s0Config
from k1s0_config.secrets import resolve_secrets


def _deep_merge(base: dict[str, object], overlay: dict[str, object]) -> dict[str, object]:
    """Recursively merge overlay into base, returning a new dict."""
    result = dict(base)
    for key, value in overlay.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = _deep_merge(result[key], value)  # type: ignore[arg-type]
        else:
            result[key] = value
    return result


def load_config(
    env: str = "default",
    config_dir: str | Path = "config",
    secrets_dir: str | Path | None = None,
) -> K1s0Config:
    """Load configuration from YAML files.

    Loads config/default.yaml as the base, then overlays config/{env}.yaml
    on top. If secrets_dir is provided, resolves any keys ending with '_file'
    by reading the referenced file content.

    Args:
        env: Environment name (default, dev, stg, prod).
        config_dir: Path to the config directory.
        secrets_dir: Optional path to secrets directory for _file suffix resolution.

    Returns:
        A K1s0Config instance with the merged configuration.

    Raises:
        FileNotFoundError: If default.yaml does not exist.
    """
    config_path = Path(config_dir)

    default_file = config_path / "default.yaml"
    if not default_file.exists():
        msg = f"Configuration file not found: {default_file}"
        raise FileNotFoundError(msg)

    with open(default_file) as f:
        data: dict[str, object] = yaml.safe_load(f) or {}

    if env != "default":
        env_file = config_path / f"{env}.yaml"
        if env_file.exists():
            with open(env_file) as f:
                env_data: dict[str, object] = yaml.safe_load(f) or {}
            data = _deep_merge(data, env_data)

    if secrets_dir is not None:
        data = resolve_secrets(data, Path(secrets_dir))

    return K1s0Config(data)
