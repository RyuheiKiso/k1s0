---
id: plan.overview.scenario_ops_on_call_rotation_handover
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.ops.ops_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# on-call rotation handover

## 一文方針

8h × 3 階層 on-call rotation の引継ぎを実施し、`rotation_handover.lock.yaml` の確認・前 shift のインシデント summary・今後の注意点を次担当者に渡して zero gap を維持する。

> 早朝 6 時、夜間 shift の ops 担当者がラップトップの Mattermost `#ops-oncall` を開く。8 時間前に引き継いだ `rotation_handover.lock.yaml` の current_shift エントリを確認し、前 shift で発火した SLO burn rate alert 1 件と ops-edge cluster の monthly check 完了を next_shift_notes に記録する。隣の席では次の担当者が既にスタンバイしており、Backstage runbook の最新版 URL を手元に用意している。

## ペルソナ要約

主役: ops 担当者（シニア級、メタ専任）、目的: 前 shift の状態を漏れなく次担当者に引き継ぎ zero gap を実現する

## 現状業務での痛み

- 口頭での引継ぎに依存しており、夜間 shift 中の alert 対応内容が次担当者に伝わらず二重対応が発生する
- インシデント summary の記録先が散在し（Slack / email / 口頭）、引継ぎ漏れが常態化している
- 次担当者が不明な場合に escalation 先が分からず対応が止まる
- `rotation_handover.lock.yaml` のような構造化ファイルがなく、引継ぎ品質が担当者の属人的スキルに依存する

## k1s0 でこう変わる

- `rotation_handover.lock.yaml` が構造化 handover record として機能し、前 shift のインシデント summary・注意点・次担当者が単一ファイルで管理される
- handover record が Mattermost `#ops-oncall` に自動投稿され、関係者全員が参照可能になる
- escalation policy が `escalation_policy.lock.yaml` で自動化され、次担当者不在時も escalation 先が明確になる
- zero gap が構造で保証されるため、引継ぎ品質が担当者スキルに依存しなくなる

## Trigger

on-call rotation shift 切替時 (8h 毎)

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 日次 (3 回)
- 典型きっかけ: 「6:00 の shift 切替で、夜間 shift 中に発火した SLO burn rate alert 1 件と ops-edge cluster 月次確認の引継ぎが必要になった」
- 頻度根拠: 8h × 3 shift = 24h ops loop の構造上、日次 3 回の handover が必ず発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 前 shift ops 担当者（主役） | シニア | リモート / 本社 IT 室 | Mattermost #ops-oncall / rotation_handover.lock.yaml | handover record 記録・次担当者へ申し送り |
| 次 shift ops 担当者 | シニア | 本社 IT 室 / リモート | rotation_handover.lock.yaml | handover record 受取・open alert 確認 |

## 個人 KPI / 達成感

- handover record の記録完了率 100%（shift 終了前 15 分以内）
- 引継ぎ漏れによる二重対応件数: 0 件/月
- next_shift_notes の記載充実度（alert summary 件数 / open incident 件数 / 注意点）

## 工数 / 関与人数 / コスト感

- 通常 handover: 10〜15 分
- incident open 引継ぎ含む: 20〜30 分
- 関与人数: 2 名（前 shift + 次 shift）

## 前提

- `rotation_handover.lock.yaml` が build artifact として存在し、current_shift / next_shift_notes フィールドが有効
- `escalation_policy.lock.yaml` が有効で次担当者の assign が自動化されている
- Mattermost `#ops-oncall` に handover record の自動投稿 webhook が設定されている

## 流れ

1. **shift 終了 15 分前**: 前 shift ops 担当者が `rotation_handover.lock.yaml` の current_shift を確認する
2. **インシデント summary 記録**: 当 shift 中に対応した alert / incident を next_shift_notes に記録する（signal_class / 対応内容 / open/closed / next action）
3. **open alert 確認**: Mattermost `#ops-oncall` の open alert を確認し、引継ぎが必要なものを next_shift_notes に追記する
4. **注意点記録**: capacity 変動 / 定期 maintenance window / chaos drill 予定を next_shift_notes に記録する
5. **次担当者へ口頭 or Mattermost DM で要約**: 1〜2 分で critical 事項を口頭伝達し、Mattermost に要約メッセージを投稿する
6. **handover record の Mattermost 投稿**: `rotation_handover.lock.yaml` の当 shift 分を Mattermost `#ops-oncall` に自動投稿する
7. **次担当者の受取確認**: 次担当者が handover record を確認し、Mattermost に :white_check_mark: リアクションで受取を記録する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T-15m | 前 shift ops | current_shift 確認・next_shift_notes 記録開始 | — |
| T-5m | 前 shift ops | open alert 最終確認・注意点追記 | — |
| T+0m | rotation system | shift 切替・次担当者 assign | Mattermost: 「#ops-oncall shift 切替: 担当者X → 担当者Y」 |
| T+1m | rotation system | handover record を #ops-oncall に自動投稿 | Mattermost: 「[handover] shift 06:00 summary: alert 1 件 closed, ops-edge check 完了」 |
| T+2m | 次 shift ops | handover record 受取・:white_check_mark: リアクション | Mattermost: リアクション確認 |
| T+5m | 次 shift ops | open alert / Prometheus dashboard 確認開始 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| FA 生産指示 | 中 | 夜間 shift 中の生産指示関連 alert の引継ぎ |
| ライン稼働監視 | 高 | ライン稼働 SLO alert の open/closed 状態引継ぎ |
| 警報配信 | 高 | 警報配信パイプラインの alert 引継ぎが最重要 |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- SLO 適合仕様
- 関連 OSS: Mattermost（通知チャネル）/ Backstage（runbook）

## 期待結果 / 観測指標

- `rotation_handover.lock.yaml` の next_shift_notes が shift 終了前に記録されている
- Mattermost `#ops-oncall` に handover record の自動投稿が届いている
- 次担当者の :white_check_mark: リアクションで受取が確認されている
- 引継ぎ漏れによる二重対応が発生していない

## 失敗時の挙動 / escalation

- **次担当者が shift 開始後 10 分以内に受取確認しない**: `escalation_policy.lock.yaml` の L2 escalation が発火し、Mattermost に ops チーム全員へ通知が飛ぶ（SLA: 10 分以内に受取確認または代替担当者 assign）
- **handover record の自動投稿が失敗する**: ops 担当者が手動で `rotation_handover.lock.yaml` の内容を Mattermost に貼り付け、webhook の復旧を infra 担当者へ依頼する
- **前 shift 中に critical incident が open のまま**: 次担当者に口頭で即時 briefing し、incident response 09 を継続する

## 失敗パターン (anti-pattern)

1. **next_shift_notes を空のまま handover する**: 次担当者が shift 状態を把握できず、初動対応が遅延する。SPA は next_shift_notes 未記載の場合に handover record 送信ボタンを非活性化する
2. **closed alert のみ記録し open alert を省略する**: 引継ぎ後に次担当者が open alert に気づかず対応漏れになる。`rotation_handover.lock.yaml` の open_alerts フィールドは自動集計されるため省略不可
3. **口頭伝達だけで handover record を省略する**: 後から参照できる記録が残らず、postmortem で経緯が追えなくなる

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [SLO監視_burn_rate_alert](02_SLO監視_burn_rate_alert.md)
- [24h_oncall_fatigue_budget管理](14_24h_oncall_fatigue_budget管理.md)
