---
id: env.tier1.tier1_frontmatter
axis: tier1
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.tier1.tier1_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- frontmatter は機械への宣言であり、`id` は配置パスから一意に導出できる必要がある。tier1 エンジニアは docs 編集のたびに全 lint が green であることを確認してから push する規律を持つ。

## id 導出規則

```
docs/05_環境構築/01_tier1/<slug>.md
  → id: env.tier1.<slug_normalized>
```

slug は小文字 ASCII + `_` のみ。日本語ファイル名からの変換例:

```
01_責務とスコープ.md → tier1_responsibility_scope
03_必須ランタイム.md → tier1_runtime
```

## 7 required field

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。1 つでも欠けると run_lint.sh / run_lint.py 双方で FAIL。

## 8 forbidden field

`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` / `version` / `created_at` / `tags`。これらを frontmatter に含めると CI が FAIL する。

## tier1 ドキュメントの frontmatter テンプレート

```yaml
---
id: env.tier1.tier1_<slug>
axis: tier1
phase: env_setup
kind: <responsibility|policy|enforcement|convention|index>
status: draft
depends_on:
  - <依存先 id>
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---
```

## status: locked の追加制約

`status: locked` のドキュメントに `TBD` / `未定` / `後述のみ` が残っていると run_lint.py check 7/8 で FAIL。locked にする前にこれらを削除すること。

## 検収コマンド

```bash
# 新規ファイルを作ったあとに必ず実行
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
