---
id: plan.tier1.scenario_l2star_conformance
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
  proof_classes: []
---

# L2\* conformance 維持

## 一文方針

tier1 担当者が L2\* 採用カテゴリ（同族 OSS 2 実装）の Testcontainers conformance test を定期実行・維持し、「merge 条件として 2 実装が green」の原則が常に物理的に成立している状態を保つ。

## Trigger（発火条件）

L2\* 同族 OSS の一方でバージョン更新 / API 変更 / 非互換変更が発生した時 / conformance test が CI で fail した時 / 四半期の定期 conformance check cadence 到来時

## 想定頻度 / 典型きっかけ

- 想定頻度: 四半期（定期 check）/ イベント駆動（同族 OSS 更新時）
- 典型きっかけ: 「L2\* の RDB カテゴリで採用している YugabyteDB と CockroachDB の conformance test が CockroachDB のマイナーアップデートで fail した。どちらの API 差異が問題か特定し修正する必要が生じた」「L2\* の Message Broker カテゴリ（Kafka + Redpanda）の conformance test を 4 半期定期 check で実施したところ、Redpanda の特定 API のタイムアウト挙動が Kafka と差異があることを発見した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 承認: dual reviewer（tier1 担当者 2 名、変更 PR の author 不可）

## 前提

- [L2\* 同族 OSS 一覧](../../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md) が最新の状態で記録済み
- 各 L2\* カテゴリの conformance test が `tests/conformance/<category>/` に存在する

## 流れ

1. 対象の L2\* カテゴリと 2 実装（OSS A / OSS B）を `04_提供機能カテゴリ.md` で確認する
2. Testcontainers でローカル実行し、fail している test を特定する
   - 「OSS A は green / OSS B は fail」なのか「両方 fail」なのかを切り分ける
   - fail している test 名とエラー内容を記録する
3. fail の原因を分類する: API 非互換 / タイムアウト差異 / デフォルト設定差異 / バグ
4. 修正方針を決定する
   - L2\* 抽象層で差分を吸収できる場合: adapter コードを更新して green に修正
   - どちらかの OSS が L2\* 要件（同族 API 同等性）を満たさない場合: `04_提供機能カテゴリ.md` のコメントに非互換事項を記録して承認
   - tier1 Library 側の実装変更が必要な場合: PR を作成して conformance test を再実行
5. 2 実装の conformance test が両方 green になることを確認する
6. conformance check 結果を `oss_conformance_check.lock.yaml` に記録する（日付 / OSS A version / OSS B version / 結果 / 対応内容）
7. dual reviewer sign-off を取得する

## 関連適合仕様 / 関連 OSS

- 関連適合仕様: [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) / [tier1 強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- 関連 OSS: Testcontainers（conformance test runner）/ Harbor（OSS image mirror）

## 期待結果 / 観測指標

- ci: 対象 L2\* カテゴリの conformance test（OSS A / OSS B 両方）all green
- artifact: `oss_conformance_check.lock.yaml` に当期チェック記録が追加済み
- sign-off: dual reviewer 2 名 sign-off 完了

## 失敗時の挙動 / escalation

- **L2\* 同族 OSS の一方が根本的に API 同等性を失った（API breaking change）**: その OSS を L2\* から除外し L3 に降格するか、代替 OSS を選定して L2\* ペアを更新する。設計決定を tier1-01（新規 OSS 採用評価）で再実施。Mattermost `#tier1-lifecycle` に報告（**SLA: 四半期 check 期間内 = 2 週間以内**）。
- **conformance test 自体に欠陥（誤った非互換検出）**: test を修正してから再実行（false positive と明記して `oss_conformance_check.lock.yaml` に記録）。

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成
- [新規 OSS 採用評価](./01_新規OSS採用評価.md) — L2\* ペアを選定した際の初回評価シナリオ
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md) — L2\* 同族保証の原則定義
