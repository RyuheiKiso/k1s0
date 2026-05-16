---
id: env.meta.author_markdownlint_textlint
axis: meta
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.meta.author_docs_lint
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# markdownlint / textlint 適用

## 一文方針

- markdownlint と textlint は CI 未配線（echo placeholder 段階）だが、作者は手元での実行責任を持つ。lint 設定ファイルは `docs/00_format/linters/` に定義済みであり、CI 配線は作者が行う。

## 現状（2026-05-13 時点）

`.github/workflows/docs_lint.yml` の `release_gate_check` job は echo のみ（`run_lint.py` の `[8/8]` 参照）。markdownlint / textlint の CI runner は未作成。

## markdownlint のローカル適用

Node.js（LTS）と `markdownlint-cli` が必要。

```bash
npm install -g markdownlint-cli
markdownlint --config docs/00_format/linters/markdownlint.json 'docs/**/*.md'
```

主要 rule（markdownlint.json より）:

| Rule | 内容 |
|---|---|
| MD003 | 見出しスタイル: atx（`#` プレフィックス）のみ |
| MD004 | リストマーカー: dash（`-`）のみ |
| MD007 | リストインデント: 2 スペース |
| MD025 | H1 は 1 ファイルに 1 つ |
| MD040 | fenced code block に言語指定必須 |

## textlint のローカル適用

```bash
npm install -g textlint
textlint --config docs/00_format/linters/textlint.config.mjs 'docs/01_企画/**/*.md'
```

textlint はターゲットが `docs/01_企画/**` ～ `docs/00_format/**` であり、`docs/90_archive/**` と `*.tpl` は除外される。日本語の禁止表現（prh rules）を検査する。

## CI 配線の作者責務

markdownlint / textlint を CI に組み込む場合は `.github/workflows/docs_lint.yml` に新 step を追加し、`release_gate.lock.yaml` の該当 cell を echo から実運用に昇格させる。変更は dual sign-off 必須。

## 検収コマンド

```bash
markdownlint --version   # CLI が存在すること
# または
npx markdownlint-cli --version
```

実際の lint は対象ファイルが増えた段階で行う。インストールされているかの確認が本ページの検収。

## 関連参照

- [05_docs_lint実行手順](05_docs_lint実行手順.md)
- [docs/00_format/linters/markdownlint.json](../../../../docs/00_format/linters/markdownlint.json)
- [12_CI完全再現](12_CI完全再現.md)
