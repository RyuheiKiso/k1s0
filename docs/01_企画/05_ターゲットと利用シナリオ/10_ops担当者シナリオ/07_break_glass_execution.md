---
id: plan.overview.scenario_ops_break_glass_execution
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

# break_glass_execution

## 一文方針

緊急時の break-glass (v1_emergency_step_up) を起票・実行し、強制 audit emit と security 担当者の事後 post-review トリガまでを責任をもって実施する。

> 土曜深夜 2 時、on-call の ops 担当者に「本番 PostgreSQL cluster のデータ破損が疑われる。cluster-admin 権限での即時調査が必要」という pager alert が届く。通常の RBAC では cluster-admin は許可されていない。ops 担当者は `break_glass_request.lock.yaml` に緊急理由と調査範囲を記録し、security 担当者に Mattermost で DM を送り dual authorization を取得する。break-glass 発動後、全操作が audit emit される。調査完了後、security 担当者の事後 post-review を自動でトリガする。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: 緊急時の break-glass を適切な手順で起票・実行し、強制 audit emit と事後 post-review まで完結させる

## 現状業務での痛み

- 緊急時に cluster-admin 権限が必要になったとき、誰に承認を取ればいいかが不明確で対応が遅延する
- break-glass 実行時の操作ログが散在し、事後に何を実行したかを追跡できないケースがある
- 緊急権限の乱用（緊急でない場面での使用）を防ぐ仕組みがない
- 事後 post-review が属人的な記憶に依存し、審査が形骸化している

## k1s0 でこう変わる

- `break_glass_request.lock.yaml` が起票テンプレートとして機能し、緊急理由・調査範囲・dual authorization の記録が構造化される
- break-glass 実行中の全操作が強制的に audit emit され、改竄不可能な証跡が残る
- security 担当者への事後 post-review トリガが自動化され、審査が必ず実施される
- `v1_emergency_step_up` の使用件数と理由がダッシュボードで可視化され、乱用の抑止力となる

## Trigger

本番 DB cluster-admin が緊急必要になった時（データ破損対応 / DR 宣言等）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次〜不定期（緊急時のみ）
- 典型きっかけ: 「本番 PostgreSQL cluster のデータ破損が疑われ、cluster-admin での即時調査が必要になった」「DR 宣言時に本番 cluster への緊急アクセスが必要になった」
- 頻度根拠: break-glass は設計上レアケース。年 0〜2 件が典型的。乱用があれば使用件数の急増で検知する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | リモート（緊急時） | Mattermost pager / break_glass_request.lock.yaml | 起票・dual authorization 取得・break-glass 実行・audit emit 確認 |
| security 担当者 | シニア | リモート（on-call） | Mattermost DM / break_glass_request.lock.yaml | dual authorization 承認・事後 post-review 実施 |

## 個人 KPI / 達成感

- dual authorization 取得時間: 15 分以内（緊急時）
- break-glass 実行中の audit emit 漏れ: 0 件
- 事後 post-review の完了率: 100%（break-glass 実行から 48 時間以内）

## 工数 / 関与人数 / コスト感

- 起票 + dual authorization: 15〜30 分
- break-glass 実行（調査含む）: 1〜4 時間
- 事後 post-review: 1〜2 時間
- 関与人数: 2 名（ops + security）

## 前提

- `break_glass_request.lock.yaml` と `v1_emergency_step_up` フロー が認証適合仕様に定義されている
- security 担当者の on-call rotation が `escalation_policy.lock.yaml` に定義されている
- OpenBao の break-glass policy が設定されており、dual authorization が必要な状態になっている

## 流れ

