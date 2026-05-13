---
id: plan.overview.scenario_legal_dpo_gdpr_72h_breach
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.legal_check
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, D]
  proof_classes: []
---

# GDPR_72h_breach通知

## 一文方針

data breach 発生時の GDPR 72h 通知（監督機関 + 本人通知 template 完成まで）を法務 DPO 担当者として主導し、GDPR Art.33/34 の deadline を厳守する。

> 火曜午後 2 時、security 担当者から「テナント B の PII データが外部 IP に exfiltrate された可能性がある」という Mattermost 通知が届く。法務 DPO 担当者は breach 認識時刻を T+0 として記録し、72h カウントダウンを開始する。security 担当者から breach 技術詳細を取得しながら、GDPR Art.33 の監督機関通知 template と Art.34 の本人通知 template を並行して draft する。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級、DPO 認定）、目的: breach 認識から 72h 以内に GDPR 監督機関通知を完了し、本人通知 template を準備する

## 現状業務での痛み

- breach 認識から 72h というタイムプレッシャーの中で、通知に必要な技術情報を security 担当者から収集するのに時間がかかる
- 監督機関通知 template が存在せず、毎回ゼロから draft するため時間が不足する
- breach の影響範囲（affected 本人数 / データ種別）の特定に ClickHouse クエリが必要で、data 担当者の協力待ちになる
- 通知後の follow-up（追加情報提供 / 本人通知完了確認）の管理が属人化している

## k1s0 でこう変わる

- GDPR 72h 通知 template が Backstage TechDocs に整備され、draft 時間を大幅に短縮できる
- `breach_notification.lock.yaml` がタイムライン管理ツールとして機能し、72h カウントダウンが可視化される
- security 担当者から技術詳細が構造化形式で提供される仕組みがあり、情報収集の待ち時間が短縮される
- 本人通知の送信記録が data 層に保管され、規制当局への完了報告に即座に活用できる

## Trigger

data breach 発生時（security 担当者から breach 疑いの通知を受けた時）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: イベント駆動（年 0〜2 件）
- 典型きっかけ: 「security 担当者から PII data exfiltration の可能性が通知された」
- 頻度根拠: breach は設計上レアケース。複数テナントへの影響がある場合は GDPR と個人情報保護法の両方の手続きが並行する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | Mattermost / breach_notification.lock.yaml | 72h カウントダウン管理・監督機関通知 draft・本人通知 template 作成 |
| security 担当者 | シニア | 本社 IT 室 / リモート | security dashboard | breach 技術詳細の提供・affected データ範囲の特定 |
| data 担当者 | シニア | 本社 IT 室 / リモート | ClickHouse | affected 本人数・データ種別の特定クエリ実行 |

## 個人 KPI / 達成感

- GDPR Art.33 監督機関通知: breach 認識から 72h 以内 100%
- 監督機関通知の完全性スコア（Art.33 必須記載事項の充足率）: 95% 以上
- 本人通知の発送完了: 監督機関通知後 速やかに

## 工数 / 関与人数 / コスト感

- breach 技術情報収集 + template draft: 4〜8 時間
- 監督機関通知送信 + 記録: 1〜2 時間
- 本人通知 template 作成 + 送信: 2〜4 時間
- 関与人数: 3〜4 名（法務 DPO + security + data + 経営承認）

## 前提

- GDPR 監督機関通知 template が Backstage TechDocs に整備されている
- `breach_notification.lock.yaml` が build artifact として存在する
- breach 技術詳細の報告フォーマットが security 担当者と合意されている

## 流れ

