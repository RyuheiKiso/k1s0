# 04. 要件定義（NFR / DX）との対応

本ファイルは `IMP-DIR-*` と要件定義書の NFR（非機能要件）・DX（開発者体験）要件の対応を明示する。

## A. 可用性（NFR-A-AVL-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-A-AVL-\*（HA） | IMP-DIR-INFRA-075（データ層） | CloudNativePG 3 replica / Kafka 3 broker |
| NFR-A-AVL-\*（DR） | IMP-DIR-OPS-095（ops DR） | Barman / scheduled backup / drills |
| NFR-A-AVL-\*（証明書自動更新） | IMP-DIR-INFRA-076 | cert-manager ClusterIssuer |
| NFR-A-AVL-\*（StorageClass Retain） | IMP-DIR-INFRA-072 | Longhorn `longhorn-retain` |

## B. 性能・拡張性（NFR-B-PERF-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-B-PERF-\*（低 CPU） | IMP-DIR-INFRA-073 | Istio Ambient（sidecar-less） |
| NFR-B-PERF-\*（BFF キャッシュ） | IMP-DIR-T3-059 | BFF Valkey キャッシュ |
| NFR-B-PERF-\*（KEDA） | IMP-DIR-INFRA-071 | infra/scaling/keda/ |
| NFR-B-PERF-\*（sparse 高速化） | IMP-DIR-SPARSE-126〜132 | partial clone / sparse index |

## C. 運用保守（NFR-C-NOP-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-C-NOP-001（採用側の小規模運用） | IMP-DIR-OPS-095 | Runbook 標準化 |
| NFR-C-NOP-\*（観測性） | IMP-DIR-INFRA-077 | LGTM + Pyroscope |
| NFR-C-NOP-\*（GitOps 単一ソース） | IMP-DIR-OPS-091〜092 | ArgoCD App-of-Apps |
| NFR-C-NOP-\*（Runbook 管理） | IMP-DIR-OPS-095 | ops/runbooks/ |
| NFR-C-NOP-\*（Chaos Engineering） | IMP-DIR-OPS-095 | ops/chaos/（Litmus） |

## D. 移行（NFR-D-MIG-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-D-MIG-\*（.NET Framework 段階移行） | IMP-DIR-T3-060 | レガシーラップ sidecar |
| NFR-D-MIG-\*（環境再現性） | IMP-DIR-COMM-115 | Dev Container 標準化 |
| NFR-D-MIG-\*（環境差分吸収） | IMP-DIR-INFRA-078, IMP-DIR-OPS-094 | Kustomize overlays |

## E. セキュリティ（NFR-E-SEC-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-E-SEC-\*（mTLS 強制） | IMP-DIR-INFRA-073, IMP-DIR-INFRA-076 | Istio STRICT mTLS + SPIRE |
| NFR-E-SEC-\*（secret 管理） | IMP-DIR-INFRA-076 | OpenBao |
| NFR-E-SEC-\*（image 署名必須） | IMP-DIR-INFRA-076 | Kyverno require-image-signature |
| NFR-E-SEC-\*（default-deny） | IMP-DIR-INFRA-072 | NetworkPolicy default-deny |

## G. データ保護・プライバシー（NFR-G-PRV-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-G-PRV-\*（PII 除去） | IMP-DIR-INFRA-077 | OTel Collector pii-redact processor |
| NFR-G-PRV-\*（監査ログ長期保存） | IMP-DIR-INFRA-075 | audit Cluster + MinIO audit-archive |
| NFR-G-PRV-\*（モバイル端末データ保護） | IMP-DIR-T3-058 | MAUI ローカルキャッシュ暗号化 |

## H. 完整性・コンプライアンス（NFR-H-COMP-\*）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| NFR-H-COMP-\*（OSS ライセンス遵守） | IMP-DIR-COMM-114 | third_party/LICENSES/ + NOTICE |
| NFR-H-COMP-\*（監査証跡） | IMP-DIR-INFRA-075 | CloudNativePG audit Cluster |

## DX. 開発者体験（DX-GP-\* / DX-CICD-\*）

### DX-GP-\*（Golden Path）

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| DX-GP-\*（雛形 CLI） | IMP-DIR-COMM-116 | tools/codegen/scaffold/ |
| DX-GP-\*（example Golden Path） | IMP-DIR-COMM-113 | examples/ |
| DX-GP-\*（Dev Container） | IMP-DIR-COMM-115 | tools/devcontainer/profiles/ |
| DX-GP-\*（Backstage） | IMP-DIR-OPS-096 | Backstage Software Catalog |
| DX-GP-\*（sparse 役割別） | IMP-DIR-SPARSE-127 | 10 役割 cone 定義 |

### DX-CICD-\*

| 要件 | IMP-DIR ID | 寄与 |
|---|---|---|
| DX-CICD-\*（path-filter） | IMP-DIR-SPARSE-130 | GitHub Actions path-filter |
| DX-CICD-\*（GitOps） | IMP-DIR-OPS-091 | ArgoCD + Argo Rollouts |
| DX-CICD-\*（IaC 宣言的管理） | IMP-DIR-OPS-097 | OpenTofu |
| DX-CICD-\*（共通 Helm chart） | IMP-DIR-OPS-093 | deploy/charts/ |
| DX-CICD-\*（テスト戦略） | IMP-DIR-COMM-112 | tests/contract/integration/fuzz/golden/ + tests/e2e/{owner,user}/（ADR-TEST-008 で 2 module 分離、実装時に配置） |

## 制約との対応

### 制約 8（.NET Framework sidecar）

| IMP-DIR ID | 寄与 |
|---|---|
| IMP-DIR-T3-060 | レガシーラップ配置 |
| IMP-DIR-INFRA-074 | Dapr sidecar 注入設定 |

## 対応未充足の要件（運用蓄積後で対応）

| 要件 | 適用段階 | 予定 |
|---|---|---|
| NFR-F-ENV-\*（システム環境・エコロジー） | リリース時点 | 消費電力・CO2e 試算（infra/scaling/keda で自動縮退と連動） |
| NFR-A-AVL-\*（multi-region） | 採用後の運用拡大時 | infra/environments/ に region 別 overlay 追加 |
| DX-GP-\*（採用後の運用拡大時 マルチテナント UI） | 採用後の運用拡大時 | tier3 web マルチテナント対応 |

本プランでは リリース時点-1a の範囲で定義される要件に限定して IMP-DIR を対応付ける。
