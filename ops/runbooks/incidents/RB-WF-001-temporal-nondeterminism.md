---
runbook_id: RB-WF-001
title: Temporal NonDeterministicWorkflowError 対応
category: WF
severity: SEV2
owner: 協力者
automation: manual
alertmanager_rule: TemporalNonDeterminismError
fmea_id: 間接対応
estimated_recovery: 暫定 15 分（rollback）/ 恒久 1 時間（versioning 適用）
last_updated: 2026-05-02
---

# RB-WF-001: Temporal NonDeterministicWorkflowError 対応

本 Runbook は Temporal で Workflow が `NonDeterministicWorkflowError` で失敗した時の対応を定める。
本エラーはコード変更が既存実行中の Workflow と非互換である症状で、インフラ操作のみでは解決しない（コード or versioning 戦略変更が必要）。
NFR-C-MON-003 / DS-OPS-RB-012 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + Argo CD app rollback 権限を保持。
- 必要ツール: `kubectl` / `temporal` CLI（`temporal-admintools` Pod 経由）/ `argocd` / Loki / Tempo へのアクセス。
- kubectl context が `k1s0-prod`。
- Temporal クラスタ自体が healthy であること（`kubectl get pods -n temporal` で全 Pod が `Running`）。
- 直前の worker デプロイが Argo CD の history に残っていること（rollback 用）。

## 2. 対象事象

- Alertmanager `TemporalNonDeterminismError` 発火（`temporal_workflow_failed_total{failure_reason="NonDeterministicWorkflowError"} > 0`）、または
- Workflow 失敗率の急増（`rate(temporal_workflow_failed_total[5m]) > 0.1`）、または
- Task Queue バックログ増加（`temporal_workflow_task_schedule_to_start_latency_bucket{le="10"} < 0.9`）。

検知シグナル:

```promql
# NonDeterministicWorkflowError の発生数（1 件でアラート）
sum(rate(temporal_workflow_failed_total{namespace="temporal", failure_reason="NonDeterministicWorkflowError"}[5m])) > 0

# ワークフロー失敗率の急増
rate(temporal_workflow_failed_total{namespace="temporal"}[5m]) > 0.1

# タスクキューのバックログ（ワーカーがエラーで止まっている兆候）
temporal_workflow_task_schedule_to_start_latency_bucket{namespace="temporal", le="10"} < 0.9
```

ダッシュボード: **Grafana → k1s0 Temporal Overview**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#incident-temporal`。

## 3. 初動手順（5 分以内）

```bash
# 失敗ワークフロー特定（temporal-admintools Pod 経由）
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow list \
  --namespace k1s0 \
  --query 'ExecutionStatus="Failed" AND StartTime > "$(date -u -Iseconds -d "1 hour ago")"' \
  --limit 20
```

```bash
# 失敗ワークフローの詳細と履歴
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow describe \
  --namespace k1s0 \
  --workflow-id <workflow-id>

kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow show \
  --namespace k1s0 \
  --workflow-id <workflow-id> \
  --run-id <run-id>
```

```bash
# ワーカー Pod のエラーログ
kubectl logs -n k1s0 -l app=temporal-worker --tail=100 | grep -i "nondetermin\|panic\|fatal"
```

```bash
# どの WorkflowType で発生しているか特定
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow list \
  --namespace k1s0 \
  --query 'ExecutionStatus="Failed"' \
  --fields WorkflowType,WorkflowId,CloseTime | head -20
```

ステークホルダー通知: Slack `#incident-temporal` に「Temporal nondeterminism、影響 Workflow Type <type>、件数 <N>」を投稿。
SEV2 のため `oncall/escalation.md` 起動は不要。ただし Workflow 種別が業務クリティカル（例: 監査用 Saga）の場合は SEV1 昇格を検討。

## 4. 原因特定手順

```logql
# Loki で詳細ログ
{namespace="k1s0", app="temporal-worker"} |= "NonDeterministicWorkflowError" | json
```

Tempo で trace を確認: Grafana → Explore → Tempo → サービス `temporal-worker`、エラーのスパンを特定。

よくある原因:

1. **条件分岐の変更**: `if/else` の条件を追加・削除してコマンド順序が変わった。ワークフロー履歴と新コードの実行パスを突き合わせる。
2. **Activity / Signal の追加・削除**: 既存のワークフロー実行中に新しい Activity を挿入。`GetVersion` API を使わずに追加した。
3. **Timer 変更**: `workflow.NewTimer` の引数（duration）を変更した。
4. **非決定的な外部状態参照**: ワークフロー内で `time.Now()` や乱数、環境変数を直接参照。これらは Activity に移動すべき。
5. **SDK バージョン不整合**: Temporal Go SDK のバージョンアップでシリアライズ形式が変わった（稀）。

