# tests/ci/path-filter-golden — path-filter 回帰テストデータ

`tools/ci/path-filter/filters.yaml` の 11 軸出力が、過去の典型的な changed-files
ケースに対して期待通り評価されることを保証する golden test 用データセット。

## 構造

各テストケースは 2 ファイル組:

- `<id>.json` — 変更ファイル一覧（`{"files": [...]}` 形式）
- `<id>.expected.json` — 期待 filter outputs（`{"<filter_key>": <bool>, ...}`）

## 命名規則

`<id>` は `PR-<3 桁番号>` を採用する。番号は採番台帳的な意味を持たず単なる識別子。
ケースの意図を明確にするため、`<id>.json` の冒頭で `_description` フィールドに 1 行説明を含めても良い（runner 側は `.files` のみ参照する）。

## 実行

```bash
tools/ci/path-filter/run-golden-test.sh           # 全件評価
tools/ci/path-filter/run-golden-test.sh --list    # ケース一覧
```

## CI

`.github/workflows/_reusable-test.yml` の golden job が PR 毎に呼ぶ前提
（IMP-CI-PF-035）。filters.yaml が変更された PR では、本データの期待出力を更新
することがレビュアー必須チェック項目になる。

## 期待値の更新方法

filters.yaml を意図的に変更した PR で expected が古くなった場合は:

1. `tools/ci/path-filter/run-golden-test.sh` を実行して FAIL したケースを特定
2. 各 FAIL ケースの actual 値を確認し、変更意図と一致するか手で精査
3. 一致するなら `<id>.expected.json` を新値で書き換える
4. 不一致なら filters.yaml の修正をやり直す

## カバレッジ方針

最低限のケースは以下を網羅する:

- 単一 tier 変更（tier1_rust / tier1_go / tier2_dotnet / tier2_go / tier3_web / tier3_native）
- contracts 単独変更（全 tier 強制ビルド経路）
- docs 単独変更（lint-only 経路）
- infra 単独変更
- sdk 単独変更
- platform 単独変更
- multi-tier 変更（典型的な BFF + SDK 共修正）
