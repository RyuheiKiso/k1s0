---
id: env.data.data_frontmatter
axis: data
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.data.data_docs_lint
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# frontmatter 規約適用

## 一文方針

- data 軸のドキュメントは `id: env.data.<slug>` / `axis: data` / `phase: env_setup` の 3 フィールドが固定であり、新規ページ作成のたびに `python3 tools/docs_lint/run_lint.py` で green を確認してから PR に含める。

## id 導出規則

```
docs/05_環境構築/05_data/<filename>.md
  → id: env.data.<slug>
```

slug は小文字 ASCII + `_` のみ。ファイル名からの変換例:

```
01_責務とスコープ.md        → data_responsibility_scope
03_必須ランタイム.md         → data_runtime
11_軸固有環境設定.md         → data_axis_specific
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

## data 軸固有の注意

- `defense_in_depth_layers` にはバックアップ・リストア関連は `[B]`、schema evolution 関連は `[A, B]` を設定することを推奨する。
- data 軸ドキュメントで SQL ファイル（`.sql`）へのリンクを書く場合は、lint の body link check 対象外のため自由に書ける。
- docker-compose.yml へのリンクも同様に lint 対象外。

## 検収コマンド

```bash
# 新規ファイルを作ったあとに必ず実行
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL|green"
```

## 関連参照

- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
