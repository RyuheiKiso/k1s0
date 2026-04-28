# Temporal NonDeterministicWorkflowError 対応 Runbook

> **alert_id**: tier1.temporal.workflow.nondeterminism-error
> **severity**: SEV2
> **owner**: tier1-platform-team
> **estimated_mttr**: 60m
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

**Mimir / Grafana** で以下を確認する。

PromQL（Mimir）:

```promql
# NonDeterministicWorkflowError の発生数（1 件でアラート）
sum(rate(temporal_workflow_failed_total{namespace="temporal", failure_reason="NonDeterministicWorkflowError"}[5m])) > 0

# ワークフロー失敗率の急増
rate(temporal_workflow_failed_total{namespace="temporal"}[5m]) > 0.1

# タスクキューのバックログ（ワーカーがエラーで止まっている兆候）
temporal_workflow_task_schedule_to_start_latency_bucket{namespace="temporal", le="10"} < 0.9
```

ダッシュボード: **Grafana → k1s0 Temporal Overview**。

alert チャンネル: PagerDuty `tier1-platform-team` → Slack `#incident-temporal`。

## 2. 初動 (Immediate Action, 〜15 分)

- [ ] Temporal 管理 UI またはコマンドで失敗したワークフローを特定する

  ```bash
  # temporal CLI（cluster 内の temporal-frontend サービスに接続）
  kubectl exec -n temporal deploy/temporal-admintools -- \
    temporal workflow list \
    --namespace k1s0 \
    --query 'ExecutionStatus="Failed" AND StartTime > "2026-04-28T00:00:00Z"' \
    --limit 20
  ```

- [ ] 失敗したワークフローの詳細と履歴を確認する

  ```bash
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

- [ ] ワーカー Pod のエラーログを確認する

  ```bash
  kubectl logs -n k1s0 -l app=temporal-worker --tail=100 | grep -i "nondetermin\|panic\|fatal"
  ```

- [ ] どのワークフロー種別（WorkflowType）で発生しているか特定する

  ```bash
  kubectl exec -n temporal deploy/temporal-admintools -- \
    temporal workflow list \
    --namespace k1s0 \
    --query 'ExecutionStatus="Failed"' \
    --fields WorkflowType,WorkflowId,CloseTime | head -20
  ```

## 3. 復旧 (Recovery, 〜60 分)

**原則: NonDeterministicWorkflowError は コードの問題であり、インフラ操作だけでは解決しない。**

**ステップ 1 — 対象ワーカーを旧バージョンにロールバックする（即時緩和）**:

```bash
# ArgoCD でワーカー Deployment を前のリビジョンに戻す
argocd app rollback k1s0-temporal-worker --revision <prev-revision>
kubectl rollout status deployment/temporal-worker -n k1s0
```

**ステップ 2 — ロールバック後も残るエラー実行を確認する**:

```bash
# ロールバック後に新規実行が正常稼働しているか確認
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow list \
  --namespace k1s0 \
  --query 'ExecutionStatus="Running"' \
  --limit 10
```

**ステップ 3 — 互換性のある修正版をデプロイして再実行する**:

修正版（GetVersion や versioning API を使用）をデプロイ後:

```bash
# 失敗したワークフローを再開（terminate → 新規起動）
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal workflow terminate \
  --namespace k1s0 \
  --workflow-id <workflow-id> \
  --reason "nondeterminism-fix-redeploy"

# ワークフローを再起動（tier1 facade のリトライ API を呼ぶ、またはイベントを再送）
```

**ステップ 4 — Temporal worker versioning を有効化する（長期対策）**:

```bash
# worker versioning で新コードを新 build ID に割り当てる
kubectl exec -n temporal deploy/temporal-admintools -- \
  temporal task-queue update-build-id-compatibility \
  --namespace k1s0 \
  --task-queue k1s0-main \
  --add-new-build-id <new-build-id> \
  --promote-set
```

## 4. 原因調査 (Root Cause Analysis)

**よくある原因**:

1. **条件分岐の変更**: `if/else` の条件を追加・削除してコマンド順序が変わった。ワークフロー履歴と新コードの実行パスを突き合わせる。
2. **Activity / Signal の追加・削除**: 既存のワークフロー実行中に新しい Activity を挿入。`GetVersion` API を使わずに追加した。
3. **Timer 変更**: `workflow.NewTimer` の引数（duration）を変更した。
4. **非決定的な外部状態参照**: ワークフロー内で `time.Now()` や乱数、環境変数を直接参照。これらは Activity に移動すべき。
5. **SDK バージョン不整合**: Temporal Go SDK のバージョンアップでシリアライズ形式が変わった（稀）。

**Loki でのログ確認**:

```logql
{namespace="k1s0", app="temporal-worker"} |= "NonDeterministicWorkflowError" | json
```

**Tempo でのトレース確認**:

Grafana → Explore → Tempo → サービス `temporal-worker`、エラーのスパンを特定。

## 5. 事後処理 (Post-incident)

- [ ] ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-temporal-nondeterminism.md`）
- [ ] ワークフロー変更時の `GetVersion` 使用を CI のプリコミットチェックに追加
- [ ] Temporal worker versioning を本番に導入（未実施の場合）
- [ ] 開発 CLAUDE.md に「ワークフロー変更は GetVersion 必須」を追記
- [ ] NFR-A-REC-002 の MTTR ログを更新

## 関連

- 関連 ADR: `docs/02_構想設計/adr/ADR-RULE-002`（Temporal 採用決定）
- Temporal 公式ドキュメント: Workflow Versioning with GetVersion
- 関連 Runbook: `ops/runbooks/incidents/dlq-backlog.md`
