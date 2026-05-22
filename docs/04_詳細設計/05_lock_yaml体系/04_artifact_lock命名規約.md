---
id: detail.lock_yaml.artifact_lock_naming_convention
axis: meta
phase: detail
kind: policy
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
trace:
  fr_ids:
  - FR-meta-005

---

# artifact lock 命名規約

## 一文方針
- 全 build artifact lock ファイルは `^[a-z][a-z0-9_]*\.lock\.yaml$` パターンで命名し、軸ごとに固定の lock 名を持つ。frontmatter `lock_artifacts` フィールドへの登録 + frontmatter_schema による物理 enforce + CI で参照整合性を検査する。

## 命名 pattern
- 正規表現: `^[a-z][a-z0-9_]*\.lock\.yaml$`
- 制約:
  - 先頭は小文字 alphabet
  - 文字種は小文字 alphabet / digit / underscore
  - 拡張子は `.lock.yaml` 固定
- 例:
  - `capabilities.lock.yaml`（tier1 Bidi 適合仕様）
  - `dry_run.lock.yaml`（tier1 移行 Pair 適合仕様）
  - `idp_capabilities.lock.yaml`（tier1 認証適合仕様）
  - `backends.lock.yaml`（tier1 鍵管理適合仕様）
  - `registries.lock.yaml`（tier1 スキーマ進化適合仕様）
  - `instruments.lock.yaml`（tier1 SLO 適合仕様）
  - `oss_inventory.lock.yaml`（tier1 OSS lifecycle 適合仕様）
  - `enforcement_points.lock.yaml`（tier1 テナント容量適合仕様）
  - `migration.lock.yaml`（tier2 テナント分離適合仕様）
  - `abac_opa.lock.yaml`（tier2 テナント分離適合仕様 — ABAC OPA policy artifact）
  - `api_neutrality.lock.yaml`（tier2 テナント分離適合仕様 — API neutrality check artifact）
  - `quota.lock.yaml`（tier2 テナント分離適合仕様 — テナント quota enforcement artifact）
  - `registry_pin.lock.yaml`（tier2 テナント分離適合仕様 — container registry pin artifact）
  - `scheduler_argo.lock.yaml`（tier2 テナント分離適合仕様 — Argo Workflows scheduler artifact）
  - `second_industry_stub.lock.yaml`（tier2 テナント分離適合仕様 — 第二業種 stub artifact）
  - `weaver.lock.yaml`（tier2 テナント分離適合仕様 — OTel Weaver semconv artifact）
  - `conflict_tree.lock.yaml`（tier3 クライアント状態適合仕様）
  - `forms_lint.lock.yaml`（tier3 クライアント状態適合仕様 — フォーム lint ルール artifact）
  - `notifications_property.lock.yaml`（tier3 クライアント状態適合仕様 — 通知 property test artifact）
  - `design_tokens_contrast.lock.yaml`（tier3 クライアント状態適合仕様 — デザイントークン contrast 検査 artifact）
  - `forbidden_export_symbols.lock.yaml`（tier3 クライアント状態適合仕様 — 公開シンボル禁止リスト artifact）
  - `sdk_inventory.lock.yaml` / `capability_matrix.lock.yaml` / `sdk_conformance.lock.yaml`（client クライアント SDK 配布適合仕様）
  - `proof_inventory.lock.yaml` / `proof_status.lock.yaml` / `counter_example.lock.yaml` / `proof_review.lock.yaml` / `proof_matrix.lock.yaml` / `assumption.lock.yaml` / `mathlib_pin.lock.yaml` / `tla_apalache_pin.lock.yaml` / `kani_cbmc_pin.lock.yaml`（formal 形式検証適合仕様）
  - `cross_cutting_registry.lock.yaml`（cross-cutting cluster bundle map）
  - `release_gate.lock.yaml`（meta-axis 軸登録適合仕様）

## frontmatter 規約
- 各適合仕様 .md の frontmatter `lock_artifacts` フィールドに lock ファイル名を列挙
- yaml list 形式
- 例:
  ```yaml
  lock_artifacts:
    - capabilities.lock.yaml
  ```

## frontmatter_schema による物理 enforce
- `00_format/frontmatter_schema.yaml` の `lock_artifacts` フィールドの正規表現 pattern: `^[a-z][a-z0-9_]*\.lock\.yaml$`
- pattern 違反は CI fail
- 例: `formal_classes.yaml`（`.lock.yaml` 拡張子なし）/ `Bidi.lock.yaml`（先頭大文字）/ `bidi-conformance.lock.yaml`（hyphen 含む）は不適合

## yaml ファイル と classes / scenarios の区別
- **build artifact（lock_artifacts に登録）**: `*.lock.yaml`、build script で生成、手書き禁止
- **軸 enum / catalog**: `classes.yaml` / `scenarios.yaml` / `phases.yaml` / `test_matrix.yaml` 等、手書き、軸の単一の真
- 区別の意図: 手書きと build artifact の混同を防ぐ

## _input.yaml 系ファイルの位置づけ

`_input.yaml` 系ファイルは `tools/lock_yaml_generator/` 配下に置く **generator の入力ファイル**（手書き）であり、`*.lock.yaml` build artifact とは明確に区別される。

### 役割と命名規則
- **命名パターン**: `<lock_stem>_input.yaml`（例: `capabilities_input.yaml`、`oss_inventory_input.yaml`）
- **配置先**: `tools/lock_yaml_generator/inputs/` または各軸 generator のサブディレクトリ
- **性質**: 手書き可・git 管理対象・frontmatter `lock_artifacts` には登録しない
- **用途**: generator スクリプトが `_input.yaml` を読み込み、validation・enrichment を加えて `*.lock.yaml` を生成する

### _input.yaml と .lock.yaml の対応関係

| _input.yaml（手書き入力） | 生成される .lock.yaml（build artifact） |
|---|---|
| `capabilities_input.yaml` | `capabilities.lock.yaml` |
| `oss_inventory_input.yaml` | `oss_inventory.lock.yaml` |
| `instruments_input.yaml` | `instruments.lock.yaml` |
| `proof_inventory_input.yaml` | `proof_inventory.lock.yaml` |

### 不変条件
- `_input.yaml` ファイルを `lock_artifacts` フィールドに登録することを禁止（build artifact ではないため）
- `_input.yaml` を `*.lock.yaml` 拡張子で命名することを禁止（pattern 違反として CI fail）
- generator を介さず `_input.yaml` から直接 `*.lock.yaml` へ手書きコピーすることを禁止

## CI 不変条件
- 全 .md の frontmatter `lock_artifacts` フィールドが pattern 適合
- 全 lock ファイルが build artifact として生成され、手書き drift がない（generated と git の差分検出）
- lock ファイルが参照されない場合、dead lock として CI fail
- frontmatter `lock_artifacts` フィールドに登録されない lock ファイルは CI fail

## 採用しない設計
- 命名 pattern の例外
- 手書き lock ファイル
- frontmatter `lock_artifacts` フィールドへの未登録 lock 配置
- yaml ファイル と lock ファイル の混在配置

## 関連参照
- [05_lock_yaml 体系 README](README.md)
- [proof_artifact 体系](01_proof_artifact体系.md)
- [counter_example 体系](02_counter_example体系.md)
- [release_gate 体系](03_release_gate体系.md)
- [immutable_archive 体系](05_immutable_archive体系.md)
