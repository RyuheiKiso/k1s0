# 50. infra レイアウト

本章は `infra/` 配下のクラスタ素構成の配置を確定する。旧 `src/tier1/infra/` はルート `infra/` に昇格済み（ADR-DIR-002）、`deploy/`（GitOps 配信定義）と `ops/`（Runbook/Chaos/DR）から明確に分離される。

## 本章の対象

Kubernetes クラスタそのものの素構成、すなわち以下を対象とする。

- Kubernetes ブートストラップ（namespace / ネットワーキング / ストレージ）
- サービスメッシュ（Istio Ambient）
- Dapr Control Plane と Dapr Component 定義
- データ層基盤（CloudNativePG / Strimzi Kafka / Valkey / MinIO）
- セキュリティ層（Keycloak / OpenBao / SPIRE / cert-manager / Kyverno）
- 観測性層（LGTM + Pyroscope + OTel Collector）
- フィーチャー管理（flagd）と HPA / KEDA
- 環境別パッチ（dev / staging / prod）

これらは Kubernetes CRD / Helm values / YAML manifest として記述され、GitOps（ArgoCD）が監視する Git リポジトリの一部として配信される。

## 本章の構成

| ファイル | 内容 |
|---|---|
| 01_infra全体配置.md | `infra/` 全体のサブディレクトリ配置 |
| 02_k8sブートストラップ.md | `infra/k8s/` の bootstrap / namespaces / networking / storage |
| 03_サービスメッシュ配置.md | `infra/mesh/` の istio-ambient / envoy-gateway |
| 04_Dapr_Component配置.md | `infra/dapr/` の control-plane / components |
| 05_データ層配置.md | `infra/data/` の cloudnativepg / kafka / valkey / minio |
| 06_セキュリティ層配置.md | `infra/security/` の keycloak / openbao / spire / cert-manager / kyverno |
| 07_観測性配置.md | `infra/observability/` の LGTM + Pyroscope |
| 08_環境別パッチ配置.md | `infra/environments/` の dev / staging / prod |

## 本章で採番する IMP-DIR ID

- IMP-DIR-INFRA-071（infra 全体配置）— `01_infra全体配置.md`
- IMP-DIR-INFRA-072（k8s ブートストラップ）— `02_k8sブートストラップ.md`
- IMP-DIR-INFRA-073（サービスメッシュ配置）— `03_サービスメッシュ配置.md`
- IMP-DIR-INFRA-074（Dapr Component 配置）— `04_Dapr_Component配置.md`
- IMP-DIR-INFRA-075（データ層配置）— `05_データ層配置.md`
- IMP-DIR-INFRA-076（セキュリティ層配置）— `06_セキュリティ層配置.md`
- IMP-DIR-INFRA-077（観測性配置）— `07_観測性配置.md`
- IMP-DIR-INFRA-078（環境別パッチ配置）— `08_環境別パッチ配置.md`

予約 IMP-DIR ID は `IMP-DIR-INFRA-079` 〜 `IMP-DIR-INFRA-090`（feature-management / scaling / multi-region / backup-restore / hardware-profile / cost-optimization / green-ops / air-gap など、運用蓄積後で採番）。本章採番範囲は `IMP-DIR-INFRA-071` 〜 `IMP-DIR-INFRA-090`。

## 対応 ADR

- ADR-DIR-002（infra 分離）
- ADR-CNCF-001〜（CNCF OSS 選定の各 ADR）
- ADR-DATA-001〜（CloudNativePG / Kafka / Valkey の各 ADR）
- ADR-SEC-001〜（Keycloak / OpenBao / SPIRE の各 ADR）
