"""tools/docs_lint/dual_review_validator.py

src/formal/dual_review/*.dual_review.lock.yaml を
docs/00_format/dual_review_schema.yaml で全件 schema 検証する。

V0 旧形 5 件 (tier1_(auth|key_mgmt|slo|oss_lifecycle|tenant_capacity).dual_review.lock.yaml、
_tsp suffix を持たないもの) は scope 外として除外する。
これらの標準形への移行は別 plan で実施する。

exit 0 = 全件 schema 適合
exit 1 = 1 件以上 violation / no in-scope files
"""
from __future__ import annotations

import re
import sys
import pathlib
import yaml
import jsonschema
from jsonschema import Draft202012Validator

# リポジトリルート (tools/docs_lint/ の 2 段上)
REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent.parent

# schema 定義の場所
SCHEMA_PATH = REPO_ROOT / "docs/00_format/dual_review_schema.yaml"

# 対象 file glob
GLOB = "src/formal/dual_review/*.dual_review.lock.yaml"

# V0 旧形 5 件のファイル名パターン (_tsp suffix を持たない tier1_* の特定 5 件)
_V0_PATTERN = re.compile(
    r"^tier1_(auth|key_mgmt|slo|oss_lifecycle|tenant_capacity)\.dual_review\.lock\.yaml$"
)


def load_schema() -> dict:
    """docs/00_format/dual_review_schema.yaml を読み込んで dict として返す。"""
    return yaml.safe_load(SCHEMA_PATH.read_text(encoding="utf-8"))


def is_v0_scope_out(path: pathlib.Path) -> bool:
    """V0 旧形 5 件のいずれかであれば True を返す。"""
    return bool(_V0_PATTERN.match(path.name))


def validate_file(path: pathlib.Path, validator: Draft202012Validator) -> list[str]:
    """単一 file を schema で検証し、violation メッセージのリストを返す。"""
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return [f"{path.name}: YAML parse error: {exc}"]
    try:
        label = path.relative_to(REPO_ROOT)
    except ValueError:
        # REPO_ROOT 外 (テスト tmp_path 等) の場合はファイル名のみ使用
        label = path.name
    return [
        f"{label}: {err.message} (at {list(err.absolute_path)})"
        for err in validator.iter_errors(data or {})
    ]


def main() -> int:
    """validator のエントリポイント。exit 0 = 全件 OK / exit 1 = violation あり。"""
    schema = load_schema()
    validator = Draft202012Validator(schema)

    files = sorted(REPO_ROOT.glob(GLOB))
    in_scope = [f for f in files if not is_v0_scope_out(f)]
    excluded = [f for f in files if is_v0_scope_out(f)]

    if not in_scope:
        print(f"ERROR: no in-scope files matched {GLOB}", file=sys.stderr)
        return 1

    all_errors: list[str] = []
    violators = 0
    for f in in_scope:
        errs = validate_file(f, validator)
        if errs:
            violators += 1
            all_errors.extend(errs)

    excluded_note = f" ({len(excluded)} V0 legacy excluded from scope)" if excluded else ""
    print(f"checked: {len(in_scope)} files{excluded_note}, violations: {violators}")
    for e in all_errors:
        print(f"  - {e}", file=sys.stderr)
    return 1 if violators else 0


if __name__ == "__main__":
    sys.exit(main())
