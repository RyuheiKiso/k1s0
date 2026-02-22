"""Vault シークレットのマージ"""

from __future__ import annotations

from .models import AppConfig


def merge_vault_secrets(config: AppConfig, secrets: dict[str, str]) -> AppConfig:
    """Vault から取得したシークレットを設定にマージして新しい AppConfig を返す。

    secrets のキーは設定パス（ドット区切り）。
    例: {"database.password": "secret123", "kafka.sasl.password": "kafka-secret"}
    """
    data = config.model_dump()
    for key_path, value in secrets.items():
        parts = key_path.split(".")
        node: dict = data  # type: ignore[type-arg]
        for part in parts[:-1]:
            if part not in node or node[part] is None:
                node[part] = {}
            node = node[part]
        node[parts[-1]] = value
    return AppConfig.model_validate(data)