1. **breach 認識 T+0 記録**: breach 疑い通知を受けた時刻を breach 認識時刻として `breach_notification.lock.yaml` に記録し、72h タイマーを開始する
2. **技術詳細収集（〜T+4h）**: security 担当者から breach の技術詳細（exfiltrate されたデータ種別 / affected テナント / 発生経路）を収集する
3. **affected 本人数特定（〜T+8h）**: data 担当者が ClickHouse クエリで affected 本人数と PII class を特定する
4. **Art.33 要件確認**: GDPR Art.33 の 4 項目（breach の性質 / affected 人数 / category / 影響 / 対処措置 / DPO 連絡先）を確認する
5. **監督機関通知 draft（〜T+24h）**: Backstage TechDocs の template を使用し、監督機関通知を draft する。security 担当者との dual review を経て内容を確定する
6. **経営承認取得（〜T+48h）**: 監督機関通知の内容を経営に確認し、承認を取得する
7. **監督機関通知送信（〜T+72h）**: EU 監督機関の指定 portal または email で通知を送信する。送信記録を `breach_notification.lock.yaml` に記録する
8. **本人通知 template 作成**: Art.34 対象（高リスク breach の場合）の本人通知 template を draft し、送信準備をする

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | 法務 DPO | breach 認識記録・72h カウントダウン開始 | 「[GDPR breach] T+0 記録: 2026-05-12 14:00 JST」 |
| T+4h | security 担当者 | 技術詳細報告 | Mattermost: 「breach 技術詳細: テナントB PII 3件 exfiltrate 確認」 |
| T+8h | data 担当者 | affected 本人数特定（ClickHouse クエリ） | — |
| T+24h | 法務 DPO | 監督機関通知 draft 完了・security dual review | — |
| T+48h | 経営 | 通知内容承認 | — |
| T+70h | 法務 DPO | 監督機関通知送信 | 「[GDPR breach] 監督機関通知送信完了: T+70h（T+72h deadline に対し 2h 余裕）」 |
| T+72h | — | GDPR Art.33 deadline | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 品質検査結果配信 | 高 | 品質検査結果の PII が breach した場合に Art.33 通知が必要 |
| SCADA テレメトリ収集 | 中 | SCADA データに個人識別可能情報が含まれる場合 |
| 受注 sub | 高 | 取引先個人情報を含む受注データの breach |

## 通知 deadline timeline

| deadline | 内容 | 送信先 | 根拠条文 |
|----------|------|--------|--------|
| T+72h（最大）| 監督機関通知 | EU 監督機関（例: BSI / CNIL） | GDPR Art.33 |
| 速やかに（不当な遅延なく） | 本人通知（高リスク breach 時） | breach 対象本人 | GDPR Art.34 |

## 監督官庁との接点

- **通知先**: テナントの主たる拠点の EU 加盟国監督機関（Lead Supervisory Authority）
- **通知方法**: 各監督機関の指定 online portal または secure email
- **follow-up**: 監督機関から追加情報要求が来た場合は 30 日以内に回答する義務がある

## dual sign-off 規律

- 法務 DPO（通知内容の法的妥当性確認）+ security 担当者（技術詳細の正確性確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（breach 技術詳細取得と連動）
- 関連 OSS: ClickHouse（affected 本人数特定）/ Backstage TechDocs（通知 template）/ Mattermost（通知チャネル）

## 期待結果 / 観測指標

- GDPR Art.33 通知が breach 認識から 72h 以内に監督機関に送信されている
- `breach_notification.lock.yaml` に送信記録・affected 本人数・データ種別が記録されている
- Art.34 対象（高リスク breach）の本人通知 template が 72h 以内に準備されている

## 失敗時の挙動 / escalation

- **T+24h 時点で技術詳細が収集できていない**: security 担当者に L3 escalation。tech lead が security チームに優先対応を指示する
- **T+48h 時点で affected 本人数が特定できていない**: 「概算値での通知（更新予定あり）」として監督機関通知を進める。Art.33(4) により追加情報を後日提供することが許容される
- **72h deadline を超過しそう**: T+60h 時点で弁護士（外部法務顧問）に連絡し、超過時の対応方針を確定する

## 失敗パターン (anti-pattern)

1. **breach 認識時刻の記録を後回しにする**: 72h カウントダウンの起点が曖昧になり、deadline 管理ができなくなる
2. **技術詳細が揃うまで通知 draft を始めない**: 技術詳細収集と template draft は並行して進める必要がある
3. **高リスク breach であるにもかかわらず Art.34 本人通知を省略する**: 監督機関からの行政処分リスクが高まる

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [個人情報保護法30d報告](02_個人情報保護法30d報告.md)
- [DSAR対応最終承認](03_DSAR対応最終承認.md)
- [security: data_breach_privacy_incident_response](../../06_security担当者シナリオ/14_data_breach_privacy_incident_response.md)
