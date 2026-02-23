"""InMemoryFeatureFlagClient 実装"""

from __future__ import annotations

from .exceptions import FeatureFlagError, FeatureFlagErrorCodes
from .models import EvaluationContext, EvaluationResult, FeatureFlag


class InMemoryFeatureFlagClient:
    """テスト用インメモリフィーチャーフラグクライアント。"""

    def __init__(self) -> None:
        self._flags: dict[str, FeatureFlag] = {}

    def set_flag(self, flag: FeatureFlag) -> None:
        """フラグを設定する。"""
        self._flags[flag.flag_key] = flag

    async def evaluate(
        self, flag_key: str, context: EvaluationContext
    ) -> EvaluationResult:
        flag = self._flags.get(flag_key)
        if flag is None:
            raise FeatureFlagError(
                FeatureFlagErrorCodes.FLAG_NOT_FOUND,
                f"フラグが見つかりません: {flag_key}",
            )
        return EvaluationResult(
            flag_key=flag_key,
            enabled=flag.enabled,
            variant=flag.variants[0].name if flag.variants else None,
            reason="FLAG_ENABLED" if flag.enabled else "FLAG_DISABLED",
        )

    async def get_flag(self, flag_key: str) -> FeatureFlag:
        flag = self._flags.get(flag_key)
        if flag is None:
            raise FeatureFlagError(
                FeatureFlagErrorCodes.FLAG_NOT_FOUND,
                f"フラグが見つかりません: {flag_key}",
            )
        return flag

    async def is_enabled(self, flag_key: str, context: EvaluationContext) -> bool:
        result = await self.evaluate(flag_key, context)
        return result.enabled
