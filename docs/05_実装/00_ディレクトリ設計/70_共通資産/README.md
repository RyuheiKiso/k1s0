# 70. 共通資産

本章は横断ディレクトリ `tools/` `tests/` `examples/` `third_party/` `.devcontainer/` `tools/codegen/` の配置を確定する。特定の tier に属さず、tier 横断で共有される資産を集約する場所。

## 本章の対象

- **tools/**: 開発者ツール・横断スクリプト
- **tests/**: tier 横断 E2E / Contract / Integration / Fuzz テスト
- **examples/**: 雛形 CLI の実稼働版 Golden Path 実装例
- **third_party/**: 社内フォーク OSS / パッチ版 vendoring
- **.devcontainer/**: 役割別 Dev Container プロファイル
- **tools/codegen/**: buf / openapi / scaffold のコード生成ツール

## 本章の構成

| ファイル | 内容 |
|---|---|
| 01_tools配置.md | `tools/` 配下のサブディレクトリ |
| 02_tests配置.md | `tests/` 配下の横断テスト配置 |
| 03_examples配置.md | `examples/` 配下の Golden Path 例 |
| 04_third_party配置.md | `third_party/` 配下の OSS フォーク管理 |
| 05_devcontainer配置.md | `.devcontainer/` 役割別プロファイル |
| 06_codegen配置.md | `tools/codegen/` のコード生成ツール |

## 本章で採番する IMP-DIR ID

- IMP-DIR-COMM-111（tools 配置）— `01_tools配置.md`
- IMP-DIR-COMM-112（tests 配置）— `02_tests配置.md`
- IMP-DIR-COMM-113（examples 配置）— `03_examples配置.md`
- IMP-DIR-COMM-114（third_party 配置）— `04_third_party配置.md`
- IMP-DIR-COMM-115（devcontainer 配置）— `05_devcontainer配置.md`
- IMP-DIR-COMM-116（codegen 配置）— `06_codegen配置.md`

予約 IMP-DIR ID は `IMP-DIR-COMM-117` 〜 `IMP-DIR-COMM-125`（運用蓄積後で採番）。本章採番範囲は `IMP-DIR-COMM-111` 〜 `IMP-DIR-COMM-125`。
