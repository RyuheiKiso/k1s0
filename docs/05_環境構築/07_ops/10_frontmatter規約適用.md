---
id: env.ops.ops_frontmatter
axis: ops
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.ops.ops_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- frontmatter は機械への宣言であり、ops 軸エンジニアが作成する全ドキュメントの id は `env.ops.ops_<slug>` パターンに従い、7 required field を完備し 8 forbidden field を含まないことが CI 通過の絶対条件である。

## id 導出規則

```
docs/05_環境構築/07_ops/<slug>.md
  → id: env.ops.ops_<slug_normalized>
```

slug は小文字 ASCII + `_` のみ。日本語ファイル名からの変換例:

```
01_責務とスコープ.md        → ops_responsibility_scope
07_テスト検証環境.md        → ops_test_environment
11_軸固有環境設定.md        → ops_axis_specific
```

## 7 required field

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。1 つでも欠けると run_lint.sh / run_lint.py 双方で FAIL。

ops 軸の最小 frontmatter 例:

```yaml
---
id: env.ops.ops_<slug>
axis: ops
phase: env_setup
kind: <responsibility|policy|enforcement|convention|index>
status: draft
depends_on:
  - env.ops.ops_<prev_slug>
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---
```

## 8 forbidden field

`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` / `version` / `created_at` / `tags` は CI で即時 FAIL になる。ドキュメントの履歴管理は git log が担う。

## ops 軸の kind 使用方針

| kind | 使用するページ |
|---|---|
| `index` | README.md のみ |
| `responsibility` | 01_責務とスコープ |
| `policy` | 02_前提OS環境 / 03_必須ランタイム / 14_Claude_Code連携 |
| `enforcement` | 04〜09 / 11〜13 |
| `convention` | 10_frontmatter規約適用 |

## 検収コマンド

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P2 完了後に有効。

```bash
# 新規ファイルを作ったあとに必ず実行
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
- [docs/00_format/conventions/frontmatter.md](../../00_format/conventions/frontmatter.md)
