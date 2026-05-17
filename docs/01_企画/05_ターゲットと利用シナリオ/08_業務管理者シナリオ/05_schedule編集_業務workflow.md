---
id: plan.overview.scenario_business_admin_schedule_edit
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D]
  proof_classes: []
---

# schedule編集_業務workflow

## 一文方針

生産計画 / 検査 cycle / 棚卸 cycle の業務 workflow schedule を Backstage プラグインから変更し、tier2 decision table override layer で設定を反映させる。

> 月末金曜日、業務管理者は翌月の生産計画変更通知を受け取る。設備メンテナンスのため第 2 週は検査 cycle を週 2 回から週 1 回に変更し、棚卸 cycle も月末から第 4 週金曜に前倒す必要がある。Backstage プラグインの業務 workflow schedule 編集画面を開き、対象 workflow の cadence 設定を変更する。preview で影響を確認し、業務担当者に通知して保存する。tier2 の decision table override layer に即座に反映されることを確認する。

## ペルソナ要約

主役: 業務管理者（シニア級）、目的: 業務計画変更に合わせて業務 workflow schedule を正確に更新し、業務担当者への影響を最小化する

## 現状業務での痛み

- 業務 workflow の schedule 変更が手動でスプレッドシートに管理されており、業務担当者への周知が口頭伝達に依存する
- schedule 変更が複数のシステムに個別に設定が必要で、設定漏れが発生すると予定通りに workflow が起動しない
- 季節変動（繁忙期 / 閑散期）に合わせた cadence 調整が都度 IT 部門への依頼になり、数日かかる
- schedule 変更の履歴が残らないため、過去に何をいつ変更したかを確認できない

## k1s0 でこう変わる

- Backstage プラグインから業務管理者が単独で schedule を変更でき、IT 部門への依頼が不要になる
- 変更 preview が変更前後の workflow 発火タイミングを視覚的に比較表示し、設定漏れを防ぐ
- schedule 変更と同時に tier2 の decision table override layer に反映され、業務担当者への push 通知が自動送信される
- 全 schedule 変更操作が audit hash chain に記録されるため、過去の変更履歴を即座に確認できる

## Trigger

生産計画変更 / 季節変動対応 / 設備メンテナンスで schedule 変更が必要な時（計画変更通知受信または月次 planning 会議での決定をトリガーとする）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「翌月の生産計画で設備メンテナンス週に検査 cycle を減らす」「年末年始の繁忙期に合わせて棚卸 cycle を前倒す」「新規製品ラインの立ち上げで生産指示 cadence を追加する」
- 頻度根拠: 製造業では月次 planning サイクルで schedule 変更が恒常的に発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 業務管理者（主役） | シニア | 事務所 | Backstage プラグイン | schedule 変更 / preview 確認 / 業務担当者通知 |
| 生産計画担当者 | — | 本社 / 事務所 | 生産計画システム | 計画変更の起点・schedule 変更要件の提示 |
| 業務担当者 | 現場スタッフ | 工場フロア | SPA 通知 | schedule 変更通知の受信・確認 |

## 個人 KPI / 達成感

- schedule 変更完了から tier2 への反映確認までの所要時間（目標: 10 分以内）
- 業務担当者への push 通知送信率（schedule 変更ごと）100%
- schedule 変更ミス（設定値の誤入力）による workflow 誤発火件数 / 月（目標: ゼロ）
- 月次 planning 会議後 1 営業日以内の schedule 変更完了率

## 工数 / 関与人数 / コスト感

- 平常（単一 workflow の cadence 変更）: 15〜30 分
- 複数 workflow の一括変更: 30〜60 分
- 失敗時（tier2 反映エラー）: +30 分〜1 時間
- 関与人数: 2〜3 名（業務管理者 + 生産計画担当者 + 業務担当者（通知受信確認））

## 前提

- 業務管理者が Backstage プラグインの schedule 編集画面へのアクセス権を保有している
- 変更対象の workflow が tier2 decision table override layer に存在している
- 生産計画担当者から schedule 変更要件（変更対象 / 変更内容 / 有効日）が書面で提示されている

## 流れ

