---
id: env.formal.formal_frontmatter
axis: formal
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.formal.formal_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- formal 軸のドキュメントは `id: env.formal.<slug>` 形式を厳守し、7 required field を全て記載し、8 forbidden field を一切含まないことが lint green の必要条件である。

## id 導出規則（formal 軸）

```
docs/05_環境構築/10_formal/<slug>.md
  → id: env.formal.<slug_normalized>
```

slug 変換例:

```
README.md               → env.formal.formal_index
01_責務とスコープ.md    → env.formal.formal_responsibility_scope
02_前提OS環境.md        → env.formal.formal_os_prerequisite
03_必須ランタイム.md    → env.formal.formal_runtime
11_軸固有環境設定.md    → env.formal.formal_axis_specific
```

slug は小文字 ASCII + `_` のみ。日本語は含めない。

## 7 required field

```yaml
id: env.formal.formal_<slug>
axis: formal
phase: env_setup
kind: <responsibility|policy|enforcement|convention|index>
status: draft
depends_on:
  - <parent_id>
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
```

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。

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

## proof_classes の記法

formal 軸ドキュメントでは `covered_by.proof_classes` に適用する proof_class を列挙する。

```yaml
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: [tla_plus, dafny, lean4]
```

proof_classes の有効な値は `tla_plus` / `dafny` / `lean4` / `kani` / `cbmc` / `stainless`。

## 検収コマンド

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P2 完了後に有効。

```bash
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

FAIL が 0 件で `=== docs_lint (Python): 8 check 全 green ===` が表示されることを確認する。

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
- [docs/00_format/conventions/frontmatter.md](../../00_format/conventions/frontmatter.md)
