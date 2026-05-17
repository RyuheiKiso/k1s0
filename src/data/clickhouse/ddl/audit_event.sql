-- ============================================================
-- ClickHouse audit_event テーブル DDL
-- 監査ログの長期保管・高速集計分析用の OLAP テーブル定義
-- ReplicatedMergeTree: レプリケーション + 効率的なデータ集約
-- ============================================================

-- k1s0 データベースが存在しない場合は作成する（冪等性を保証）
CREATE DATABASE IF NOT EXISTS k1s0 ON CLUSTER k1s0_cluster;

-- ============================================================
-- ローカルテーブル定義（各ノードに配置されるシャードテーブル）
-- ============================================================

-- audit_event ローカルテーブルを作成（各 shard/replica に 1 インスタンス）
CREATE TABLE IF NOT EXISTS k1s0.audit_event_local ON CLUSTER k1s0_cluster (
    -- 監査イベントの一意識別子（PostgreSQL の UUID を文字列で保持）
    event_id        String,
    -- イベント記録時刻（ClickHouse の DateTime64 でマイクロ秒精度）
    event_at        DateTime64(6, 'UTC'),
    -- テナント識別子（UUID 文字列: シャーディングキーとして使用）
    tenant_id       String,
    -- 監査イベント種別（例: 'row_inserted', 'login_success'）
    event_kind      String,
    -- イベント発生元のアクター識別子（ユーザーまたはサービス名）
    actor           String,
    -- イベント対象のリソース識別子（テーブル名・集約 ID 等。空の場合もある）
    resource        String,
    -- 監査イベントの詳細データ（JSON 文字列形式）
    payload         String,
    -- 前の監査イベントのハッシュ値（hash chain 検証に使用）
    previous_hash   Nullable(String),
    -- このイベントの SHA-256 ハッシュ値（tamper detection に使用）
    current_hash    String,
    -- データ取り込み日（パーティション分割のキー）
    -- toDate(event_at) で自動算出してパーティション管理を簡素化
    event_date      Date MATERIALIZED toDate(event_at)
)
-- ReplicatedMergeTree: 自動レプリケーションと効率的なデータ圧縮
-- ZooKeeper (Keeper) パスとレプリカ名を動的に設定
ENGINE = ReplicatedMergeTree(
    -- Keeper 上のテーブルメタデータパス（クラスター・データベース・テーブル名を含む）
    '/clickhouse/tables/{cluster}/{database}/{table}/{shard}',
    -- このレプリカの識別子（ホスト名を使用して一意性を確保）
    '{replica}'
)
-- パーティション: 月単位でデータを分割（長期保持データのクエリ性能を最適化）
PARTITION BY toYYYYMM(event_date)
-- プライマリキー: テナント・時刻・イベント種別の複合キー
-- カラムナーストレージの効率的なスキップインデックスを生成する
ORDER BY (tenant_id, event_at, event_kind, event_id)
-- TTL: 90 日後に自動削除（GDPR データ保持ポリシーに対応）
TTL event_date + INTERVAL 90 DAY
-- テーブル設定
SETTINGS
    -- インデックスの粒度（デフォルト 8192 を調整して I/O を最適化）
    index_granularity = 8192,
    -- 圧縮アルゴリズム: ZSTD（ストレージ効率と解凍速度のバランス）
    compress_marks = 1;

-- ============================================================
-- 分散テーブル定義（クエリルーター: 全 shard に分散してクエリを実行）
-- ============================================================

-- audit_event 分散テーブル（クライアントはこのテーブルにのみクエリを発行する）
CREATE TABLE IF NOT EXISTS k1s0.audit_event ON CLUSTER k1s0_cluster
-- Distributed エンジン: 全シャードへのクエリ分散・結果集約を自動実行
AS k1s0.audit_event_local
ENGINE = Distributed(
    -- ルーティング先のクラスター名（cluster.yaml で定義した名前）
    k1s0_cluster,
    -- ルーティング先のデータベース名
    k1s0,
    -- ルーティング先のローカルテーブル名
    audit_event_local,
    -- シャーディングキー: tenant_id のハッシュで均等分散
    -- 同一テナントのデータを同一シャードに集約してクエリ効率を向上
    cityHash64(tenant_id)
);

-- ============================================================
-- マテリアライズドビュー: イベント種別ごとの集計テーブル
-- ============================================================

-- テナント別・日別・イベント種別別の集計（ダッシュボード高速化）
CREATE MATERIALIZED VIEW IF NOT EXISTS k1s0.audit_event_daily_summary_local
ON CLUSTER k1s0_cluster
ENGINE = ReplicatedSummingMergeTree(
    -- Keeper パス（サマリーテーブル専用のパスを使用）
    '/clickhouse/tables/{cluster}/{database}/audit_event_daily_summary/{shard}',
    -- レプリカ識別子
    '{replica}'
)
-- パーティション: 月単位で分割
PARTITION BY toYYYYMM(event_date)
-- 集約キー: テナント・日付・イベント種別
ORDER BY (tenant_id, event_date, event_kind)
-- マテリアライズドビューのデータソース
AS SELECT
    -- テナント識別子（集計キー）
    tenant_id,
    -- イベント記録日（集計の粒度）
    toDate(event_at) AS event_date,
    -- イベント種別（集計キー）
    event_kind,
    -- イベント件数（SummingMergeTree が自動集計する）
    count() AS event_count
FROM k1s0.audit_event_local
GROUP BY tenant_id, event_date, event_kind;