1. 生産計画担当者から schedule 変更要件（変更対象 workflow / 新 cadence / 有効開始日）を受け取る
2. Backstage プラグインの業務 workflow schedule 編集画面を開き、対象 workflow を選択する
3. 変更前の cadence 設定を確認し、要件と照合する
4. 新しい cadence 設定（発火タイミング / 頻度 / 有効日）を入力する
5. preview ボタンを押し、変更後の workflow 発火タイムラインを確認する（今後 2 週間の発火予定が表示される）
6. preview が要件通りであることを確認し、「保存」ボタンを押下する
7. tier2 decision table override layer への反映を確認する（反映 status が「適用済み」になること）
8. 業務担当者への push 通知が自動送信されたことを確認する
9. Mattermost `#workflow-schedule` に変更内容を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 業務管理者 | 変更要件受領・Backstage 開く | — |
| T+5m | 業務管理者 | 対象 workflow 選択・変更前設定確認 | — |
| T+15m | 業務管理者 | 新 cadence 設定入力 | — |
| T+17m | 業務管理者 | preview 確認（今後 2 週間の発火タイムライン） | Backstage: 「次回発火: 来週火曜 09:00」 |
| T+20m | 業務管理者 | 保存ボタン押下 | — |
| T+20m30s | tier2 | decision table override layer 反映 | Backstage: 「schedule 適用済み。audit ログ発行済み」 |
| T+21m | SPA | 業務担当者へ push 通知送信 | 端末（業務担当者）: 「検査 cycle schedule が変更されました（有効: 来月第 1 週より）」 |
| T+25m | 業務管理者 | Mattermost 報告 | Mattermost `#workflow-schedule`: 「検査 cycle 変更完了（要件 #SCH-2024-031）」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| FA 生産指示 | 高 | 生産指示の発行 cadence が schedule 変更で直接変わる |
| 品質検査結果配信 | 高 | 検査 cycle の cadence 変更が検査実施頻度・結果配信タイミングに直結する |
| 在庫最新値 | 中 | 棚卸 cycle の schedule 変更が在庫最新値の更新タイミングに影響する |
| 受注 sub | 中 | 棚卸 schedule が変わると受注可能在庫の評価タイミングが変動する |

## Backstage プラグイン操作 UI

- **業務 workflow schedule 編集画面**: テナント内の全 workflow 一覧。各行に現 cadence 設定と最終変更日時を表示
- **cadence 設定フォーム**: cron 式 / 相対指定（毎週月曜 09:00 等）の両方に対応。入力値の human-readable 変換をリアルタイム表示
- **preview パネル**: 変更後の今後 14 日間の発火タイムラインをカレンダー形式で表示。変更前との差分をハイライト
- **反映 status バッジ**: 「保存中」→「適用済み」→「業務担当者通知送信済み」の status をリアルタイム更新

## 業務管理者の決定権限境界

- **実施可（単独で完結）**: 既存 workflow の cadence 変更 / 有効日変更 / 一時停止 / 再開
- **escalation 必要（tier2 担当者）**: 新規 workflow の作成 / workflow ロジックの変更 / decision table の構造変更

## 関連適合仕様

- [テナント容量適合仕様](../../../04_詳細設計/01_適合仕様/09_テナント容量適合仕様.md)

## 期待結果 / 観測指標 / 受入条件

- tier2 decision table override layer への schedule 反映が保存後 30 秒以内に完了する
- 業務担当者への push 通知が schedule 変更後 1 分以内に送信される
- audit ログに変更者 / 変更内容 / 有効日が記録されている
- preview と実際の workflow 発火タイミングが一致している
- 受入条件: 上記 4 点が E2E テストで全て pass

## 失敗時の挙動 / escalation

- **tier2 反映エラー**: Backstage が「schedule 適用に失敗しました」と表示し、retry ボタンを提示する。3 回 retry 失敗後: escalation 先 tier2 担当者 Mattermost `#tier2-incident`（SLA: 30 分以内）
- **cron 式文法エラー**: Backstage がリアルタイムで「無効な cron 式」を表示し、保存を拒否する。human-readable 表示で入力値を確認して修正する
- **業務担当者通知未送信**: push 通知の送信 status が「失敗」になった場合、Mattermost `#workflow-schedule` に手動で通知内容を投稿する

## 失敗パターン（3 例）

1. **preview を確認せずに保存する**: 有効日の入力ミスにより、意図より早く / 遅く schedule が変わってしまう。preview パネルで今後 14 日の発火タイムラインを必ず確認する SOP とする
2. **変更要件書なしで口頭指示のみで schedule を変更する**: 変更根拠が audit に残らず、後から変更の正当性を説明できなくなる。変更前に要件番号（#SCH-YYYY-NNN）を受領することを必須条件とする
3. **複数 workflow を一括変更する際に 1 件ずつ preview 確認をスキップする**: 一部 workflow に誤設定が入ってもレビュー時に気づかない。一括変更時も各 workflow の preview を順番に確認する手順を SOP に明記する

## 関連参照

- [業務管理者シナリオ INDEX](README.md)
- [06_decision_table_override編集](06_decision_table_override編集.md)
- `arch.tier2.tier2_index`
- `req.team.tier_engineer_requirement`
