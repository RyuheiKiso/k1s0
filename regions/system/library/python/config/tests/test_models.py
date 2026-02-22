"""pydantic モデルのユニットテスト"""

import pytest
from k1s0_config.models import (
    AppConfig,
    AppSection,
    DatabaseSection,
    ServerSection,
)
from pydantic import ValidationError


def test_app_section_required_fields() -> None:
    """AppSection の必須フィールド確認。"""
    section = AppSection(name="my-service")
    assert section.name == "my-service"
    assert section.version == "0.1.0"
    assert section.environment == "development"


def test_app_section_missing_name() -> None:
    """name なしで ValidationError が発生すること。"""
    with pytest.raises(ValidationError):
        AppSection.model_validate({})  # type: ignore[call-arg]


def test_server_section_defaults() -> None:
    """ServerSection のデフォルト値確認。"""
    section = ServerSection()
    assert section.host == "0.0.0.0"
    assert section.port == 8080
    assert section.read_timeout == 30


def test_server_section_invalid_port() -> None:
    """不正ポートで ValidationError が発生すること。"""
    with pytest.raises(ValidationError):
        ServerSection(port=0)
    with pytest.raises(ValidationError):
        ServerSection(port=65536)


def test_database_section_required_fields() -> None:
    """DatabaseSection の必須フィールド確認。"""
    section = DatabaseSection(name="mydb", user="admin")
    assert section.name == "mydb"
    assert section.user == "admin"
    assert section.host == "localhost"


def test_app_config_minimal() -> None:
    """最小構成の AppConfig 作成。"""
    config = AppConfig(app=AppSection(name="test"))
    assert config.app.name == "test"
    assert config.database is None
    assert config.kafka is None


def test_app_config_full() -> None:
    """フル構成の AppConfig 作成。"""
    data = {
        "app": {"name": "full-service", "version": "1.0.0"},
        "server": {"port": 9090},
        "database": {"name": "testdb", "user": "postgres"},
    }
    config = AppConfig.model_validate(data)
    assert config.app.name == "full-service"
    assert config.server.port == 9090
    assert config.database is not None
    assert config.database.name == "testdb"
