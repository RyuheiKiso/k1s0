"""k1s0 config library."""

from .exceptions import ConfigError, ConfigErrorCodes
from .loader import load
from .merger import deep_merge
from .models import (
    AppConfig,
    AppSection,
    AuthSection,
    DatabaseSection,
    GrpcSection,
    KafkaSection,
    ObservabilitySection,
    RedisSection,
    ServerSection,
)
from .vault import merge_vault_secrets

__all__ = [
    "AppSection",
    "ServerSection",
    "GrpcSection",
    "DatabaseSection",
    "KafkaSection",
    "RedisSection",
    "ObservabilitySection",
    "AuthSection",
    "AppConfig",
    "load",
    "deep_merge",
    "merge_vault_secrets",
    "ConfigError",
    "ConfigErrorCodes",
]
