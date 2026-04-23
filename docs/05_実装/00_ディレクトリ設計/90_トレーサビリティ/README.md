# 90. トレーサビリティ

本章はディレクトリ設計（IMP-DIR-\*）と、既存設計（DS-SW-COMP-\*）・ADR・要件定義（NFR-\* / DX-\*）との対応関係を明示する。

## 本章の目的

実装フェーズのディレクトリ設計を導入するにあたり、「どの設計 ID が既存のどの設計と紐付くか」「どの ADR に基づくか」「どの要件を満たすか」の 4 次元トレーサビリティを保持する。これによりレビュー時・改訂時の影響範囲を追跡可能にする。

## 本章の構成

| ファイル | 内容 |
|---|---|
| 01_IMP-DIR_ID一覧.md | IMP-DIR-\* 全 ID の索引 |
| 02_DS-SW-COMP_121-135_との対応.md | DS-SW-COMP-120-135 との対応関係（改訂影響） |
| 03_ADR_との対応.md | ADR との対応表 |
| 04_要件定義_NFR_DX_との対応.md | NFR-\* / DX-\* 要件 ID との対応表 |

## 本章に含めないもの

実装コード中の CODEOWNERS / .gitattributes 等の運用ファイル実体は含めず、それらは `docs/05_実装/00_ディレクトリ設計/` 他章で記述されたサンプルコードを参照する。

## 対応 IMP-DIR ID 範囲

本章自体は設計 ID 対象外（トレーサビリティ章は「索引」のため）。他章で採番した ID を整理・検証するメタ章。

## 自動検証（CI）

トレーサビリティと相互リンクの整合性は、以下の自動検査で常時担保する。手動レビュー任せにすると、改訂時の link rot や ID 抜け落ちを見逃すため、Phase 1a 開始までに `.github/workflows/docs-lint.yaml` として組み込む。

### lychee による相対リンク検査

`docs/05_実装/00_ディレクトリ設計/` 配下の全 Markdown が他ファイル・図への相対リンクを大量に持つため、PR 時に [lychee](https://github.com/lycheeverse/lychee) を実行して壊れたリンクを検出する。`mdbook-linkcheck` 相当の機能を Markdown native に提供し、Rust 製のため CI 上の起動が高速。

```yaml
# .github/workflows/docs-lint.yaml（抜粋）
- name: Link check (docs)
  uses: lycheeverse/lychee-action@v2
  with:
    args: >-
      --no-progress
      --base ./docs
      --exclude-path docs/99_壁打ち
      './docs/**/*.md'
    fail: true
```

検出対象:

- 他 Markdown への相対リンクが存在しない（typo / リネーム見落とし）
- `img/*.svg` / `img/*.drawio` の相対参照が存在しない（drawio 編集後に SVG エクスポート漏れ）
- `../../../04_概要設計/` 等の親階層またぎリンク先が存在しない（`04_概要設計` 配下のリファクタ時に検出される）

### IMP-DIR ID 一覧との整合検査

`tools/ci/check-imp-dir-ids.sh`（Phase 1a で配置）で以下を検査する。grep 抽出と `01_IMP-DIR_ID一覧.md` の表との照合により、文書本文に存在する ID と一覧表が一致することを保証する。

```bash
# tools/ci/check-imp-dir-ids.sh（概要）
# 1. 全 .md から IMP-DIR-(ROOT|T1|T2|T3|INFRA|OPS|COMM|SPARSE)-NNN を抽出
# 2. 01_IMP-DIR_ID一覧.md の表セルと突き合わせ
# 3. 文書本文に出現するが一覧表にない ID（採番漏れ）を fail
# 4. 一覧表にあるが文書本文に出現しない ID（実装漏れ・予約は exempt）を warning
```

### drawio / SVG 対応検査

`tools/ci/check-drawio-svg-pair.sh`（Phase 1a で配置）で `*.drawio` と同名 `*.svg` の存在ペアを検査する。drawio を編集したまま SVG エクスポートを忘れた場合に CI が落ちる。

これら 3 種の検査が組み合わさることで、本章「トレーサビリティ」の信頼性が継続的に担保される。
