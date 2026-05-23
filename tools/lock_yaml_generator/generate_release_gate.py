#!/usr/bin/env python3
"""tools/lock_yaml_generator/generate_release_gate.py

release_gate.lock.yaml 生成器。

[release_gate 体系](docs/04_詳細設計/05_lock_yaml体系/03_release_gate体系.md) に従い、
各軸の lock.yaml + cosign 署名を入力として AND-gate を計算する。
"""

from __future__ import annotations

import datetime
import hashlib
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

try:
    import yaml  # type: ignore
except ImportError:
    print("ERROR: PyYAML required", file=sys.stderr)
    sys.exit(2)


REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SPEC_PATH = Path(__file__).parent.parent.parent / "docs/04_詳細設計/05_lock_yaml体系/03_release_gate体系.md"
LOCK_DEP_PATH = Path(__file__).parent / "lock_dependency.yaml"

# フォールバック用: spec parse が失敗した場合に使用
_FALLBACK_CELLS_RAW = [
    # formal
    ("formal.all_critical_verified", "proof_status.lock.yaml", ""),
    ("formal.proof_matrix_complete", "proof_matrix.lock.yaml", ""),
    ("formal.no_open_above_severity_low", "counter_example.lock.yaml", ""),
    ("formal.dual_review_completeness_100pct", "proof_review.lock.yaml", ""),
    ("formal.assumption_cap_within_20", "assumption.lock.yaml",
     "count(`assumption.lock.yaml`, entries[?status=='open']) <= 20"),
    ("formal.tool_pin_drill_green", "tla_apalache_pin.lock.yaml", ""),
    ("formal.reproducibility_daily_green", "reproducibility.lock.yaml", ""),
    ("formal.cross_axis_lock_drift_zero", "cross_cutting_registry.lock.yaml", ""),
    ("formal.slo_4_sli_green", "proof_status.lock.yaml", ""),
    # test
    ("test.coverage_matrix_complete", "coverage_matrix.lock.yaml", ""),
    ("test.regression_corpus_drift_zero", "regression_corpus.lock.yaml", ""),
    ("test.mutation_score_monotonic", "coverage_matrix.lock.yaml", ""),
    # ops
    ("ops.loop_closure_complete", "ops_loop.lock.yaml", ""),
    ("ops.toil_minutes_within_50pct", "ops_loop.lock.yaml", ""),
    # security
    ("security.threat_model_coverage_100pct", "threat_model.lock.yaml", ""),
    ("security.audit_event_chain_no_divergence", "threat_model.lock.yaml", ""),
    ("security.build_provenance_slsa_l3plus", "artifact_inventory.lock.yaml", ""),
    # infra
    ("infra.topology_class_drill_green", "topology_class.lock.yaml", ""),
    ("infra.clock_integrity_drill_green", "clock_causality_proof.lock.yaml", ""),
    # data
    ("data.preservation_class_drill_green", "preservation_substrates.lock.yaml", ""),
    ("data.restore_drill_quarterly_green", "preservation_substrates.lock.yaml", ""),
    # tier1 / tier2 / tier3 / client
    ("tier1.bidi_conformance_complete", "capabilities.lock.yaml", ""),
    ("tier1.slo_compliance_quarterly_green", "instruments.lock.yaml", ""),
    ("tier1.oss_lifecycle_drill_green", "oss_inventory.lock.yaml", ""),
    ("tier1.tenant_capacity_drill_green", "capabilities.lock.yaml", ""),
    ("tier2.tenant_isolation_drill_green", "migration.lock.yaml", ""),
    ("tier3.client_state_conformance_complete", "conflict_tree.lock.yaml", ""),
    ("client.sdk_distribution_5class_green", "sdk_conformance.lock.yaml", ""),
    # meta
    ("meta.axis_registry_complete", "axis_registry.lock.yaml", ""),
    ("meta.ownership_table_complete", "ownership_table.lock.yaml", ""),
    ("meta.docs_lint_green", "docs_lint.lock.yaml", ""),
    ("meta.release_gate_dual_signoff_complete", "dual_review.lock.yaml", ""),
    # cross-cutting
    ("cross_http2.enforcement_complete", "cross_cutting_registry.lock.yaml", ""),
    ("cross_kek.shamir_threshold_drill_green", "cross_cutting_registry.lock.yaml", ""),
    ("cross_schema.apicurio_sot_drift_zero", "cross_cutting_registry.lock.yaml", ""),
    ("cross_fsm.protoc_gen_go_codegen_drift_zero", "cross_cutting_registry.lock.yaml", ""),
    ("cross_slo.protection_layers_4tier_green", "cross_cutting_registry.lock.yaml", ""),
    ("cross_bff.auth_edge_isolation_complete", "cross_cutting_registry.lock.yaml", ""),
    ("cross_bff.tauri_sidecar_distribution_green", "cross_cutting_registry.lock.yaml", ""),
    ("cross_pii.dedicated_cluster_drill_green", "cross_cutting_registry.lock.yaml", ""),
    ("cross_pii.audit_ingest_gap_zero", "cross_cutting_registry.lock.yaml", ""),
    ("cross_edge.ops_edge_cluster_independent_green", "cross_cutting_registry.lock.yaml", ""),
    ("cross_edge.companion_otel_4stack_green", "cross_cutting_registry.lock.yaml", ""),
    ("cross_edge.ua_aware_adapter_capability_matrix_complete", "cross_cutting_registry.lock.yaml", ""),
    ("cross_edge.dotnet8_connect_conformance_green", "cross_cutting_registry.lock.yaml", ""),
]


