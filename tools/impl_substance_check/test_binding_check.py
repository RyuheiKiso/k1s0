"""tools/impl_substance_check/test_binding_check.py

coverage_matrix.lock.yaml の 90 cell に対して、artifact_manifest.yaml が指す
source_path が repo 上に実在するか、last_verified_at が 90 days 以内かを検査する。

Stage 3 の exit criteria:
- 全 90 cell の artifact_pointer が存在する artifact_manifest.yaml を指す
- 全 artifact_manifest.yaml の artifacts[].source_path が実在する (file or dir)
- 全 cell の last_green_at が 90 days 以内 (stale 防止)
"""

from __future__ import annotations

import datetime
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required: pip install PyYAML")

AGE_LIMIT_DAYS: int = 90


@dataclass
class BindingViolation:
    cell_id: str
    rule: str
    detail: str

    def __str__(self) -> str:
        return f"[BINDING VIOLATION] {self.cell_id} / {self.rule}: {self.detail}"


@dataclass
class BindingResult:
    total_cells: int = 0
    passed_cells: int = 0
    violations: list[BindingViolation] = field(default_factory=list)

    @property
    def failed_cells(self) -> int:
        return len({v.cell_id for v in self.violations})

    def is_ok(self) -> bool:
        return len(self.violations) == 0


def _parse_dt(dt_str: str) -> datetime.datetime:
    """ISO8601 文字列を UTC datetime に変換する。"""
    # Python 3.10 以前の fromisoformat は Z を扱えないため手動で処理する
    s = dt_str.strip().rstrip("Z")
    if "+" in s:
        s = s.split("+")[0]
    return datetime.datetime.fromisoformat(s).replace(tzinfo=datetime.timezone.utc)


def _check_age(cell_id: str, dt_str: str) -> list[BindingViolation]:
    """last_green_at が AGE_LIMIT_DAYS 以内かを確認する。"""
    violations: list[BindingViolation] = []
    try:
        dt = _parse_dt(dt_str)
        now = datetime.datetime.now(tz=datetime.timezone.utc)
        age_days = (now - dt).days
        if age_days > AGE_LIMIT_DAYS:
            violations.append(BindingViolation(
                cell_id=cell_id,
                rule="age_limit",
                detail=f"last_green_at is {age_days} days old (limit={AGE_LIMIT_DAYS}): {dt_str}",
            ))
    except (ValueError, AttributeError) as exc:
        violations.append(BindingViolation(
            cell_id=cell_id,
            rule="age_parse_error",
            detail=f"cannot parse last_green_at {dt_str!r}: {exc}",
        ))
    return violations


def _check_manifest(
    cell_id: str,
    manifest_path: Path,
    repo_root: Path,
) -> list[BindingViolation]:
    """artifact_manifest.yaml を読み込んで source_path の実在確認を実施する。"""
    violations: list[BindingViolation] = []

    if not manifest_path.exists():
        violations.append(BindingViolation(
            cell_id=cell_id,
            rule="manifest_exists",
            detail=f"artifact_manifest.yaml not found: {manifest_path}",
        ))
        return violations

    try:
        manifest: dict[str, Any] = yaml.safe_load(manifest_path.read_text(encoding="utf-8")) or {}
    except yaml.YAMLError as exc:
        violations.append(BindingViolation(
            cell_id=cell_id,
            rule="manifest_parse",
            detail=f"YAML parse error in {manifest_path}: {exc}",
        ))
        return violations

    artifacts: list[dict[str, Any]] = manifest.get("artifacts") or []
    if not artifacts:
        violations.append(BindingViolation(
            cell_id=cell_id,
            rule="artifacts_nonempty",
            detail=f"artifacts list is empty in {manifest_path}",
        ))
        return violations

    for art in artifacts:
        source_path_str: str = art.get("source_path", "")
        if not source_path_str:
            violations.append(BindingViolation(
                cell_id=cell_id,
                rule="source_path_defined",
                detail=f"artifact {art.get('artifact_id', '?')} has no source_path",
            ))
            continue
        source_path = repo_root / source_path_str
        if not source_path.exists():
            violations.append(BindingViolation(
                cell_id=cell_id,
                rule="source_path_exists",
                detail=f"source_path not found: {source_path_str}",
            ))

    return violations


def check_all_cells(
    coverage_matrix_path: Path,
    repo_root: Path,
) -> BindingResult:
    """coverage_matrix.lock.yaml の全 cell を検査する。"""
    result = BindingResult()

    data: dict[str, Any] = yaml.safe_load(coverage_matrix_path.read_text(encoding="utf-8")) or {}
    cells: list[dict[str, Any]] = data.get("cells") or []
    result.total_cells = len(cells)

    for cell in cells:
        cell_id: str = cell.get("cell_id", "unknown")
        cell_violations: list[BindingViolation] = []

        # artifact_pointer の実在確認
        artifact_pointer_str: str = cell.get("artifact_pointer", "")
        if not artifact_pointer_str:
            cell_violations.append(BindingViolation(
                cell_id=cell_id,
                rule="artifact_pointer_defined",
                detail="artifact_pointer field is missing",
            ))
        else:
            manifest_path = repo_root / artifact_pointer_str
            cell_violations.extend(_check_manifest(cell_id, manifest_path, repo_root))

        # last_green_at の age 確認
        last_green_at: str = cell.get("last_green_at", "")
        if not last_green_at:
            cell_violations.append(BindingViolation(
                cell_id=cell_id,
                rule="last_green_at_defined",
                detail="last_green_at field is missing",
            ))
        else:
            cell_violations.extend(_check_age(cell_id, last_green_at))

        # drill_state の確認
        drill_state: str = cell.get("drill_state", "")
        if drill_state != "v1_verified_with_artifact_pointer":
            cell_violations.append(BindingViolation(
                cell_id=cell_id,
                rule="drill_state",
                detail=f"drill_state must be 'v1_verified_with_artifact_pointer', got {drill_state!r}",
            ))

        result.violations.extend(cell_violations)
        if not cell_violations:
            result.passed_cells += 1

    return result


def main(
    coverage_matrix_path: str | None = None,
    repo_root_path: str | None = None,
) -> int:
    """CLI エントリポイント。"""
    repo_root = Path(repo_root_path) if repo_root_path else Path(__file__).parent.parent.parent
    cm_path = (
        Path(coverage_matrix_path)
        if coverage_matrix_path
        else repo_root / "src/test/lock/coverage_matrix.lock.yaml"
    )

    if not cm_path.exists():
        print(f"ERROR: coverage_matrix.lock.yaml not found: {cm_path}")
        return 1

    result = check_all_cells(cm_path, repo_root)
    print(f"test_binding_check: {result.passed_cells}/{result.total_cells} cells passed")

    if result.violations:
        print(f"VIOLATIONS ({len(result.violations)}):")
        for v in result.violations:
            print(f"  {v}")
        return 1

    print("OK: all cells have valid artifact bindings")
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
