-- k1s0 tier2 audit テーブルスキーマ（ClickHouse 版）
-- audit_event を ClickHouse に長期保存するためのスキーマ定義
-- hash_digest / prev_digest カラムを含む（業務エラー監査 08 の hash chain）

-- k1s0_audit データベースを作成する
CREATE DATABASE IF NOT EXISTS k1s0_audit;

-- audit_events テーブルを作成する（ClickHouse カラム指向エンジン）
-- MergeTree エンジンを使用してテナント別・時系列パーティショニングを実現する
CREATE TABLE IF NOT EXISTS k1s0_audit.audit_events
(
    -- エントリの主キー（UUID）
    id UUID,
    -- 関連する aggregate の ID（UUID）
    aggregate_id UUID,
    -- テナント ID（テナント別パーティショニングに使用する）
    tenant_id UUID,
    -- アクター識別子（Keycloak subject）
    actor_id String,
    -- セッション目的（business_op / support / export / migration / emergency）
    purpose LowCardinality(String),
    -- テーブルクラス（TenantScoped / TenantMaster / PlatformGlobal / PiiSegregated）
    table_class LowCardinality(String),
    -- 操作内容のペイロード（JSON 文字列として格納する）
    payload String,
    -- 書込日時（ミリ秒精度、UTC タイムゾーン付き）
    created_at DateTime64(3, 'UTC'),
    -- hash chain: このエントリの SHA-256 ダイジェスト（null = hash 計算未実施）
    hash_digest Nullable(String),
    -- hash chain: 直前のエントリの hash_digest（null = チェーン先頭）
    prev_digest Nullable(String),
    -- hash chain: テナント内での連番（null = 連番未設定）
    chain_sequence Nullable(Int64)
)
ENGINE = MergeTree()
-- テナント別・月次パーティショニング（クエリの局所化に使用する）
PARTITION BY (toYYYYMM(created_at), tenant_id)
-- テナント ID と時系列でソートキーを構成する（テナント別時系列クエリを高速化する）
ORDER BY (tenant_id, created_at, id)
-- audit_event の保持期間: 7 年 (2555 日) = TTL で自動削除する
TTL created_at + INTERVAL 2555 DAY
-- 設定: 重複排除を有効にする（Kafka CDC の at-least-once 配信に対応する）
SETTINGS deduplicate_blocks_in_dependent_materialized_views = 1;

-- hash_chain_verification テーブル: hash chain 検証結果を格納する
-- バックグラウンドジョブが定期的に verify_chain を実行して結果を記録する
CREATE TABLE IF NOT EXISTS k1s0_audit.hash_chain_verification
(
    -- テナント ID（検証対象テナント）
    tenant_id UUID,
    -- 検証実施日時
    verified_at DateTime64(3, 'UTC'),
    -- 検証ステータス（ok / tampered）
    status LowCardinality(String),
    -- 最後に検証したエントリの chain_sequence
    last_verified_sequence Int64,
    -- 改ざん検出時のエラーメッセージ（null = 正常）
    error_message Nullable(String)
)
ENGINE = MergeTree()
-- テナント別・日次パーティショニング
PARTITION BY (toYYYYMMDD(verified_at), tenant_id)
-- テナント ID と検証日時でソートキーを構成する
ORDER BY (tenant_id, verified_at);
