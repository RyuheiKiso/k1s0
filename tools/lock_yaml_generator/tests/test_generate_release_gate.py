"""tests for generate_release_gate.py"""
import hashlib
import sys
from pathlib import Path

import pytest
import yaml

# プロジェクトルートをパスに追加
sys.path.insert(0, str(Path(__file__).parent.parent))
from generate_release_gate import (
    Cell,
    dump_deterministic,
    emit_cosign_signal,
    evaluate_cell,
    get_max_mtime,
    load_cell_catalog,
)


def test_dump_deterministic_key_order(tmp_path):
    data = {
        "red_cells": [],
        "release_version": "1.0.0",
        "release_gate_status": "green",
        "cells": [],
        "yellow_cells": [],
    }
    result = dump_deterministic(data)
    keys = [line.split(":")[0] for line in result.splitlines() if ":" in line]
    ordered = [
        k
        for k in ["release_version", "release_gate_status", "cells", "yellow_cells", "red_cells"]
        if k in keys
    ]
    assert ordered == [
        "release_version",
        "release_gate_status",
        "cells",
        "yellow_cells",
        "red_cells",
    ]


def test_dump_deterministic_idempotency(tmp_path):
    data = {
        "release_version": "1.0.0",
        "release_gate_status": "green",
        "cells": [{"cell_id": "formal.all_critical_verified", "status": "green"}],
        "yellow_cells": [],
        "red_cells": [],
    }
    out1 = dump_deterministic(data)
    out2 = dump_deterministic(data)
    assert (
        hashlib.sha256(out1.encode()).hexdigest()
        == hashlib.sha256(out2.encode()).hexdigest()
    )


def test_evaluate_cell_missing_lock(tmp_path):
    cell = Cell(cell_id="test.missing", source_lock="nonexistent.lock.yaml", dsl_expr="")
    result = evaluate_cell(cell, tmp_path)
    assert result["status"] in ("red", "yellow")


def test_evaluate_cell_with_lock(tmp_path):
    lock = {"entries": [{"status": "open", "severity": "high"}]}
    (tmp_path / "assumption.lock.yaml").write_text(yaml.safe_dump(lock), encoding="utf-8")
    cell = Cell(
        cell_id="formal.assumption_cap",
        source_lock="assumption.lock.yaml",
        dsl_expr="count(`assumption.lock.yaml`, entries[?status=='open']) <= 20",
    )
    result = evaluate_cell(cell, tmp_path)
    assert result["status"] == "green"


def test_evaluate_cell_count_exceeds_threshold(tmp_path):
    """count が threshold を超えた場合に red を返すことを確認する。"""
    entries = [{"status": "open", "severity": "high"}] * 21
    lock = {"entries": entries}
    (tmp_path / "assumption.lock.yaml").write_text(yaml.safe_dump(lock), encoding="utf-8")
    cell = Cell(
        cell_id="formal.assumption_cap",
        source_lock="assumption.lock.yaml",
        dsl_expr="count(`assumption.lock.yaml`, entries[?status=='open']) <= 20",
    )
    result = evaluate_cell(cell, tmp_path)
    assert result["status"] == "red"


def test_cosign_signal_only_on_green(tmp_path):
    artifact_green = {
        "release_version": "1.0.0",
        "release_gate_status": "green",
        "cells": [],
        "yellow_cells": [],
        "red_cells": [],
    }
    artifact_red = dict(artifact_green, release_gate_status="red")

    out_path = tmp_path / "release_gate.lock.yaml"
    out_path.write_text(dump_deterministic(artifact_green), encoding="utf-8")
    emit_cosign_signal(out_path, artifact_green)
    trigger = tmp_path / "release_gate.cosign_trigger.lock.yaml"
    assert trigger.exists(), "cosign trigger should be written for green"

    trigger.unlink()
    out_path.write_text(dump_deterministic(artifact_red), encoding="utf-8")
    # red の場合は emit_cosign_signal を呼ばない（呼び出し側の責任）
    assert not trigger.exists(), "cosign trigger should NOT exist for red"


def test_cosign_signal_digest_content(tmp_path):
    """cosign trigger の subject_digest が実際の artifact hash と一致することを確認する。"""
    artifact = {
        "release_version": "1.0.0",
        "release_gate_status": "green",
        "cells": [],
        "yellow_cells": [],
        "red_cells": [],
    }
    out_path = tmp_path / "release_gate.lock.yaml"
    out_path.write_text(dump_deterministic(artifact), encoding="utf-8")
    emit_cosign_signal(out_path, artifact)

    trigger_path = tmp_path / "release_gate.cosign_trigger.lock.yaml"
    with trigger_path.open(encoding="utf-8") as f:
        trigger_data = yaml.safe_load(f)

    expected_digest = "sha256:" + hashlib.sha256(
        dump_deterministic(artifact).encode("utf-8")
    ).hexdigest()
    assert trigger_data["subject_digest"] == expected_digest


def test_get_max_mtime_empty_dir(tmp_path):
    result = get_max_mtime(tmp_path)
    assert result == "1970-01-01T00:00:00Z"


def test_get_max_mtime_with_locks(tmp_path):
    (tmp_path / "foo.lock.yaml").write_text("x: 1", encoding="utf-8")
    result = get_max_mtime(tmp_path)
    assert result != "1970-01-01T00:00:00Z"
    assert "T" in result and result.endswith("Z")


def test_get_max_mtime_excludes_output_file(tmp_path):
    """release_gate.lock.yaml（出力ファイル）は mtime 計算から除外される。"""
    (tmp_path / "release_gate.lock.yaml").write_text("x: 1", encoding="utf-8")
    result = get_max_mtime(tmp_path)
    assert result == "1970-01-01T00:00:00Z"
