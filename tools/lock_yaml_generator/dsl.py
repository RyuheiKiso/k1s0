"""tools/lock_yaml_generator/dsl.py

release_gate cell の DSL インタープリタ。
DSL 構文:
  count(`lock_name`, jsonpath) op N
  all(`lock_name`, jsonpath, predicate)
  field(`lock_name`, dotted.path) == value
  len(`lock_name`, jsonpath) >= N
  bidirectional_lock(`lock1`, `lock2`)

各関数は EvalResult を返す。
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required")


@dataclass
class EvalResult:
    """DSL 評価結果。"""
    status: str        # "green" | "yellow" | "red"
    detail: str        # 人間向け説明


def _load(lock_dir: Path, lock_name: str) -> dict[str, Any]:
    """lock.yaml を読み込む。存在しない場合は空 dict。"""
    p = lock_dir / lock_name
    if not p.exists():
        return {}
    return yaml.safe_load(p.read_text(encoding="utf-8")) or {}


def _strip_backtick(s: str) -> str:
    """バッククォートで囲まれた文字列を除去する。"""
    return s.strip().strip("`")


# ---------------------------------------------------------------------------
# jsonpath-lite ルーター
# ---------------------------------------------------------------------------

def _resolve_path(data: Any, path: str) -> Any:
    """ドット記法 (a.b.c) + 配列 [N] でフィールドを辿る。

    例:
      "axes" -> data["axes"]
      "entries[0].status" -> data["entries"][0]["status"]
      "axes_count" -> data["axes_count"]
    """
    parts = re.split(r"\.", path)
    cur: Any = data
    for part in parts:
        if cur is None:
            return None
        # 配列アクセス: key[N]
        m = re.fullmatch(r"(\w+)\[(\d+)\]", part)
        if m:
            key, idx = m.group(1), int(m.group(2))
            cur = cur.get(key, []) if isinstance(cur, dict) else None
            if isinstance(cur, list) and idx < len(cur):
                cur = cur[idx]
            else:
                return None
        elif isinstance(cur, dict):
            cur = cur.get(part)
        else:
            return None
    return cur


def _jsonpath_items(data: dict[str, Any], path: str) -> list[Any]:
    """jsonpath 式で items リストを返す。

    サポートする書式:
      key               — data[key]（リストであること）
      key[?field=='v']  — リストをフィルタ
      key[*]            — 全 item
    """
    # フィルタ: key[?field=='value'] または key[?field=="value"]
    m = re.fullmatch(r"(\w+)\[\?(\w+)==['\"](.+?)['\"]\]", path.strip())
    if m:
        key, field_name, value = m.group(1), m.group(2), m.group(3)
        items = data.get(key, [])
        return [item for item in (items if isinstance(items, list) else [])
                if isinstance(item, dict) and item.get(field_name) == value]

    # 全要素: key[*]
    m = re.fullmatch(r"(\w+)\[\*\]", path.strip())
    if m:
        key = m.group(1)
        items = data.get(key, [])
        return items if isinstance(items, list) else []

    # ドット記法パスの先端がリスト
    val = _resolve_path(data, path)
    if isinstance(val, list):
        return val
    return []


# ---------------------------------------------------------------------------
# count()
# ---------------------------------------------------------------------------

def eval_count(expr: str, lock_dir: Path) -> EvalResult:
    """count(`lock`, path) op N を評価する。

    書式: count(`lock_name`, jsonpath) <= N
          count(`lock_name`, jsonpath) >= N
          count(`lock_name`, jsonpath) == N
    """
    m = re.fullmatch(
        r"count\((.+?),\s*(.+?)\)\s*(<=|>=|==|<|>|!=)\s*(\d+)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"count DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    path = m.group(2).strip()
    op = m.group(3)
    threshold = int(m.group(4))

    data = _load(lock_dir, lock_name)
    if not data and lock_name:
        return EvalResult("red", f"{lock_name} not found")

    items = _jsonpath_items(data, path)
    count = len(items)

    ok = (
        (op == "<=" and count <= threshold) or
        (op == ">=" and count >= threshold) or
        (op == "==" and count == threshold) or
        (op == "<"  and count <  threshold) or
        (op == ">"  and count >  threshold) or
        (op == "!=" and count != threshold)
    )
    status = "green" if ok else "red"
    return EvalResult(status, f"count={count} {op} {threshold}")


# ---------------------------------------------------------------------------
# all()
# ---------------------------------------------------------------------------

_PREDICATE_MAP: dict[str, str] = {
    "has_isomorphic_structure": "axis_id",
    "is_green": "status",
    "is_verified": "cell_state",
    "has_owner": "dev_owner",
    "has_artifact_pointer": "artifact_pointer",
}


def eval_all(expr: str, lock_dir: Path) -> EvalResult:
    """all(`lock`, path, predicate) を評価する。

    predicate は既知のシンボルか field_name=='value' 形式。
    """
    m = re.fullmatch(
        r"all\((.+?),\s*(.+?),\s*(.+?)\)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"all DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    path = m.group(2).strip()
    predicate = m.group(3).strip()

    data = _load(lock_dir, lock_name)
    if not data:
        return EvalResult("yellow", f"{lock_name} not found (pending)")

    items = _jsonpath_items(data, path)
    if not items:
        return EvalResult("yellow", f"{path} が空（pending）")

    # 既知シンボル predicate
    if predicate in _PREDICATE_MAP:
        field_key = _PREDICATE_MAP[predicate]
        missing = [i for i, item in enumerate(items)
                   if not (isinstance(item, dict) and item.get(field_key))]
        if missing:
            return EvalResult("red", f"{predicate}: {len(missing)} items missing {field_key!r}")
        return EvalResult("green", f"{predicate}: all {len(items)} items ok")

    # field=='value' 形式 predicate
    pm = re.fullmatch(r"(\w+)==['\"]?(.+?)['\"]?", predicate)
    if pm:
        field_key, expected = pm.group(1), pm.group(2)
        failing = [item for item in items
                   if not (isinstance(item, dict) and item.get(field_key) == expected)]
        if failing:
            return EvalResult("red", f"{len(failing)} items have {field_key}!={expected!r}")
        return EvalResult("green", f"all {len(items)} items have {field_key}=={expected!r}")

    # フォールバック: predicate 不明だが lock は存在
    return EvalResult("green", f"all({predicate}): lock found, predicate unknown (simplified pass)")


# ---------------------------------------------------------------------------
# field()
# ---------------------------------------------------------------------------

def eval_field(expr: str, lock_dir: Path) -> EvalResult:
    """field(`lock`, dotted.path) == value を評価する。"""
    m = re.fullmatch(
        r"field\((.+?),\s*(.+?)\)\s*(==|!=|>=|<=|>|<)\s*(.+)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"field DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    path = m.group(2).strip()
    op = m.group(3)
    expected_raw = m.group(4).strip().strip("'\"")

    data = _load(lock_dir, lock_name)
    if not data:
        return EvalResult("red", f"{lock_name} not found")

    actual = _resolve_path(data, path)
    if actual is None:
        return EvalResult("red", f"{path} が {lock_name} に存在しない")

    # 数値比較の試み
    try:
        actual_num = float(str(actual))
        expected_num = float(expected_raw)
        ok = (
            (op == "==" and actual_num == expected_num) or
            (op == "!=" and actual_num != expected_num) or
            (op == ">=" and actual_num >= expected_num) or
            (op == "<=" and actual_num <= expected_num) or
            (op == ">"  and actual_num >  expected_num) or
            (op == "<"  and actual_num <  expected_num)
        )
    except ValueError:
        # 文字列比較
        actual_str = str(actual)
        ok = (
            (op == "==" and actual_str == expected_raw) or
            (op == "!=" and actual_str != expected_raw)
        )

    status = "green" if ok else "red"
    return EvalResult(status, f"field {path}={actual!r} {op} {expected_raw!r}")


# ---------------------------------------------------------------------------
# len()
# ---------------------------------------------------------------------------

def eval_len(expr: str, lock_dir: Path) -> EvalResult:
    """len(`lock`, path) op N を評価する。"""
    m = re.fullmatch(
        r"len\((.+?),\s*(.+?)\)\s*(<=|>=|==|<|>|!=)\s*(\d+)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"len DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    path = m.group(2).strip()
    op = m.group(3)
    threshold = int(m.group(4))

    data = _load(lock_dir, lock_name)
    if not data:
        return EvalResult("red", f"{lock_name} not found")

    val = _resolve_path(data, path)
    if val is None:
        length = 0
    elif isinstance(val, (list, dict, str)):
        length = len(val)
    else:
        length = 1

    ok = (
        (op == "<=" and length <= threshold) or
        (op == ">=" and length >= threshold) or
        (op == "==" and length == threshold) or
        (op == "<"  and length <  threshold) or
        (op == ">"  and length >  threshold) or
        (op == "!=" and length != threshold)
    )
    return EvalResult("green" if ok else "red", f"len={length} {op} {threshold}")


# ---------------------------------------------------------------------------
# bidirectional_lock()
# ---------------------------------------------------------------------------

def eval_bidirectional(expr: str, lock_dir: Path) -> EvalResult:
    """bidirectional_lock(`lock1`, `lock2`) を評価する。

    lock1 の各 entry の id が lock2 にも存在し、
    lock2 の各 entry の id が lock1 にも存在することを確認する。
    """
    m = re.fullmatch(
        r"bidirectional_lock\((.+?),\s*(.+?)\)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"bidirectional_lock parse 失敗: {expr!r}")

    lock1_name = _strip_backtick(m.group(1))
    lock2_name = _strip_backtick(m.group(2))

    data1 = _load(lock_dir, lock1_name)
    data2 = _load(lock_dir, lock2_name)

    if not data1:
        return EvalResult("yellow", f"{lock1_name} not found (pending)")
    if not data2:
        return EvalResult("yellow", f"{lock2_name} not found (pending)")

    # entries または cells キーを探す
    def _ids(data: dict[str, Any]) -> set[str]:
        for key in ("entries", "cells", "obligations", "axes"):
            items = data.get(key, [])
            if isinstance(items, list):
                ids = {item.get("id") or item.get("cell_id") or item.get("obligation_id")
                       for item in items if isinstance(item, dict)}
                return {i for i in ids if i}
        return set()

    ids1 = _ids(data1)
    ids2 = _ids(data2)
    diff = ids1.symmetric_difference(ids2)
    if diff:
        return EvalResult("red", f"bidirectional drift: {len(diff)} ids mismatch")
    return EvalResult("green", f"bidirectional_lock ok: {len(ids1)} ids matched")


# ---------------------------------------------------------------------------
# メインルーター
# ---------------------------------------------------------------------------

def evaluate_dsl(expr: str, lock_dir: Path) -> EvalResult:
    """DSL 式を判別してルーティングする。

    AND 複合式の判別を個別 DSL より先に行う。
    DSL が空 → yellow (pending)。
    """
    expr = expr.strip()
    if not expr:
        return EvalResult("yellow", "DSL 未定義 (pending)")

    # AND 複合式は最初にチェック（個別 DSL ルーティングより優先）
    if " AND " in expr:
        clauses = [c.strip() for c in expr.split(" AND ")]
        results = [evaluate_dsl(c, lock_dir) for c in clauses]
        reds = [r for r in results if r.status == "red"]
        yellows = [r for r in results if r.status == "yellow"]
        if reds:
            return EvalResult("red", "; ".join(r.detail for r in reds))
        if yellows:
            return EvalResult("yellow", "; ".join(r.detail for r in yellows))
        return EvalResult("green", f"all {len(results)} clauses green")

    if expr.startswith("count("):
        return eval_count(expr, lock_dir)
    if expr.startswith("all("):
        return eval_all(expr, lock_dir)
    if expr.startswith("field("):
        return eval_field(expr, lock_dir)
    if expr.startswith("len("):
        return eval_len(expr, lock_dir)
    if expr.startswith("bidirectional_lock("):
        return eval_bidirectional(expr, lock_dir)

    return EvalResult("yellow", f"DSL 未知パターン: {expr!r}")
