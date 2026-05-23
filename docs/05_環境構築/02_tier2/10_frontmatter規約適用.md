---
id: env.tier2.tier2_frontmatter
axis: tier2
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.tier2.tier2_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- frontmatter は機械への宣言であり、`id` は配置パスから一意に導出できる必要がある。tier2 エンジニアは docs 編集のたびに全 lint が green であることを確認してから push する規律を持つ。

## id 導出規則

```
docs/05_環境構築/02_tier2/<slug>.md
  → id: env.tier2.<slug_normalized>
```

slug は小文字 ASCII + `_` のみ。日本語ファイル名からの変換例:

```
01_責務とスコープ.md → tier2_responsibility_scope
07_テスト検証環境.md → tier2_test_environment
```

## 7 required field

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。1 つでも欠けると run_lint.sh / run_lint.py 双方で FAIL。

## 8 forbidden field

`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` / `version` / `created_at` / `tags`。これらを frontmatter に含めると CI が FAIL する。

## tier2 ドキュメントの frontmatter テンプレート

```yaml
---
id: env.tier2.tier2_<slug>
axis: tier2
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

## 検収コマンド

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P2 完了後に有効。

```bash
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
