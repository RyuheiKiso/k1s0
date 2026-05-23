---
id: env.meta.author_docs_lint
axis: meta
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.meta.author_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# docs_lint 実行手順

## 一文方針

- bash 版（run_lint.sh）と Python 版（run_lint.py）の 2 スクリプトを両方実行し、それぞれ全 check が green になることを手元での push 前条件とする。

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P2 完了後に有効。

## bash 版: run_lint.sh（7 check）

```bash
bash tools/docs_lint/run_lint.sh
```

| check | 内容 |
|---|---|
| 1/7 | frontmatter schema 検査（required / forbidden / lock_artifacts pattern） |
| 2/7 | id 一意性検査 |
| 3/7 | id 導出整合（phase prefix のみ、簡易） |
| 4/7 | depends_on 参照整合 |
| 5/7 | cross-link dangling 検査 |
| 6/7 | 禁止表現検査（段階的 release 表現） |
| 7/7 | 空セクション / TBD 残存（status: locked のみ） |

## Python 版: run_lint.py（8 check）

```bash
source .venv/bin/activate
python3 tools/docs_lint/run_lint.py
```

Python 版は bash 版の 7 check に加え、以下を追加する。

| check | 内容 |
|---|---|
| 4/8 | 循環依存検出（DFS による DAG 検査、bash 版にはない） |
| 8/8 | repository layout 検査（root allowlist / src/ 軸 allowlist / _crosscutting slug pattern / img/ 拡張子） |

PyYAML 未インストールの場合、run_lint.py は `ERROR: PyYAML required. Install: pip install pyyaml` を出力して exit 2 する。

## EXCLUDE_PATTERNS の確認

```
docs/90_archive/ → 除外
docs/00_format/    → 除外
docs/README.md     → 除外（frontmatter 不要）
```

これらは lint 対象外であるため、90_archive や 00_format 配下に新規ドキュメントを置いても frontmatter なしで許容される。

## よくある FAIL と対処

| FAIL メッセージ | 原因 | 対処 |
|---|---|---|
| `frontmatter 不在 (先頭 --- なし)` | frontmatter 先頭 `---` がない | ファイル先頭に `---\n` を追加 |
| `required fields 不在: ['id']` | id フィールド欠損 | 7 required field を全て追加 |
| `id 重複` | 同じ id が複数ファイルにある | slug を変更して一意にする |
| `id prefix 'xxx' != expected 'env'` | 05_環境構築/ 配下のファイルで id が `env.` で始まっていない | id を `env.<axis>.<slug>` に修正 |
| `depends_on 'xxx' 解決不能` | 依存先 id が存在しない | 先に依存先ファイルを作成するか depends_on から削除 |
| `dangling link: xxx.md` | body 中のリンクが存在しないファイルを指している | リンク先ファイルを作成するかリンクを修正 |
| `循環依存検出` | depends_on が循環している | 依存方向を見直す |
| `root: 許可外ファイル: xxx` | root に想定外のファイルを置いた | ファイルを適切なディレクトリに移動 |

## 検収コマンド

```bash
bash tools/docs_lint/run_lint.sh 2>&1 | tail -1
# 期待: === docs_lint: 7 check 全 green ===

python3 tools/docs_lint/run_lint.py 2>&1 | tail -1
# 期待: === docs_lint (Python): 8 check 全 green ===
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [09_frontmatter規約適用](09_frontmatter規約適用.md)
- [12_CI完全再現](12_CI完全再現.md)
- [tools/docs_lint/run_lint.sh](../../../../tools/docs_lint/run_lint.sh)
- [tools/docs_lint/run_lint.py](../../../../tools/docs_lint/run_lint.py)
