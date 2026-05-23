"""tests for dsl.py — DSL インタープリタの全 case を検証する。"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest
import yaml

sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))
from tools.lock_yaml_generator.dsl import (
    EvalResult,
    evaluate_dsl,
    eval_count,
    eval_all,
    eval_field,
    eval_len,
    eval_bidirectional,
    eval_ratio,
    eval_hard_fail_if_zero,
)


# ---------------------------------------------------------------------------
# count()
# ---------------------------------------------------------------------------

def test_count_lte_green(tmp_path):
    data = {"entries": [{"status": "open"}] * 5}
    (tmp_path / "assumption.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_count("count(`assumption.lock.yaml`, entries[?status=='open']) <= 20", tmp_path)
    assert r.status == "green"
    assert "5" in r.detail


def test_count_lte_red(tmp_path):
    data = {"entries": [{"status": "open"}] * 21}
    (tmp_path / "assumption.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_count("count(`assumption.lock.yaml`, entries[?status=='open']) <= 20", tmp_path)
    assert r.status == "red"
    assert "21" in r.detail


def test_count_gte_green(tmp_path):
    data = {"specs": [{"cluster_id": "cross_http2"}] * 2}
    (tmp_path / "cross_cutting_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_count(
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_http2']) >= 1",
        tmp_path
    )
    assert r.status == "green"


def test_count_gte_red(tmp_path):
    data = {"specs": []}
    (tmp_path / "cross_cutting_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_count(
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_http2']) >= 1",
        tmp_path
    )
    assert r.status == "red"


def test_count_eq_green(tmp_path):
    data = {"cells": [{"drill_state": "green"}] * 5}
    (tmp_path / "topology.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_count("count(`topology.lock.yaml`, cells[?drill_state=='green']) == 5", tmp_path)
    assert r.status == "green"


def test_count_lock_missing_red(tmp_path):
    r = eval_count("count(`no_such.lock.yaml`, entries[?status=='open']) <= 20", tmp_path)
    assert r.status == "red"
    assert "not found" in r.detail


def test_count_parse_fail_yellow(tmp_path):
    r = eval_count("invalid DSL expression", tmp_path)
    assert r.status == "yellow"
    assert "parse" in r.detail


# ---------------------------------------------------------------------------
# all()
# ---------------------------------------------------------------------------

def test_all_has_isomorphic_structure_green(tmp_path):
    data = {"axes": [{"axis_id": "01"}, {"axis_id": "02"}]}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_all(
        "all(`axis_registry.lock.yaml`, axes, has_isomorphic_structure)",
        tmp_path
    )
    assert r.status == "green"


def test_all_has_isomorphic_structure_red(tmp_path):
    data = {"axes": [{}, {}]}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_all(
        "all(`axis_registry.lock.yaml`, axes, has_isomorphic_structure)",
        tmp_path
    )
    assert r.status == "red"


def test_all_lock_missing_yellow(tmp_path):
    r = eval_all("all(`no_such.lock.yaml`, axes, has_isomorphic_structure)", tmp_path)
    assert r.status == "yellow"


def test_all_field_eq_predicate_green(tmp_path):
    data = {"cells": [{"status": "green"}, {"status": "green"}]}
    (tmp_path / "release_gate.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_all("all(`release_gate.lock.yaml`, cells, status=='green')", tmp_path)
    assert r.status == "green"


def test_all_field_eq_predicate_red(tmp_path):
    data = {"cells": [{"status": "green"}, {"status": "red"}]}
    (tmp_path / "release_gate.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_all("all(`release_gate.lock.yaml`, cells, status=='green')", tmp_path)
    assert r.status == "red"


# ---------------------------------------------------------------------------
# field()
# ---------------------------------------------------------------------------

def test_field_eq_int_green(tmp_path):
    data = {"axes_count": 19, "cap_slot_max": 20}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_field("field(`axis_registry.lock.yaml`, axes_count) == 19", tmp_path)
    assert r.status == "green"


def test_field_eq_int_red(tmp_path):
    data = {"axes_count": 18}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_field("field(`axis_registry.lock.yaml`, axes_count) == 19", tmp_path)
    assert r.status == "red"


def test_field_eq_str_green(tmp_path):
    data = {"status": "green"}
    (tmp_path / "docs_lint.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_field("field(`docs_lint.lock.yaml`, status) == green", tmp_path)
    assert r.status == "green"


def test_field_eq_str_red(tmp_path):
    data = {"status": "red"}
    (tmp_path / "docs_lint.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_field("field(`docs_lint.lock.yaml`, status) == green", tmp_path)
    assert r.status == "red"


def test_field_lock_missing_red(tmp_path):
    r = eval_field("field(`no_such.lock.yaml`, status) == green", tmp_path)
    assert r.status == "red"


def test_field_path_missing_red(tmp_path):
    data = {"other": "value"}
    (tmp_path / "some.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_field("field(`some.lock.yaml`, status) == green", tmp_path)
    assert r.status == "red"


# ---------------------------------------------------------------------------
# len()
# ---------------------------------------------------------------------------

def test_len_gte_green(tmp_path):
    data = {"cells": [1, 2, 3] * 30}  # 90 cells
    (tmp_path / "coverage_matrix.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_len("len(`coverage_matrix.lock.yaml`, cells) >= 90", tmp_path)
    assert r.status == "green"


def test_len_gte_red(tmp_path):
    data = {"cells": [1, 2]}
    (tmp_path / "coverage_matrix.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_len("len(`coverage_matrix.lock.yaml`, cells) >= 90", tmp_path)
    assert r.status == "red"


def test_len_lock_missing_red(tmp_path):
    r = eval_len("len(`no_such.lock.yaml`, cells) >= 1", tmp_path)
    assert r.status == "red"


# ---------------------------------------------------------------------------
# bidirectional_lock()
# ---------------------------------------------------------------------------

def test_bidirectional_both_empty_green(tmp_path):
    d1 = {"entries": []}
    d2 = {"entries": []}
    (tmp_path / "lock1.lock.yaml").write_text(yaml.safe_dump(d1))
    (tmp_path / "lock2.lock.yaml").write_text(yaml.safe_dump(d2))
    r = eval_bidirectional("bidirectional_lock(`lock1.lock.yaml`, `lock2.lock.yaml`)", tmp_path)
    assert r.status == "green"


def test_bidirectional_matching_green(tmp_path):
    d1 = {"entries": [{"id": "a"}, {"id": "b"}]}
    d2 = {"entries": [{"id": "a"}, {"id": "b"}]}
    (tmp_path / "formal.lock.yaml").write_text(yaml.safe_dump(d1))
    (tmp_path / "test.lock.yaml").write_text(yaml.safe_dump(d2))
    r = eval_bidirectional("bidirectional_lock(`formal.lock.yaml`, `test.lock.yaml`)", tmp_path)
    assert r.status == "green"


def test_bidirectional_mismatch_red(tmp_path):
    d1 = {"entries": [{"id": "a"}, {"id": "b"}]}
    d2 = {"entries": [{"id": "a"}]}
    (tmp_path / "formal.lock.yaml").write_text(yaml.safe_dump(d1))
    (tmp_path / "test.lock.yaml").write_text(yaml.safe_dump(d2))
    r = eval_bidirectional("bidirectional_lock(`formal.lock.yaml`, `test.lock.yaml`)", tmp_path)
    assert r.status == "red"


def test_bidirectional_missing_yellow(tmp_path):
    r = eval_bidirectional("bidirectional_lock(`no1.lock.yaml`, `no2.lock.yaml`)", tmp_path)
    assert r.status == "yellow"


# ---------------------------------------------------------------------------
# evaluate_dsl — メインルーター
# ---------------------------------------------------------------------------

def test_evaluate_dsl_count(tmp_path):
    data = {"entries": [{"status": "open"}] * 5}
    (tmp_path / "assumption.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl(
        "count(`assumption.lock.yaml`, entries[?status=='open']) <= 20",
        tmp_path
    )
    assert r.status == "green"


def test_evaluate_dsl_field(tmp_path):
    data = {"axes_count": 19}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl("field(`axis_registry.lock.yaml`, axes_count) == 19", tmp_path)
    assert r.status == "green"


def test_evaluate_dsl_and_compound(tmp_path):
    data = {"axes_count": 19}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl(
        "field(`axis_registry.lock.yaml`, axes_count) == 19 AND field(`axis_registry.lock.yaml`, axes_count) == 19",
        tmp_path
    )
    assert r.status == "green"


def test_evaluate_dsl_and_compound_partial_fail(tmp_path):
    data = {"axes_count": 19}
    (tmp_path / "axis_registry.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl(
        "field(`axis_registry.lock.yaml`, axes_count) == 19 AND field(`axis_registry.lock.yaml`, axes_count) == 20",
        tmp_path
    )
    assert r.status == "red"


def test_evaluate_dsl_empty_yields_yellow(tmp_path):
    r = evaluate_dsl("", tmp_path)
    assert r.status == "yellow"


def test_evaluate_dsl_unknown_yellow(tmp_path):
    r = evaluate_dsl("some_unknown_dsl(a, b)", tmp_path)
    assert r.status == "yellow"


# ---------------------------------------------------------------------------
# ratio()
# ---------------------------------------------------------------------------

def test_ratio_lte_green(tmp_path):
    # 20 accepted_with_assumption / 100 total = 20% <= 20% → green
    data = {
        "total_cells": 100,
        "cells": [{"cell_state": "v1_accepted_with_assumption"}] * 20
                + [{"cell_state": "v1_baseline_verified"}] * 80,
    }
    (tmp_path / "proof_status.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_ratio(
        "ratio(`proof_status.lock.yaml`, cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 0.20",
        tmp_path
    )
    assert r.status == "green"
    assert "20.0%" in r.detail


def test_ratio_lte_red(tmp_path):
    # 61 accepted_with_assumption / 100 total = 61% > 20% → red
    data = {
        "total_cells": 100,
        "cells": [{"cell_state": "v1_accepted_with_assumption"}] * 61
                + [{"cell_state": "v1_baseline_verified"}] * 39,
    }
    (tmp_path / "proof_status.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_ratio(
        "ratio(`proof_status.lock.yaml`, cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 0.20",
        tmp_path
    )
    assert r.status == "red"
    assert "61.0%" in r.detail


def test_ratio_division_by_zero_red(tmp_path):
    data = {"total_cells": 0, "cells": []}
    (tmp_path / "proof_status.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_ratio(
        "ratio(`proof_status.lock.yaml`, cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 0.20",
        tmp_path
    )
    assert r.status == "red"
    assert "division by zero" in r.detail


def test_ratio_lock_missing_red(tmp_path):
    r = eval_ratio(
        "ratio(`no_such.lock.yaml`, cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 0.20",
        tmp_path
    )
    assert r.status == "red"
    assert "not found" in r.detail


def test_ratio_parse_fail_yellow(tmp_path):
    r = eval_ratio("invalid ratio expression", tmp_path)
    assert r.status == "yellow"
    assert "parse" in r.detail


def test_evaluate_dsl_ratio(tmp_path):
    data = {
        "total_cells": 100,
        "cells": [{"cell_state": "v1_accepted_with_assumption"}] * 10
                + [{"cell_state": "v1_baseline_verified"}] * 90,
    }
    (tmp_path / "proof_status.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl(
        "ratio(`proof_status.lock.yaml`, cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 0.20",
        tmp_path
    )
    assert r.status == "green"


# ---------------------------------------------------------------------------
# hard_fail_if_zero()
# ---------------------------------------------------------------------------

def test_hard_fail_if_zero_non_empty_green(tmp_path):
    data = {"entries": [{"id": "e1", "status": "closed"}]}
    (tmp_path / "regression_corpus.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_hard_fail_if_zero(
        "hard_fail_if_zero(`regression_corpus.lock.yaml`, entries)",
        tmp_path
    )
    assert r.status == "green"
    assert "1 item" in r.detail


def test_hard_fail_if_zero_empty_list_red(tmp_path):
    data = {"entries": [], "total_count": 0}
    (tmp_path / "regression_corpus.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_hard_fail_if_zero(
        "hard_fail_if_zero(`regression_corpus.lock.yaml`, entries)",
        tmp_path
    )
    assert r.status == "red"
    assert "hard fail" in r.detail


def test_hard_fail_if_zero_missing_field_red(tmp_path):
    data = {"total_count": 0}  # entries フィールド自体が存在しない
    (tmp_path / "regression_corpus.lock.yaml").write_text(yaml.safe_dump(data))
    r = eval_hard_fail_if_zero(
        "hard_fail_if_zero(`regression_corpus.lock.yaml`, entries)",
        tmp_path
    )
    assert r.status == "red"
    assert "hard fail" in r.detail


def test_hard_fail_if_zero_lock_missing_red(tmp_path):
    r = eval_hard_fail_if_zero(
        "hard_fail_if_zero(`no_such.lock.yaml`, entries)",
        tmp_path
    )
    assert r.status == "red"
    assert "not found" in r.detail


def test_hard_fail_if_zero_parse_fail_yellow(tmp_path):
    r = eval_hard_fail_if_zero("invalid expression", tmp_path)
    assert r.status == "yellow"
    assert "parse" in r.detail


def test_evaluate_dsl_hard_fail_and_count_compound(tmp_path):
    # entries 非空 AND open エントリゼロ → green
    data = {"entries": [{"id": "e1", "status": "closed"}], "total_count": 1}
    (tmp_path / "regression_corpus.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl(
        "hard_fail_if_zero(`regression_corpus.lock.yaml`, entries) AND "
        "count(`regression_corpus.lock.yaml`, entries[?status=='open']) == 0",
        tmp_path
    )
    assert r.status == "green"


def test_evaluate_dsl_hard_fail_empty_entries_red(tmp_path):
    # entries 空 → hard_fail_if_zero が red → compound は red
    data = {"entries": [], "total_count": 0}
    (tmp_path / "regression_corpus.lock.yaml").write_text(yaml.safe_dump(data))
    r = evaluate_dsl(
        "hard_fail_if_zero(`regression_corpus.lock.yaml`, entries) AND "
        "count(`regression_corpus.lock.yaml`, entries[?status=='open']) == 0",
        tmp_path
    )
    assert r.status == "red"
    assert "hard fail" in r.detail
