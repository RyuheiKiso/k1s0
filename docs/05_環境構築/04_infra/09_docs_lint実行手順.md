---
id: env.infra.infra_docs_lint
axis: infra
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.infra.infra_lint_format
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# docs_lint 実行手順

## 一文方針

- bash 版（run_lint.sh）と Python 版（run_lint.py）の 2 スクリプトを両方実行し、それぞれ全 check が green になることを infra 軸ドキュメントの push 前条件とする。

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

Python 版は bash 版の 7 check に加え、循環依存検出と repository layout 検査を追加する。

## infra 軸ドキュメントに特有の注意点

infra 軸のドキュメントは `docs/05_環境構築/04_infra/` 配下に配置する。id は `env.infra.<slug>` パターンで統一する。manifest ファイル（`.yaml`）へのリンクは lint の body link check 対象外のため安全。

```bash
# infra 軸のみの FAIL を確認
python3 tools/docs_lint/run_lint.py 2>&1 | grep -E "FAIL.*(04_infra|env\.infra)"
```

## よくある FAIL と対処

| FAIL メッセージ | 原因 | 対処 |
|---|---|---|
| `frontmatter 不在` | frontmatter 先頭 `---` がない | ファイル先頭に `---\n` を追加 |
| `id prefix 'xxx' != expected 'env'` | 05_環境構築/ 配下で id が `env.` で始まっていない | id を `env.infra.<slug>` に修正 |
| `depends_on 'xxx' 解決不能` | 依存先 id が存在しない | 依存先ファイルを先に作成 |
| `dangling link: xxx.md` | body 中のリンクが存在しないファイルを指している | リンク先を作成またはリンクを修正 |

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
