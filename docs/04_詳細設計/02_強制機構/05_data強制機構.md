---
id: detail.data.data_enforcement
axis: data
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.data.data_index
  - arch.data.datastore_composition_policy
  - arch.data.schema_operations_policy
  - arch.data.preservation_policy
  - arch.data.encryption_policy
  - arch.data.replication_policy
  - arch.data.lifecycle_policy
  - arch.data.recovery_drill_policy
  - arch.data.migration_policy
  - detail.data.preservation_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts:
  - data_enforcement.lock.yaml
---

# data 強制機構

## 一文方針
- data 層の規律は Kyverno admission policy + DB engine 物理機構 + sqlx-cli prepare + Apicurio Registry compatibility rule の 4 種で物理 enforce、手書き policy は CI fail、policy 自体は build artifact から生成、最終 safety net は OS / DB engine / Kafka broker / Ceph の物理層機能（synchronous_commit / min.insync.replicas / object lock / LUKS）。

## 5 層 defense-in-depth

### 層 A: compile（コード / SQL の type / schema check）
- sqlx prepare: migration 適用後の DB に対して SQL を compile-time check
- Apicurio CLI: schema fetch 時に subject の compatibility rule を verify
- Buf: Protobuf schema lint
- go-migrate dry-run: ClickHouse migration の syntax check
- manifest: kubeval / kubeconform / OpenAPI schema
- `preservation_substrates.lock.yaml` / `restore_drill.lock.yaml`: build artifact、手書き禁止

### 層 B: lint（ポリシー / 規約 check）

#### Conftest custom rule（Rego policy）
- 全 entity table が tenant_id / version_id / created_at / updated_at / deleted_at 列を持つ
- 全 KafkaTopic CR が preservation_class 注釈持つ
- 全 CloudNativePG Cluster CR が preservation_class 注釈持つ
- 全 ClickHouse table が TTL 注釈持つ
- 全 PII column が envelope encryption 注釈
- 全 Apicurio subject が compatibility rule 持つ

#### Kyverno admission policy が build artifact から生成
- 手書き policy は CI fail

### 層 C: integration test（Testcontainers / kuttl / chainsaw）
- 全 admission policy が intended state を block / pass することを test
- chaos test の自動 replay（Litmus / restore_drill / crypto-erase / failover_drill）
- 全 instance の class instantiation が classes.yaml に整合
- migration の expand-contract phase が CI で順序保証

### 層 D: runtime（cluster 上の actual enforcement）
- Kyverno admission webhook: deploy / migration / DDL Job 実行前に block
- sqlx-cli の advisory lock: 並行 apply 防止
- go-migrate の version table: apply 順序保証
- Apicurio Registry: subject 登録時 compatibility verify
- Argo CD reconciliation: drift 検出 + self-heal

### 層 E: 物理 enforcement（DB engine / OS / OSS の物理機構）
- PostgreSQL synchronous_commit / WAL fsync: 物理 durability を OS layer で強制
- Kafka min.insync.replicas / unclean.leader.election=false: split-brain / data-loss を broker 層で物理 block
- ClickHouse Replicated MergeTree + keeper Raft: part metadata の整合を keeper が物理保証
- Ceph RGW Object Lock（Compliance mode）: retention 期限内の物理 delete を不可能化
- Longhorn LUKS encryption: unencrypted disk write を物理 block
- PostgreSQL FORCE_RLS（tier2/32）: bypass を権限層で物理 block
- OpenBao HSM-backed KEK: DEK の rotation を物理強制（cryptographic erasure）

## Kyverno admission policy 一覧（data 担当部分）

