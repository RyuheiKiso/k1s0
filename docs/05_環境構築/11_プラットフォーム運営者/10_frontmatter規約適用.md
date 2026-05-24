---
id: env.ops.platform_operator_frontmatter
axis: ops
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.ops.platform_operator_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- プラットフォーム運営者が関わる docs/ ドキュメントは `id: env.ops.platform_operator_<slug>` 形式を厳守し、7 required field を全て記載し、8 forbidden field を一切含まないことが lint green の必要条件である。

## id 導出規則（ops プラットフォーム運営者）

```
docs/05_環境構築/11_プラットフォーム運営者/<slug>.md
  → id: env.ops.platform_operator_<slug_normalized>
```

slug 変換例:

```
README.md               → env.ops.platform_operator_index
01_責務とスコープ.md    → env.ops.platform_operator_responsibility_scope
02_前提OS環境.md        → env.ops.platform_operator_os_prerequisite
11_軸固有環境設定.md    → env.ops.platform_operator_axis_specific
```

slug は小文字 ASCII + `_` のみ。日本語は含めない。

## 7 required field

```yaml
id: env.ops.platform_operator_<slug>
axis: ops
phase: env_setup
kind: <responsibility|policy|enforcement|convention|index>
status: draft
depends_on:
  - <parent_id>
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
```

## 8 forbidden field

以下のフィールドを frontmatter に含めると lint FAIL となる：

- `changelog`
- `last_updated`
- `last_modified`
- `author`
- `reviewers`
- `version`
- `created_at`
- `tags`

## 検収コマンド

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P2 完了後に有効。

```bash
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
- [docs/00_format/conventions/frontmatter.md](../../00_format/conventions/frontmatter.md)
