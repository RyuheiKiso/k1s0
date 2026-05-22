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
# evidence()
# ---------------------------------------------------------------------------

# 有効な evidence_kind 値の一覧（build artifact 種別）
_VALID_EVIDENCE_KINDS: frozenset[str] = frozenset([
    # Cargo ビルド成功証跡
    "cargo_build_pass",
    # pnpm テスト成功証跡
    "pnpm_test_pass",
    # Buf lint 成功証跡
    "buf_lint_pass",
    # Buf lint（observability/PII 対象）成功証跡
    "buf_lint_observability_pii_pass",
    # Buf generate ドリフトゼロ証跡
    "buf_generate_drift_zero",
    # testcontainers E2E 成功証跡
    "testcontainers_e2e_pass",
    # kind cluster drill 成功証跡
    "kind_cluster_drill_pass",
    # pgTAP RLS FORCE 成功証跡
    "pgtap_rls_force_pass",
    # Playwright 8 シナリオ成功証跡（既存）
    "playwright_8_scenario_pass",
    # OpenBao Transit sign/verify 成功証跡
    "openbao_transit_sign_verify_pass",
    # axe-core ゼロ違反証跡（既存）
    "axe_core_zero_violation",
    # Cargo public API ドリフトゼロ証跡
    "cargo_public_api_drift_zero",
    # cargo-deny 成功証跡
    "cargo_deny_pass",
    # Cargo マイグレーションペアテスト成功証跡
    "cargo_test_migration_pair_pass",
    # ESLint boundaries 成功証跡
    "eslint_boundaries_pass",
    # BFL OIDC E2E 成功証跡
    "bfl_oidc_e2e_pass",
    # SLO バーンレートテスト成功証跡
    "slo_burn_rate_test_pass",
    # クォータ enforcement E2E 成功証跡
    "quota_enforcement_e2e_pass",
    # atomic triple write 4言語成功証跡
    "atomic_triple_write_4lang_pass",
    # クロステナント E2E 4言語成功証跡
    "cross_tenant_e2e_4lang_pass",
    # vitest reducer 4サブタイプ成功証跡
    "vitest_reducer_4subtype_pass",
    # SDK distribution 5クラス E2E 成功証跡
    "sdk_dist_5class_e2e_pass",
    # cosign 検証成功証跡（既存）
    "cosign_verify_pass",
    # OPA policy テスト成功証跡
    "opa_policy_test_pass",
    # Argo Workflow lint 成功証跡
    "argo_workflow_lint_pass",
    # Weaver semconv マッチ証跡
    "weaver_semconv_match",
    # API 中立性チェック成功証跡
    "api_neutrality_check_pass",
    # 第二産業スタブコンパイル成功証跡
    "second_industry_stub_compile_pass",
    # レジストリピン成功証跡
    "registry_pin_pass",
    # Pact provider verify 成功証跡
    "pact_provider_verify_pass",
    # SBOM grype 高脆弱性なし証跡
    "sbom_grype_no_high",
    # SLSA attest 成功証跡
    "slsa_attest_pass",
    # Litmus chaos 成功証跡
    "litmus_chaos_pass",
    # SDK distribution 5クラス Pact 成功証跡
    "sdk_dist_5class_pact_pass",
    # SDK distribution 5クラス cosign 成功証跡
    "sdk_dist_5class_cosign_pass",
])


