---
id: req.overview.oss_catalog
axis: overview
phase: requirement
kind: oss
status: draft
depends_on:
  - arch.formal.formal_index
  - detail.tier1.oss_lifecycle_conformance
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# OSS 採用一覧

## 一文方針
- 本ドキュメントは 19 軸全体で採用する OSS の単一の真として、機能カテゴリ × v1_l1plus_primary（単一深耕）の対応を一箇所に固定する。OSS ライセンス階層（[OSS ライセンス規律](02_OSSライセンス規律.md)）の (a)〜(e) いずれかに必ず帰属し、(d) AGPL/SSPL は不採用、横並び OSS は持たない。

## 至高路線における立ち位置
- **単一 OSS L1+ 深耕**: 各機能カテゴリに 1 OSS のみ採用、横並びは不採用。差し替えは OSS ライフサイクルイベントとして扱う。
- **「機能削減なし」**: 本一覧から OSS を削除する場合は必ず移行 toolchain + migration drill を提供する。
- **「文章のみの選定」を禁止**: 採用 OSS は全て [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) の `oss_lifecycle.lock.yaml` に entry 登録、major version migration 演習が必須。

## 軸別採用 OSS

### tier1 軸（v1_l1plus_primary、単一深耕）

#### Server 系 / Library / Companion ランタイム
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Server 系言語 | Rust（stable） | MIT / Apache 2.0 |
| Operator / Controller 言語 | Go 1.22 + controller-runtime / kubebuilder | Apache 2.0 |
| Library 対応言語 | Rust / C# (.NET 8+) / Go / TypeScript | MIT / Apache 2.0 / BSD-3 / Apache 2.0 |
| Companion ランタイム | .NET Framework 4.6.2+（Companion 専用） | MIT |

#### Proto / Schema / Codegen
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Schema 単一の真 | protobuf + Buf | BSD-3 / Apache 2.0 |
| Buf custom lint | buf | Apache 2.0 |
| Schema Registry（primary） | Apicurio Registry | Apache 2.0 |
| Schema Registry（L2\* pair） | Karapace | Apache 2.0 |
| OTel SemConv | OpenTelemetry Weaver | Apache 2.0 |
| DDL migration | sqlx-cli | Apache 2.0 / MIT |

#### L1+ 機能カテゴリ（単一深耕、移行コミットメント）
| カテゴリ | 本命 | 移行 pair |
|---|---|---|
| Relational Store / Single-leader | PostgreSQL（CloudNativePG）+ PgBouncer + pg_partman | StackGres |
| Vector Search | pgvector | PG ecosystem 内のみ toolchain 保証（族外 standalone は project_spawn） |
| Messaging / EventBus | Apache Kafka（Strimzi）+ Debezium | RedPanda |
| Workflow / Long-running Saga | Temporal | Cadence |
| Rule Engine | ZEN Engine | 自製 DSL backend（Rust 実装、v1_inhouse_authoritative） |

#### L2\* 機能カテゴリ（同族保証、2 実装 conformance）
| カテゴリ | 同族メンバ |
|---|---|
| Profiling | Parca + Pyroscope |
| Configuration / Feature Flag | flagd + GO Feature Flag（GrowthBook 等の OpenFeature provider 族） |
| Schema Registry | Apicurio Registry + Karapace |

#### L3 機能カテゴリ（OSS 中立、wire 互換）
| カテゴリ | 採用 OSS（複数選択肢可） |
|---|---|
| Logging / Tracing / Metrics | OpenTelemetry SDK + OTLP |
| Authentication / Authorization | Keycloak |
| Secret Management | OpenBao + External Secrets Operator |
| KeyValue / Cache | Valkey（族: Dragonfly / KeyDB） |
| Object Storage | Rook + Ceph（RGW）（族: MinIO / SeaweedFS） |
| RPC / Gateway | Envoy Gateway + gRPC |

#### Transport Adapter Layer 実装
| adapter | substrate | 実装 OSS |
|---|---|---|
| grpc_native | HTTP/2 + gRPC | 言語別 gRPC 実装 |
| connect_bidi | HTTP/2 + Connect-RPC / HTTP/3 + Connect-RPC | `@connectrpc/connect-web` / `@connectrpc/connect-node` / `connectrpc.com/connect` / `connect-kotlin` / `connect-swift` / `k1s0.Connect.NetCore`（v1_inhouse_authoritative） |
| grpc_web | HTTP/1.1 + gRPC-Web filter | grpc-web |
| web_transport | HTTP/3 + QUIC + WebTransport | quinn / s2n-quic（v1 opt-in） |
| sse_paired | HTTP/1.1 chunked + SSE | tier1 Rust ハンドラ（SSE フレーミング直接生成）+ Companion パーサ |
| paired_post_sse | POST /open + SSE + POST /send + POST /close | tier1 Rust + 自製 |
| long_poll | HTTP/1.1 cursor poll | tier1 Rust 自製 |
| webhook | inbound POST | tier1 Rust 自製 |
| messaging_bridge | Kafka / AMQP | Apache Kafka（Strimzi） |

#### 鍵管理 / 認証 backend
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| OIDC / token issuer | Keycloak | Apache 2.0 |
| Secret store | OpenBao | MPL 2.0 |
| Workload identity | SPIRE / SPIFFE | Apache 2.0 |
| Cert manager | cert-manager | Apache 2.0 |
| HSM PKCS#11（production） | Thales Luna / YubiHSM 2 / Entrust nShield / Nitrokey NetHSM（multi-vendor、FIPS 140-3 Level 3） | Vendor / OSS |
| HSM（dev / drill 専用） | SoftHSM2 | BSD-2 |
| Shamir Secret Sharing | vendored Rust crate `shamir-secret-sharing` | Apache 2.0 |

