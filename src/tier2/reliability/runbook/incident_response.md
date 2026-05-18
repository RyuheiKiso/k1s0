# tier2 インシデント対応ランブック

設計方針 18（信頼性運用準備）に基づくインシデント対応手順。詳細は `docs/03_概要設計/03_tier2設計方針/README.md` を参照。

## Phase 1: 検知 (Detect)

- アラートソース確認: Prometheus AlertManager / PagerDuty / Grafana
- 影響テナント特定: `SELECT tenant_id, COUNT(*) FROM audit_log WHERE created_at > NOW() - INTERVAL '5 minutes' GROUP BY tenant_id`
- SLO ダッシュボード確認: error rate / latency p99 / availability

## Phase 2: トリアージ (Triage)

- 重大度判定: P1（全テナント影響）/ P2（一部テナント影響）/ P3（単一テナント影響）
- 担当者アサイン: Slack `#k1s0-incidents` チャンネルに投稿
- インシデント ID 払い出し: `INC-YYYYMMDD-NNN` 形式
- ステータスページ更新

## Phase 3: 緊急軽減 (Mitigate)

- トラフィック制御: Istio VirtualService で障害ポッドへのルーティングを停止する
- フォールバック有効化: tier2 サーキットブレーカーの OPEN 状態を確認する
- テナント隔離: 影響テナントの RLS 追加述語で他テナントへの波及を防ぐ
- 緊急アクセス要求: `EmergencyAccess` AdminOperation でデュアル承認を経て実施する

## Phase 4: 根本解決 (Resolve)

- 原因調査: `kubectl logs` / Jaeger トレース / pgaudit ログ照合
- パッチ適用: feature branch → PR → CI green → staging 検証 → prod deploy
- Chaos 実験再実行: Litmus `tier2-pod-delete-engine` で回帰確認
- SLO 回復確認: error rate が SLO 閾値以下に戻ったことを確認する

## Phase 5: ポストモーテム (Postmortem)

- 発生から解決までのタイムラインを docs/postmortem/INC-YYYYMMDD-NNN.md に記録する
- 根本原因（5-Why 分析）を明記する
- 再発防止アクションを GitHub Issue として登録し担当者を割り当てる
- ポストモーテムレビュー会議を 48 時間以内に開催する
