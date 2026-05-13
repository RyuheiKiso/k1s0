---
id: env.test.test_frontmatter
axis: test
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.test.test_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- frontmatter は機械への宣言であり、test 軸エンジニアが作成する全ドキュメントの id は `env.test.test_<slug>` パターンに従い、7 required field を完備し 8 forbidden field を含まないことが CI 通過の絶対条件である。

## id 導出規則

```
docs/05_環境構築/09_test/<slug>.md
  → id: env.test.test_<slug_normalized>
```

slug は小文字 ASCII + `_` のみ。日本語ファイル名からの変換例:

```
01_責務とスコープ.md        → test_responsibility_scope
07_テスト検証環境.md        → test_test_environment
11_軸固有環境設定.md        → test_axis_specific
```

## 7 required field

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。

test 軸の最小 frontmatter 例:

```yaml
---
id: env.test.test_<slug>
axis: test
phase: env_setup
kind: <responsibility|policy|enforcement|convention|index>
status: draft
depends_on:
  - env.test.test_<prev_slug>
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---
```

## 8 forbidden field

`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` / `version` / `created_at` / `tags` は CI で即時 FAIL になる。

## test 軸の kind 使用方針

| kind | 使用するページ |
|---|---|
| `index` | README.md のみ |
| `responsibility` | 01_責務とスコープ |
| `policy` | 02_前提OS環境 / 03_必須ランタイム / 14_Claude_Code連携 |
| `enforcement` | 04〜09 / 11〜13 |
| `convention` | 10_frontmatter規約適用 |

## 検収コマンド

```bash
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
- [docs/00_format/conventions/frontmatter.md](../../00_format/conventions/frontmatter.md)