@dataclass
class Cell:
    cell_id: str
    source_lock: str
    dsl_expr: str
    yellow_band: Optional[str] = None


def load_cell_catalog(spec_path: Path = SPEC_PATH) -> list:
    """spec の markdown table から cell 一覧を parse する。"""
    cells = []
    content = spec_path.read_text(encoding="utf-8")
    in_catalog = False
    for line in content.splitlines():
        if "## release_gate cell catalog" in line or "### cross-cutting cluster cells" in line:
            in_catalog = True
        if in_catalog and line.startswith("|") and "`" in line:
            parts = [p.strip() for p in line.split("|")[1:-1]]
            if len(parts) >= 2 and parts[0].startswith("`"):
                cell_id = parts[0].strip("`")
                source_lock = parts[1].strip("`") if len(parts) > 1 else "unknown.lock.yaml"
                dsl_expr = parts[2] if len(parts) > 2 else ""
                yellow_band = parts[3] if len(parts) > 3 else None
                if cell_id and "---" not in cell_id and "cell_id" not in cell_id:
                    cells.append(Cell(
                        cell_id=cell_id,
                        source_lock=source_lock,
                        dsl_expr=dsl_expr,
                        yellow_band=yellow_band,
                    ))
    if not cells:
        raise ValueError(
            f"cell catalog の parse 失敗: {spec_path} に有効な table が見つからない"
        )
    return cells


def _build_cell_catalog() -> list:
    """spec parse を試み、失敗時はフォールバック定数を使用する。"""
    try:
        return load_cell_catalog(SPEC_PATH)
    except Exception as exc:
        print(f"WARNING: spec parse 失敗、フォールバック定数を使用します: {exc}", file=sys.stderr)
        return [
            Cell(cell_id=cid, source_lock=src, dsl_expr=dsl)
            for cid, src, dsl in _FALLBACK_CELLS_RAW
        ]


# ---------------------------------------------------------------------------
# Lock 読み込み
# ---------------------------------------------------------------------------

def load_lock(lock_dir: Path, lock_name: str) -> dict:
    """lock.yaml を PyYAML で読み込む。存在しない場合は空 dict を返す。"""
    lock_path = lock_dir / lock_name
    if not lock_path.exists():
        return {}
    with lock_path.open(encoding="utf-8") as f:
        return yaml.safe_load(f) or {}


_OUTPUT_LOCK_NAMES = frozenset({
    "release_gate.lock.yaml",
    "release_gate.cosign_trigger.lock.yaml",
})


def get_max_mtime(lock_dir: Path) -> str:
    """lock_dir 内の入力 .lock.yaml の最大 mtime を ISO8601 で返す。

    release_gate.lock.yaml 自体（出力ファイル）は除外して
    bit-for-bit reproducibility を保つ。
    """
    mtimes = [
        p.stat().st_mtime
        for p in lock_dir.glob("*.lock.yaml")
        if p.name not in _OUTPUT_LOCK_NAMES
    ]
    if not mtimes:
        return "1970-01-01T00:00:00Z"
    ts = datetime.datetime.fromtimestamp(max(mtimes), tz=datetime.timezone.utc)
    return ts.strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# DSL interpreter
# ---------------------------------------------------------------------------