エスカレーション: 原因が SDK バージョン不整合（稀）の場合は Temporal コミュニティに issue 提起。それ以外は Workflow 著者を Slack でメンション。

## 5. 復旧手順

**原則: NonDeterministicWorkflowError は コードの問題であり、インフラ操作だけでは解決しない。**

### Step 1 — 対象ワーカーを旧バージョンにロールバックする（即時緩和、暫定 15 分）

```bash
# ArgoCD でワーカー Deployment を前のリビジョンに戻す
argocd app rollback k1s0-temporal-worker --revision <prev-revision>
kubectl rollout status deployment/temporal-worker -n k1s0
```

または `ops/scripts/rollback.sh` を使用:

```bash
ops/scripts/rollback.sh \
  --app k1s0-temporal-worker \
  --revision <prev-good-sha> \
  --reason "Temporal nondeterminism, RB-WF-001"
```

### Step 2 — ロールバック後も残るエラー実行を確認

```bash
# ロールバック後に新規実行が正常稼働しているか確認
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow list \
  --namespace k1s0 \
  --query 'ExecutionStatus="Running"' \
  --limit 10
```

### Step 3 — 互換性のある修正版をデプロイして再実行する

修正版（GetVersion or worker versioning を使用）をデプロイ後:

```bash
# 失敗したワークフローを再開（terminate → 新規起動）
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow terminate \
  --namespace k1s0 \
  --workflow-id <workflow-id> \
  --reason "nondeterminism-fix-redeploy"

# ワークフローを再起動（tier1 facade のリトライ API を呼ぶ、またはイベントを再送）
```

### Step 4 — Temporal worker versioning を有効化する（長期対策、恒久 1 時間）

```bash
# worker versioning で新コードを新 build ID に割り当てる
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal task-queue update-build-id-compatibility \
  --namespace k1s0 \
  --task-queue k1s0-main \
  --add-new-build-id <new-build-id> \
  --promote-set
```

## 6. 検証手順

復旧完了の判定基準:

- 直近 30 分の `temporal_workflow_failed_total{failure_reason="NonDeterministicWorkflowError"}` 増分が 0。
- Workflow 失敗率 `rate(temporal_workflow_failed_total[5m]) < 0.01` が 15 分間継続。
- Task Queue バックログが解消（`temporal_workflow_task_schedule_to_start_latency_bucket{le="10"} > 0.99`）。
- 直近 10 分の Loki クエリ `{app="temporal-worker"} |= "NonDeterministicWorkflowError"` が 0 件。
- 失敗していた Workflow 全件が再実行で `Completed` に到達したか、`Terminated` で監視除外済み。
- 業務集計値（監査レコード数等）が想定範囲内（Workflow 業務影響の確認）。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-WF-001.md`）。
- ワークフロー変更時の `GetVersion` 使用を CI のプリコミットチェックに追加（`tools/lint/temporal-determinism.sh` 整備）。
- Temporal worker versioning を本番に導入（未実施の場合）。プラン化と PR 起案。
- 開発 CLAUDE.md に「ワークフロー変更は GetVersion 必須」を追記。
- NFR-A-REC-002 の MTTR ログを更新。
- 月次 Chaos Drill 対象に「Workflow 中の Activity 追加デプロイ」シナリオを追加。

## 8. 関連 Runbook

- 関連 ADR: [ADR-RULE-002（Temporal 採用決定）](../../../docs/02_構想設計/adr/ADR-RULE-002-temporal.md)
- 関連 NFR: [NFR-C-MON-003](../../../docs/03_要件定義/30_非機能要件/C_運用.md), [NFR-A-REC-002](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §DS-OPS-RB-012
- 公式ドキュメント: Temporal Workflow Versioning with GetVersion / Build ID-based Versioning
- 連鎖 Runbook:
  - [`RB-MSG-002-dlq-backlog.md`](RB-MSG-002-dlq-backlog.md) — Workflow 失敗が PubSub DLQ に流れる場合
  - [`RB-OPS-002-argocd-out-of-sync.md`](RB-OPS-002-argocd-out-of-sync.md)（予定） — ArgoCD rollback で sync 不整合が発生した場合
