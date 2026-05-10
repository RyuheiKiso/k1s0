---
id: detail.data.ops_dx
axis: data
phase: detail
kind: ops_dx
status: draft
depends_on:
  - arch.data.data_index
  - arch.data.datastore_composition_policy
  - arch.data.schema_operations_policy
  - detail.data.data_enforcement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# data 運用 UI と開発者体験

## 一文方針
- data 層の運用 UI は infra Headlamp / Backstage の上に薄く乗る形で提供し、独自の UI 系 OSS は採用しない。全運用操作は GitOps + Argo CD reconciliation 経路、UI は読み取り / 限定的 trigger 専用。開発者体験は Testcontainers（実 instance）+ sqlx prepare（type-safe SQL）+ Apicurio CLI（schema fetch）+ Backstage software template の 4 点で構成する。

## 至高路線における立ち位置
- 独自の UI 系 OSS 採用しない（infra UI に薄く乗る）
- mock DB / 共有 dev DB 採用しない（Testcontainers default）
- GUI 操作による DB 変更禁止（pgAdmin / Adminer / Kafka UI は読み取り限定）
- write 系操作は admission policy で本番経路の手動 write を block

## 運用 UI の構成

### 層 1: infra 提供
- Headlamp: cluster 全体の k8s 資源 viewer、namespace 単位 CR 閲覧
- Backstage: service catalog、software template、API doc viewer
- Kubeshark: API トラフィック viewer（read only）

### 層 2: data 層が提供する dashboard plugin

#### Backstage plugin（自製）
- preservation_class 一覧（class instance、instance attribute、状態）
- `restore_drill.lock.yaml` の green/red ビュー
- replication lag SLI 一覧
- migration progress（phase A/B/C 状態）
- retention 残時間 / purge 予定
- rotation 進捗（KEK rotation の wrapped DEK 再 wrap 進捗）

#### Headlamp plugin（自製）
- CloudNativePG Cluster CR の primary/standby 位置、replication lag
- Strimzi Kafka CR の broker / KRaft controller 健康状態
- Apicurio subject の compatibility rule 状態
- ClickHouse cluster の shard / replica / keeper quorum

### 層 3: observability
- Apache Superset / Perses（infra 提供）の dashboard として data 層 SLI（replication lag / WAL archive lag / backup age）を可視化

## DBA 操作経路
- read 系: psql / kafka-console-consumer / clickhouse-client / valkey-cli は Backstage の Web shell（読み取り限定 role）で発行可能。query log は ClickHouse に永続
- write 系: admission policy で本番経路の手動 write を block。break-glass のみで一時 superuser 権限を OpenBao response wrapping で発行、使用後 1 時間以内に postmortem PR 起票必須
- 緊急時運用: infra 14 強制機構 の break-glass と同じ経路で、role escalation は audit signal を必ず emit（`data.break_glass.event`）

## 開発者体験（local 開発）
- Testcontainers: tier1 / tier2 / tier3 開発者は local test で 実 PostgreSQL / Kafka / Valkey / ClickHouse / Apicurio container を起動、mock しない
- testcontainer 構成は data 層が template として提供（Backstage software template）:
    - `data-testcontainers-postgres`: CloudNativePG イメージで起動、初期 schema 適用済
    - `data-testcontainers-kafka`: Strimzi イメージで KRaft mode、Apicurio container 同梱
    - `data-testcontainers-clickhouse`: keeper 内蔵
    - `data-testcontainers-valkey`: cluster mode 4 node
- container image は Harbor から pull、Cosign signed
- sqlx prepare: CI で sqlx-cli が migration 適用済 DB に対して type check を行う、生成した `.sqlx/` メタファイルを git に commit

## schema 開発体験

### tier1 / tier2 開発者の workflow
1. migration ファイルを `<service>/migrations/` に追加
2. Apicurio Registry に新 subject を local で push（test container）
3. local Testcontainers で sqlx prepare → unit test 実行
4. PR: CI が schema diff、compatibility check、expand-contract phase 確認
5. merge → Argo CD が staging cluster に migration apply
6. release: production rollout、preservation_class の dual phase 期間を経て contract

### schema diff visualization
- atlas（参照のみ）が CI で diff 生成、PR comment に Markdown table を post

## migration progress UI
- Backstage plugin（migration progress）:
    - 進行中の migration 一覧（service / phase / 進捗 %）
    - backfill rows 残量
    - dual phase 残日数（preservation_class 別 min を basis に表示）
    - contract 可能日（dual 期間経過 + backfill 完了）
- 開発者は plugin で「contract phase に進める PR を起票してよい」timing を確認

## schema browser
- Backstage plugin（schema browser）:
    - 全 service の table / topic / subject 一覧
    - 各 column / field の preservation 注釈、PII 分類、encryption flag
    - schema 進化履歴（version timeline）
    - Apicurio subject の compatibility rule 状態

## DBA 視点の monitoring dashboard
- Apache Superset / Perses dashboard（自製）:
    - replication lag dashboard（per cluster / topic）
    - WAL archive lag
    - backup age（latest base backup からの経過）
    - migration in-flight count
    - purge job 完了率 / 失敗履歴
    - rotation 進捗

## 採用しない方針
- 独自の UI 系 OSS（pgAdmin / Adminer / Kafka UI 等の手動 write 用途）: 採用しない
- mock DB / 共有 dev DB: 採用しない
- GUI 操作による DB 変更: 禁止
- write 系操作の手動経路: 禁止（break-glass のみ）

## 関連参照
- [data 設計方針 index](../../03_概要設計/06_data設計方針/README.md)
- [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md)
- [data 強制機構](../02_強制機構/05_data強制機構.md)
