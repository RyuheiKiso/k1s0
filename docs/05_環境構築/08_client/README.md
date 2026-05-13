---
id: env.client.client_index
axis: client
phase: env_setup
kind: index
status: draft
depends_on:
  - env.overview.environment_setup_index
  - plan.development_team_structure
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# client 軸エンジニア 環境構築 index

## 一文方針

- client 軸エンジニアは 5 distribution_class（v1_full_native_with_companion / v1_legacy_dotnet_framework / v1_browser_spa_typescript / v1_thick_native_via_tauri / v1_thin_business_api_only）の SDK 配布管理 / SDK conformance test / browser matrix 対応の 3 責務を持ち、Tauri CLI / Playwright / dotnet 8 SDK が手元で動くことが環境構築の検収条件である。

## 至高路線における立ち位置

- client 軸の環境は「5 distribution_class が全て手元で smoke 起動できる状態」を到達目標とする。1 class だけ green では不合格であり、5 class 全件 smoke + Playwright browser matrix 3 ブラウザ起動成功が唯一の完了定義である。
- 14 ページは client 責務の 14 側面に 1:1 対応し、それぞれが独立した検収コマンドを持つ。一括ではなくページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: Rust / Node / .NET 全ランタイムが揃ってから 05_主要OSS導入に進む。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | client ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / Windows 11 / Rust toolchain | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | Node 20 / Rust / .NET 8 / Tauri CLI / Playwright | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | Tauri CLI / Playwright / WiX Toolset | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + Tauri / .NET / Playwright extension | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | 5 distribution_class smoke / browser matrix | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | ESLint / cargo clippy / dotnet format | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh + run_lint.py | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | 5 class 全件 smoke / SDK conformance cadence | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | Tauri / Playwright / browser matrix 合否 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / memory / skills | policy |

## 検収条件

以下を全て満たすまで「client 軸エンジニアの環境構築完了」とはならない。

1. `bash tools/docs_lint/run_lint.sh` → exit 0
2. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`
3. `cargo tauri --version` が応答する
4. `pnpm playwright --version` が応答する
5. Playwright browser matrix（Chrome + Firefox + Webkit）3 ブラウザ起動成功
6. SDK conformance test pass

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): client 軸エンジニアロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
