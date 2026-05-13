---
id: env.ops.ops_docs_lint
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.ops_lint_format
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# docs_lint 実行手順

## 一文方針

- bash 版（run_lint.sh）と Python 版（run_lint.py）の 2 スクリプトを両方実行し、それぞれ全 check が green になることを手元での push 前条件とする。

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

Python 版は bash 版の 7 check に加え以下を追加する。

| check | 内容 |
|---|---|
| 4/8 | 循環依存検出（DFS による DAG 検査） |
| 8/8 | repository layout 検査（root allowlist / src/ 軸 allowlist） |

## ops 軸固有の FAIL と対処

| FAIL メッセージ | 原因 | 対処 |
|---|---|---|
| `id prefix 'ops' != expected 'env'` | 07_ops/ 配下で id が `env.ops.` で始まっていない | id を `env.ops.ops_<slug>` に修正 |
| `dangling link: 07_テスト検証環境.md` | リンク先ファイルが存在しない | ファイルを作成するかリンクを修正 |
| `depends_on 'env.ops.ops_xxx' 解決不能` | 依存先 id が存在しない | 依存先ファイルを先に作成する |

## EXCLUDE_PATTERNS の確認

```
docs/90_knowledge/ → 除外
docs/00_format/    → 除外
docs/README.md     → 除外（frontmatter 不要）
```

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
