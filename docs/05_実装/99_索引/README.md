# 99. 索引

本章は `05_実装/` 全体の横断索引を提供する。各章で採番される IMP-\* 接頭辞別の ID 一覧、概要設計 ID（DS-SW-COMP-\*）との対応、ADR との対応、NFR との対応、Backstage カタログ（catalog-info.yaml）との対応を集約する。本章は `00_ディレクトリ設計/90_トレーサビリティ/` の IMP-DIR-\* 索引と並列関係にあり、両者の守備範囲を重複させない。

## 本章の位置付け

実装ドキュメントは 12 章に分散しており、それぞれが独立に IMP-\* ID を採番する（IMP-BUILD / IMP-CODEGEN / IMP-CI / IMP-DEP / IMP-DEV / IMP-OBS / IMP-REL / IMP-SUP / IMP-SEC / IMP-POL / IMP-DX / IMP-TRACE の 12 接頭辞）。横断的な問合せ（「この NFR を満たす実装 ID はどれか」「この ADR の影響を受ける章はどこか」）に答えるため、単一の索引ファイルが必要となる。本章は新規 ID 採番時の最終更新先であり、章内の変更は `00_ディレクトリ設計/90_トレーサビリティ/` と同様の運用で反映する。

本章は人間が読むためではなく、監査時と改訂影響範囲の特定時に参照される。したがって密度を優先し、冗長な散文は置かない。上流の `04_概要設計/80_トレーサビリティ/05_構想設計ADR相関マトリクス.md` と双方向で整合する。

![IMP-ID 体系 (12 接頭辞) と対応章の相関図](img/99_IMP-ID体系_12接頭辞.svg)

## OSS リリース時点での確定範囲

- リリース時点: IMP-\* 接頭辞別の ID 一覧 / ADR / NFR / DS-SW-COMP との対応表 / Backstage catalog 対応 / 整合性 CI（IMP-TRACE-CI-010〜019 = 台帳 grand total 検算・90_対応索引相互整合・孤立 ID 検出・採番重複検出・予約帯外採番検出 / pre-commit + GitHub Actions 二段検証）/ catalog-info.yaml スキーマ検証（IMP-TRACE-CAT-020〜029 = 必須属性 / annotation / lifecycle 許可リスト / owner Group 実在 / Scaffold 出力 bit 一致 / Off-Path 検出 / `ci-overall` 必須化）
- リリース時点 以降: 各章の ID 採番に追従して随時更新

## RACI

| 役割 | 責務 |
|---|---|
| 全章主担当（責任共有） | 採番時の索引更新 |
| DX（調整窓口 / C） | 索引整合性のレビュー、catalog-info.yaml 対応 |

## 節構成予定

```
99_索引/
├── README.md
├── 00_方針/                 # 索引運用原則 (IMP-TRACE-POL-001〜007)
├── 00_IMP-ID一覧/           # 接頭辞別（BUILD / CODEGEN / CI / DEP / DEV / OBS / REL / SUP / SEC / POL / DX / TRACE）
├── 10_ADR対応表/
├── 20_DS-SW-COMP対応表/
├── 30_NFR対応表/
├── 40_Backstage_catalog対応/
├── 50_整合性CI/             # 索引と参照網の自動検証 (IMP-TRACE-CI-010〜019)
├── 60_catalog-info検証/     # catalog-info.yaml スキーマ検証 (IMP-TRACE-CAT-020〜029)
└── 90_改訂履歴/
```

## IMP ID 予約

本章で採番する実装 ID は `IMP-TRACE-*`（予約範囲: IMP-TRACE-001 〜 IMP-TRACE-099）。

## 対応 ADR / 概要設計 ID / NFR

- ADR: 全 ADR 横断
- DS-SW-COMP: 全概要設計 ID 横断
- NFR: 全 NFR 横断

## 関連章

- 全 12 章（本章は索引）
- `00_ディレクトリ設計/90_トレーサビリティ/` — ディレクトリ設計単独の索引（IMP-DIR-\*）
- `04_概要設計/80_トレーサビリティ/` — 上流の ADR / 要件 / 設計 ID マトリクス
