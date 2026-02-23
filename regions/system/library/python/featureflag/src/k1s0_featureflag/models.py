"""featureflag データモデル"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class FlagVariant:
    """フラグバリアント。"""

    name: str
    value: str
    weight: int = 0


@dataclass
class FeatureFlag:
    """フィーチャーフラグ。"""

    id: str
    flag_key: str
    description: str = ""
    enabled: bool = False
    variants: list[FlagVariant] = field(default_factory=list)


@dataclass
class EvaluationContext:
    """フラグ評価コンテキスト。"""

    user_id: str | None = None
    tenant_id: str | None = None
    attributes: dict[str, str] = field(default_factory=dict)


@dataclass
class EvaluationResult:
    """フラグ評価結果。"""

    flag_key: str
    enabled: bool
    variant: str | None = None
    reason: str = ""
