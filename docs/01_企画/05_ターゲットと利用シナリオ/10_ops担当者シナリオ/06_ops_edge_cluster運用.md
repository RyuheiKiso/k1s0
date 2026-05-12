---
id: plan.overview.scenario_ops_edge_cluster
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

# ops_edge_cluster運用

## 一文方針

ops-edge cluster (target cluster outage 時の escalation last-mile 用) の正常稼働を確認し、OS native API (DPAPI/Keychain/APNs/FCM) 経由の通知経路が維持されていることを定期確認する。

> 毎月第 1 月曜、ops 担当者が ops-edge cluster の月次定期確認チェックリストを開く。Kubernetes の ops-edge namespace を確認し、DPAPI / Keychain / APNs / FCM の各通知経路が正常に応答しているかをヘルスチェックスクリプトで確認する。target cluster がダウンした際にこの ops-edge 経路だけが最後の通知手段になるため、正常稼働の確認は単月も欠かせない。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: ops-edge cluster の正常稼働を確認し、OS native API 通知経路が維持されていることを月次 + イベント駆動で確認する

## 現状業務での痛み

- target cluster 障害時に ops-edge cluster も障害になって初めて「バックアップ経路が機能しない」ことが発覚するケースがある
- OS native API（DPAPI / Keychain / APNs / FCM）の接続確認が運用 SOP に含まれておらず、設定変更後の動作確認が漏れる
- ops-edge cluster の定期確認が属人的な記憶に依存し、担当者交代時に確認漏れが発生する
- target cluster 障害時の escalation 手順が ops-edge cluster 経由で定義されていない

## k1s0 でこう変わる

- ops-edge cluster の月次定期確認チェックリストが Backstage runbook に定義され、担当者交代後も確認が継続される
- OS native API の通知経路ヘルスチェックスクリプトが自動化され、定期確認の toil が削減される
- target cluster 障害発生時に ops-edge escalation 手順が自動で発火し、通知経路の切替が即時化される
- 月次確認結果が `ops_edge_healthcheck.lock.yaml` に記録され、証跡が残る

## Trigger

月次定期確認 / target cluster 障害発生時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次 + イベント駆動
- 典型きっかけ: 「月次定期確認日到来」「target cluster で availability_loss incident が発生し、ops-edge 経由の通知切替が必要になった」
- 頻度根拠: 月次定期確認は SRE 運用規律として月 1 回必須。target cluster 障害は年 0〜3 件程度

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | Backstage runbook / ops-edge cluster dashboard | 月次確認チェックリスト実施・障害時通知経路切替 |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Kubernetes dashboard | ops-edge cluster の infra 層確認（escalation 受け） |

## 個人 KPI / 達成感

- 月次定期確認の実施率: 100%
- `ops_edge_healthcheck.lock.yaml` の月次記録率: 100%
- target cluster 障害時の ops-edge 通知経路切替時間: 5 分以内

## 工数 / 関与人数 / コスト感

- 月次定期確認: 20〜30 分
- target cluster 障害時の通知切替: 5〜15 分
- 関与人数: 1〜2 名（ops + 必要時 infra）

## 前提

- ops-edge cluster が target cluster とは独立した infra に構築されている
- OS native API（DPAPI / Keychain / APNs / FCM）の接続設定が ops-edge cluster に正しく設定されている
- `ops_edge_healthcheck.lock.yaml` が build artifact として存在する
- Backstage runbook に月次確認チェックリストが定義されている

## 流れ

1. **チェックリスト起動**: Backstage runbook の `ops-edge-monthly-check` を開く
2. **ops-edge cluster 稼働確認**: `kubectl get pods -n ops-edge` で全 pod の Running 状態を確認する
3. **DPAPI / Keychain 通知経路確認**: Windows / macOS 端末への push 通知テストを実行し、応答を確認する
4. **APNs 通知経路確認**: iOS 端末への APNs 経由 push 通知テストを実行し、APNS-Status 200 を確認する
5. **FCM 通知経路確認**: Android 端末への FCM 経由 push 通知テストを実行し、success レスポンスを確認する
6. **`ops_edge_healthcheck.lock.yaml` 記録**: 確認結果を YAML に記録し、GitHub commit で証跡を残す
7. **Mattermost 報告**: Mattermost `#ops-infrastructure` に月次確認完了を報告する
8. **（障害発生時）通知経路切替**: target cluster 障害検知時に ops-edge 経由の通知経路に手動で切替え、5 signal の alerting が ops-edge 経由で到達することを確認する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | ops 担当者 | Backstage runbook 起動・チェックリスト開始 | — |
| T+5m | ops 担当者 | ops-edge cluster pod 稼働確認 | — |
| T+10m | ops 担当者 | DPAPI / Keychain 通知テスト実行 | — |
| T+15m | ops 担当者 | APNs / FCM 通知テスト実行 | — |
| T+20m | ops 担当者 | ops_edge_healthcheck.lock.yaml 記録・GitHub commit | — |
| T+25m | ops 担当者 | Mattermost 報告 | 「[ops-edge] 月次確認完了。全経路 OK」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | target cluster 障害時の警報配信 last-mile が ops-edge 経路 |
| ライン稼働監視 | 高 | ライン異常通知の ops-edge バックアップ経路維持 |
| FA 生産指示 | 中 | 緊急生産停止指示の ops-edge 通知経路確保 |

## 関連適合仕様 / 関連 OSS

- ops_edge_cluster 適合仕様
- 運用ループ適合仕様
- 関連 OSS: Kubernetes（ops-edge cluster）/ Mattermost（通知）/ APNs / FCM（OS native API）

## 期待結果 / 観測指標

- `ops_edge_healthcheck.lock.yaml` に月次確認結果が記録されている
- 全 OS native API 通知経路（DPAPI / Keychain / APNs / FCM）のヘルスチェックが pass している
- target cluster 障害時の通知切替が 5 分以内に完了している

## 失敗時の挙動 / escalation

- **APNs / FCM 通知テストが失敗する**: ops 担当者が接続設定を確認し、infra 担当者に証明書更新 / API key 更新を escalation する。SLA: 24 時間以内に通知経路復旧
- **ops-edge cluster の pod が Pending / CrashLoopBackOff になっている**: infra 担当者に Kubernetes 障害対応を escalation する。SLA: 月次確認日から 48 時間以内に復旧
- **target cluster 障害時に ops-edge 通知切替が 5 分以内にできない**: `escalation_policy.lock.yaml` の L3 escalation を発火し、tech lead と infra チームリーダーに通知する

## 失敗パターン (anti-pattern)

1. **月次定期確認をスキップする**: ops-edge 通知経路の設定変更（証明書失効等）に気づかず、target cluster 障害時に last-mile 通知が機能しない
2. **DPAPI / Keychain のみ確認して APNs / FCM を省略する**: モバイル端末への通知が断絶したまま運用が継続されるリスクがある
3. **確認結果をメモ書きに残してロック YAML に記録しない**: 証跡が残らず、監査や postmortem で経緯が追えなくなる

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [incident_response_7class主導](09_incident_response_7class主導.md)
- [SLO監視_burn_rate_alert](02_SLO監視_burn_rate_alert.md)
