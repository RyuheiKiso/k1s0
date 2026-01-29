"""Secret file loader using the _file suffix convention."""

from __future__ import annotations

from pathlib import Path


def resolve_secrets(data: dict[str, object], secrets_dir: Path) -> dict[str, object]:
    """Resolve secret references in configuration data.

    Any key ending with '_file' is treated as a path reference to a secret.
    The value is read from the file and stored under the key without the '_file' suffix.
    The original '_file' key is removed.

    Args:
        data: Configuration dictionary to process.
        secrets_dir: Base directory for resolving relative secret file paths.

    Returns:
        A new dictionary with secrets resolved.
    """
    result: dict[str, object] = {}
    for key, value in data.items():
        if isinstance(value, dict):
            result[key] = resolve_secrets(value, secrets_dir)  # type: ignore[arg-type]
        elif isinstance(key, str) and key.endswith("_file") and isinstance(value, str):
            secret_path = Path(value)
            if not secret_path.is_absolute():
                secret_path = secrets_dir / secret_path
            if secret_path.exists():
                resolved_key = key.removesuffix("_file")
                result[resolved_key] = secret_path.read_text().strip()
            else:
                result[key] = value
        else:
            result[key] = value
    return result