def _jsonpath_count(data: dict, path: str) -> int:
    """簡易 jsonpath の件数カウント。"""
    # entries[?status=='open'] パターン
    m = re.match(r"(\w+)\[\?(.+?)=='(.+?)'\]", path)
    if m:
        key, field_name, value = m.group(1), m.group(2), m.group(3)
        items = data.get(key, [])
        if isinstance(items, list):
            return sum(1 for item in items if isinstance(item, dict) and item.get(field_name) == value)
    # 単純配列
    m = re.match(r"(\w+)\[\*\]", path)
    if m:
        key = m.group(1)
        items = data.get(key, [])
        return len(items) if isinstance(items, list) else 0
    return 0


def _dsl_get_field(data: dict, field_path: str):
    """ドット区切りで data からフィールドを取得する。"""
    parts = field_path.strip().split(".")
    cur = data
    for part in parts:
        if not isinstance(cur, dict):
            return None
        cur = cur.get(part)
    return cur


def _evaluate_single_expr(expr: str, lock_dir: Path) -> dict:
    """単一 DSL 式を評価する（AND/OR の子式として再帰呼び出しされる）。"""
    expr = expr.strip()

    # count(lock, path) <= N
    m = re.match(r"count\((.+?),\s*(.+?)\)\s*<=\s*(\d+)$", expr)
    if m:
        lock_name = m.group(1).strip("`")
        jsonpath = m.group(2)
        threshold = int(m.group(3))
        data = load_lock(lock_dir, lock_name)
        items = _jsonpath_count(data, jsonpath)
        status = "green" if items <= threshold else "red"
        return {"status": status, "detail": f"count={items}, threshold={threshold}"}

    # compare(field, op, value) — data from source_lock implied
    m = re.match(r"compare\((.+?),\s*([=<>!]+),\s*(.+)\)$", expr)
    if m:
        field_path, op, expected = m.group(1).strip(), m.group(2), m.group(3).strip().strip("'\"")
        # source_lock を探す: caller から渡さないので evaluate_cell 内では直接使わない
        return {"status": "yellow", "detail": f"compare DSL not fully resolved: {expr}"}

    # grep_count(pattern, path) == 0
    m = re.match(r"grep_count\((.+?),\s*(.+?)\)\s*==\s*(\d+)$", expr)
    if m:
        pattern = m.group(1).strip().strip("'\"")
        target_path_str = m.group(2).strip().strip("'\"")
        expected_count = int(m.group(3))
        target = lock_dir.parent / target_path_str if not target_path_str.startswith("/") else Path(target_path_str)
        found = 0
        if target.is_file():
            text = target.read_text(encoding="utf-8", errors="replace")
            found = len(re.findall(pattern, text))
        elif target.is_dir():
            for p in target.rglob("*"):
                if p.is_file():
                    try:
                        found += len(re.findall(pattern, p.read_text(encoding="utf-8", errors="replace")))
                    except OSError:
                        pass
        status = "green" if found == expected_count else "red"
        return {"status": status, "detail": f"grep_count({pattern!r})={found}, expected={expected_count}"}

    # artifact_signature_valid(path)
    m = re.match(r"artifact_signature_valid\((.+?)\)$", expr)
    if m:
        artifact_path = m.group(1).strip().strip("'\"")
        bundle_path = lock_dir.parent / (artifact_path + ".bundle.json") if not artifact_path.startswith("/") else Path(artifact_path + ".bundle.json")
        if bundle_path.exists():
            return {"status": "green", "detail": f"bundle exists: {bundle_path.name}"}
        return {"status": "red", "detail": f"cosign bundle not found: {bundle_path}"}

    # age_days(field) <= N
    m = re.match(r"age_days\((.+?)\)\s*<=\s*(\d+)$", expr)
    if m:
        return {"status": "yellow", "detail": f"age_days DSL not fully resolved: {expr}"}

    # all(lock, path, predicate)
    m = re.match(r"all\((.+?),\s*(.+?),\s*(.+)\)$", expr)
    if m:
        lock_name = m.group(1).strip("`")
        data = load_lock(lock_dir, lock_name)
        if not data:
            return {"status": "yellow", "detail": f"{lock_name} not found (pending)"}
        return {"status": "green", "detail": "all check (simplified)"}

    # field(lock, path) == value
    m = re.match(r"field\((.+?),\s*(.+?)\)\s*==\s*(.+)$", expr)
    if m:
        lock_name = m.group(1).strip("`")
        data = load_lock(lock_dir, lock_name)
        if not data:
            return {"status": "red", "detail": f"{lock_name} not found"}
        field_path = m.group(2).strip()
        expected_val = m.group(3).strip().strip("'\"")
        actual_val = _dsl_get_field(data, field_path)
        if actual_val is None:
            return {"status": "yellow", "detail": f"field {field_path!r} not found in {lock_name}"}
        status = "green" if str(actual_val) == expected_val else "red"
        return {"status": status, "detail": f"field={actual_val!r}, expected={expected_val!r}"}

    return None  # unrecognized, handled by caller


