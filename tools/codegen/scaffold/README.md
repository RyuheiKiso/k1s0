# tools/codegen/scaffold — Scaffold CLI 補助ツール

`src/platform/scaffold/` の k1s0-scaffold CLI を運用する際に使う補助
スクリプトを集約する。テンプレート本体は `src/{tier2,tier3}/templates/` に
配置されており（IMP-CODEGEN-SCF-030〜037 / 設計書: `docs/05_実装/20_コード
生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md`）、本ディレクトリには
**snapshot 更新スクリプトなどの補助ツール**のみが入る。

## 配置スクリプト

| ファイル | 役割 |
|---|---|
| `regenerate-golden.sh` | `tests/golden/scaffold-outputs/<ServiceType>/expected.tar.gz` を再生成する。`skeleton/` 配下のテンプレート (.hbs) を意図的に変更した PR で実行し、`tests/golden/diff-tool/compare-outputs.sh` の差分検出と整合させる。 |

## CODEOWNERS

`tools/codegen/scaffold/` への変更は `.github/CODEOWNERS` で SRE + Security
の二重承認が必須（IMP-CODEGEN-SCF 設計書）。本ディレクトリのスクリプト
だけでなく、本ディレクトリを介して再生成される
`tests/golden/scaffold-outputs/` の expected.tar.gz も同レベルの審査対象。

## 関連

- 設計: `docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md`
- テンプレート本体: `src/tier2/templates/`, `src/tier3/templates/`
- CLI 本体: `src/platform/scaffold/`
- Golden test: `tests/golden/scaffold-outputs/`, `tests/golden/diff-tool/compare-outputs.sh`