#### 採用しない OSS
- **gRPC-over-WebSocket 自製 frame protocol**: gRPC 公式仕様外、Connect-RPC L1+ 単一深耕に統一
- **HashiCorp Vault**: BUSL、OpenBao 一本
- **OPA Gatekeeper**: Kyverno L1+ 単一深耕
- **Confluent Schema Registry v5+**: Confluent Community License、Apicurio Registry / Karapace で吸収
- **MongoDB / Elasticsearch / OpenSearch / Redis / MySQL / Cassandra**: tier2/data 軸の非採用と整合
- **Standalone vector engine（Qdrant / Milvus / Weaviate）**: pgvector の transactional ANN / SQL JOIN を持たないため、L1+ toolchain 保証範囲外
- **cloud lock-in 移行先候補**（Aurora / Neon / AWS MSK / Confluent Cloud / Temporal Cloud）: 1.0.0 primary pair に採用しない（v2 以降）
- **`google.protobuf.Any` の External Proto 直接利用**: oneof / `bytes` + content_type に置換
- **Transcoder upstream の SSE 派生 option 依存**: tier1 Rust ハンドラ内で SSE フレーミング直接生成
- **community plugin（jaegertracing/jaeger-clickhouse）**: archive 済 + Jaeger v2 で plugin protocol 廃止、Jaeger v2 first-party 内蔵 ClickHouse storage を採用

### tier2 軸（v1_l1plus_primary、単一深耕）

#### tier2 業務 service / Library 言語
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| 主要言語（業務 service / Library） | Rust / C# (.NET 8+) / Go / TypeScript | MIT / Apache 2.0 / BSD-3 / Apache 2.0 |
| 内部 RPC | Internal Proto + gRPC（tier1 Library 経由） | Apache 2.0 |
| Repository abstraction | sqlx（Rust） / EF Core（C#） / sqlx-go（Go） / typeorm（TS）（tier1 Library 経由） | Apache 2.0 / MIT |
| Workflow 抽象 | tier1 Workflow Library（Temporal client） | Apache 2.0 |
| Rule Engine 抽象 | tier1 Rule Engine Library（ZEN Engine client） | Apache 2.0 |

#### 業界 pack 並立 / Domain Event
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Domain Event Avro schema | Apicurio Registry + Buf for Apicurio plugin（tier1 経由） | Apache 2.0 |
| Schema GitOps SoT | git + Argo CD + Apicurio Operator | Apache 2.0 |
| 第二業界 stub | 自製 service_industry_stub（CI 専用） | Apache 2.0 |
| 命名禁則 lint | 言語別 cargo public-api / PublicApiAnalyzers / api-extractor / go-apidiff | Apache 2.0 / MIT |
| 依存方向 lint | cargo-deny / Roslyn analyzer / depguard / eslint-plugin-boundaries | Apache 2.0 / MIT |

#### マルチテナント / RLS
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| RLS engine | PostgreSQL Row Level Security FORCE（tier1 Library 経由） | PostgreSQL |
| pgaudit | pgaudit | PostgreSQL |
| connection pool | PgBouncer（per-tenant pool + tenant sharding） | ISC |
| session GUC 注入 | tier1 Library Repository abstraction | Apache 2.0 |
| atomic 三表書込 | tier1 Library transaction abstraction | Apache 2.0 |

#### 状態遷移 FSM
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Rust phantom type | tier1 Library codegen（既存） | Apache 2.0 |
| C# phantom type + Roslyn analyzer | tier1 Library codegen + 自製 K1s0Analyzer | Apache 2.0 |
| TypeScript branded type + ESLint custom rule | tier1 Library codegen + 自製 eslint-plugin-k1s0 | Apache 2.0 |
| Go typestate via nominal types | 自製 `protoc-gen-k1s0-go-fsm`（v1_inhouse_authoritative） | Apache 2.0 |
| Go bypass 防止 | golangci-lint forbidigo + go-exhaustive | MIT / Apache 2.0 |

#### 共有 Pod / DB SLO 保護四層 + 自動昇格
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| 層 SLO-A: per-tenant rate limiter | Envoy Gateway Local Rate Limit filter（tier1 経由） | Apache 2.0 |
| 層 SLO-B: cgroup quota | Linux cgroup v2（OS native） | GPL / OS |
| 層 SLO-C: PG pool tenant 分離 | PgBouncer multi-pool + tenant sharding | ISC |
| 層 SLO-D: Kafka quota | Apache Kafka Quota（client_id quota） | Apache 2.0 |
| 自動昇格 trigger | Argo Workflow + ArgoCD + Backstage Software Template | Apache 2.0 |
| 専用 Pod / DB Provision | CloudNativePG Cluster CR + k8s Deployment | Apache 2.0 |

#### 業務エラー監査 / hash chain
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| audit_local table | PostgreSQL（tier1 Library 経由、PII 不在 schema） | PostgreSQL |
| audit relay | tier1 Library audit relay（at-least-once + ReplacingMergeTree） | Apache 2.0 |
| audit SoR | ClickHouse（tier1 経由） | Apache 2.0 |
| hash chain | SHA256 + RFC 3161 trusted timestamp + Sigstore transparency log | Apache 2.0 |
| heartbeat / gap monitor | 自製 + Vector + ClickHouse | Apache 2.0 / MPL 2.0 |

#### 権限モデル / 認証
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Authentication | Keycloak（tier1 経由） | Apache 2.0 |
| Authorization (ABAC) | Keycloak ABAC policy + tier1 Backend-for-Library | Apache 2.0 |
| OIDC | Keycloak | Apache 2.0 |
| MFA / WebAuthn | Keycloak built-in | Apache 2.0 |
| Delegation / Emergency Override | tier1 Library AuthContext + 32 適合仕様 GUC | Apache 2.0 |