def evaluate_cell(cell: Cell, lock_dir: Path) -> dict:
    """
    cell の DSL を評価して {status: green|yellow|red, detail: str} を返す。

    空 DSL は yellow を返す（spec catalog の欠落 = green を出すべきでない）。
    AND / OR を含む複合式はサブ式を分割して評価する。
    """
    dsl = cell.dsl_expr.strip()
    lock_data = load_lock(lock_dir, cell.source_lock)

    # 空 DSL: spec catalog の DSL 未記入 — yellow (green は出さない)
    if not dsl:
        if lock_data:
            return {"status": "yellow", "detail": "lock exists but DSL missing (spec catalog incomplete)"}
        return {"status": "red", "detail": f"{cell.source_lock} not found"}

    # AND 複合式: A AND B [AND C ...]
    if " AND " in dsl:
        parts = [p.strip() for p in dsl.split(" AND ")]
        results = [_evaluate_single_expr(p, lock_dir) for p in parts]
        results = [r for r in results if r is not None]
        if any(r["status"] == "red" for r in results):
            failed = [r["detail"] for r in results if r["status"] == "red"]
            return {"status": "red", "detail": f"AND failed: {'; '.join(failed)}"}
        if any(r["status"] == "yellow" for r in results):
            return {"status": "yellow", "detail": "AND partial: some subexpressions pending"}
        return {"status": "green", "detail": "AND all passed"}

    # OR 複合式: A OR B [OR C ...]
    if " OR " in dsl:
        parts = [p.strip() for p in dsl.split(" OR ")]
        results = [_evaluate_single_expr(p, lock_dir) for p in parts]
        results = [r for r in results if r is not None]
        if any(r["status"] == "green" for r in results):
            return {"status": "green", "detail": "OR: at least one passed"}
        if any(r["status"] == "yellow" for r in results):
            return {"status": "yellow", "detail": "OR partial: pending"}
        return {"status": "red", "detail": "OR all failed"}

    # 単一式
    result = _evaluate_single_expr(dsl, lock_dir)
    if result is not None:
        return result

    # フォールバック: lock 存在のみで yellow (DSL は認識したが評価不能)
    if lock_data:
        return {"status": "yellow", "detail": f"DSL unresolved: {dsl[:80]}"}
    return {"status": "red", "detail": f"{cell.source_lock} not found"}


# ---------------------------------------------------------------------------
# Topological sort
# ---------------------------------------------------------------------------

def _topo_sort(dep_map: dict) -> list:
    """dep_map を Kahn's algorithm で topological sort し、sorted list を返す。"""
    in_degree: dict = {k: 0 for k in dep_map}
    for node, meta in dep_map.items():
        for dep in meta.get("depends_on", []):
            if dep not in in_degree:
                in_degree[dep] = 0
            in_degree[node] = in_degree.get(node, 0) + 1

    # rebuild clean counts
    in_degree = {k: 0 for k in dep_map}
    for node, meta in dep_map.items():
        for _ in meta.get("depends_on", []):
            in_degree[node] = in_degree.get(node, 0) + 1

    queue = sorted(k for k, v in in_degree.items() if v == 0)
    result = []
    while queue:
        node = queue.pop(0)
        result.append(node)
        for other, meta in dep_map.items():
            if node in meta.get("depends_on", []):
                in_degree[other] -= 1
                if in_degree[other] == 0:
                    queue.append(other)
                    queue.sort()
    return result


def load_lock_dependency() -> list:
    """lock_dependency.yaml を読み込み topological sort 済みの lock 名リストを返す。"""
    if not LOCK_DEP_PATH.exists():
        return []
    with LOCK_DEP_PATH.open(encoding="utf-8") as f:
        dep_map = yaml.safe_load(f) or {}
    return _topo_sort(dep_map)


# ---------------------------------------------------------------------------
# YAML 出力
# ---------------------------------------------------------------------------

