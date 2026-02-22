"""Schema Registry モデルのユニットテスト"""

from k1s0_schema_registry.models import (
    CompatibilityMode,
    RegisteredSchema,
    SchemaRegistryConfig,
    SchemaType,
)


def test_schema_type_values() -> None:
    """SchemaType の値が正しいこと。"""
    assert SchemaType.AVRO.value == "AVRO"
    assert SchemaType.JSON.value == "JSON"
    assert SchemaType.PROTOBUF.value == "PROTOBUF"


def test_compatibility_mode_values() -> None:
    """CompatibilityMode の値が正しいこと。"""
    assert CompatibilityMode.BACKWARD.value == "BACKWARD"
    assert CompatibilityMode.FORWARD.value == "FORWARD"
    assert CompatibilityMode.FULL.value == "FULL"
    assert CompatibilityMode.NONE.value == "NONE"


def test_registered_schema_defaults() -> None:
    """RegisteredSchema のデフォルト値確認。"""
    schema = RegisteredSchema(id=1, subject="test-value", version=1, schema='{"type":"string"}')
    assert schema.schema_type == SchemaType.AVRO


def test_schema_registry_config_defaults() -> None:
    """SchemaRegistryConfig のデフォルト値確認。"""
    config = SchemaRegistryConfig(url="http://localhost:8081")
    assert config.username == ""
    assert config.compatibility_mode == CompatibilityMode.BACKWARD
    assert config.timeout_seconds == 10.0
