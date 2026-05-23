"""src/_meta/axis_registry/registry_loader.py

axis_registry/registry.yaml を読み込み、型付き dataclass に変換する。
generate_axis_registry.py から import して利用される。
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required: pip install PyYAML")

REGISTRY_PATH = Path(__file__).parent / "registry.yaml"

VALID_KINDS: frozenset[str] = frozenset(["primary_layer", "cross_cutting_cluster", "meta"])
VALID_PROOF_TOOLS: frozenset[str] = frozenset([
    "tla_apalache", "lean4", "kani", "stainless", "dafny",
    "proptest", "hypothesis", "fast_check", "cbmc",
])
VALID_PROOF_CLASSES: frozenset[str] = frozenset([
    "v1_temporal_safety_proof",
    "v1_property_axiom",
    "v1_type_invariant",
    "v1_contract_pair",
    "v1_scenario_replay",
    "v1_fault_chaos",
    "v1_program_correctness_proof",
])


@dataclass
class ProofObligation:
    obligation_id: str
    proof_class: str
    tool_kind: str

    def validate(self) -> list[str]:
        errors: list[str] = []
        if not self.obligation_id:
            errors.append("obligation_id must not be empty")
        if self.proof_class not in VALID_PROOF_CLASSES:
            errors.append(
                f"unknown proof_class: {self.proof_class!r}. "
                f"valid: {sorted(VALID_PROOF_CLASSES)}"
            )
        if self.tool_kind not in VALID_PROOF_TOOLS:
            errors.append(
                f"unknown tool_kind: {self.tool_kind!r}. "
                f"valid: {sorted(VALID_PROOF_TOOLS)}"
            )
        return errors


@dataclass
class AxisEntry:
    axis_id: str
    axis_name: str
    kind: str
    cap_slot: int
    legacy_id: list[str] = field(default_factory=list)
    conformance_specs: list[str] = field(default_factory=list)
    enforcement_spec: str = ""
    lock_artifacts: list[str] = field(default_factory=list)
    defense_in_depth_layers: list[str] = field(default_factory=list)
    proof_obligations: list[ProofObligation] = field(default_factory=list)

    def validate(self) -> list[str]:
        errors: list[str] = []
        if not self.axis_id:
            errors.append("axis_id must not be empty")
        if not self.axis_name:
            errors.append(f"axis {self.axis_id}: axis_name must not be empty")
        if self.kind not in VALID_KINDS:
            errors.append(
                f"axis {self.axis_id}: unknown kind: {self.kind!r}. "
                f"valid: {sorted(VALID_KINDS)}"
            )
        if not (1 <= self.cap_slot <= 20):
            errors.append(
                f"axis {self.axis_id}: cap_slot {self.cap_slot} out of range [1, 20]"
            )
        if not self.conformance_specs:
            errors.append(f"axis {self.axis_id}: conformance_specs must have at least 1 entry")
        if not self.lock_artifacts:
            errors.append(f"axis {self.axis_id}: lock_artifacts must have at least 1 entry")
        for ob in self.proof_obligations:
            errors.extend(ob.validate())
        return errors


@dataclass
class AxisRegistry:
    axes: list[AxisEntry]
    cap_slot_max: int
    cap_slot_reserved: list[dict[str, Any]] = field(default_factory=list)

    @property
    def axes_count(self) -> int:
        return len(self.axes)

    @property
    def remaining_slots(self) -> int:
        used = len(self.axes)
        reserved = len(self.cap_slot_reserved)
        return self.cap_slot_max - used - reserved + len(self.cap_slot_reserved)

    def validate(self) -> list[str]:
        errors: list[str] = []
        for ax in self.axes:
            errors.extend(ax.validate())
        return errors


def _parse_proof_obligation(raw: dict[str, Any]) -> ProofObligation:
    return ProofObligation(
        obligation_id=str(raw.get("obligation_id", "")),
        proof_class=str(raw.get("proof_class", "")),
        tool_kind=str(raw.get("tool_kind", "")),
    )


def _parse_axis_entry(raw: dict[str, Any]) -> AxisEntry:
    proof_obligations = [
        _parse_proof_obligation(ob)
        for ob in (raw.get("proof_obligations") or [])
    ]
    return AxisEntry(
        axis_id=str(raw.get("axis_id", "")),
        axis_name=str(raw.get("axis_name", "")),
        kind=str(raw.get("kind", "")),
        cap_slot=int(raw.get("cap_slot", 0)),
        legacy_id=[str(x) for x in (raw.get("legacy_id") or [])],
        conformance_specs=[str(x) for x in (raw.get("conformance_specs") or [])],
        enforcement_spec=str(raw.get("enforcement_spec", "")),
        lock_artifacts=[str(x) for x in (raw.get("lock_artifacts") or [])],
        defense_in_depth_layers=[str(x) for x in (raw.get("defense_in_depth_layers") or [])],
        proof_obligations=proof_obligations,
    )


def load_registry(registry_path: Path = REGISTRY_PATH) -> AxisRegistry:
    """registry.yaml を読み込んで AxisRegistry を返す。"""
    if not registry_path.exists():
        raise FileNotFoundError(f"registry.yaml not found: {registry_path}")
    raw = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
    axes_raw = raw.get("axes") or []
    axes = [_parse_axis_entry(ax) for ax in axes_raw]
    return AxisRegistry(
        axes=axes,
        cap_slot_max=int(raw.get("cap_slot_max", 20)),
        cap_slot_reserved=list(raw.get("cap_slot_reserved") or []),
    )