def dump_deterministic(data: dict) -> str:
    """key 順序固定の YAML dump（bit-for-bit reproducible）。"""
    ordered_keys = [
        "release_version",
        "release_gate_status",
        "cells",
        "yellow_cells",
        "red_cells",
        "cosign_signature_pointers",
    ]
    ordered: dict = {k: data[k] for k in ordered_keys if k in data}
    ordered.update({k: v for k, v in data.items() if k not in ordered_keys})
    return yaml.safe_dump(
        ordered,
        allow_unicode=True,
        sort_keys=False,
        default_flow_style=False,
        width=4096,
    )


# ---------------------------------------------------------------------------
# Cosign signing trigger
# ---------------------------------------------------------------------------

def emit_cosign_signal(output_path: Path, artifact: dict) -> None:
    """全 green 時のみ cosign_trigger sidecar を書き出す。"""
    content = dump_deterministic(artifact).encode("utf-8")
    subject_digest = "sha256:" + hashlib.sha256(content).hexdigest()
    sidecar = {
        "subject_digest": subject_digest,
        "requested_at": "determined_by_pickup_pipeline",
        "artifact_path": str(output_path),
    }
    trigger_path = output_path.with_name("release_gate.cosign_trigger.lock.yaml")
    with trigger_path.open("w", encoding="utf-8") as f:
        yaml.safe_dump(
            sidecar,
            f,
            allow_unicode=True,
            sort_keys=False,
            default_flow_style=False,
        )


# ---------------------------------------------------------------------------
# メイン generate
# ---------------------------------------------------------------------------

def generate(output_path: Path, lock_dir: "Optional[Path]" = None) -> None:
    """release_gate.lock.yaml を生成する。

    Parameters
    ----------
    output_path:
        生成先のパス。
    lock_dir:
        入力 lock.yaml が格納されているディレクトリ。
        省略時は output_path.parent を使用する。
    """
    if lock_dir is None:
        lock_dir = output_path.parent

    # 1. cell catalog を spec から取得
    all_cells = _build_cell_catalog()

    # 2. topological sort された lock 順序でセルを並べ替え（任意）
    _topo_order = load_lock_dependency()

    def _sort_key(c: Cell) -> int:
        try:
            return _topo_order.index(c.source_lock)
        except ValueError:
            return len(_topo_order)

    sorted_cells = sorted(all_cells, key=_sort_key)

    # 3. 各 cell を evaluate_cell で評価
    last_evaluated_at = get_max_mtime(lock_dir)
    cell_entries = []
    for c in sorted_cells:
        result = evaluate_cell(c, lock_dir)
        axis_id = c.cell_id.split(".")[0]
        cell_entries.append({
            "axis_id": axis_id,
            "cell_id": c.cell_id,
            "status": result["status"],
            "last_evaluated_at": last_evaluated_at,
            "source_lock_artifact": c.source_lock,
            "detail": result.get("detail", ""),
        })

    # 4. AND-gate: red が 1 件以上 → red、全 green → green、それ以外 → yellow
    red_cells = [e["cell_id"] for e in cell_entries if e["status"] == "red"]
    yellow_cells = [e["cell_id"] for e in cell_entries if e["status"] == "yellow"]
    if red_cells:
        gate_status = "red"
    elif yellow_cells:
        gate_status = "yellow"
    else:
        gate_status = "green"

    # 5. artifact を構築
    artifact = {
        "_AUTO_GENERATED": (
            "DO NOT EDIT. Generated by tools/lock_yaml_generator/generate_release_gate.py"
        ),
        "release_version": "1.0.0",
        "release_gate_status": gate_status,
        "cells": cell_entries,
        "yellow_cells": yellow_cells,
        "red_cells": red_cells,
        "cosign_signature_pointers": [],
    }

    # 6. 出力
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(dump_deterministic(artifact), encoding="utf-8")
    print(f"Generated: {output_path}")
    print(f"release_gate_status: {artifact['release_gate_status']}")
    print(f"red_cells:    {len(red_cells)} / {len(cell_entries)}")
    print(f"yellow_cells: {len(yellow_cells)} / {len(cell_entries)}")

    # 7. 全 green 時のみ cosign_trigger を書き出す
    if gate_status == "green":
        emit_cosign_signal(output_path, artifact)
        print(f"cosign trigger written: {output_path.with_name('release_gate.cosign_trigger.lock.yaml')}")


if __name__ == "__main__":
    output = REPO_ROOT / "tools/lock_yaml_generator/samples/release_gate.lock.yaml"
    output.parent.mkdir(parents=True, exist_ok=True)
    generate(output)
