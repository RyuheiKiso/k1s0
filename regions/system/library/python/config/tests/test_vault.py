"""Vault シークレットマージのユニットテスト"""

from k1s0_config.models import AppConfig, AppSection, DatabaseSection
from k1s0_config.vault import merge_vault_secrets


def test_merge_database_password() -> None:
    """database.password がシークレットでマージされること。"""
    config = AppConfig(
        app=AppSection(name="test"),
        database=DatabaseSection(name="mydb", user="admin"),
    )
    result = merge_vault_secrets(config, {"database.password": "secret123"})
    assert result.database is not None
    assert result.database.password == "secret123"


def test_merge_does_not_mutate_original() -> None:
    """元の設定が変更されないこと。"""
    config = AppConfig(
        app=AppSection(name="test"),
        database=DatabaseSection(name="mydb", user="admin"),
    )
    merge_vault_secrets(config, {"database.password": "new-secret"})
    assert config.database is not None
    assert config.database.password == ""


def test_merge_empty_secrets() -> None:
    """空のシークレット辞書でも正常動作すること。"""
    config = AppConfig(app=AppSection(name="test"))
    result = merge_vault_secrets(config, {})
    assert result.app.name == "test"