#### 採用しない OSS
- **MongoDB / Cassandra / Elasticsearch**: tier1 / data 軸の非採用と整合
- **Redis**: BUSL、Valkey 一本（tier1 経由）
- **PostgreSQL TDE 拡張**: OSS 未成熟、PV LUKS + column envelope（pii_segregated）の二重で吸収
- **Liquibase / Flyway**: sqlx-cli + Apicurio で吸収（tier1 経由）
- **GraphQL Apollo / Relay**: tier1 が gRPC + Connect-RPC + REST/SSE で吸収、GraphQL は v1 不採用
- **OPA Gatekeeper**: Kyverno L1+ 単一深耕（tier1 / infra 経由）
- **Casbin / Oso 等の外部 ABAC engine**: Keycloak ABAC policy 一本
- **手書き SQL アプリ層露出**: sqlx の `query!` 系 macro は tier1 Library crate 内部のみ
- **AGPL / SSPL 系 OSS** の業務資産配信

### tier3 軸（v1_l1plus_primary、単一深耕）

#### Web SPA（フロントエンド）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| 言語 | TypeScript | Apache 2.0 |
| UI framework | React | MIT |
| ビルド / バンドラ | Vite | MIT |
| パッケージマネージャ | pnpm（内部 Verdaccio mirror） | MIT |
| form | react-hook-form（tier2 が schema validator 提供） | MIT |
| ルーティング | React Router | MIT |
| state | tier2 SDK 内蔵 4 layer reducer + React hooks | Apache 2.0 |
| HTTP / fetch | native fetch + tier2 SDK | - |
| WebSocket / SSE | native + tier2 SDK | - |
| OTel SDK | `@opentelemetry/sdk-trace-web` + `@opentelemetry/auto-instrumentations-web` | Apache 2.0 |

#### デスクトップ exe（クロスプラットフォーム）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| .NET | .NET 8 LTS + WPF / Avalonia | MIT |
| Tauri | Tauri | MIT / Apache 2.0 |
| Rust | rustc stable | MIT / Apache 2.0 |
| Tauri 配布 | Tauri bundle（msi / dmg / AppImage） | MIT / Apache 2.0 |

#### レガシー（.NET Framework 4.6.2+）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| 言語 | C# / VB.NET | MIT |
| UI framework | WinForms / WPF | MIT |
| HTTP | HttpClient + WinHttpHandler（ALPN h2 必須環境） | MIT |
| OTel | OpenTelemetry .NET Auto-Instrumentation（CLR Profiler）+ `k1s0.Companion.NetFx.OTelExt` | Apache 2.0 |
| Companion runtime | `k1s0.Companion.NetFx`（client 軸提供） | Apache 2.0 |

#### BFF auth-edge
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| BFF runtime | Rust + axum / .NET 8 LTS + ASP.NET Core | Apache 2.0 / MIT |
| OIDC client | Keycloak.NET / openidconnect-rs（tier1 経由） | Apache 2.0 / MIT |
| session store | Valkey（tier1 経由） | BSD-3 |
| AES-256-GCM encryption | OpenBao Transit（tier1 経由） | MPL 2.0 |

#### Tauri Companion sidecar
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| sidecar runtime | Tauri | MIT / Apache 2.0 |
| WebUSB 相当 | rusb crate | MIT |
| Web Bluetooth 相当 | btleplug crate | Apache 2.0 / MIT |
| Web Serial 相当 | serialport crate | MIT |
| OPC UA | opc-ua crate | MPL 2.0 |
| Modbus | tokio-modbus crate | Apache 2.0 / MIT |
| OS keystore | OS native（DPAPI / Keychain / Secret Service） | OS |
| MDM 配布 | Microsoft Intune / Jamf / Workspace ONE / 自製 channel（Backstage） | 商用 / Apache 2.0 |

#### テスト
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| E2E | Playwright | Apache 2.0 |
| a11y | axe-core | MPL 2.0 |
| Web Vitals | Lighthouse CI | Apache 2.0 |
| contract test | Pact + Pact Broker OSS | MIT |
| property based test | fast-check (TS) / FsCheck (.NET) / proptest (Rust) | MIT / BSD-3 / Apache 2.0 |
| integration | Testcontainers | MIT |

#### 観測 / セキュリティ
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| OTel Collector | opentelemetry-collector（tier1 経由） | Apache 2.0 |
| RUM SoR | ClickHouse（tier1 経由） | Apache 2.0 |
| Web Vitals 集計 | Apache Superset（ad-hoc）/ Perses dashboard | Apache 2.0 |
| CSP / セキュリティヘッダ | tier1 Gateway + Backstage Software Template | Apache 2.0 |
| SBOM | Syft + CycloneDX | Apache 2.0 |
| 脆弱性スキャン | Trivy | Apache 2.0 |
| 署名 | Cosign | Apache 2.0 |

#### 採用しない OSS
- **Apollo / Relay GraphQL**: tier1 経由 gRPC + Connect-RPC + REST/SSE で吸収
- **axios / superagent**: tier2 SDK 経由必須（生 fetch / HTTP getter は禁止）
- **Redux / MobX / Recoil / Pinia / NgRx**: tier2 SDK 内蔵 4 layer reducer に集約
- **言語別 APM agent（NewRelic / DataDog / AppInsights）**: OTel L1+ 単一深耕
- **Snyk / WhiteSource**: Trivy + Grype + Syft で吸収
- **Cypress / Selenium / TestCafe / Puppeteer / WebdriverIO**: Playwright L1+ 単一深耕
- **iOS Swift / Android Kotlin**: v1.0.0 出荷外（言語スタック未対応）
- **AGPL / SSPL 系 OSS**: 業務 UI 配信から除外
- **HashiCorp Vault**: BUSL、OpenBao 一本（tier1 経由）

### infra 軸（v1_l1plus_primary、単一深耕）

#### cluster コア
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| コンテナオーケストレーション | Kubernetes | Apache 2.0 |
| コンテナランタイム | containerd | Apache 2.0 |
| Control Plane VIP | kube-vip | Apache 2.0 |
| ベアメタル LB | MetalLB | Apache 2.0 |

