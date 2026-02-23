"""k1s0 featureflag library."""

from .client import FeatureFlagClientProtocol
from .exceptions import FeatureFlagError, FeatureFlagErrorCodes
from .memory import InMemoryFeatureFlagClient
from .models import EvaluationContext, EvaluationResult, FeatureFlag, FlagVariant

__all__ = [
    "EvaluationContext",
    "EvaluationResult",
    "FeatureFlag",
    "FeatureFlagClientProtocol",
    "FeatureFlagError",
    "FeatureFlagErrorCodes",
    "FlagVariant",
    "InMemoryFeatureFlagClient",
]