1. **緊急事態確認**: 通常 RBAC では対応不可能であることを確認し、break-glass が正当化できる理由を明文化する
2. **`break_glass_request.lock.yaml` 起票**: 緊急理由・調査範囲・想定作業時間・影響テナントを YAML に記録する
3. **security 担当者への dual authorization 依頼**: Mattermost DM で security on-call 担当者に dual authorization を依頼し、`break_glass_request.lock.yaml` の URL を共有する
4. **dual authorization 取得**: security 担当者が `break_glass_request.lock.yaml` に承認署名を追加し、GitHub commit で記録する
5. **break-glass 実行**: `v1_emergency_step_up` を実行し、cluster-admin 権限を取得する。OpenBao が自動的に全操作の audit emit を開始する
6. **調査実行**: 最小権限の原則に従い、必要な調査操作のみを実行する。不要な操作は行わない
7. **権限返却**: 調査完了後、cluster-admin 権限を即時に返却する（TTL: 最大 2 時間）
8. **事後 post-review トリガ**: security 担当者への事後 post-review を自動トリガし、audit emit ログの URL を共有する
9. **Mattermost 報告**: Mattermost `#security-break-glass` に実行内容と調査結果を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | ops 担当者 | pager alert 確認・break-glass 必要性の判断 | — |
| T+5m | ops 担当者 | break_glass_request.lock.yaml 起票 | GitHub: commit |
| T+10m | ops 担当者 | security 担当者に Mattermost DM | 「緊急 break-glass 承認依頼: 本番 PG データ破損調査」 |
| T+20m | security 担当者 | dual authorization 承認 | GitHub: commit (承認署名) |
| T+25m | ops 担当者 | v1_emergency_step_up 実行 | audit emit 開始 |
| T+120m | ops 担当者 | 調査完了・cluster-admin 権限返却 | — |
| T+125m | system | security 担当者への事後 post-review 自動トリガ | Mattermost: 「[break-glass post-review] audit log URL」 |
| T+130m | ops 担当者 | #security-break-glass に報告 | 「break-glass 実行完了。調査内容: xxx」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| FA 生産指示 | 高 | 生産指示 DB 破損時の緊急 cluster-admin アクセス |
| SCADA テレメトリ収集 | 高 | テレメトリ DB 障害時の緊急調査 |
| 警報配信 | 中 | 警報配信 DB 障害での緊急対応 |

## 関連適合仕様 / 関連 OSS

- 認証適合仕様
- security 強制機構
- 関連 OSS: OpenBao（break-glass policy）/ Mattermost（通知）/ GitHub（audit trail）

## 期待結果 / 観測指標

- `break_glass_request.lock.yaml` に緊急理由と dual authorization が記録されている
- break-glass 実行中の全操作が audit emit されている
- 事後 post-review が 48 時間以内に完了している
- 権限が最大 TTL（2 時間）以内に返却されている

## 失敗時の挙動 / escalation

- **security on-call 担当者が 15 分以内に応答しない**: `escalation_policy.lock.yaml` の L3 escalation が発火し、security チームリーダーと tech lead に通知する。SLA: 30 分以内に dual authorization 取得
- **audit emit が失敗する（OpenBao 障害等）**: break-glass 実行を中断し、infra 担当者に OpenBao の復旧を escalation する。audit emit なしの break-glass 実行は禁止
- **権限 TTL 内に調査が完了しない**: ops 担当者が security 担当者に延長の dual authorization を取得し、TTL を 1 時間延長する（最大 1 回）

## 失敗パターン (anti-pattern)

1. **dual authorization なしに break-glass を実行しようとする**: `v1_emergency_step_up` は技術的に dual authorization なしでは実行できないよう設計されている。手順をバイパスしようとしない
2. **break-glass を「便利な管理ツール」として日常使用する**: 使用件数の急増が security dashboard に表示され、tech lead レビューの対象になる
3. **事後 post-review を省略する**: audit emit ログが review されず、乱用や操作ミスが発覚しないリスクがある。事後 post-review は強制実施

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [security: break_glass_emergency_step_up_post_review](../../06_security担当者シナリオ/10_break_glass_emergency_step_up_post_review.md)
- [incident_response_7class主導](09_incident_response_7class主導.md)