#### サービスメッシュ / API Gateway / CNI
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| サービスメッシュ | Istio | Apache 2.0 |
| API Gateway | Envoy Gateway | Apache 2.0 |
| CNI | Calico（BGP mode）| Apache 2.0 |

#### GitOps / 配信
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| GitOps CD | Argo CD | Apache 2.0 |
| プログレッシブデリバリー | Argo Rollouts | Apache 2.0 |
| イベント駆動自動化 | Argo Events | Apache 2.0 |
| CI（主）| GitHub Actions runner | Apache 2.0 |
| CI（代替）| Tekton | Apache 2.0 |

#### サプライチェーン
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| コンテナレジストリ | Harbor | Apache 2.0 |
| 脆弱性スキャン | Trivy | Apache 2.0 |
| イメージ署名 | Cosign（Sigstore）| Apache 2.0 |
| SBOM 生成 | Syft | Apache 2.0 |
| SBOM 脆弱性スキャン | Grype | Apache 2.0 |
| ポリシーエンジン | Kyverno | Apache 2.0 |
| 証明書管理 | cert-manager | Apache 2.0 |
| シークレット管理 | OpenBao | MPL 2.0 |
| シークレット同期 | External Secrets Operator | Apache 2.0 |
| Build sandbox | Bazel + Nix + Buildkit | Apache 2.0 / LGPL-2.1 / Apache 2.0 |
| Build attestation | Witness / Gitsign / Tekton Chains | Apache 2.0 |
| Hardening verifier | kube-bench / docker-bench-security | Apache 2.0 |

#### スケール / Chaos
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| イベント駆動オートスケーラー | KEDA | Apache 2.0 |
| Chaos Engineering | Litmus | Apache 2.0 |

#### IaC / マニフェスト
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| IaC | OpenTofu | MPL 2.0 |
| マニフェスト管理（自製）| Kustomize | Apache 2.0 |
| マニフェスト管理（3rd party）| Helm | Apache 2.0 |
| ローカル開発 | Tilt | Apache 2.0 |

#### ストレージ
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| オブジェクトストレージ | Rook + Ceph（RGW）| Apache 2.0 / LGPL-2.1 |
| 分散ブロックストレージ | Longhorn | Apache 2.0 |

#### 運用 UI / 開発者ポータル
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| k8s Web UI | Headlamp | Apache 2.0 |
| API トラフィックビューア | Kubeshark | Apache 2.0 |
| k8s 状態メトリクス | kube-state-metrics | Apache 2.0 |
| 開発者ポータル | Backstage | Apache 2.0 |
| ad-hoc 探索 | Apache Superset | Apache 2.0 |
| メトリクス可視化 | Perses | Apache 2.0 |

#### 時刻同期
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| NTP daemon | chrony | GPLv2 |
| PTP daemon | linuxptp（ptp4l / phc2sys / ts2phc）| GPLv2 |
| eBPF observability | Cilium Tetragon | Apache 2.0 |
| HLC runtime（自製）| k1s0_hlc_lib | Apache 2.0 |
| GPS source | GPSD + LinuxPPS | BSD-3 / GPLv2 |

#### 採用しない OSS
- Grafana / Tempo / Loki / k6 / Grafana OnCall（AGPL-3）
- OpenSearch / Kibana
- HashiCorp Vault
- Confluent Schema Registry v5+
- Linkerd / Cilium Service Mesh
- Flux / Spinnaker
- Pulumi / Crossplane / Ansible / Chef / Puppet
- Chaos Mesh
- HPA v1
- Karpenter（EKS）/ Knative
- OPA Gatekeeper
- GlusterFS / OpenEBS Replicated Engine
- HDFS / Hadoop
- EBS / Persistent Disk 等
- Octant / Lens

### data 軸（v1_l1plus_primary、単一深耕）

#### RDB（OLTP）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| RDB engine | PostgreSQL | PostgreSQL |
| k8s Operator | CloudNativePG | Apache 2.0 |
| 接続プール | PgBouncer | ISC |
| vector index | pgvector | PostgreSQL |
| migration tool | sqlx-cli | Apache 2.0 / MIT |

#### Stream / Messaging
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Stream broker | Apache Kafka | Apache 2.0 |
| k8s Operator | Strimzi | Apache 2.0 |
| CDC | Debezium | Apache 2.0 |
| Schema Registry | Apicurio Registry | Apache 2.0 |
| Schema Registry（代替）| Karapace | Apache 2.0 |

#### Cache / KV
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| In-memory cache | Valkey | BSD-3 |
| k8s Operator | Valkey Operator / KubeBlocks | Apache 2.0 |

#### Analytics（OLAP / signal SoR）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Columnar OLAP | ClickHouse | Apache 2.0 |
| k8s Operator | Altinity ClickHouse Operator | Apache 2.0 |
| 分散調整 | clickhouse-keeper | Apache 2.0 |

#### Backup / Restore
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Postgres backup | Barman（CloudNativePG 内蔵）| GPL-3 |
| Volume snapshot | CSI Snapshotter + Longhorn snapshot | Apache 2.0 |
| Object lock / WORM | Ceph RGW Object Lock | Apache 2.0 / LGPL-2.1 |
| cluster 跨ぎ backup | Velero | Apache 2.0 |

#### Migration / DDL Tooling
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Postgres migration | sqlx-cli | Apache 2.0 / MIT |
| Kafka schema migration | Apicurio CLI | Apache 2.0 |
| ClickHouse migration | go-migrate（ClickHouse driver）| MIT |
| Schema diff（参照のみ）| atlas | Apache 2.0 |

#### 観測補助
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Postgres exporter | postgres_exporter | Apache 2.0 |
| Kafka exporter | kafka-exporter + Strimzi metrics | Apache 2.0 |
| Valkey exporter | redis_exporter（Valkey 互換）| MIT |
| ClickHouse exporter | ClickHouse 内蔵 system table + Prom | Apache 2.0 |

