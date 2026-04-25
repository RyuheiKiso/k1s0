# 00. ディレクトリ設計

本章は k1s0 モノレポ全体のディレクトリ配置を実装段階確定版として固定する。`src/` 配下のソースコード一次ディレクトリに加え、`infra/` / `deploy/` / `ops/` / `tools/` / `tests/` / `examples/` / `third_party/` / `docs/` を含むリポジトリルート全体のレイアウトを規定し、スパースチェックアウト cone mode による役割別運用、CODEOWNERS による path-pattern ベースの責任分界、依存方向の一方向化を同時に保証する。

## 本章を読む順序

実装開始前の初回読者は以下の順で読むことを推奨する。

1. [00_設計方針/](00_設計方針/) — 設計原則・世界トップ企業事例・ID 体系・スパースチェックアウト概論
2. [10_ルートレイアウト/](10_ルートレイアウト/) — リポジトリ最上位の配置
3. [20_tier1レイアウト/](20_tier1レイアウト/) — tier1 層（リリース時点 確定範囲）
4. [30_tier2レイアウト/](30_tier2レイアウト/) / [40_tier3レイアウト/](40_tier3レイアウト/) — tier2 / tier3 層（リリース時点 / 採用後の運用拡大時 骨組み）
5. [50_infraレイアウト/](50_infraレイアウト/) / [60_operationレイアウト/](60_operationレイアウト/) — クラスタ素構成 / GitOps 配信 / 運用領域
6. [70_共通資産/](70_共通資産/) — tools / tests / examples / third_party
7. [80_スパースチェックアウト運用/](80_スパースチェックアウト運用/) — cone 定義と運用手順
8. [90_トレーサビリティ/](90_トレーサビリティ/) — IMP-DIR ID 索引・DS-SW-COMP との対応・ADR との対応

## 構成一覧

```
00_ディレクトリ設計/
├── README.md                        # 本ファイル
├── 00_設計方針/                     # 原則・事例・ID 体系・スパース概論
├── 10_ルートレイアウト/             # リポジトリルート
├── 20_tier1レイアウト/              # src/tier1 / src/contracts / src/sdk
├── 30_tier2レイアウト/              # src/tier2 （.NET / Go ドメインサービス）
├── 40_tier3レイアウト/              # src/tier3（Web / MAUI / BFF）
├── 50_infraレイアウト/              # infra（クラスタ素構成）
├── 60_operationレイアウト/          # deploy（GitOps）+ ops（Runbook / Chaos / DR）
├── 70_共通資産/                     # tools / tests / examples / third_party
├── 80_スパースチェックアウト運用/   # cone mode / partial clone / sparse index
└── 90_トレーサビリティ/             # IMP-DIR ID 索引 / DS-SW-COMP・ADR 対応
```

## 本章の対応 ADR

- [ADR-DIR-001: contracts 昇格](../../02_構想設計/adr/ADR-DIR-001-contracts-elevation.md)
- [ADR-DIR-002: infra 分離](../../02_構想設計/adr/ADR-DIR-002-infra-separation.md)
- [ADR-DIR-003: スパースチェックアウト cone mode 採用](../../02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)

## 本章の対応概要設計 ID

`DS-SW-COMP-120` / `DS-SW-COMP-121` / `DS-SW-COMP-122` / `DS-SW-COMP-124` / `DS-SW-COMP-129` / `DS-SW-COMP-132` / `DS-SW-COMP-135` を改訂または継承する。対応関係は [90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md](90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md) 参照。

## 確定段階

- k1s0 リリース時点: 本章全体のディレクトリレイアウトを確定
- リリース時点: tier1 実装対応範囲の IMP-DIR を全採番完了
- リリース時点: tier2 / tier3 実装対応範囲の IMP-DIR を順次採番
- リリース時点: スパースチェックアウト必須化可否を再評価
