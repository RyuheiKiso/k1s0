# 10. ルートレイアウト

本章は k1s0 リポジトリの最上位（ルート）に直接置かれるディレクトリ・ファイルの配置を確定する。実装段階の開発者が `git clone` して最初に目にする階層であり、本章の構成が開発者体験の第一印象を決める。

## 本章の位置付け

[../00_設計方針/01_ディレクトリ設計原則.md](../00_設計方針/01_ディレクトリ設計原則.md) で定めた 7 軸の原則を、ルートレベルの具体的な配置に落とし込む。本章が確定したレイアウトは `src/` `infra/` `deploy/` `ops/` `tools/` `tests/` `examples/` `third_party/` `docs/` の 9 主要ディレクトリと、それらを補完するルート直下ファイル群で構成される。

## 構成

- [01_ルート直下ファイル.md](01_ルート直下ファイル.md) — .github / CLAUDE.md / README.md / LICENSE 等
- [02_src配下の層別分割.md](02_src配下の層別分割.md) — src/ 内部の contracts / tier1 / sdk / tier2 / tier3 / platform
- [03_横断ディレクトリ.md](03_横断ディレクトリ.md) — infra / deploy / ops / tools / tests / examples / third_party
- [04_設定ファイル配置規約.md](04_設定ファイル配置規約.md) — .gitattributes / CODEOWNERS / editorconfig 等
- [05_依存方向ルール.md](05_依存方向ルール.md) — tier3 → tier2 → (sdk ← contracts) → tier1 → infra

## 本章で採番する IMP-DIR ID

- IMP-DIR-ROOT-008（ルート直下ファイル一覧）— `01_ルート直下ファイル.md`
- IMP-DIR-ROOT-009（src 配下の層別分割）— `02_src配下の層別分割.md`
- IMP-DIR-ROOT-010（横断ディレクトリ）— `03_横断ディレクトリ.md`
- IMP-DIR-ROOT-011（設定ファイル配置規約）— `04_設定ファイル配置規約.md`
- IMP-DIR-ROOT-012（依存方向ルール）— `05_依存方向ルール.md`

原則レベル IMP-DIR-ROOT-001 〜 007 は [../00_設計方針/01_ディレクトリ設計原則.md](../00_設計方針/01_ディレクトリ設計原則.md) で採番済み。残り IMP-DIR-ROOT-013 〜 020 は 運用蓄積後で採番する予約枠。

## 本章の対応 ADR / 概要設計

- ADR-DIR-001 / ADR-DIR-002 / ADR-DIR-003
- DS-SW-COMP-119 / DS-SW-COMP-120（改訂後）

## 関連図

- [img/リポジトリルート全体ツリー.drawio](img/リポジトリルート全体ツリー.drawio)
- [img/依存方向一方向化.drawio](img/依存方向一方向化.drawio)
