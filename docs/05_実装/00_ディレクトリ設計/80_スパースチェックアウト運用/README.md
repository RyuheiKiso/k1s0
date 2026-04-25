# 80. スパースチェックアウト運用

本章は Git sparse-checkout を用いた役割別ワークスペース運用を確定する。ADR-DIR-003 で採用決定した「cone mode + partial clone + sparse index」を標準運用として位置付け、10 役割の cone 定義を全文掲載する。

## 本章の目的

k1s0 のモノレポは tier1/2/3 + infra + deploy + ops + tools + tests + examples + third_party + docs/ を包含し、リリース時点 段階でも 50 万行を超える想定。全開発者が全コードを checkout する必要はなく、以下の恩恵がある。

- IDE 起動時間の短縮（JetBrains IntelliJ、VS Code の初期 indexing が 10 倍速）
- ローカル git 操作の高速化（status / switch / log）
- 役割と関心事の明確化（tier1 Rust 開発者は tier2 の業務ロジックを見る必要がない）
- 誤編集の予防（cone に入ってないディレクトリは書換え不可）

## 本章の構成

| ファイル | 内容 |
|---|---|
| 01_cone_mode設計原則.md | cone mode 採用理由と設計原則 |
| 02_役割別cone定義.md | 10 役割の cone 定義全文 |
| 03_初期クローンとオンボーディング.md | 新規開発者のセットアップ手順 |
| 04_役割切替運用.md | 役割間の切替と複数役割兼任 |
| 05_CI戦略とpath_filter統合.md | GitHub Actions path-filter との統合 |
| 06_注意点と既知問題.md | submodule / LFS / 生成コード / Windows 問題 |
| 07_partial_clone_sparse_index.md | partial clone と sparse index の導入 |

## 本章で採番する IMP-DIR ID

- IMP-DIR-SPARSE-126（cone mode 設計原則）— `01_cone_mode設計原則.md`
- IMP-DIR-SPARSE-127（役割別 cone 定義）— `02_役割別cone定義.md`
- IMP-DIR-SPARSE-128（初期クローンとオンボーディング）— `03_初期クローンとオンボーディング.md`
- IMP-DIR-SPARSE-129（役割切替運用）— `04_役割切替運用.md`
- IMP-DIR-SPARSE-130（CI 戦略と path-filter 統合）— `05_CI戦略とpath_filter統合.md`
- IMP-DIR-SPARSE-131（注意点と既知問題）— `06_注意点と既知問題.md`
- IMP-DIR-SPARSE-132（partial clone / sparse index）— `07_partial_clone_sparse_index.md`

予約 IMP-DIR ID は `IMP-DIR-SPARSE-133` 〜 `IMP-DIR-SPARSE-145`（運用蓄積後で採番）。本章採番範囲は `IMP-DIR-SPARSE-126` 〜 `IMP-DIR-SPARSE-145`。

## 対応 ADR

- ADR-DIR-003（スパースチェックアウト cone mode 採用）
