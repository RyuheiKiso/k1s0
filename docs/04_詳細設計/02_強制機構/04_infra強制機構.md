---
id: detail.infra.infra_enforcement
axis: infra
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.infra.infra_index
  - arch.infra.cluster_composition_policy
  - arch.infra.network_policy
  - arch.infra.storage_policy
  - arch.infra.security_policy
  - arch.infra.gitops_delivery_policy
  - arch.infra.autoscale_policy
  - arch.infra.chaos_engineering_policy
  - arch.infra.iac_policy
  - detail.infra.cluster_topology_conformance
  - detail.infra.clock_integrity_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts:
  - infra_enforcement.lock.yaml
---

# infra 強制機構

## 一文方針
- infra の規律は **PR 段の Conftest（OPA Rego、GitHub Actions Required Status Check で merge block）+ deploy 段の Kyverno admission policy（cluster で deploy reject）の二段** で物理 enforce する。手書き policy は CI fail、policy 自体は build artifact から生成、最終 safety net は OS / Kubernetes / OSS の物理層機能（etcd quorum / Ceph CRUSH map / Kafka rack awareness）。

## 5 層 defense-in-depth

### 層 A: compile（IaC コードの type / schema check）
- OpenTofu plan: terraform fmt / tflint
- Kustomize / Helm: kustomize build / helm lint
- manifest: kubeval / kubeconform / OpenAPI schema
- `cluster_inventory.lock.yaml` / `failover_drill.lock.yaml`: build artifact、手書き禁止

### 層 B: lint（ポリシー / 規約 check）
- Conftest custom rule（Rego policy）:
    - 全 Application に topology_class / quota_class / preservation_class 注釈必須
    - Kyverno admission policy が build artifact から生成されている（手書き policy は CI fail）

### 層 C: integration test（Testcontainers / kuttl / chainsaw）
- 全 admission policy が intended state を block / pass することを test
- chaos test の自動 replay（Litmus / failover_drill / restore_drill）
- 全 cluster の `cluster_inventory.lock.yaml` が `classes.yaml` に整合

### 層 D: runtime（cluster 上の actual enforcement）
- Kyverno admission webhook: deploy 前に block
- PSA（Pod Security Admission）: Pod 起動前 block
- NetworkPolicy / Istio AuthorizationPolicy: runtime 通信 block
- Argo CD reconciliation: drift 検出 + self-heal

### 層 E: 物理 enforcement（OS / Kubernetes / OSS の物理機能）
- etcd quorum: split-brain を物理的に防ぐ
- Ceph CRUSH map: zone failure_domain を物理層で強制
- Kafka rack awareness: min.insync.replicas を rack 跨ぎで物理強制
- CloudNativePG synchronous replication: WAL 物理層
- kube-apiserver TLS / mTLS: unencrypted 通信を物理層で reject
- LUKS / Ceph at-rest encryption: disk 物理層で暗号化
- kernel SELinux / AppArmor / seccomp: syscall 物理層で block

## Kyverno admission policy 一覧（infra 担当部分）

