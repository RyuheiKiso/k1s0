"""tests/test_dsl_invariants.py

再 gaming 防止構造の不変条件テスト。
以下の 5 不変条件を検証する:
  1. _GLOBAL_INVARIANTS が 4 要素存在する
  2. meta_invariant.* cell が _CELL_CATALOG に 4 件存在する
  3. _meta/lock/ 内に SoT 重複がない (orphan file 削除後の状態が維持されている)
  4. 4 evaluator が dsl.py から export されている
  5. CANONICAL_SOT_TABLE が主要 lock_name を全て含む
"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from tools.lock_yaml_generator.generate_release_gate_v2 import (
    _GLOBAL_INVARIANTS,
    _CELL_CATALOG,
)
from tools.lock_yaml_generator.dsl import (
    eval_artifact_substance,
    eval_no_stale_reference,
    eval_no_vacuous_green,
    eval_env_dependent_ratio_cap,
)
from tools.lock_yaml_generator.lock_registry import CANONICAL_SOT_TABLE

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent


def test_global_invariants_present():
    """_GLOBAL_INVARIANTS に 4 要素が存在する。"""
    assert len(_GLOBAL_INVARIANTS) == 4
    inv_ids = [i[0] for i in _GLOBAL_INVARIANTS]
    assert "sot_uniqueness_pre" in inv_ids
    assert "no_vacuous_green_pre" in inv_ids
    assert "artifact_substance_pre" in inv_ids
    assert "env_dependent_cap_pre" in inv_ids


def test_meta_invariant_cells_in_catalog():
    """_CELL_CATALOG に meta_invariant.* cell が 4 件存在する。"""
    meta_inv_cells = [cell_id for cell_id, _, _ in _CELL_CATALOG
                      if cell_id.startswith("meta_invariant.")]
    assert len(meta_inv_cells) == 4, f"Expected 4 meta_invariant cells, got: {meta_inv_cells}"
    assert "meta_invariant.sot_uniqueness" in meta_inv_cells
    assert "meta_invariant.no_vacuous_green" in meta_inv_cells
    assert "meta_invariant.artifact_substance_floor" in meta_inv_cells
    assert "meta_invariant.env_dependent_ratio_cap" in meta_inv_cells


def test_no_meta_lock_duplicate_paths():
    """_meta/lock/ 内に CANONICAL_SOT_TABLE で canonical が formal/lock/ または test/lock/ に
    指定されている lock_name が存在しないことを確認する（孤児削除が維持されている）。"""
    meta_lock_dir = REPO_ROOT / "src" / "_meta" / "lock"
    if not meta_lock_dir.exists():
        pytest.skip("_meta/lock/ directory does not exist")

    orphan_canonical_names = {
        name for name, canonical in CANONICAL_SOT_TABLE.items()
        if canonical in ("src/formal/lock", "src/test/lock")
    }
    found_orphans = []
    for lock_name in orphan_canonical_names:
        orphan_path = meta_lock_dir / lock_name
        if orphan_path.exists():
            found_orphans.append(str(orphan_path.relative_to(REPO_ROOT)))

    assert not found_orphans, (
        f"Orphan lock files found in _meta/lock/ (gaming pattern):\n"
        + "\n".join(f"  {p}" for p in found_orphans)
    )


def test_invariant_evaluator_signatures():
    """dsl.py に 4 evaluator が callable として export されている。"""
    assert callable(eval_artifact_substance)
    assert callable(eval_no_stale_reference)
    assert callable(eval_no_vacuous_green)
    assert callable(eval_env_dependent_ratio_cap)


def test_canonical_sot_table_complete():
    """CANONICAL_SOT_TABLE が重要な lock_name をカバーしている。"""
    required_entries = {
        "proof_status.lock.yaml",
        "proof_inventory.lock.yaml",
        "proof_matrix.lock.yaml",
        "proof_review.lock.yaml",
        "assumption.lock.yaml",
        "counter_example.lock.yaml",
        "coverage_matrix.lock.yaml",
        "regression_corpus.lock.yaml",
        "release_gate.lock.yaml",
        "axis_registry.lock.yaml",
        "build_evidence.lock.yaml",
    }
    missing = required_entries - set(CANONICAL_SOT_TABLE.keys())
    assert not missing, f"CANONICAL_SOT_TABLE is missing entries: {missing}"


# ---------------------------------------------------------------------------
# 4 evaluator の動作テスト
# ---------------------------------------------------------------------------

import yaml


def test_eval_artifact_substance_green(tmp_path):
    """substance ある artifact_pointer ファイルを持つ cell が green になる。"""
    lock_dir = tmp_path
    artifact_dir = tmp_path / "artifacts"
    artifact_dir.mkdir()
    # substance: 10 行以上・ban_keywords なし
    (artifact_dir / "good.yaml").write_text("key: value\n" * 10)

    data = {
        "cells": [
            {"drill_state": "v1_verified_with_artifact_pointer",
             "artifact_pointer": "artifacts/good.yaml"}
        ]
    }
    (lock_dir / "coverage_matrix.lock.yaml").write_text(yaml.safe_dump(data))

    r = eval_artifact_substance(
        'artifact_substance(`coverage_matrix.lock.yaml`, '
        "cells[?drill_state=='v1_verified_with_artifact_pointer'], "
        'artifact_pointer, 8, "") >= 1',
        lock_dir,
    )
    assert r.status == "green", r.detail


def test_eval_artifact_substance_red_placeholder(tmp_path):
    """3 行 placeholder README が red になる。"""
    lock_dir = tmp_path
    placeholder_dir = tmp_path / "placeholder_artifacts"
    placeholder_dir.mkdir()
    (placeholder_dir / "README.md").write_text("# placeholder\n\nartifact placeholder。drill 実行後に結果 URI を記録する。\n")

    data = {
        "cells": [
            {"drill_state": "v1_verified_with_artifact_pointer",
             "artifact_pointer": "placeholder_artifacts/README.md"}
        ]
    }
    (lock_dir / "coverage_matrix.lock.yaml").write_text(yaml.safe_dump(data))

    r = eval_artifact_substance(
        'artifact_substance(`coverage_matrix.lock.yaml`, '
        "cells[?drill_state=='v1_verified_with_artifact_pointer'], "
        'artifact_pointer, 8, "placeholder|TODO|drill 実行後に") >= 1',
        lock_dir,
    )
    # 3 行かつ "placeholder" / "drill 実行後に" を含む → substance なし → red
    assert r.status == "red", r.detail


def test_eval_no_stale_reference_green_single(tmp_path):
    """canonical のみ存在する場合 green。"""
    (tmp_path / "formal_proof_status.lock.yaml").write_text("status: green\n")
    r = eval_no_stale_reference(
        "no_stale_reference(`proof_status.lock.yaml`, `formal_proof_status.lock.yaml`, 7)",
        tmp_path,
    )
    assert r.status == "green", r.detail


def test_eval_no_stale_reference_red_duplicate(tmp_path):
    """両方存在する場合 red (SoT duplicate)。"""
    (tmp_path / "proof_status.lock.yaml").write_text("status: green\n")
    (tmp_path / "formal_proof_status.lock.yaml").write_text("status: green\n")
    r = eval_no_stale_reference(
        "no_stale_reference(`proof_status.lock.yaml`, `formal_proof_status.lock.yaml`, 7)",
        tmp_path,
    )
    assert r.status == "red", r.detail


def test_eval_no_vacuous_green_green(tmp_path):
    """min_count 以上の items があれば green。"""
    data = {"cells": [{"id": f"c{i}"} for i in range(95)]}
    (tmp_path / "proof_matrix.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_no_vacuous_green(
        "no_vacuous_green(`proof_matrix.lock.yaml`, cells, 95)",
        tmp_path,
    )
    assert r.status == "green", r.detail


def test_eval_no_vacuous_green_red_empty(tmp_path):
    """cells: [] で red になる。"""
    data = {"cells": [], "total_cells": 0}
    (tmp_path / "proof_matrix.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_no_vacuous_green(
        "no_vacuous_green(`proof_matrix.lock.yaml`, cells, 95)",
        tmp_path,
    )
    assert r.status == "red", r.detail


def test_eval_env_dependent_ratio_cap_green(tmp_path):
    """env_dependent が 20% (2/10) で cap 30% → green。"""
    entries = [{"build_evidence_id": f"entry_{i}_envdep_local"} for i in range(2)]
    entries += [{"build_evidence_id": f"entry_{i}_local_verified"} for i in range(8)]
    data = {"evidence_entries": entries}
    (tmp_path / "build_evidence.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_env_dependent_ratio_cap(
        'env_dependent_ratio_cap(`build_evidence.lock.yaml`, evidence_entries, build_evidence_id, "_envdep_") <= 0.30',
        tmp_path,
    )
    assert r.status == "green", r.detail


def test_eval_env_dependent_ratio_cap_red(tmp_path):
    """env_dependent が 70% (7/10) で cap 30% → red。"""
    entries = [{"build_evidence_id": f"entry_{i}_envdep_"} for i in range(7)]
    entries += [{"build_evidence_id": f"entry_{i}_verified"} for i in range(3)]
    data = {"evidence_entries": entries}
    (tmp_path / "build_evidence.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_env_dependent_ratio_cap(
        'env_dependent_ratio_cap(`build_evidence.lock.yaml`, evidence_entries, build_evidence_id, "_envdep_") <= 0.30',
        tmp_path,
    )
    assert r.status == "red", r.detail
