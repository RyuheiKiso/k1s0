"""tools/lock_yaml_generator/lib/catalog_validator.py

catalog yaml SoT を用いた cross-validation ロジック。

各 tier1 generator が担当 catalog yaml を SoT input として読み込み、
「catalog で宣言された id 集合 ⊆ drill 実行された id 集合」を検証する。

catalog yaml が存在しない場合は graceful degradation し、
status=catalog_absent を返す（release_gate への影響なし）。

主要な公開関数:
  compute_sot_hash      — catalog yaml の SHA-256 を返す
  load_catalog          — catalog yaml を読み込む（不在時は None）
  validate_catalog_against_input — catalog 宣言 id vs drill 実行 id の包含関係検証
"""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required: pip install PyYAML")

# 検証結果のステータス定数
STATUS_PASSED = "passed"
STATUS_MISSING_DRILLS = "missing_drills"
STATUS_CATALOG_ABSENT = "catalog_absent"


def compute_sot_hash(catalog_path: Path) -> str:
    """catalog yaml ファイルの SHA-256 を "sha256:<hex>" 形式で返す。

    ファイルをバイト列として読み込み、SHA-256 を計算する。
    ファイルが存在しない場合は ValueError を送出する。

    Args:
        catalog_path: 対象 catalog yaml ファイルの絶対パス。

    Returns:
        "sha256:<64 hex chars>" 形式の文字列。

    Raises:
        ValueError: catalog_path が存在しない場合。
    """
    # ファイル存在確認
    if not catalog_path.exists():
        raise ValueError(f"catalog_path does not exist: {catalog_path}")
    # バイト列を読み込んで SHA-256 を計算する
    raw_bytes = catalog_path.read_bytes()
    hex_digest = hashlib.sha256(raw_bytes).hexdigest()
    return f"sha256:{hex_digest}"


def load_catalog(catalog_path: Path) -> dict[str, Any] | None:
    """catalog yaml を読み込む。存在しない場合は None を返す。

    graceful degradation のために、ファイルが存在しない場合や
    空ファイルの場合は None を返す。YAML パースエラーは伝播させる。

    Args:
        catalog_path: 対象 catalog yaml ファイルの絶対パス。

    Returns:
        catalog yaml の内容 dict、または None（ファイル不在・空の場合）。
    """
    # ファイルが存在しない場合は None を返す
    if not catalog_path.exists():
        return None
    # YAML テキストを読み込む
    text = catalog_path.read_text(encoding="utf-8")
    # 空ファイルまたはコメントのみの場合は None を返す
    parsed = yaml.safe_load(text)
    if parsed is None:
        return None
    return parsed


def _extract_catalog_ids(
    catalog: dict[str, Any],
    class_key: str,
    catalog_id_key: str = "id",
) -> list[str]:
    """catalog dict から class_key 下の id 集合を抽出する。

    catalog[class_key] が list の場合は各要素の catalog_id_key キーを取り出す。
    存在しない場合は空リストを返す。

    Args:
        catalog:         load_catalog() で得た dict。
        class_key:       catalog 内の class リストが格納されているキー名
                         （例: "conformance_classes", "signals", "pairs"）。
        catalog_id_key:  各エントリの id を格納しているキー名
                         （デフォルト "id"、migration/pairs.yaml は "pair_id"）。

    Returns:
        id 文字列のリスト。
    """
    # class_key に対応する値を取り出す
    entries = catalog.get(class_key, [])
    # リストでない場合は空リストとして扱う
    if not isinstance(entries, list):
        return []
    # 各エントリの catalog_id_key キーを取り出す（存在しない場合はスキップ）
    result: list[str] = []
    for entry in entries:
        if isinstance(entry, dict):
            raw_id = entry.get(catalog_id_key)
            if raw_id is not None:
                result.append(str(raw_id))
    return result