| policy 名 | 内容 |
|---|---|
| `inspect-image-signature` | Cosign signed でない image を reject |
| `require-pod-security-restricted` | restricted profile 適用 |
| `require-network-policy` | default deny NetworkPolicy 必須 |
| `require-istio-strict-mtls` | mTLS STRICT 必須 |
| `require-topology-spread` | 全 Deployment / StatefulSet に topology_spread_constraints 必須 |
| `require-pdb` | 全 Deployment / StatefulSet に PodDisruptionBudget 必須 |
| `require-topology-class-annotation` | 全 Application に `infra.topology.topology_class` 必須 |
| `require-clock-integrity-class-annotation` | 全 node Pod に `infra.clock_integrity.clock_integrity_class` 注釈必須 |
| `require-quota-class-annotation` | 全 RPC method / Library public API に `tier1.capacity.quota_class` 必須 |
| `require-preservation-class-annotation` | 全 data 軸 schema に `data.preservation.preservation_class` 必須 |
| `block-on-failover-drill-red` | `failover_drill.lock.yaml` の red cluster に新規 deploy 拒否 |
| `block-on-clock-drill-red` | `clock_drill.lock.yaml` の cadence 越え node に新規 deploy 拒否 |
| `block-on-restore-drill-red` | data01 `restore_drill.lock.yaml` の red cluster に新規 deploy 拒否 |
| `block-on-error-budget-freeze` | 13 SLO error budget freeze 状態の Service に新規 canary 拒否 |
| `block-on-cve-backlog-threshold` | 14 OSS lifecycle の cve_backlog_threshold 違反 image を reject |
| `block-dimension-override` | quota_class / topology_class / preservation_class / clock_integrity_class の dimension を coding で flag した PR を reject |
| `require-encrypted-pv` | encryption=true でない PV を reject |
| `block-host-network-host-pid` | 例外 allowlist 以外で hostNetwork / hostPID / hostPath を reject |
| `require-readonly-root-fs` | root filesystem read-only 必須 |
| `require-non-root-user` | runAsNonRoot=true 必須 |
| `require-image-from-harbor` | Harbor 以外の registry からの pull を reject |
| `block-manual-secret-creation` | External Secrets 経由でない Secret 作成を reject |
| `require-priority-class` | 明示的 PriorityClass 必須 |
| `block-host-time-api-usage` | wall-clock TTL API（`SystemTime::now()` 等）の TTL 判定使用を reject |

## admission policy のライフサイクル
- 全 admission policy は build artifact（`infra/policy/kyverno-policies.yaml`）から生成、手書き禁止
- generator の input:
    - `classes.yaml`（quota / topology / preservation / clock_integrity 各軸）
    - `cluster_inventory.lock.yaml`
    - `cve_backlog.lock.yaml`（14）
    - `failover_drill.lock.yaml`
    - `restore_drill.lock.yaml`（data01）
    - `clock_drill.lock.yaml`
    - `error_budget.lock.yaml`（13）
- drift: runtime に admission policy が build artifact と divergence した場合、Argo CD が drift として検出 + self-heal

## break-glass
- 緊急時に admission policy を bypass する経路:
    1. OpenBao response wrapping で短期 cluster-admin token を発行
    2. 発行操作は 09 観測可能性 SoR に audit signal を emit（`infra.break_glass.event`）
    3. 使用後 1 時間以内に postmortem PR 起票必須
    4. break-glass 使用は 13 SLO error budget consumption として記録

## CI 不変条件（infra 全体）
- 違反は merge 不可

| # | 整合 |
|---|---|
| 整合 1 | 全 Kustomize overlay / Helm chart に topology_class 注釈 |
| 整合 2 | 全 Application に owning team / oncall 連絡先 annotation |
| 整合 3 | `cluster_inventory.lock.yaml` は build artifact と git の完全一致 |
| 整合 4 | `failover_drill.lock.yaml` の全 cluster で last_green_at が drill_cadence_days 以内（v1_single_zone を除く）|
| 整合 5 | 全 admission policy が build artifact から生成 |
| 整合 6 | Harbor 以外の registry 参照ゼロ |
| 整合 7 | signed でない image 参照ゼロ |
| 整合 8 | trivy / grype scan で critical / high CVE 未対応ゼロ |
| 整合 9 | 全 OSS バージョンが `oss_inventory.lock.yaml` と整合 |
| 整合 10 | 13 軸 + meta（00）の各 spec の build artifact と infra の `cluster_inventory.lock.yaml` / `failover_drill.lock.yaml` / `clock_inventory.lock.yaml` が双方向 lock |

## 採用しない強制機構
- OPA Gatekeeper: Kyverno L1+ 単一深耕、横並びは持たない
- 文章 only の運用ルール: 禁止
- Falco / Tetragon は runtime threat detection の L1+ として security 層所有として cross 参照のみ

## 関連参照
- [infra 設計方針 index](../../03_概要設計/05_infra設計方針/README.md)
- [クラスタ位相適合仕様](../01_適合仕様/12_クラスタ位相適合仕様.md)
- [時刻整合適合仕様](../01_適合仕様/13_時刻整合適合仕様.md)
- [infra 運用 UI](../04_運用UI開発者体験/01_infra運用UI.md)
