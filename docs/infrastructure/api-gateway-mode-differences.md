# Kong モード差異: 開発（DB-less）vs 本番（DB-backed）

## KONG-03 監査対応

| 環境 | Kong モード | 設定管理 |
|------|------------|---------|
| Docker Compose（開発） | DB-less (`KONG_DATABASE: "off"`) | 宣言型 YAML（`infra/kong/kong.dev.yaml`） |
| Kubernetes 本番 | DB-backed (`KONG_DATABASE: postgres`) | Admin API + deck sync |

## 差異と注意点

### ルーティング挙動の差異
- DB-less は `kong.dev.yaml` の宣言型設定を使用。Admin API への変更は再起動で失われる。
- DB-backed は PostgreSQL に設定を永続化。deck sync（`.github/workflows/kong-sync.yaml`）でGitHub管理。

### 開発環境での検証について
Docker Compose でのルーティング検証は本番 DB-backed 環境での動作を完全には再現しない。
重大なルーティング変更は staging 環境（DB-backed）でも必ず検証すること。

### 設計上の意図
開発環境では設定変更の高速サイクルを優先して DB-less を採用している。
本番環境では設定永続化・クラスタリング・Admin API の活用のために DB-backed を使用する。

---

## MED-011: Kong サービス名と Istio VirtualService 名のマッピング

Kong と Istio では同一サービスに対して異なる命名規約を採用している。
運用時の混乱を防ぐため、以下にマッピング表を記載する。

### 命名規約の差異

| 項目 | Kong | Istio VirtualService |
|------|------|---------------------|
| 命名形式 | `{service}-v1`（バージョンサフィックス付き） | Kubernetes Service 名と同一 |
| 管理ファイル | `infra/kong/kong.dev.yaml`（開発）、Helm values（本番） | `infra/istio/virtual-service.yaml` |

### サービス名マッピング表

| Kong サービス名 | Istio VirtualService 名 | upstream（Docker） | 備考 |
|---------------|------------------------|-------------------|------|
| `auth-v1` | `auth` | `auth-rust` | — |
| `config-v1` | `config` | `config-rust` | — |
| `saga-v1` | `saga` | `saga-rust` | — |
| `dlq-manager-v1` | `dlq-manager` | `dlq-manager` | — |
| `master-v1` | `master-maintenance` | `master-maintenance-rust` | **Kong と Istio で名前が異なる**。Kong は API パス `/api/v1/master` に対応するため `master` を使用し、Istio は K8s Service 名 `master-maintenance` を使用する |
| `featureflag-v1` | `featureflag` | `featureflag-rust` | — |
| `tenant-v1` | `tenant` | `tenant-rust` | — |
| `ratelimit-v1` | `ratelimit` | `ratelimit-rust` | — |
| `vault-svc-v1` | `vault` | `vault-rust` | Kong は `vault-svc` を使用（vault インフラとの名前衝突回避） |
| `api-registry-v1` | `api-registry` | `api-registry-rust` | — |
| `app-registry-v1` | `app-registry` | `app-registry-rust` | — |
| `event-monitor-v1` | `event-monitor` | `event-monitor-rust` | — |
| `event-store-v1` | `event-store` | `event-store-rust` | — |
| `file-v1` | `file` | `file-rust` | — |
| `navigation-v1` | `navigation` | `navigation-rust` | — |
| `notification-v1` | `notification` | `notification-rust` | — |
| `policy-v1` | `policy` | `policy-rust` | — |
| `quota-v1` | `quota` | `quota-rust` | — |
| `rule-engine-v1` | `rule-engine` | `rule-engine-rust` | — |
| `scheduler-v1` | `scheduler` | `scheduler-rust` | — |
| `search-v1` | `search` | `search-rust` | — |
| `service-catalog-v1` | `service-catalog` | `service-catalog-rust` | — |
| `session-v1` | `session` | `session-rust` | — |
| `workflow-v1` | `workflow` | `workflow-rust` | — |
| `graphql-gateway-v1` | `graphql-gateway` | `graphql-gateway-rust` | — |
| `project-master-v1` | `project-master` | `project-master-rust` | — |
| `task-v1` | `task` | `task-rust` | — |
| `board-v1` | `board` | `board-rust` | — |
| `activity-v1` | `activity` | `activity-rust` | — |
| `bff-proxy`（Kong プラグインなし） | `bff-proxy` | `bff-proxy` | Kong はリバースプロキシのみ担当 |
| `ai-gateway-v1` | — | `ai-gateway-rust` | Istio VirtualService は将来追加予定 |
| `ai-agent-v1` | — | `ai-agent-rust` | Istio VirtualService は将来追加予定 |

### 注意事項

- `master-v1`（Kong） ↔ `master-maintenance`（Istio）は最も混乱しやすいマッピングである
- Kong の `vault-svc-v1` は HashiCorp Vault コンテナ（`vault`）との名前衝突を避けるため `-svc` サフィックスを付与している
- Kong サービス名の `-v1` サフィックスは API バージョニングを示すが、Istio 側では K8s Service 名をそのまま使用する