def validate_catalog_against_input(
    catalog: dict[str, Any] | None,
    input_data: dict[str, Any],
    class_key: str = "classes",
    input_id_key: str = "cell_id",
    input_list_key: str | None = None,
    catalog_path: Path | None = None,
    catalog_id_key: str = "id",
) -> dict[str, Any]:
    """catalog の classes に宣言された id 集合 ⊆ input の id 集合 を検証する。

    catalog が None の場合（ファイル不在）は status=catalog_absent を返す。
    catalog の class id が input の id 集合に全て含まれる場合は status=passed。
    欠落がある場合は status=missing_drills と missing リストを返す。

    Args:
        catalog:         load_catalog() の戻り値（None = catalog 不在）。
        input_data:      generator の load_inputs() が返す dict。
        class_key:       catalog 内の class リストキー名
                         （例: "conformance_classes", "signals", "pairs"）。
        input_id_key:    input_data 内の各エントリから id を取得するキー名
                         （例: "cell_id", "signal_id", "pair_id"）。
        input_list_key:  input_data 内のリストキー名
                         （None の場合は空リストとして扱う）。
        catalog_path:    catalog yaml ファイルのパス（hash 計算用、None でスキップ）。
        catalog_id_key:  catalog 各エントリの id キー名
                         （デフォルト "id"、migration/pairs.yaml は "pair_id"）。

    Returns:
        以下の形式の dict::

            {
                "status": "passed" | "missing_drills" | "catalog_absent",
                "missing": [...],        # catalog にあって drill されていない id
                "catalog_sot_hash": str | None,
            }
    """
    # catalog が存在しない場合は graceful degradation
    if catalog is None:
        return {
            "status": STATUS_CATALOG_ABSENT,
            "missing": [],
            "catalog_sot_hash": None,
        }

    # catalog の宣言 id 集合を取り出す（catalog_id_key を使用）
    declared_ids: list[str] = _extract_catalog_ids(catalog, class_key, catalog_id_key)

    # input の実行済み id 集合を取り出す
    if input_list_key is not None:
        # input_data[input_list_key] からリストを取得する
        raw_list: list[dict[str, Any]] = input_data.get(input_list_key, [])
    else:
        # input_list_key が None の場合は空リストとして扱う
        raw_list = []

    # input エントリから id を取り出す
    drilled_ids: set[str] = set()
    for entry in raw_list:
        if isinstance(entry, dict):
            raw_id = entry.get(input_id_key)
            if raw_id is not None:
                drilled_ids.add(str(raw_id))

    # catalog 宣言 id のうち drill されていない id を missing とする
    missing: list[str] = [
        cid for cid in declared_ids if cid not in drilled_ids
    ]

    # catalog yaml の SHA-256 を計算する
    sot_hash: str | None = None
    if catalog_path is not None:
        try:
            sot_hash = compute_sot_hash(catalog_path)
        except ValueError:
            # catalog_path が存在しない場合は hash なし
            sot_hash = None

    # 結果を返す
    if missing:
        status = STATUS_MISSING_DRILLS
    else:
        status = STATUS_PASSED

    return {
        "status": status,
        "missing": missing,
        "catalog_sot_hash": sot_hash,
    }


def build_catalog_metadata(
    classes_result: dict[str, Any],
    scenarios_result: dict[str, Any],
    classes_path: str,
    scenarios_path: str,
) -> dict[str, Any]:
    """classes と scenarios の cross-validation 結果から metadata dict を構築する。

    両方の validation 結果を統合して、lock yaml の metadata キーに埋め込む
    dict を返す。status は "passed" > "missing_drills" > "catalog_absent" の優先順で決定する。

    Args:
        classes_result:  validate_catalog_against_input() の classes 側戻り値。
        scenarios_result: validate_catalog_against_input() の scenarios 側戻り値。
        classes_path:    catalog classes yaml の repo 相対パス文字列（ドキュメント用）。
        scenarios_path:  catalog scenarios yaml の repo 相対パス文字列（ドキュメント用）。

    Returns:
        lock yaml の metadata キーに格納する dict。
    """
    # classes と scenarios の status を統合する
    # catalog_absent が両方の場合は catalog_absent
    # いずれか一方でも missing_drills なら missing_drills
    # 両方 passed なら passed
    c_status = classes_result["status"]
    s_status = scenarios_result["status"]

    if c_status == STATUS_PASSED and s_status == STATUS_PASSED:
        combined_status = STATUS_PASSED
    elif c_status == STATUS_CATALOG_ABSENT and s_status == STATUS_CATALOG_ABSENT:
        combined_status = STATUS_CATALOG_ABSENT
    elif STATUS_MISSING_DRILLS in (c_status, s_status):
        combined_status = STATUS_MISSING_DRILLS
    else:
        # 片方が absent、もう片方が passed の場合は passed として扱う
        combined_status = STATUS_PASSED

    # classes の sot_hash（classes が主要 catalog）
    classes_hash = classes_result.get("catalog_sot_hash")
    scenarios_hash = scenarios_result.get("catalog_sot_hash")

    # 代表 hash: classes の hash を優先する
    representative_hash = classes_hash or scenarios_hash

    # metadata dict を組み立てる
    metadata: dict[str, Any] = {
        "catalog_validation_status": combined_status,
        "catalog_sot_hash": representative_hash,
        "catalog_classes_path": classes_path,
        "catalog_scenarios_path": scenarios_path,
    }

    # missing があれば警告として追記する
    missing_classes = classes_result.get("missing", [])
    missing_scenarios = scenarios_result.get("missing", [])
    if missing_classes:
        metadata["catalog_classes_missing"] = missing_classes
    if missing_scenarios:
        metadata["catalog_scenarios_missing"] = missing_scenarios

    return metadata