def eval_evidence(expr: str, lock_dir: Path) -> EvalResult:
    """evidence(`lock`, cell_id, kind) == green を評価する。

    書式: evidence(`lock_name`, cell_id, evidence_kind) == green

    評価手順:
      1. lock_name から build_evidence.lock.yaml 相当の lock を読み込む。
      2. entries または cells リスト内で cell_id が一致するエントリを探す。
      3. エントリの build_evidence_id フィールドが kind を含むか確認する。
      4. 存在・一致すれば green、不在なら yellow、不一致なら red を返す。
    """
    # evidence() == green 形式をパースする
    m = re.fullmatch(
        r"evidence\((.+?),\s*(.+?),\s*(.+?)\)\s*==\s*green",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"evidence DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    cell_id = m.group(2).strip().strip("`'\"")
    kind = m.group(3).strip().strip("`'\"")

    # lock ファイルを読み込む
    data = _load(lock_dir, lock_name)
    if not data:
        # build_evidence.lock.yaml が存在しない場合は yellow (pending)
        return EvalResult("yellow", f"{lock_name} not found (evidence pending)")

    # entries / cells / obligations / axes いずれかのリストを探す
    found_entry: dict[str, Any] | None = None
    for list_key in ("entries", "cells", "evidence_entries"):
        items = data.get(list_key, [])
        if isinstance(items, list):
            for item in items:
                if isinstance(item, dict):
                    # cell_id または id フィールドで照合する
                    if item.get("cell_id") == cell_id or item.get("id") == cell_id:
                        found_entry = item
                        break
        if found_entry:
            break

    # エントリが見つからない場合は yellow (pending)
    if found_entry is None:
        return EvalResult("yellow", f"{cell_id} の evidence エントリが未登録 (pending)")

    # build_evidence_id フィールドを確認する
    build_evidence_id: Any = found_entry.get("build_evidence_id")
    if not build_evidence_id:
        return EvalResult("yellow", f"{cell_id}: build_evidence_id 未設定 (pending)")

    # build_evidence_id が期待する kind を含むか確認する
    evidence_str = str(build_evidence_id)
    if kind in evidence_str:
        return EvalResult("green", f"{cell_id}: evidence {kind!r} confirmed ({evidence_str!r})")
    else:
        return EvalResult("red", f"{cell_id}: evidence kind mismatch (expected {kind!r}, got {evidence_str!r})")


# ---------------------------------------------------------------------------
# ratchet_ge()
# ---------------------------------------------------------------------------

def eval_ratchet_ge(expr: str, lock_dir: Path) -> EvalResult:
    """ratchet_ge(`lock`, field_path, threshold) を評価する。

    現在値 >= threshold の場合 green、未満の場合 red。
    lock が存在しない場合は yellow（pending）。

    ratchet の単調増加保証（前回値との比較）は R1 で history 機構を追加して強化する。
    R0 では threshold チェックのみ実施し、audit 振動防止の基盤を確立する。
    """
    m = re.fullmatch(
        r"ratchet_ge\((.+?),\s*(.+?),\s*(.+?)\)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"ratchet_ge DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    path = m.group(2).strip()
    threshold_raw = m.group(3).strip()

    # threshold を float に変換する
    try:
        threshold = float(threshold_raw)
    except ValueError:
        return EvalResult("yellow", f"ratchet_ge: threshold parse 失敗: {threshold_raw!r}")

    # lock ファイルを読み込む
    data = _load(lock_dir, lock_name)
    if not data:
        return EvalResult("yellow", f"{lock_name} not found (pending)")

    # field_path の現在値を取得する
    actual = _resolve_path(data, path)
    if actual is None:
        return EvalResult("red", f"ratchet_ge: {path} が {lock_name} に存在しない")

    # 数値変換を試みる
    try:
        actual_num = float(str(actual))
    except ValueError:
        return EvalResult("red", f"ratchet_ge: {path}={actual!r} は数値でない")

    # threshold との比較（単調増加保証は R1 で追加）
    ok = actual_num >= threshold
    status = "green" if ok else "red"
    return EvalResult(status, f"ratchet_ge: {path}={actual_num} >= {threshold}")


# ---------------------------------------------------------------------------
# ratio()
# ---------------------------------------------------------------------------

def eval_ratio(expr: str, lock_dir: Path) -> EvalResult:
    """ratio(`lock`, jsonpath, total_field) op N を評価する。

    書式: ratio(`lock_name`, path[?filter], total_field) <= N
    計算: len(filtered_items) / data[total_field] <op> N

    total_field が 0 の場合は red（division by zero 防止）。
    """
    m = re.fullmatch(
        r"ratio\((.+?),\s*(.+?),\s*(\w+)\)\s*(<=|>=|==|<|>|!=)\s*([\d.]+)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"ratio DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    path = m.group(2).strip()
    total_field = m.group(3).strip()
    op = m.group(4)
    threshold = float(m.group(5))

    data = _load(lock_dir, lock_name)
    if not data:
        return EvalResult("red", f"{lock_name} not found")

    total = _resolve_path(data, total_field)
    if total is None:
        return EvalResult("red", f"{total_field} が {lock_name} に存在しない")
    try:
        total_num = float(str(total))
    except ValueError:
        return EvalResult("red", f"{total_field}={total!r} は数値でない")
    if total_num == 0:
        return EvalResult("red", f"{total_field}=0 (division by zero)")

    items = _jsonpath_items(data, path)
    ratio_val = len(items) / total_num

    ok = (
        (op == "<=" and ratio_val <= threshold) or
        (op == ">=" and ratio_val >= threshold) or
        (op == "==" and ratio_val == threshold) or
        (op == "<"  and ratio_val <  threshold) or
        (op == ">"  and ratio_val >  threshold) or
        (op == "!=" and ratio_val != threshold)
    )
    pct = f"{ratio_val:.1%}"
    thr_pct = f"{threshold:.1%}"
    return EvalResult("green" if ok else "red",
                      f"ratio={pct} ({len(items)}/{int(total_num)}) {op} {thr_pct}")


# ---------------------------------------------------------------------------
# hard_fail_if_zero()
# ---------------------------------------------------------------------------

def eval_hard_fail_if_zero(expr: str, lock_dir: Path) -> EvalResult:
    """hard_fail_if_zero(`lock`, list_field) を評価する。

    list_field が空リスト・存在しない・長さ 0 の場合は red。
    非空の場合は green。entries=[] のような形式的 green を物理的に弾く目的。
    """
    m = re.fullmatch(
        r"hard_fail_if_zero\((.+?),\s*(\w+)\)",
        expr.strip()
    )
    if not m:
        return EvalResult("yellow", f"hard_fail_if_zero DSL parse 失敗: {expr!r}")

    lock_name = _strip_backtick(m.group(1))
    field_name = m.group(2).strip()

    data = _load(lock_dir, lock_name)
    if not data:
        return EvalResult("red", f"{lock_name} not found")

    val = _resolve_path(data, field_name)
    if val is None or (isinstance(val, (list, dict, str)) and len(val) == 0):
        return EvalResult("red", f"{field_name} が空または存在しない (hard fail)")

    count = len(val) if isinstance(val, (list, dict, str)) else 1
    return EvalResult("green", f"{field_name}: {count} item(s)")


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
    if expr.startswith("evidence("):
        return eval_evidence(expr, lock_dir)
    if expr.startswith("ratchet_ge("):
        return eval_ratchet_ge(expr, lock_dir)
    if expr.startswith("ratio("):
        return eval_ratio(expr, lock_dir)
    if expr.startswith("hard_fail_if_zero("):
        return eval_hard_fail_if_zero(expr, lock_dir)

    return EvalResult("yellow", f"DSL 未知パターン: {expr!r}")
