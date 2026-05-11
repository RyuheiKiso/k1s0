# tools/lock_yaml_generator

build artifact `*.lock.yaml` の generator script skeleton + sample artifact。

## 一文方針
- 各軸の `*.lock.yaml`（`capabilities.lock.yaml` / `dry_run.lock.yaml` / `instruments.lock.yaml` / `oss_inventory.lock.yaml` / `enforcement_points.lock.yaml` / `migration.lock.yaml` / `conflict_tree.lock.yaml` / `sdk_inventory.lock.yaml` / `capability_matrix.lock.yaml` / `sdk_conformance.lock.yaml` / `proof_*.lock.yaml` / `release_gate.lock.yaml`）を build artifact として生成する script の skeleton + 1 sample（`release_gate.lock.yaml`）を提供する。

## 構成
- `generate_release_gate.py`: `release_gate.lock.yaml` 生成 skeleton（各軸の lock.yaml + cosign 署名 を AND-gate）
- `samples/release_gate.lock.yaml`: 1.0.0 ship blocker AND-gate sample（all cells red 初期値）

## 1.0.0 ship blocker
- 全 19 軸の cell が green
- cosign signed tag が物理 prerequisite
- `meta.docs_lint_green` cell（[tools/docs_lint](../docs_lint/README.md)）green
- `meta.release_gate_dual_signoff_complete` cell（dual reviewer cosign signature）

## 関連参照
- [release_gate 体系](../../docs/04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)
- [artifact_lock 命名規約](../../docs/04_詳細設計/05_lock_yaml体系/04_artifact_lock命名規約.md)
- [immutable_archive 体系](../../docs/04_詳細設計/05_lock_yaml体系/05_immutable_archive体系.md)
