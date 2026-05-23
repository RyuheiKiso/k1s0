---
id: env.formal.formal_docs_lint
axis: formal
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.formal.formal_lint_format
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# docs_lint 実行手順

## 一文方針

- formal 軸エンジニアは docs/ 配下のドキュメントに変更を加えた際、bash 版（run_lint.sh）と Python 版（run_lint.py）の両方を実行し、全 check が green になることを push 前の必須条件とする。

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

bash 版 7 check に加え、以下を追加する。

| check | 内容 |
|---|---|
| 4/8 | 循環依存検出（DFS による DAG 検査） |
| 8/8 | repository layout 検査（root allowlist / src/ 軸 allowlist / img/ 拡張子） |

## formal 軸固有の FAIL パターン

| FAIL メッセージ | 原因 | 対処 |
|---|---|---|
| `id prefix 'formal' != expected 'env'` | id が `formal.` で始まっている | `env.formal.<slug>` に修正する |
| `depends_on 'env.formal.formal_xxx' 解決不能` | 依存先ファイルが未作成 | 依存先ファイルを先に作成する |
| `dangling link: 05_主要OSS導入.md` | リンク先ファイルが存在しない | ファイルを作成するかリンクを修正する |

## 検収コマンド

```bash
bash tools/docs_lint/run_lint.sh 2>&1 | tail -1
# 期待: === docs_lint: 7 check 全 green ===

python3 tools/docs_lint/run_lint.py 2>&1 | tail -1
# 期待: === docs_lint (Python): 8 check 全 green ===
```

## 関連参照

- [08_lintとformat適用](08_lintとformat適用.md)
- [10_frontmatter規約適用](10_frontmatter規約適用.md)
- [12_CI完全再現](12_CI完全再現.md)
