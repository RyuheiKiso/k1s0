# `tools/git-hooks/` — pre-commit hook 実装

本ディレクトリは k1s0 リポジトリの **commit 時ローカル検証** を担う pre-commit hook 群を保持する。CI ゲート（02-07 reusable workflow）の前段として、commit 前に違反を弾く即時フィードバック機構。

## 関連設計

- 計画: [`plan/02_開発環境整備/05_pre-commit_hooks有効化.md`](../../plan/02_開発環境整備/05_pre-commit_hooks有効化.md)
- 関連 ID: IMP-CI-008（pre-commit / CI 二重防御）
- 上位設定: [`/.pre-commit-config.yaml`](../../.pre-commit-config.yaml)

## 配置

```
tools/git-hooks/
├── README.md                        # 本ファイル
├── install.sh                       # `pre-commit install` のラッパ
├── japanese-header-guard.py         # src/ 配下の日本語ヘッダコメント必須
├── file-length-guard.py             # 1 ファイル 500 行以内
└── link-check-wrapper.py            # tools/_link_check.py を pre-commit から呼ぶラッパ
```

関連配置:

```
/.pre-commit-config.yaml             # pre-commit のルート設定
/.gitleaks.toml                      # gitleaks の許容 / 追加 rule
/.markdownlint-cli2.jsonc            # markdownlint-cli2 の rule 調整
tools/_link_check.py                 # 既存: docs/ md リンク全件検証
tools/_link_fix.py                   # 既存: ファイル移動後のリンク自動修復
tools/_export_svg.py                 # 既存: drawio → SVG 出力
```

## インストール（開発者向け）

```bash
# pre-commit が PATH 上にあることを確認（Dev Container postCreate.sh で導入される予定）
pre-commit --version

# hook を配置（pre-commit / commit-msg / pre-push の各 stage）
./tools/git-hooks/install.sh

# 配置 + 既存ファイルすべてを一度走査（初回検証）
./tools/git-hooks/install.sh --run-all

# hook を解除
./tools/git-hooks/install.sh --uninstall
```

## hook 一覧

| hook id | 役割 | 違反時の挙動 | 個別 skip |
|---|---|---|---|
| `trailing-whitespace` | 行末空白除去 | 自動修正 | `SKIP=trailing-whitespace` |
| `end-of-file-fixer` | ファイル末尾改行保証 | 自動修正 | `SKIP=end-of-file-fixer` |
| `check-yaml` / `check-json` / `check-toml` | 構文チェック | reject | 個別 skip |
| `check-added-large-files` | 1MB 超のファイル検出 | reject | — |
| `check-merge-conflict` | conflict marker 残存検出 | reject | — |
| `mixed-line-ending` | LF 強制 | 自動修正（`*.bat` 除く） | — |
| `markdownlint-cli2` | docs/ markdown 構文 | reject | `SKIP=markdownlint-cli2` |
| `gitleaks` | 秘密情報検出 | reject | （skip 非推奨） |
| `shellcheck` | シェルスクリプト静的解析 | reject（warning level） | `SKIP=shellcheck` |
| `japanese-header-guard` | src/ 配下のコードに日本語ヘッダコメント必須 | reject | `SKIP=japanese-header-guard` |
| `file-length-guard` | 1 ファイル 500 行以内（src/ infra/ deploy/ tools/ tests/） | reject | （skip 非推奨） |
| `link-check` | docs/ 内部リンク検証 | reject | `SKIP=link-check` |
| `drawio-svg-staleness` | `*.drawio` 更新時に対応 SVG が古い場合 reject | reject | `SKIP=drawio-svg-staleness` |

## 緊急時の hook 無効化

CI ゲート（02-07）が hook と同等の検査を `pre-commit run --all-files` で再実行するため、ローカル skip は CI で必ず検出される。それでも commit を急ぐ場合の手段:

```bash
# 単一 hook を skip
SKIP=markdownlint-cli2 git commit -m "..."

# 全 hook を skip（最終手段、PR 必須レビュー時に説明責任）
git commit --no-verify -m "..."
```

`--no-verify` を常用するのは Paved Road 違反。緊急 hot-fix 時のみとし、PR description で理由を述べること。

## 自作 hook の仕様

### `japanese-header-guard.py`

`src/CLAUDE.md` 規約「コードファイルの先頭には、必ず日本語でファイルの説明コメントを記載すること」の自動化。最低限「先頭の説明コメントブロックに CJK 文字（ひらがな / カタカナ / 漢字）が含まれること」を検証する。

- 対象拡張子: `.go` / `.rs` / `.cs` / `.ts` / `.tsx` / `.js` / `.jsx` / `.proto` / `.py` / `.sh`
- 自動生成（`Code generated` / `DO NOT EDIT` / `@generated` 等を含む）はスキップ
- shebang / SPDX-License-Identifier / Copyright ヘッダはスキップして説明コメントを探索
- 行ごとの日本語コメント要否（src/CLAUDE.md の本来規約）は AI レビューに委ねる

### `file-length-guard.py`

`src/CLAUDE.md` 規約「1 ファイルあたりの行数は 500 行以内とすること。いかなる例外も認めない」の自動化。

- 対象トップディレクトリ: `src/` `infra/` `deploy/` `tools/` `tests/`
- 例外: `docs/` `plan/` `examples/` `.github/` `third_party/`、`*.lock` / `*.sum` / package-lock.json 等
- 自動生成（先頭 5 行に `Code generated` 等）はスキップ
- `--limit` で上限を変更可能（default 500）

### `link-check-wrapper.py`

`tools/_link_check.py`（既存）を pre-commit から呼ぶラッパ。差分 md が `docs/` 配下にある場合のみ全件走査を起動する（差分 md 以外のリンクが影響を受ける可能性があるため、結局 full）。

## 関連

- pre-commit 設定: [`/.pre-commit-config.yaml`](../../.pre-commit-config.yaml)
- gitleaks 設定: [`/.gitleaks.toml`](../../.gitleaks.toml)
- markdownlint 設定: [`/.markdownlint-cli2.jsonc`](../../.markdownlint-cli2.jsonc)
- 既存 link-checker: [`/tools/_link_check.py`](../_link_check.py)
- src コーディング規約: [`/src/CLAUDE.md`](../../src/CLAUDE.md)
