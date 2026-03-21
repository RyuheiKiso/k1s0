# データベース Sharding 設計

## 概要

現状の単一 PostgreSQL インスタンス構成から、サービス規模拡大に備えた
段階的な Sharding 戦略を定義する。

## 現状分析

### データベース構成（2026年3月時点）
- **System Tier**: 22スキーマが `k1s0_system` データベースに同居
- **Business Tier**: `k1s0_business` データベース（domain-master スキーマ）
- **Service Tier**: 3スキーマ（inventory/order/payment）が `k1s0_service` に同居
- 全て単一 PostgreSQL 17.2 インスタンスで稼働

### 課題
- スケールアップ（垂直スケーリング）の限界が明確
- 1つのスロークエリが全サービスに影響
- DB メンテナンス（VACUUM/ANALYZE）が全スキーマに影響

## 移行フェーズ

### Phase 1: 垂直分割（ティア別インスタンス分離）

**目標**: ティアごとに独立した PostgreSQL インスタンスを用意する
**判断基準**: DB CPU 使用率が 70% を超えた時点で実施を検討する

```
[現状]                    [Phase 1 後]
PostgreSQL (単一)    →    PostgreSQL-system   (k1s0_system)
 - k1s0_system            PostgreSQL-business (k1s0_business)
 - k1s0_business          PostgreSQL-service  (k1s0_service)
 - k1s0_service
```

**移行手順**:
1. 新インスタンスを並行起動し、`pg_logical` レプリケーションで同期
2. サービスの `DATABASE_URL` を config-server 経由で切り替える（ダウンタイムゼロ）
3. 旧インスタンスへの接続がゼロになったことを確認してから停止

**実装変更箇所**:
- `infra/helm/*/values.yaml` の `database.host` を更新
- `regions/system/database/` の `sqlx.toml` を更新
- 接続プール設定: PgBouncer（transaction mode）を各インスタンスの前段に配置

### Phase 2: 水平分割（テナントベース Sharding）

**目標**: テナント ID でデータを複数の PostgreSQL インスタンスに分散する
**判断基準**: 単一インスタンスのストレージが 1TB を超えた時点で検討する

**対象スキーマ**（テナント境界が明確なもの優先）:
- `auth` スキーマ: `tenant_id` カラムでシャーディング
- `event-store` スキーマ: `tenant_id` × 時系列でパーティショニング
- `audit-log`: 時系列パーティショニング（月次）

**シャーディングキー**:
```sql
-- tenant_id の hash でシャードを決定する（一貫したハッシュ）
shard_id = crc32(tenant_id) % SHARD_COUNT
```

**アプリケーション変更**:
- k1s0-migration ライブラリにシャードルーティング機能を追加
- 全リポジトリ実装で `tenant_id` を WHERE 条件に含める（既存実装を確認）

### Phase 3: CQRS + Read Replica

**目標**: 読み取り負荷が高いサービスの Read/Write を分離する
**判断基準**: 読み取りクエリの P99 レイテンシが 500ms を超えた時点で検討する

**対象サービス**（読み取り比率が高いもの）:
- `service-catalog`: サービス一覧・検索
- `search`: 全文検索（OpenSearch への移行も検討）
- `api-registry`: API スキーマ参照

**実装方針**:
```
Write → Primary PostgreSQL
Read  → Read Replica（ストリーミングレプリケーション、遅延 < 100ms）
```

## 接続管理

### 現状の接続プール設定
各サービスは sqlx の接続プールを使用し、環境変数 `DATABASE_URL` で接続先を指定する。

### Phase 1 以降の接続管理
```yaml
# config.yaml スキーマ（Phase 1 対応後）
database:
  primary_url: ${DATABASE_URL}      # 書き込み用
  replica_url: ${DATABASE_REPLICA_URL}  # 読み取り用（Phase 3）
  pool:
    max_connections: 20
    min_connections: 2
    connect_timeout_secs: 30
```

## 監視

Sharding 実施後は以下のメトリクスを追加で監視する:
- 各シャードのクエリ数バランス（偏りが 30% を超えたら再シャーディングを検討）
- クロスシャードクエリの発生頻度（ゼロが理想）
- レプリケーション遅延（Read Replica、目標 < 100ms）

## 関連ドキュメント

- データベーススキーマ設計: `regions/system/database/`
- マイグレーション設計: `regions/service/*/database/postgres/migrations/`
- デプロイ設計: `docs/architecture/deployment/プログレッシブデリバリー設計.md`
