---
id: env.security.security_frontmatter
axis: security
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.security.security_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- security 軸のドキュメントは `id: env.security.<slug>` / `axis: security` / `phase: env_setup` の 3 フィールドが固定であり、新規ページ作成のたびに `python3 tools/docs_lint/run_lint.py`（P2 deliverable） で green を確認してから PR に含める。

## id 導出規則

```
docs/05_環境構築/06_security/<filename>.md
  → id: env.security.<slug>
```

slug は小文字 ASCII + `_` のみ。ファイル名からの変換例:

```
01_責務とスコープ.md        → security_responsibility_scope
03_必須ランタイム.md         → security_runtime
11_軸固有環境設定.md         → security_axis_specific
```

## 7 required field

`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` が全て必須。1 つでも欠けると run_lint.sh / run_lint.py 双方で FAIL。

## 8 forbidden field

`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` / `version` / `created_at` / `tags`。これらを frontmatter に書いた場合は CI fail になる。

## kind の選択基準

| kind | 使用する場面 |
|---|---|
| `responsibility` | ロールの責務・非責務を定義するページ |
| `policy` | OS / ランタイム要件のような「守るべき条件」を定義するページ |
| `enforcement` | 実行コマンド・手順・検証が主体のページ |
| `convention` | 命名規則・パターン・規約を定義するページ |
| `index` | README.md（配下ドキュメント一覧） |

## security 軸固有の注意

- `defense_in_depth_layers` には penetration testing 関連は `[B, E]`、鍵管理関連は `[E]`、脅威モデル関連は `[A, B, E]` を設定することを推奨する。
- security ドキュメントでは secret の実際の値・鍵マテリアルを本文に記載しない。detect-secrets がスキャンするため、ダミー値も `<placeholder>` 形式にする。

## 検収コマンド

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P2 完了後に有効。

```bash
# 新規ファイルを作ったあとに必ず実行
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