| policy 名 | 内容 |
|---|---|
| `require-preservation-class-annotation` | 全 data CR に `data.preservation.preservation_class` 注釈必須 |
| `require-tenant-id-column` | 全 entity table に tenant_id 列必須 |
| `require-version-id-column` | 全 entity table に version_id 列必須 |
| `require-soft-delete-column` | 全 entity table に deleted_at TIMESTAMPTZ NULL 必須 |
| `require-envelope-encryption-on-pii` | PII 分類された列が envelope encryption 注釈を持つ |
| `require-blind-index-for-pii-search` | PII 列で equality search が必要な場合 blind index 必須 |
| `require-rls-policy` | tenant_id 列を持つ table に POLICY + FORCE_RLS 必須 |
| `require-jsonb-validator` | JSONB 列に check constraint or application Validator 必須 |
| `require-replicated-clickhouse` | 全 ClickHouse table が Replicated MergeTree |
| `require-clickhouse-ttl` | 全 ClickHouse table が TTL clause 持つ |
| `require-kafka-rack-awareness` | Kafka broker.rack 必須 |
| `require-kafka-min-insync` | min.insync.replicas が preservation_class 別 min 以上 |
| `require-kafka-unclean-leader-disable` | unclean.leader.election.enable=false |
| `require-postgres-synchronous-commit` | preservation_class 別 min 以上の synchronous_commit |
| `block-non-forward-migration` | down migration を block |
| `block-destructive-ddl-without-contract-phase` | DROP COLUMN / DROP TABLE が contract phase id を持たなければ block |
| `block-on-restore-drill-red` | data01 `restore_drill.lock.yaml` の red cluster に新規 deploy 拒否 |
| `block-on-rotation-overdue` | 11 鍵管理の rotation cadence 超過時 block |
| `block-manual-ddl` | psql / kafka-topics.sh / clickhouse-client の手動 DDL を audit + block |
| `block-retention-shorter-than-class` | retention 値が preservation_class 既定より短い場合 block |
| `block-encryption-disabled-pv` | PV encryption=false を block |
| `require-replication-lag-sli` | 全 instance に replication_lag SLI emit 必須 |
| `require-outbox-table-naming` | tier2 業務 Service の table 命名が outbox_<context>_v<n> 規約に従う |
| `require-apicurio-compatibility-rule` | 全 subject に rule 必須 |
| `require-cosign-signed-image-data-operator` | data 層 Operator image は Cosign signed のみ許可 |

## admission policy のライフサイクル
- 全 admission policy は build artifact（`data/policy/kyverno-policies.yaml`）から生成、手書き禁止
- generator の input:
    - `classes.yaml`（preservation 各軸）
    - `preservation_substrates.lock.yaml`（infra topology ↔ data preservation の双方向 lock）
    - `restore_drill.lock.yaml`（drill 状態）
    - `rotation_progress.lock.yaml`（11 鍵管理）
    - `oss_inventory.lock.yaml`（14 OSS lifecycle）
    - `error_budget.lock.yaml`（13 SLO）
- drift: runtime に admission policy が build artifact と divergence した場合、Argo CD が drift として検出 + self-heal

## break-glass
- 緊急時に admission policy を bypass する経路:
    1. OpenBao response wrapping で短期 cluster-admin 相当 + DB superuser token を発行
    2. 発行操作は 09 観測可能性 SoR に audit signal を emit（`data.break_glass.event`）
    3. 使用後 1 時間以内に postmortem PR 起票必須
    4. break-glass 使用は 13 SLO error budget consumption として記録
    5. break-glass 経由の DDL も migration 経路として履歴化

## CI 不変条件（data 全体）
- 違反は merge 不可

| # | 整合 |
|---|---|
| 整合 1 | 全 instance に preservation_class 注釈 |
| 整合 2 | `preservation_substrates.lock.yaml` は build artifact と git の完全一致 |
| 整合 3 | `restore_drill.lock.yaml` の全 cluster で last_green_at が drill_cadence_days 以内（v1_local_only を除く）|
| 整合 4 | 全 admission policy が build artifact から生成 |
| 整合 5 | 全 entity table に tenant_id / version_id / created_at / updated_at / deleted_at 列 |
| 整合 6 | 全 PII 分類列に envelope encryption 注釈 + blind index（必要時）|
| 整合 7 | 全 KafkaTopic に min.insync.replicas / replicationFactor / cleanup.policy / retention.ms が preservation_class 規定値 |
| 整合 8 | 全 ClickHouse Replicated table に TTL clause |
| 整合 9 | 全 OSS バージョンが `oss_inventory.lock.yaml` と整合 |
| 整合 10 | 13 軸 + meta（00）の各 spec の build artifact と data 層の preservation_substrates / restore_drill が双方向 lock |
| 整合 11 | infra topology_class と data preservation_class の bind 表が双方向 lock |
| 整合 12 | 全 schema migration が forward-only |
| 整合 13 | 全 destructive DDL が contract phase id 必須 |

## 採用しない強制機構
- OPA Gatekeeper: Kyverno L1+ 単一深耕
- Atomic Schema 単独運用: sqlx-cli + Apicurio で吸収
- 文章 only の運用ルール: 禁止
- 「business 都合で envelope encryption を一時 off」: 禁止

## 関連参照
- [data 設計方針 index](../../03_概要設計/06_data設計方針/README.md)
- [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md)
- [data 運用 UI](../04_運用UI開発者体験/02_data運用UI.md)
- [PII 専用クラスタ](../03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md)