#### 採用しない OSS
- MySQL / MariaDB
- MongoDB（SSPL）
- Elasticsearch / OpenSearch
- Redis（BUSL-1.1）
- Cassandra / ScyllaDB
- Confluent Schema Registry v5+
- HashiCorp Vault
- DynamoDB / Cosmos DB / cloud managed DB
- PostgreSQL TDE 拡張
- pglogical multi-master
- Kafka Cluster Linking（Confluent）
- Liquibase / Flyway
- pgAdmin / Adminer / Kafka UI（手動 write 用途）

### security 軸（v1_l1plus_primary、単一深耕）

#### サプライチェーン真正性（build / sign / verify）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Container signing | cosign | Apache 2.0 |
| SBOM 生成 | Syft | Apache 2.0 |
| SLSA provenance / attestation | in-toto / witness | Apache 2.0 |
| binary provenance | SLSA-GitHub-generator | Apache 2.0 |
| build-time integrity | Tekton Chains | Apache 2.0 |

#### 脆弱性 / CVE 検知
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Image vuln scanner | Trivy | Apache 2.0 |
| Image vuln scanner（補強）| Grype | Apache 2.0 |
| Rust dep audit | cargo-audit | Apache 2.0 / MIT |
| Go dep / vuln check | govulncheck | BSD-3 |
| Node dep audit | pnpm audit | Artistic 2.0 |
| SAST（言語横断）| Semgrep（OSS rules）| LGPL-2.1 |
| Secret scan | gitleaks | MIT |
| Secret scan（補強）| trufflehog | AGPL-3 |
| IaC scan | Trivy IaC / Checkov | Apache 2.0 |

#### runtime 脅威検知
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| syscall / process detect | Falco | Apache 2.0 |
| eBPF based observability | Tetragon | Apache 2.0 |

#### policy / admission（cross-axis）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Admission policy engine | Kyverno（infra 提供）| Apache 2.0 |
| Mutation engine | Kyverno | Apache 2.0 |
| CEL based validation | Kubernetes ValidatingAdmissionPolicy | Apache 2.0 |
| Policy testing | chainsaw / kuttl | Apache 2.0 |

#### audit / detection SoR
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| audit signal SoR | ClickHouse（data 提供）| Apache 2.0 |
| log shipping / pipeline | Vector | MPL 2.0 |
| WORM object lock | Ceph RGW Object Lock（infra 提供）| Apache 2.0 / LGPL-2.1 |

#### secret / 鍵 hygiene
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Secret store | OpenBao（infra 提供）| MPL 2.0 |
| External Secrets | External Secrets Operator | Apache 2.0 |
| KEK 階層 | OpenBao Transit（tier1/11 提供）| MPL 2.0 |

#### IR / on-call
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| on-call escalation | 自製 escalation engine（Argo Workflows + Mattermost）| Apache 2.0 |
| IR ticketing | incident.io OSS / GitHub Issue | (custom OSS) |
| postmortem | Markdown + GitHub PR | (workflow) |
| runbook | Backstage TechDocs（infra 提供）| Apache 2.0 |

#### 採用しない OSS
- HashiCorp Vault（BUSL）
- Wazuh（Elastic 依存）
- Anchore Enterprise / Sysdig Secure / Aqua Security 商用版
- GitHub Advanced Security の商用 license SAST
- JFrog Xray
- immudb 単独運用
- 商用 IR ticketing（PagerDuty / OpsGenie 等の有償面）
- OPA Gatekeeper 単独運用

### ops 軸（v1_l1plus_primary、単一深耕）

#### SLO platform / error budget
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| SLO definition / budget | Sloth | Apache 2.0 |
| SLO recording rule generator | Pyrra | Apache 2.0 |
| Time-series store | VictoriaMetrics | Apache 2.0 |
| PromQL adapter | Prometheus | Apache 2.0 |

#### alerting / routing
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Alert routing | Alertmanager | Apache 2.0 |
| Burn rate alert generator | Sloth + Pyrra | Apache 2.0 |
| Synthetic monitoring | Playwright | Apache 2.0 |
| Probe / blackbox monitor | blackbox_exporter | Apache 2.0 |

#### on-call / escalation
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| on-call escalation engine | 自製 engine（Argo Workflows + Tekton Triggers + Mattermost）| Apache 2.0 |
| rotation schedule | roster.lock.yaml + Argo Workflows CronWorkflow | Apache 2.0 |
| ack receipt | Mattermost (web slash command + mobile push) + Tekton Trigger HTTP callback | MIT |
| notification channel | Mattermost (Team Edition) + Telegram OSS bot | MIT / GPLv3 |
| Identity provider | Keycloak | Apache 2.0 |
| voice / SMS（ops-edge）| Asterisk + 自社契約 SIP trunk 二系統 | GPL-2.0 |
| mobile push（ops-edge）| APNs + FCM | OS native（例外規律で許容）|

#### incident management / postmortem
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| IR ticketing | incident.io OSS / GitHub Issue | (custom OSS) |
| IR command bridge | Mattermost slash command + Tekton Triggers | MIT |
| postmortem 文書 | Markdown + GitHub PR | (workflow) |
| postmortem template | Backstage TechDocs | Apache 2.0 |
| action item 追跡 | GitHub Issue + Project | (workflow) |

#### progressive delivery / change management
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Progressive delivery | Argo Rollouts | Apache 2.0 |
| Canary analysis | Argo Rollouts AnalysisTemplate | Apache 2.0 |
| GitOps | Argo CD | Apache 2.0 |
| Workflow / pipeline | Argo Workflows | Apache 2.0 |
| Pull-request preview env | Argo CD ApplicationSet | Apache 2.0 |
| Feature flag | flagd / OpenFeature | Apache 2.0 |

