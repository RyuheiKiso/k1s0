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

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が buf CI ダッシュボードを確認し、四半期定期 check の cadence が到来しており L2* の Message Broker カテゴリ（Kafka + Redpanda）の conformance test で Redpanda の特定 API タイムアウト差異が検出されていることに気付く。手元には Testcontainers CI ログと `oss_conformance_check.lock.yaml`、Mattermost 越しに dual reviewer 2 名がいる。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: L2* 同族 OSS 2 実装が常に conformance test で green を維持し「merge 条件として 2 実装が等価」の物理的成立を保つ

## 現状業務での痛み

- L2* の 2 実装が独立してバージョンアップされ、一方が degraded 状態になっていても発覚が本番障害後になる
- conformance test が四半期ごとに手動実行されるため、バージョン更新直後の API 差異が長期間放置される
- タイムアウト差異やデフォルト設定差異を「互換あり」と誤判定し、切り替え時に動作不良が発生する
- L2* 同族 OSS の一方が API 同等性を失った場合に代替 OSS 選定のフローが明確でない

## k1s0 でこう変わる

- Testcontainers conformance test を定期 CI（四半期）で自動実行し、2 実装の drift を即時検出する
- 修正方針（adapter 吸収 / 非互換事項記録 / tier1 Library 実装変更）を判定フローとして明文化し、対応ブレをなくす
- `oss_conformance_check.lock.yaml` に当期チェック記録（バージョン / 結果 / 対応内容）を蓄積し、トレーサビリティを確保する
- OSS が L2* 要件を失った場合に L3 降格 / 代替 OSS 選定フローを明文化し、迅速な対処を可能にする

## Trigger（発火条件）

L2\* 同族 OSS の一方でバージョン更新 / API 変更 / 非互換変更が発生した時 / conformance test が CI で fail した時 / 四半期の定期 conformance check cadence 到来時

## 想定頻度 / 典型きっかけ

- 想定頻度: 四半期（定期 check）/ イベント駆動（同族 OSS 更新時）
- 典型きっかけ: 「L2\* の RDB カテゴリで採用している YugabyteDB と CockroachDB の conformance test が CockroachDB のマイナーアップデートで fail した。どちらの API 差異が問題か特定し修正する必要が生じた」「L2\* の Message Broker カテゴリ（Kafka + Redpanda）の conformance test を 4 半期定期 check で実施したところ、Redpanda の特定 API のタイムアウト挙動が Kafka と差異があることを発見した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 承認: dual reviewer（tier1 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | buf CI / Testcontainers CI | L2* カテゴリ確認・fail 特定・修正方針決定・conformance check 記録 |
| 承認（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | conformance 修正内容レビュー・sign-off |
| 承認（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | conformance 修正内容レビュー・sign-off |

## 個人 KPI / 達成感

- L2* conformance test（OSS A / OSS B 両方）all green 率
- `oss_conformance_check.lock.yaml` の四半期更新完了率
- dual reviewer 2 名 sign-off 完了
- OSS A / OSS B のバージョン差異（非互換）0 件

## 工数 / 関与人数 / コスト感

- 初回（定期 check・adapter 吸収で解決）: 半日、関与 3 名（主役 + dual reviewer 2 名）
- 平常（バージョン更新起動・タイムアウト差異修正）: 1 日、関与 3 名
- 失敗時（OSS が L2* 要件喪失・代替 OSS 選定）: 1〜2 週間、関与 4〜5 名（+ 新規 OSS 採用評価フロー）

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | 四半期 cadence 確認・対象 L2* カテゴリと 2 実装確認 | `L2* check: Message Broker / Kafka v3.6 / Redpanda v24.1` |
| 2h | tier1 担当者 | Testcontainers ローカル実行・fail 特定・原因分類 | `Redpanda タイムアウト差異検出 / 原因: デフォルト設定差異` |
| 3〜4h | tier1 担当者 | 修正方針決定・adapter コード更新 or 非互換記録 | `adapter 吸収で対応 / PR 作成中` |
| 5〜6h | tier1 担当者 | 2 実装 conformance test 両方 green 確認・lock.yaml 記録 | `OSS A/B conformance: both green` |
| 1 日 | dual reviewer A/B | sign-off | `dual sign-off 完了 / oss_conformance_check.lock.yaml 更新済` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA テレメトリ**: L2* Message Broker（Kafka + Redpanda）の conformance 維持は SCADA テレメトリの配信パイプラインがどちらの実装でも同等に動作することを保証し、OSS 切替時の設備データ欠落を防ぐ。
- **警報配信**: L2* RDB カテゴリの同族保証は警報履歴の永続化 / 検索 API が 2 実装で等価に動作することを保証し、警報配信の信頼性監査に必要な履歴完全性を支える。

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

## 失敗パターン (anti-pattern)

- **タイムアウト差異を「誤差範囲」として黙認**: adapter 吸収もせず非互換事項の記録もしないまま放置し、OSS 切り替え時に動作不良が発生する。差異は必ず adapter 吸収か非互換事項の明示記録のどちらかで対処し、黙認を構造的に禁止する。
- **conformance test に欠陥（false positive）を放置**: 誤検出のまま「conformance test が unreliable」という認識が広まり、test の信頼性が低下する。false positive と明記して `oss_conformance_check.lock.yaml` に記録し、test を修正して再実行することを必須にする。
- **OSS A が fail したまま OSS B のみ green で「OK」と判断**: 一方が fail した状態で L2* 同族保証が破れているにもかかわらず運用が続く。「2 実装両方 green でない = L2* 同族保証の失敗」として即時対処を強制する。

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成
- [新規 OSS 採用評価](./01_新規OSS採用評価.md) — L2\* ペアを選定した際の初回評価シナリオ
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md) — L2\* 同族保証の原則定義
- [L1+ migration dry-run](./12_L1plus_migration_dryrun.md) — L2* conformance が green の状態を保つことが L1+ dry-run 成功の前提条件
