"""tools/docs_lint/frontmatter_validator.py

適合仕様 frontmatter の SemVer / status enum / version monotonicity 検証モジュール。
run_lint.py から利用されるほか、単体テスト可能な純関数を提供する。
"""
# Python 3.10 未満の型ヒント互換のため annotations をインポート
from __future__ import annotations
# 正規表現モジュールのインポート
import re

# SemVer パターン（major.minor.patch 形式のみ許可）
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+$")
# status フィールドの有効な enum 値セット
STATUS_ENUM = frozenset({"draft", "published", "locked"})


def validate_status(status: str) -> bool:
    """status フィールドが有効な enum 値かを検証する。"""
    # STATUS_ENUM に含まれる値のみ True を返す
    return status in STATUS_ENUM


def validate_version(version: str) -> bool:
    """version フィールドが SemVer 形式かを検証する。"""
    # SEMVER_RE パターンにマッチする場合のみ True を返す
    return bool(SEMVER_RE.match(str(version)))


def validate_version_monotonicity(versions: list[str]) -> bool:
    """version が単調増加かを検証する（同一 file の履歴ではなく複数 file 列）。
    単純な lexicographic 比較で十分（major.minor.patch の数値比較）。
    空リストは True とする。
    """
    # タプル形式にパースした version を格納するリスト
    parsed = []
    # 各 version 文字列を検証してタプルに変換
    for v in versions:
        # SemVer 形式でない場合は即座に False を返す
        if not validate_version(v):
            return False
        # "major.minor.patch" を整数タプル (major, minor, patch) に変換
        parts = tuple(int(x) for x in v.split("."))
        parsed.append(parts)
    # 隣接する要素間の単調増加を検証
    for i in range(1, len(parsed)):
        # 前の version より小さい場合は単調増加違反
        if parsed[i] < parsed[i - 1]:
            return False
    # 全要素が単調増加であれば True を返す
    return True
