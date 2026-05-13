---
id: env.meta.author_frontmatter_discipline
axis: meta
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.meta.author_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- frontmatter は機械への宣言であり、`id` は配置パスから一意に導出できる必要がある。作者は schema 変更のたびに `frontmatter_schema.yaml` / `run_lint.py` / `run_lint.sh` / `conventions/frontmatter.md` を同時に更新し、変更後に全 lint が green であることを確認する責務を持つ。

## id 導出規則

```
docs/<phase_dir>/<axis_or_subdir>/<slug>.md
  → id: <phase_short>.<axis>.<slug_normalized>
```

| path_prefix | phase_short |
|---|---|
| `01_企画/` | `plan` |
| `02_要件定義/` | `req` |
| `03_概要設計/` | `arch` |
| `04_詳細設計/` | `detail` |
| `05_環境構築/` | `env` |
| `00_format/` | `format` |

slug は小文字 ASCII + `_` のみ。日本語ファイル名からの変換例:

```
03_必須ランタイム.md → author_runtime
09_frontmatter規約適用.md → author_frontmatter_discipline
```

## 7 required field

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。1 つでも欠けると run_lint.sh / run_lint.py 双方で FAIL。

## 8 forbidden field

`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` / `version` / `created_at` / `tags`。`additionalProperties: false` により schema 検証段階で拒否。

## env_setup phase 拡張の作者責任

`05_環境構築/` 以外の新 phase を追加する場合、作者が以下を同時に変更する。

1. `docs/00_format/frontmatter_schema.yaml` の id pattern と phase enum
2. `tools/docs_lint/run_lint.py` の `PHASE_PREFIX_BY_PATH`
3. `tools/docs_lint/run_lint.sh` の case 文
4. `docs/00_format/conventions/frontmatter.md` の phase_short enum 説明

変更後 `python3 tools/docs_lint/run_lint.py` が green になることを確認してから PR を作る。

## status: locked の追加制約

`status: locked` のドキュメントに `TBD` / `未定` / `後述のみ` が残っていると run_lint.py check 7/8 で FAIL。1.0.0 ship blocker 対象のドキュメントを locked にする前にこれらを削除すること。

## 検収コマンド

```bash
# 新規ファイルを作ったあとに必ず実行
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [05_docs_lint実行手順](05_docs_lint実行手順.md)
- [10_crosslinkと依存グラフ](10_crosslinkと依存グラフ.md)
- [docs/00_format/conventions/frontmatter.md](../../00_format/conventions/frontmatter.md)
- [docs/00_format/frontmatter_schema.yaml](../../00_format/frontmatter_schema.yaml)