#### runbook 実行 / 自動化
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Runbook 文書 | Backstage TechDocs | Apache 2.0 |
| Runbook 実行 engine | Tekton Pipelines | Apache 2.0 |
| Runbook trigger | Tekton Triggers + Alertmanager webhook | Apache 2.0 |
| ChatOps | Mattermost slash command + Tekton trigger | MIT |
| Workflow scheduling | Argo Workflows CronWorkflow | Apache 2.0 |
| Chaos drill | Litmus | Apache 2.0 |
| Game day scenario | Litmus + Tekton | Apache 2.0 |

#### observability / dashboard
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Dashboard | Perses | Apache 2.0 |
| Trace store | ClickHouse | Apache 2.0 |
| Trace UI | Jaeger v2 内蔵 ClickHouse storage | Apache 2.0 |
| Log store | ClickHouse + Vector | Apache 2.0 / MPL 2.0 |
| Audit / ops_event store | ClickHouse | Apache 2.0 |
| Log shipping | Vector | MPL 2.0 |

#### 採用しない OSS
- PagerDuty / Opsgenie / Splunk On-Call の有償 SLA 機能
- Datadog / New Relic / Dynatrace / AppDynamics
- Splunk Enterprise / Sumo Logic
- Chronosphere / Honeycomb
- LaunchDarkly / Split.io / Optimizely
- Spinnaker
- GitOps Toolkit（Flux）
- Concourse / GoCD
- Backstage 以外の internal developer portal
- Tempo / Loki / Grafana / Grafana k6 / Grafana OnCall（AGPL-3）

### client 軸（v1_l1plus_primary、単一深耕）

#### 言語別 SDK ランタイム / 配布
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| .NET ランタイム | .NET 8 LTS / .NET Framework 4.6.2 | MIT |
| Java ランタイム | OpenJDK Temurin 21 LTS | GPLv2+CE |
| Node.js ランタイム | Node.js 20 LTS | MIT |
| Browser TypeScript | TypeScript | Apache 2.0 |
| Tauri Native | Tauri | MIT / Apache 2.0 |
| Rust ランタイム | rustc stable | MIT / Apache 2.0 |
| Python ランタイム | CPython 3.12+ | PSF |
| Ruby ランタイム | Ruby 3.3+ | Ruby / BSD-2 |
| Go ランタイム | Go 1.22+ | BSD-3 |

#### gRPC / wire 実装
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| gRPC C++ コア | gRPC | Apache 2.0 |
| gRPC .NET（8 LTS のみ） | grpc-dotnet（Grpc.Net.Client） | Apache 2.0 |
| gRPC Java | grpc-java | Apache 2.0 |
| gRPC Node | @grpc/grpc-js | Apache 2.0 |
| gRPC-Web TypeScript | grpc-web | Apache 2.0 |
| gRPC Python | grpcio | Apache 2.0 |
| gRPC Ruby | grpc | Apache 2.0 |
| gRPC Go | google.golang.org/grpc | Apache 2.0 |
| gRPC Rust | tonic | MIT |
| Connect TS（Browser / Node） | @connectrpc/connect-web / @connectrpc/connect-node | Apache 2.0 |
| Connect Go | connectrpc.com/connect | Apache 2.0 |
| Connect Kotlin | connect-kotlin | Apache 2.0 |
| Connect Swift | connect-swift | Apache 2.0 |
| Connect .NET 8 LTS | k1s0.Connect.NetCore（in-house authoritative） | Apache 2.0 |
| HTTP/3 + WebTransport（browser） | native browser API | - |
| HTTP/3 + WebTransport（server） | quinn / s2n-quic | Apache 2.0 |
| HTTP/3 + WebTransport（.NET 8+） | System.Net.Quic | MIT |
| WebSocket（TS） | native WebSocket | - |
| WebSocket（.NET） | System.Net.WebSockets | MIT |
| SSE parser（Browser） | native EventSource | - |
| SSE parser（.NET / 他言語） | 自製（言語別、Buf 生成側で固定） | Apache 2.0 |

#### Auto-Instrumentation / 観測可能性
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| OTel .NET Auto-Instrumentation | opentelemetry-dotnet-instrumentation | Apache 2.0 |
| Companion OTel 拡張（.NET FW） | signalfx/splunk-otel-dotnet fork（k1s0.Companion.NetFx.OTelExt） | Apache 2.0 |
| OTel Java Agent | opentelemetry-java-instrumentation | Apache 2.0 |
| OTel JS Auto-Instrumentation | @opentelemetry/auto-instrumentations-node | Apache 2.0 |
| OTel Browser SDK | @opentelemetry/sdk-trace-web + @opentelemetry/auto-instrumentations-web | Apache 2.0 |
| OTel Python | opentelemetry-python | Apache 2.0 |
| OTel Ruby | opentelemetry-ruby | Apache 2.0 |
| OTel Go | go.opentelemetry.io/otel | Apache 2.0 |
| OTel Rust | opentelemetry-rust | Apache 2.0 |
| OTel Collector（sidecar） | opentelemetry-collector | Apache 2.0 |

#### スキーマ / コード生成
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| protoc | protobuf | BSD-3 |
| Buf | buf | Apache 2.0 |
| protoc-gen-go | protobuf-go | BSD-3 |
| protoc-gen-ts | ts-proto | Apache 2.0 |
| grpc-tools（.NET） | Grpc.Tools | Apache 2.0 |
| grpc-tools（Java） | protoc-gen-grpc-java | Apache 2.0 |
| protoc-gen-python | protobuf python | BSD-3 |
| protoc-gen-ruby | grpc protoc plugin | Apache 2.0 |
| protoc-gen-rust | tonic-build | MIT |
| Apicurio Registry SDK clients | apicurio-registry-clients | Apache 2.0 |

