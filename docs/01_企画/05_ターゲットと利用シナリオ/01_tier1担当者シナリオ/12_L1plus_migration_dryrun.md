---
id: plan.tier1.scenario_l1plus_migration_dryrun
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - C
    - E
  proof_classes: []
---

# L1+ 移行 dry-run

## 一文方針

tier1 担当者が L1+ 採用 OSS の年次 dry-run を実施し、移行 toolchain（schema diff / dual-write / observability 連続性 / 業務コード移行ガイド）が green であることを物理証明として `oss_lifecycle.lock.yaml` に記録する。

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が Backstage Catalog を確認し、Connect-RPC（L1+ Bidi transport）の年次 dry-run cadence が到来していることに気付く。手元には `oss_lifecycle.lock.yaml` と Jaeger v2 の trace dashboard、Mattermost 越しに dual reviewer 2 名・tier2 担当者・security 担当者がいる。

## Trigger（発火条件）

L1+ OSS の年次 dry-run cadence 到来時（tier1 設計方針「L1+ 移行コミットメント: 移行 toolchain を年次 dry-run green で物理証明」が原則）

## 想定頻度 / 典型きっかけ

- 想定頻度: 年次（L1+ 採用 OSS の本数分、例: 5 OSS 採用 = 年 5 回）
- 典型きっかけ: 「Connect-RPC（L1+ Bidi transport）の移行 toolchain dry-run cadence が到来し、仮の fork（v-next）へのスキーマ diff / dual-write / observability 連続性 / 業務コード移行ガイドを 1 週間で実証することになった」「CloudNativePG（L1+ RDB）の年次 dry-run で PostgreSQL 代替実装への移行手順が green であることを確認した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: tier2 担当者（業務コード移行ガイドの確認）/ security 担当者（観測 chain の継続性確認）
- 承認: dual reviewer（tier1 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | Backstage Catalog | dry-run 実施・schema diff・dual-write 検証・移行ガイド draft・lock.yaml 記録 |
| 関与（tier2 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | 業務コード移行ガイドのレビュー・フィードバック |
| 関与（security 担当者）| 中堅 | 本社 IT 室 | security alert dashboard | 観測 chain の継続性確認（trace_id 途切れ検証） |
| 承認（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | dry-run 記録レビュー・sign-off |
| 承認（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | dry-run 記録レビュー・sign-off |

## 前提

- 対象 L1+ OSS が `04_提供機能カテゴリ.md` に登録済み（[提供機能カテゴリ](../../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md)）
- [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)に dry-run 対象 OSS と cadence が記録済み

## 流れ

1. 対象 L1+ OSS と假想の「移行先 OSS」を `oss_lifecycle.lock.yaml` で確認する（実際の移行ではなく「移行できることを証明」するテスト）
2. schema diff を実施する: 現行 OSS の API / プロトコルと假想移行先 OSS の API 差分を洗い出し、差分がゼロか許容範囲か判定する
3. dual-write のフィージビリティを Testcontainers で検証する: 現行 OSS と假想移行先の両方に同時書き込みが可能かを自動 test で確認する（Testcontainers conformance test を両 OSS 向けに作成）
4. observability 連続性を確認する: trace_id / span が移行前後で途切れないことを Jaeger v2 で確認する（移行シミュレーション環境で分散トレースが連続していること）
5. 業務コード移行ガイドを draft する: tier2 / tier3 が「現行 OSS を假想移行先 OSS に切り替える際に変更するコード箇所」を言語別に列挙し、tier2 担当者にレビューを依頼する
6. dry-run の全工程を `oss_lifecycle.lock.yaml` に記録する（schema diff 結果 / Testcontainers green / observability 連続性確認 / ガイド draft 完了）
7. dual reviewer sign-off を取得する

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA テレメトリ**: L1+ transport（Connect-RPC 等）の dry-run は SCADA テレメトリ経路の移行実現可能性を毎年物理証明するものであり、実際の EOL 時に設備データ収集が途切れないことを担保する。
- **受注**: L1+ RDB（CloudNativePG 等）の dry-run 成功は受注データの永続化・整合性を支えるストレージ移行が実際に機能することを年次証明し、受注業務の事業継続性を裏付ける。

## 関連適合仕様 / 関連 OSS

- 関連適合仕様: [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) / [tier1 強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- 関連 OSS: Testcontainers（dual-write 検証）/ Jaeger v2（observability 連続性）/ Buf（proto diff）

## 期待結果 / 観測指標

- artifact: `oss_lifecycle.lock.yaml` に dry-run green エントリ（年次更新）が記録済み
- ci: Testcontainers conformance test（現行 + 假想移行先の 2 実装）all green
- doc: 業務コード移行ガイド draft が tier2 担当者レビュー済み
- sign-off: dual reviewer 2 名 sign-off 完了

## 失敗時の挙動 / escalation

- **Testcontainers conformance fail（dual-write が不可能）**: 假想移行先 OSS との API 差異が大きすぎる → L1+ の移行コミットメントを更新し、移行先 OSS 候補を再選定。ops 担当者に Mattermost `#tier1-lifecycle` で報告（**SLA: dry-run 期間内 = 1 週間以内**）。**postmortem 期限: 3 営業日以内**。
- **observability 連続性が確認できない（trace_id が途切れる）**: 観測 Library の更新が必要。tier1-05（新コンポーネント追加）でシムを実装してから再試行。
- **年次 dry-run 未実施のまま翌年に持ち越し**: 1.0.0 ship blocker と同等の重大度として Backstage ticket を起票し tier1 リードに報告（**SLA: cadence 超過翌日以内に escalate**）。

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成と担当者プロフィール
- [OSS ライフサイクルイベント対応](./02_OSSライフサイクルイベント対応.md) — 実際の EoL / ライセンス変更発生時の移行実施シナリオ
- [新規 OSS 採用評価](./01_新規OSS採用評価.md) — L1+ を選定した際の初回評価シナリオ（dry-run の前提）
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md) — L1+ 移行コミットメントの原則定義の SoT
- [L2* conformance 維持](./13_L2star_conformance維持.md) — migration dry-run と conformance test の定期実行は同一サイクルで計画する
