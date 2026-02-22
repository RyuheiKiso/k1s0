"""k1s0 schema_registry library."""

from .client import SchemaRegistryClient
from .exceptions import SchemaRegistryError, SchemaRegistryErrorCodes
from .http_client import HttpSchemaRegistryClient
from .models import (
    CompatibilityMode,
    RegisteredSchema,
    SchemaRegistryConfig,
    SchemaType,
)

__all__ = [
    "SchemaRegistryClient",
    "HttpSchemaRegistryClient",
    "SchemaType",
    "CompatibilityMode",
    "RegisteredSchema",
    "SchemaRegistryConfig",
    "SchemaRegistryError",
    "SchemaRegistryErrorCodes",
]