#### パッケージレジストリ / 配信
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Container registry mirror | Harbor（infra 提供） | Apache 2.0 |
| .NET package mirror | Harbor NuGet repository | Apache 2.0 |
| Java package mirror | Harbor Maven repository | Apache 2.0 |
| Node.js package mirror | Verdaccio | MIT |
| Python package mirror | Harbor PyPI repository | Apache 2.0 |
| RubyGems mirror | Harbor RubyGems repository | Apache 2.0 |
| Go module proxy | Athens | MIT |
| cargo registry mirror | Harbor（OCI cargo registry） | Apache 2.0 |
| OCI artifact 署名 | Cosign | Apache 2.0 |
| SBOM 生成 | Syft | Apache 2.0 |
| SBOM 脆弱性スキャン | Grype | Apache 2.0 |

#### 開発者体験
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| ローカル開発 | Tilt | Apache 2.0 |
| Companion local mock | 自製 k1s0 Companion Mock | Apache 2.0 |
| dev container | VS Code Remote Containers / GitHub Codespaces | MIT |
| contract test | Pact + Pact Broker OSS | MIT |
| E2E | Playwright | Apache 2.0 |
| integration | Testcontainers | MIT |
| 開発者ポータル | Backstage（infra 提供） | Apache 2.0 |
| docs | MkDocs Material | MIT |

#### 採用しない OSS
- Apollo / Relay GraphQL（v1 で GraphQL 経路を採用しない）
- axios / superagent（SDK は内部 transport を持つため）
- Redux / MobX / Recoil / Pinia / NgRx（state は 39 適合仕様 4 layer + SDK reducer に集約）
- Snyk / WhiteSource（商用、Trivy / Grype のみ）
- gRPC-over-WebSocket 自製 frame protocol（Connect-RPC L1+ 単一深耕に統一）
- 言語別 APM agent（NewRelic / DataDog / AppInsights）
- OPA Gatekeeper（Kyverno L1+ 深耕）

### test 軸（v1_l1plus_primary、単一深耕）

#### unit / framework（言語別 1 OSS L1+）
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Python unit | pytest | MIT |
| TypeScript / JavaScript unit | Vitest | MIT |
| Go unit | testing (stdlib) + testify | BSD-3 / MIT |
| Rust unit | cargo test (stdlib) | Apache 2.0 |
| Java / Kotlin unit | JUnit 5 | EPL-2.0 |
| C# / .NET unit | xUnit.net | Apache 2.0 |

#### property-based test
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Python property | Hypothesis | MPL-2.0 |
| TypeScript property | fast-check | MIT |
| Go property | gopter | MIT |
| Rust property | proptest | Apache 2.0 |
| Java / Kotlin property | jqwik | EPL-2.0 |
| C# / .NET property | FsCheck | BSD-3 |

#### contract test
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Consumer-driven contract | Pact | MIT |
| Pact Broker | Pact Broker (OSS) | MIT |
| Schema breaking detector | Buf (buf-breaking) | Apache 2.0 |

#### e2e / scenario replay
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Browser e2e | Playwright | Apache 2.0 |
| API e2e | Playwright API request | Apache 2.0 |
| Synthetic / journey monitor | Playwright | Apache 2.0 |

#### integration / Testcontainers
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Container fixture | Testcontainers | MIT |
| k8s integration | chainsaw | Apache 2.0 |
| k8s integration（alt）| kuttl | Apache 2.0 |

#### chaos / fault injection
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Chaos orchestration | Litmus | Apache 2.0 |
| Network fault injection | Toxiproxy | MIT |
| Container kill / network | Chaos Mesh（double-bound）| Apache 2.0 |
| Stress | stress-ng | GPL-2.0 |

#### mutation testing
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Java / Kotlin mutation | PIT (pitest) | Apache 2.0 |
| Python mutation | mutmut | BSD-3 |
| TypeScript / JavaScript / .NET mutation | Stryker Mutator | Apache 2.0 |
| Go mutation | go-mutesting | MIT |
| Rust mutation | cargo-mutants | MIT |

#### load / performance
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Load generator | Apache JMeter | Apache 2.0 |
| Load DSL（alt）| Locust（double-bound）| MIT |
| HTTP/3 + WebTransport load | k1s0-perf-h3wt（自製 Rust）| Apache 2.0（依存: quinn / wtransport / tokio）|
| Browser load | Playwright | Apache 2.0 |

#### coverage / report
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Python coverage | coverage.py | Apache 2.0 |
| TS / JS coverage | c8 / istanbul | ISC / BSD-3 |
| Go coverage | go test -cover | BSD-3 |
| Rust coverage | cargo-llvm-cov | Apache 2.0 |
| Java / Kotlin coverage | JaCoCo | EPL-2.0 |
| .NET coverage | coverlet | MIT |

#### fuzz
| カテゴリ | プロダクト | ライセンス |
|---|---|---|
| Coverage-guided binary fuzz | AFL++ | Apache 2.0 |
| Go fuzz | go test -fuzz (stdlib) | BSD-3 |
| Rust fuzz | cargo-fuzz / libFuzzer | MIT / NCSA |
| TS / JavaScript fuzz | jazzer.js | Apache 2.0 |
| Python fuzz | atheris | Apache 2.0 |
| .NET / C# fuzz | SharpFuzz | Apache 2.0 |

#### 採用しない OSS
- BrowserStack / Sauce Labs / LambdaTest / Applitools / Functionize / Mabl / TestRail
- Gremlin / AWS FIS / Harness Chaos
- Codecov SaaS / Coveralls SaaS / SonarQube Server commercial
- Diffblue Cover Pro / Sealights
- Pactflow（Pact 商用）
- k6 + xk6-webtransport（k6 が AGPL-3）
- Grafana k6 / Cypress / Selenium / TestCafe / Puppeteer / WebdriverIO
- Spinnaker / Flagger
- Tempo / Loki / Grafana
- Spring Cloud Contract / Karate
- nose2 / Jest / Mocha / Jasmine / Ginkgo / NUnit / MSTest / Spock / TestNG

