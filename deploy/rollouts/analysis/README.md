# deploy/rollouts/analysis — 共通 ClusterAnalysisTemplate 5 本（IMP-REL-AT-040〜049）

ADR-REL-001（PD 必須化）/ ADR-CICD-002（Argo Rollouts）/ `docs/05_実装/70_リリース設計/40_AnalysisTemplate/01_AnalysisTemplate設計.md` に従い、
k1s0 全サービスが共通利用する 5 本の `ClusterAnalysisTemplate` を一元配置する。
サービス固有 AnalysisTemplate は本セットを `templateRef` で継承し、固有指標のみを差分追加する。

## なぜ共通 5 本セットか

各サービスが独自に AnalysisTemplate を書くと、PromQL クエリの記法揺れ・閾値設定の不統一・provider 接続文字列の重複が発生し、
SRE が SLI 改訂時にすべてのサービスをスキャンしなければならない事態が生じる。
共通テンプレを cluster スコープで一元配置することで、SLO 改訂時の影響面が「共通テンプレを 1 箇所変更」に集約される。

## ファイル一覧

| ファイル | IMP-ID | 主題 | failureLimit / count | 失敗条件 |
|---|---|---|---|---|
| `at-common-error-rate.yaml` | IMP-REL-AT-040 | error rate baseline 2σ 超過 | 2 / 5 | 5xx 比率が 30 分 baseline + 2σ を超過 |
| `at-common-latency-p99.yaml` | IMP-REL-AT-041 | レイテンシ p99 SLO 連動 | 2 / 5 | p99 が args 渡し SLO 値超過 |
| `at-common-cpu.yaml` | IMP-REL-AT-042 | CPU 飽和の早期検知 | 8 / 10 | CPU 使用率 80% を 10 分中 8 分以上 |
| `at-common-dependency-down.yaml` | IMP-REL-AT-043 | 依存断短絡判定 | 1 / 3 | Postgres/Kafka/Valkey の `up` が 0 |
| `at-common-error-budget-burn.yaml` | IMP-REL-AT-044 | エラーバジェット fast burn | 2 / 3 | `api:burn_rate_1h` が 2x 超過（14 日枯渇ペース） |

## デプロイ

```sh
# Argo CD が cluster スコープで apply（k1s0-platform AppProject から）
kubectl apply -f deploy/rollouts/analysis/

# 確認
kubectl get clusteranalysistemplates
```

## サービス側の参照

```yaml
# deploy/charts/<service>/templates/rollout.yaml
spec:
  strategy:
    canary:
      steps:
        - setWeight: 25
        - analysis:
            templates:
              - templateName: at-common-error-rate
                clusterScope: true
              - templateName: at-common-latency-p99
                clusterScope: true
            args:
              - name: service-name
                value: tier1-state
              - name: namespace
                value: k1s0-tier1
              - name: slo-latency-seconds
                value: "0.1"
```

サービス固有テンプレ（namespace スコープ AnalysisTemplate）は `deploy/charts/<service>/templates/analysis-template.yaml` に配置し、
共通セットを `templateRef` で参照しつつドメイン固有指標（Temporal completion rate / ZEN Engine 確信度等）を差分追加する。

## カバレッジと監査

共通 5 本でカバーできる SLO 評価範囲は約 80%（error / latency / capacity / dependency / budget の 5 軸）。
残 20% はサービス固有テンプレが補完する。月次カバレッジ計測は `tools/scorecard/at-coverage.sh`（IMP-REL-AT-048）。

## 関連設計

- [ADR-CICD-002](../../../docs/02_構想設計/adr/ADR-CICD-002-argo-rollouts.md) — Argo Rollouts
- [ADR-REL-001](../../../docs/02_構想設計/adr/ADR-REL-001-progressive-delivery-required.md) — PD 必須化
- [ADR-OBS-001](../../../docs/02_構想設計/adr/ADR-OBS-001-grafana-lgtm.md) — Grafana LGTM
- [docs/05_実装/70_リリース設計/40_AnalysisTemplate/01_AnalysisTemplate設計.md](../../../docs/05_実装/70_リリース設計/40_AnalysisTemplate/01_AnalysisTemplate設計.md)
