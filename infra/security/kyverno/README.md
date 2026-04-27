# infra/security/kyverno

ADR-POL-001（Policy as Code、Kyverno 二重オーナー設計）に従い、
Kubernetes の admission control / background scan / mutation を Kyverno で運用。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | Kyverno Helm values（admission 3 replica + background/cleanup/reports 各 2 replica + ServiceMonitor + ValidatingAdmissionPolicy） |
| `baseline-policies.yaml` | k1s0 全クラスタに適用する baseline ClusterPolicy 4 件（runAsNonRoot / privileged 禁止 / k1s0.io/component label / resource requests） |

## ローカル開発との差分

| 観点 | dev | prod |
|---|---|---|
| 各 controller replica | 1 | admission 3 / 他 2（HA） |
| ServiceMonitor | 無効 | 有効 |
| baseline policy | （未配置） | 4 件適用（runAsNonRoot / privileged 禁止 / 必須 label / resource requests） |

## ADR-POL-001 二重オーナー設計

Policy 二重所有: 「security 担当」が baseline / runtime 制限を、「platform 担当」が k1s0 固有
規約（component label / namespace 命名）を持つ。本ディレクトリは security 側 baseline と
共通規約を含む。tier 別固有 policy は `infra/security/kyverno/policies/<tier>/` に配置予定（plan 06-XX）。

## 関連

- [ADR-POL-001](../../../docs/02_構想設計/adr/ADR-POL-001-kyverno-policy.md)
