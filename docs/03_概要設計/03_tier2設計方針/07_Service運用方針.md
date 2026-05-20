---
id: arch.tier2.service_operation_policy
axis: tier2
phase: architecture
kind: policy
status: published
version: 1.0.0
depends_on:
  - arch.tier2.tier2_index
  - arch.tier2.multi_tenant_policy
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier2 Service 運用方針

## 一文方針
- tier2 機能カテゴリは Library 形態（in-process）または Service 形態（独立 Pod）で配布する。同一機能を両形態で提供する場合は共通 conformance test suite で両方 green を merge 条件とし、Service 形態は共有 Pod / 共有 DB のマルチテナント運用を default に保護四層 + 自動昇格 trigger で SLO 違反波及を抑制する。

## 配布形態の選択
- **Library 形態**: tier3 業務 process に in-process linkage（言語別 package）
  - 利点: latency 最小、tier1 Library と同 process 内で context propagation
  - 欠点: tier3 process の resource を共有
- **Service 形態**: 独立 Pod として動作（Helm / Operator manifest）
  - 利点: tier3 と独立した resource 管理、tier3 不在環境でも動作
  - 欠点: network latency、context propagation オーバーヘッド
- 両形態提供カテゴリは [Library Service 等価性](../../04_詳細設計/02_強制機構/02_tier2強制機構.md) 層 7 を必須

## 共有 Pod / 共有 DB のマルチテナント運用
- 既定: 複数テナントが同居する共有 Pod / 共有 DB
- 専用 Pod / 専用 DB への昇格条件は [マルチテナント方針](05_マルチテナント方針.md) を参照
- 昇格時にも業務コードは無変更（リポジトリ抽象が接続先を切替）

## SLO 違反波及の抑制と自動昇格
- 「テナント間 SLO 違反波及確率を error-budget の 5% 以下に抑える」を SLO 化
- 抑制超過時は自動昇格 trigger が 5 min 以内に専用 Pod / 専用 DB へ promote
- 詳細は [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md) を参照

## 採用しない設計
- per-tenant rate limiter で「SLO 違反波及なし担保」と表現する文言（「保証」「担保」表現は禁止）
- 共有運用での hysteresis なき自動昇格（flapping 抑止が必須）
- 手動 Provision での専用 Pod 切替（Backstage + ArgoCD 自動化前提）

## Operator reconcile loop 仕様

`K1s0ServiceReconciler` は `K1s0Service` CRD を監視し、以下の順序で 6 種類の Kubernetes リソースを同期する。

### 観測対象（Observe）

| 観測リソース | 確認内容 |
|---|---|
| `K1s0Service` CRD | `spec.tenantId` / `spec.quotaClass` / `spec.featureFlags` の変化 |
| `Deployment` | レプリカ数・コンテナリソース制限が `quotaClass` 相当か |
| `HorizontalPodAutoscaler` | `minReplicas` / `maxReplicas` / CPU utilization threshold が `quotaClass` 相当か |
| `Ingress` | tenant サブドメイン（`{tenantId}.tier2.k1s0.internal`）・TLS Secret が一致するか |
| `ServiceAccount` | `{name}-sa` が存在するか |
| `NetworkPolicy` | `{name}-netpol` — 同一 Namespace 内 Pod からの Ingress のみ許可しているか |

### 変更内容（Actuate）

| ステップ | 変更操作 | 変更条件 |
|---|---|---|
| 1 | Status を `Provisioning` に更新 | Reconcile 開始時（常に実行） |
| 2 | ServiceAccount `{name}-sa` を Create | 存在しない場合 |
| 3 | Deployment を Create / Update | 存在しない場合は Create。存在する場合は `spec.replicas` と containers[0].resources を quotaClass 相当に Update |
| 4 | Service（ClusterIP）を Create | 存在しない場合 |
| 5 | NetworkPolicy `{name}-netpol` を Create | 存在しない場合 |
| 6 | HPA `{name}-hpa` を Create / Update | 存在しない場合は Create。存在する場合は `minReplicas` / `maxReplicas` / `metrics` を Update |
| 7 | Ingress `{name}-ingress` を Create / Update | 存在しない場合は Create。存在する場合は `rules` と `tls` を Update |
| 8 | Status を `Active` に更新 | 全ステップ成功後 |

### quotaClass → リソースマッピング

| quotaClass | CPU req/limit | Memory req/limit | min replicas | max replicas |
|---|---|---|---|---|
| `standard` | 100m / 500m | 128Mi / 512Mi | 2 | 5 |
| `enterprise` | 250m / 2 | 512Mi / 2Gi | 3 | 10 |
| `unlimited` | 500m / 4 | 1Gi / 4Gi | 3 | 50 |
| （不明） | 100m / 500m | 128Mi / 512Mi | 2 | 5 |

HPA CPU utilization threshold は全 quotaClass 共通 70%。

### reconcile 間隔と requeue 条件

| 条件 | 動作 |
|---|---|
| 全ステップ成功 | `RequeueAfter: 5 minutes`（定期状態チェック） |
| `tenantId` 未設定 | `RequeueAfter: 10 minutes`（警告ログ後スキップ） |
| `quotaClass` 未設定 | `RequeueAfter: 10 minutes`（警告ログ後スキップ） |
| いずれかのステップでエラー | 即時リトライ（controller-runtime の exponential backoff に委ねる） |
| `K1s0Service` リソースが存在しない | 正常終了（削除済みとみなす） |

全リソースに `OwnerReference（controller=true, blockOwnerDeletion=true）` を付与し、`K1s0Service` 削除時にカスケード GC される。

## 関連参照
- [tier2 設計方針 index](README.md)
- [マルチテナント方針](05_マルチテナント方針.md)
- [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md)
- [tier2 強制機構](../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
