"""featureflag ライブラリのユニットテスト"""

import pytest
from k1s0_featureflag import (
    EvaluationContext,
    EvaluationResult,
    FeatureFlag,
    FeatureFlagError,
    FeatureFlagErrorCodes,
    FlagVariant,
    InMemoryFeatureFlagClient,
)


def make_flag(key: str, enabled: bool, variants: list[FlagVariant] | None = None) -> FeatureFlag:
    return FeatureFlag(
        id=f"id-{key}",
        flag_key=key,
        description=f"Description for {key}",
        enabled=enabled,
        variants=variants or [],
    )


async def test_evaluate_enabled_flag() -> None:
    """有効フラグの評価。"""
    client = InMemoryFeatureFlagClient()
    client.set_flag(make_flag("feature-a", True))
    result = await client.evaluate("feature-a", EvaluationContext())
    assert result.flag_key == "feature-a"
    assert result.enabled is True
    assert result.reason == "FLAG_ENABLED"


async def test_evaluate_disabled_flag() -> None:
    """無効フラグの評価。"""
    client = InMemoryFeatureFlagClient()
    client.set_flag(make_flag("feature-b", False))
    result = await client.evaluate("feature-b", EvaluationContext())
    assert result.enabled is False
    assert result.reason == "FLAG_DISABLED"


async def test_evaluate_nonexistent_flag_returns_error() -> None:
    """存在しないフラグはエラー。"""
    client = InMemoryFeatureFlagClient()
    with pytest.raises(FeatureFlagError) as exc_info:
        await client.evaluate("no-such-flag", EvaluationContext())
    assert exc_info.value.code == FeatureFlagErrorCodes.FLAG_NOT_FOUND


async def test_get_flag_by_key() -> None:
    """フラグ情報取得。"""
    client = InMemoryFeatureFlagClient()
    client.set_flag(make_flag("feature-c", True, [FlagVariant("control", "off", 50)]))
    flag = await client.get_flag("feature-c")
    assert flag.flag_key == "feature-c"
    assert flag.enabled is True
    assert len(flag.variants) == 1


async def test_is_enabled_true() -> None:
    """is_enabled = True。"""
    client = InMemoryFeatureFlagClient()
    client.set_flag(make_flag("on-flag", True))
    assert await client.is_enabled("on-flag", EvaluationContext()) is True


async def test_is_enabled_false() -> None:
    """is_enabled = False。"""
    client = InMemoryFeatureFlagClient()
    client.set_flag(make_flag("off-flag", False))
    assert await client.is_enabled("off-flag", EvaluationContext()) is False


async def test_set_flag_and_retrieve() -> None:
    """set_flag 後に get で取得。"""
    client = InMemoryFeatureFlagClient()
    with pytest.raises(FeatureFlagError):
        await client.get_flag("dynamic")
    client.set_flag(make_flag("dynamic", True))
    flag = await client.get_flag("dynamic")
    assert flag.flag_key == "dynamic"


async def test_variant_in_evaluation_result() -> None:
    """バリアントが結果に含まれること。"""
    client = InMemoryFeatureFlagClient()
    variants = [FlagVariant("control", "off", 50), FlagVariant("treatment", "on", 50)]
    client.set_flag(make_flag("ab-test", True, variants))
    result = await client.evaluate("ab-test", EvaluationContext())
    assert result.variant == "control"


def test_evaluation_context_defaults() -> None:
    """EvaluationContext のデフォルト値。"""
    ctx = EvaluationContext()
    assert ctx.user_id is None
    assert ctx.tenant_id is None
    assert ctx.attributes == {}


def test_evaluation_context_with_values() -> None:
    """EvaluationContext の値設定。"""
    ctx = EvaluationContext(user_id="u1", tenant_id="t1", attributes={"role": "admin"})
    assert ctx.user_id == "u1"
    assert ctx.tenant_id == "t1"
    assert ctx.attributes["role"] == "admin"
