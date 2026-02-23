"""FeatureFlagClient プロトコル"""

from __future__ import annotations

from typing import Protocol

from .models import EvaluationContext, EvaluationResult, FeatureFlag


class FeatureFlagClientProtocol(Protocol):
    """フィーチャーフラグクライアントプロトコル。"""

    async def evaluate(
        self, flag_key: str, context: EvaluationContext
    ) -> EvaluationResult: ...

    async def get_flag(self, flag_key: str) -> FeatureFlag: ...

    async def is_enabled(self, flag_key: str, context: EvaluationContext) -> bool: ...