### formal 軸（v1_l1plus_primary、単一深耕、横並びなし）

| 機能カテゴリ | v1_l1plus_primary | license | linkage_model |
|---|---|---|---|
| temporal logic（safety + liveness） | TLA+ + Apalache | MIT / Apache 2.0 | process_boundary |
| state machine protocol code-gen | P language | MIT | process_boundary |
| program logic (Scala) | Stainless | Apache 2.0 | static |
| program logic (multi-lang) | Dafny | MIT | process_boundary |
| proof assistant (math + crypto) | Lean 4 + mathlib | Apache 2.0 | process_boundary |
| runtime model check (Rust) | Kani | Apache 2.0 / MIT | static |
| runtime model check (C / C++) | CBMC | BSD-4-Clause | process_boundary |

#### 各 OSS の用途と class binding

##### TLA+ + Apalache
- 担当 proof_class: v1_temporal_safety_proof（primary）/ v1_temporal_liveness_proof（primary）/ v1_refinement_proof（co-primary with Stainless）
- 担当軸: 07 / 08 / 10 / 11 / 39 / infra01 / infra16 / ops15
- artifact: TLA+ `.tla` spec / Apalache `.json` trace / TLC inv proof log
- 単一深耕根拠: Apalache を symbolic model checker として primary、TLC は exhaustive enumeration の補助。商用 SPIN / NuSMV / UPPAAL は採用しない

##### P language
- 担当 proof_class: v1_temporal_safety_proof（補助）/ v1_refinement_proof（補助）
- 担当軸: 07 / 08 / 10 / ops15
- artifact: P `.p` spec / P checker output / generated TLA+ + C# harness
- 単一深耕根拠: 分散 protocol を P で書き、P checker で property check + TLA+ への自動 lowering で symbolic model check との双方向 verification

##### Stainless
- 担当 proof_class: v1_program_correctness_proof（primary for Scala / non-Rust 系）/ v1_refinement_proof（co-primary）
- 担当軸: 09 / 12 / 13 / 14 / 15 / 32 / 39 / security15 / security17 / client15 / ops15 / test15
- artifact: Stainless `.scala` + `.stainless` certificate
- 単一深耕根拠: Scala 系の program logic 専用、refinement type で information flow lattice も表現

##### Dafny
- 担当 proof_class: v1_program_correctness_proof
- 担当軸: 09 / 12 / 13 / 14 / 15 / 32 / 39 / security15 / security17 / client15 / ops15 / test15
- artifact: Dafny `.dfy` + `.doo`
- 単一深耕根拠: multi-lang code-gen 専用（複数言語に投影される component）

##### Lean 4 + mathlib
- 担当 proof_class: v1_program_correctness_proof（primary for math + crypto）
- 担当軸: 11 / data01 / infra16 / security15 / security17 / client15
- artifact: Lean `.lean` + `.olean` + mathlib revision pin
- 単一深耕根拠: cryptographic invariant、number-theoretic property、HLC algebra 専用

##### Kani
- 担当 proof_class: v1_runtime_modelcheck_proof（primary）
- 担当軸: 07 / 10 / 11 / 12 / 15 / 32 / infra01 / infra16 / security15 / security17 / client15 / test15 / ops15
- artifact: Kani `.rs` harness + `.json` report
- 単一深耕根拠: Rust 実装の bounded model checking 専用

##### CBMC
- 担当 proof_class: v1_runtime_modelcheck_proof（primary）
- 担当軸: infra01 / infra16 / security15
- artifact: CBMC `.c` harness + `.xml` + `.json` report
- 単一深耕根拠: C / C++ 実装の bounded model checking 専用（eBPF / HSM driver / kernel module / PTP daemon）

#### formal 軸 OSS 採用しない設計
- **商用 proof assistant**: Coq Inria 派生 / Isabelle/HOL の Isabelle UK 商用 fork / F* Microsoft Research / SPARK Pro AdaCore 商用版 → 採用しない
- **商用 model checker**: SPIN proprietary / NuSMV 商用 / UPPAAL Pro → 採用しない
- **SMT solver の単独運用**: Z3 / CVC5 を直接 driver で叩く → 採用しない（Stainless / Dafny / Kani / CBMC が SMT solver を内包）
- **GUI proof editor**: Why3 IDE 等 → 採用しない、LSP + IDE plugin の標準 OSS 経路のみ

### meta 軸（共通基盤 OSS）
| 機能カテゴリ | OSS | license |
|---|---|---|
| 署名 | cosign | Apache 2.0 |
| secrets | OpenBao | MPL 2.0 |
| admission | Kyverno | Apache 2.0 |
| pipeline | Tekton | Apache 2.0 |
| GitOps | Argo CD | Apache 2.0 |
| workflow | Argo Workflows | Apache 2.0 |
| object storage | Ceph RGW | LGPL 2.1 |
| metrics SoR | ClickHouse | Apache 2.0 |
| dashboard | Perses | Apache 2.0 |

## OS 固有依存例外（[制約と前提](../06_制約と前提/01_OS固有依存例外.md) 参照）
- **APNs / FCM**: ops-edge escalation cluster の last-mile 通知に限定許容（[OSS ライセンス規律](02_OSSライセンス規律.md) (e) 例外）

## OSS ライフサイクルへの登録
- 全採用 OSS は [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) の `oss_lifecycle.lock.yaml` に entry 登録
- major version migration 演習が必須（formal 軸は 6 month cadence）
- license 変更 / 改廃 / サポート終了時の差し替え toolchain を tier1 が成果物として提供

## 関連参照
- [OSS ライセンス規律](02_OSSライセンス規律.md)
- [OSS ライフサイクル](03_OSSライフサイクル.md)
- [言語スタック](04_言語スタック.md)
- [自製 in-house 方針](05_自製in-house方針.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
