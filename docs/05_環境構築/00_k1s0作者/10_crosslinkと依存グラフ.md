---
id: env.meta.author_crosslink_dependency
axis: meta
phase: env_setup
kind: convention
status: draft
depends_on:
  - env.meta.author_frontmatter_discipline
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# crosslink と依存グラフ

## 一文方針

- `depends_on` は宣言的な DAG 依存関係であり、body 内のリンクと同期させる必要はないが dangling（存在しない id への参照）は CI fail になる。cyclic な depends_on も Python 版 lint で検出される。

## depends_on の役割

- あるドキュメントが別ドキュメントの内容を前提とする場合、`depends_on` に相手の `id` を列挙する。
- 空配列 `[]` 許容。
- 参照先 id が存在しない dangling は `run_lint.py` check 4/8 で FAIL。
- 循環（A → B → A）は `run_lint.py` check 4/8 の DFS 検出で FAIL（bash 版は非検出）。

## body リンクの規約

`[テキスト](相対パス)` 形式（リンク先は .md ファイル）のみ許容。以下は禁止。

| 禁止事項 | 理由 |
|---|---|
| 絶対パス `[text](/docs/...)` | cross-link dangling check が機能しない |
| アンカー付き絶対パス `[text](/docs/...#section)` | 同上 |
| 外部 URL をプロジェクト内リンクとして書く | check が pass できない |

body リンクが存在しないファイルを指している場合、`run_lint.py` check 5/8 で FAIL。

## 依存方向の原則

環境構築フォルダ内の depends_on の基本方向:

```
01_責務とスコープ
  ↑ 02_前提OS環境
  ↑ 03_必須ランタイム
    ↑ 04_リポジトリ取得手順
      ↑ 05_docs_lint実行手順
        ↑ 06_markdownlint_textlint適用
      ↑ 07_drawio作図ツール手順
      ↑ 08_lock_yaml生成器手順
      ↑ 09_frontmatter規約適用
        ↑ 10_crosslinkと依存グラフ（本ファイル）
          ↑ 11_19軸登録手順
            ↑ 12_CI完全再現
              ↑ 13_署名とReleaseGate
                ↑ 14_Claude_Code連携
```

## 循環依存の発生を防ぐ作業手順

1. 新規ドキュメントを作成する前に依存先ドキュメントを先に作成する（依存先が存在する状態にする）。
2. depends_on を書いたら `python3 tools/docs_lint/run_lint.py` で循環 + dangling を確認する。
3. 循環が検出されたら `循環依存検出: A -> B -> A` のメッセージを確認し、どちらかの depends_on から edge を削除する。

## 検収コマンド

```bash
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "循環|dangling|FAIL|green"
```

循環 / dangling が 0 件かつ `8 check 全 green` が出れば OK。

## 関連参照

- [09_frontmatter規約適用](09_frontmatter規約適用.md)
- [11_19軸登録手順](11_19軸登録手順.md)
- [docs/00_format/conventions/crosslink.md](../../00_format/conventions/crosslink.md)
