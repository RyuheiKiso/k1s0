"""src/_meta/axis_registry/cap_enforcer.py

axis_registry の cap 制約を物理的に強制する。

制約一覧:
- cap_slot_max == 20 (v1 固定)
- axes_count == 19 (v1 での登録数)
- cap_slot 重複なし
- cap_slot の空き (remaining) == 1
- axis_id 重複なし
- axis_name 重複なし
- kind ごとの制約: primary_layer は 1-10、cross_cutting_cluster は 11-18、meta は 19

違反があれば CapViolation のリストを返す。空リスト = 全制約 pass。
"""

from __future__ import annotations

from dataclasses import dataclass

from .registry_loader import AxisRegistry, load_registry

V1_CAP_SLOT_MAX: int = 20
V1_AXES_COUNT: int = 19
V1_REMAINING: int = 1

KIND_SLOT_RANGES: dict[str, tuple[int, int]] = {
    "primary_layer": (1, 10),
    "cross_cutting_cluster": (11, 18),
    "meta": (19, 20),
}


@dataclass
class CapViolation:
    rule: str
    detail: str

    def __str__(self) -> str:
        return f"[CAP VIOLATION] {self.rule}: {self.detail}"


def enforce_cap(registry: AxisRegistry) -> list[CapViolation]:
    """cap 制約を全チェックして CapViolation のリストを返す。空リスト = 全 pass。"""
    violations: list[CapViolation] = []

    if registry.cap_slot_max != V1_CAP_SLOT_MAX:
        violations.append(CapViolation(
            rule="cap_slot_max",
            detail=f"expected {V1_CAP_SLOT_MAX}, got {registry.cap_slot_max}",
        ))

    if registry.axes_count != V1_AXES_COUNT:
        violations.append(CapViolation(
            rule="axes_count",
            detail=f"expected {V1_AXES_COUNT}, got {registry.axes_count}",
        ))

    # cap_slot 重複チェック
    slots_seen: dict[int, str] = {}
    for ax in registry.axes:
        if ax.cap_slot in slots_seen:
            violations.append(CapViolation(
                rule="cap_slot_unique",
                detail=(
                    f"cap_slot {ax.cap_slot} is used by both "
                    f"{slots_seen[ax.cap_slot]!r} and {ax.axis_name!r}"
                ),
            ))
        else:
            slots_seen[ax.cap_slot] = ax.axis_name

    # axis_id 重複チェック
    ids_seen: dict[str, str] = {}
    for ax in registry.axes:
        normalized = ax.axis_id.lstrip("0") or "0"
        if normalized in ids_seen:
            violations.append(CapViolation(
                rule="axis_id_unique",
                detail=f"axis_id {ax.axis_id!r} is duplicate (matches {ids_seen[normalized]!r})",
            ))
        else:
            ids_seen[normalized] = ax.axis_id

    # axis_name 重複チェック
    names_seen: set[str] = set()
    for ax in registry.axes:
        if ax.axis_name in names_seen:
            violations.append(CapViolation(
                rule="axis_name_unique",
                detail=f"axis_name {ax.axis_name!r} appears more than once",
            ))
        else:
            names_seen.add(ax.axis_name)

    # kind ごとの slot 範囲チェック
    for ax in registry.axes:
        if ax.kind in KIND_SLOT_RANGES:
            lo, hi = KIND_SLOT_RANGES[ax.kind]
            if not (lo <= ax.cap_slot <= hi):
                violations.append(CapViolation(
                    rule="kind_slot_range",
                    detail=(
                        f"axis {ax.axis_name!r} (kind={ax.kind!r}) "
                        f"has cap_slot={ax.cap_slot} outside [{lo}, {hi}]"
                    ),
                ))

    # reserved slots が実装軸に重複していないか
    reserved_slots: set[int] = {
        int(r.get("slot", 0)) for r in registry.cap_slot_reserved
    }
    for ax in registry.axes:
        if ax.cap_slot in reserved_slots:
            violations.append(CapViolation(
                rule="reserved_slot_collision",
                detail=f"axis {ax.axis_name!r} occupies reserved cap_slot {ax.cap_slot}",
            ))

    # remaining スロット数チェック (reserved は remaining の内訳に含まれる: cap_slot_max - axes_count)
    used_slots = set(slots_seen.keys())
    true_remaining = V1_CAP_SLOT_MAX - len(used_slots)
    if true_remaining != V1_REMAINING:
        violations.append(CapViolation(
            rule="remaining_slots",
            detail=f"expected {V1_REMAINING} remaining slot, got {true_remaining}",
        ))

    return violations


def enforce_cap_from_file() -> list[CapViolation]:
    """registry.yaml を読み込んで cap 制約を強制する (CLI ショートカット)。"""
    registry = load_registry()
    return enforce_cap(registry)


def main() -> int:
    violations = enforce_cap_from_file()
    if violations:
        for v in violations:
            print(v)
        return 1
    print("OK: all cap constraints satisfied (19 axes / cap=20 / remaining=1)")
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
