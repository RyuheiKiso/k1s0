"""src/data/clickhouse/analytics_client.py

ClickHouse 分析クライアントモジュール。
製造業プラットフォームの生産メトリクス・品質データ・センサー時系列を
ClickHouse に挿入・クエリするインターフェースを提供する。
仕様: docs/03_概要設計/06_data設計方針/README.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# OS 操作に使用する os をインポートする
import os
# システム操作に使用する sys をインポートする
import sys
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone, timedelta
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, Iterator, List, Optional, Tuple
# URL 操作に使用する urllib をインポートする
import urllib.request
# URL エラー処理に使用する urllib.error をインポートする
import urllib.error
# URL パース・エンコードに使用する urllib.parse をインポートする
from urllib.parse import urlencode, quote

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)

# ClickHouse HTTP インターフェースのデフォルトポート
_DEFAULT_CH_HTTP_PORT = 8123
# ClickHouse のデフォルトデータベース名
_DEFAULT_CH_DATABASE = "manufacturing"
# デフォルトの接続タイムアウト（秒）
_DEFAULT_TIMEOUT_SECONDS = 30
# ClickHouse バッチ挿入のデフォルトサイズ
_DEFAULT_BATCH_SIZE = 1000
# 環境変数からの ClickHouse ホスト取得キー
_ENV_CH_HOST = "CLICKHOUSE_HOST"
# 環境変数からの ClickHouse パスワード取得キー
_ENV_CH_PASSWORD = "CLICKHOUSE_PASSWORD"


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class ClickHouseConfig:
    """ClickHouse 接続設定を表すデータクラス。"""

    # ClickHouse サーバーのホスト名またはIP
    host: str
    # HTTP インターフェースのポート番号
    port: int = _DEFAULT_CH_HTTP_PORT
    # 接続するデータベース名
    database: str = _DEFAULT_CH_DATABASE
    # 認証ユーザー名
    user: str = "default"
    # 認証パスワード
    password: str = ""
    # 接続タイムアウト（秒）
    timeout: int = _DEFAULT_TIMEOUT_SECONDS
    # TLS を使用するかどうか
    use_tls: bool = False

    def base_url(self) -> str:
        """ClickHouse HTTP エンドポイントの Base URL を返す。"""
        # プロトコルを決定する
        scheme = "https" if self.use_tls else "http"
        # Base URL を構築して返す
        return f"{scheme}://{self.host}:{self.port}/"

    @classmethod
    def from_env(cls) -> "ClickHouseConfig":
        """環境変数から ClickHouseConfig を生成して返す。"""
        # ホスト名を環境変数から取得する
        host = os.environ.get(_ENV_CH_HOST, "localhost")
        # パスワードを環境変数から取得する
        password = os.environ.get(_ENV_CH_PASSWORD, "")
        # ClickHouseConfig を生成して返す
        return cls(host=host, password=password)


@dataclass
class ManufacturingMetric:
    """製造メトリクスの1レコードを表すデータクラス。"""

    # 記録時刻（ISO 8601 形式）
    recorded_at: str
    # 生産ライン ID
    line_id: str
    # テナント ID
    tenant_id: str
    # メトリクスの種別（例: "throughput", "defect_rate", "temperature"）
    metric_type: str
    # メトリクスの数値
    value: float
    # メトリクスの単位（例: "units/hour", "%", "celsius"）
    unit: str = ""
    # メトリクスのタグ辞書（追加属性）
    tags: Dict[str, str] = field(default_factory=dict)

    def to_row(self) -> Tuple:
        """ClickHouse 挿入用のタプル形式に変換して返す。"""
        # タグを JSON 文字列に変換する
        tags_json = json.dumps(self.tags, ensure_ascii=False)
        # タプル形式で返す
        return (
            self.recorded_at,
            self.line_id,
            self.tenant_id,
            self.metric_type,
            self.value,
            self.unit,
            tags_json,
        )


@dataclass
class SensorReading:
    """センサーの読み取り値を表すデータクラス。"""

    # 読み取り時刻（ナノ秒精度の UNIX タイムスタンプ）
    timestamp_ns: int
    # センサーの一意識別子
    sensor_id: str
    # センサーが設置されているゾーン
    zone_id: str
    # テナント ID
    tenant_id: str
    # センサーの読み取り値
    value: float
    # 読み取りの品質スコア（0.0〜1.0）
    quality: float = 1.0
    # センサーの状態フラグ（0=正常, 1=警告, 2=異常）
    status_flag: int = 0

    def recorded_at_iso(self) -> str:
        """ナノ秒タイムスタンプを ISO 8601 文字列に変換して返す。"""
        # ナノ秒を秒に変換して datetime に変換する
        dt = datetime.fromtimestamp(self.timestamp_ns / 1e9, tz=timezone.utc)
        # ISO 8601 形式に変換して返す
        return dt.isoformat()


@dataclass
class QueryResult:
    """ClickHouse クエリ結果を表すデータクラス。"""

    # クエリの SQL テキスト
    query: str
    # クエリ結果の行リスト
    rows: List[List[Any]]
    # カラム名のリスト
    column_names: List[str]
    # クエリ実行時間（秒）
    elapsed_seconds: float = 0.0
    # 読み込んだ行数
    rows_read: int = 0
    # 読み込んだバイト数
    bytes_read: int = 0

    @property
    def row_count(self) -> int:
        """結果の行数を返す。"""
        # 行リストの長さを返す
        return len(self.rows)

    def as_dicts(self) -> List[Dict[str, Any]]:
        """結果行をカラム名をキーとした辞書リストで返す。"""
        # 行ごとに辞書を生成してリストで返す
        return [
            dict(zip(self.column_names, row))
            for row in self.rows
        ]

    def first_row_dict(self) -> Optional[Dict[str, Any]]:
        """最初の行をカラム名をキーとした辞書で返す（結果が空の場合は None）。"""
        # 行が存在する場合は最初の行を辞書で返す
        if not self.rows:
            # 行が存在しない場合は None を返す
            return None
        # 最初の行を辞書で返す
        return dict(zip(self.column_names, self.rows[0]))


# ===========================================================================
# DDL テンプレート定義
# ===========================================================================

# 製造メトリクステーブルの DDL
_MANUFACTURING_METRICS_DDL = """
CREATE TABLE IF NOT EXISTS {database}.manufacturing_metrics
(
    recorded_at DateTime64(3, 'UTC'),
    line_id LowCardinality(String),
    tenant_id LowCardinality(String),
    metric_type LowCardinality(String),
    value Float64,
    unit LowCardinality(String),
    tags String CODEC(ZSTD(1))
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(recorded_at)
ORDER BY (tenant_id, line_id, metric_type, recorded_at)
TTL recorded_at + INTERVAL 365 DAY
SETTINGS index_granularity = 8192;
"""

# センサー読み取りテーブルの DDL
_SENSOR_READINGS_DDL = """
CREATE TABLE IF NOT EXISTS {database}.sensor_readings
(
    timestamp_ns UInt64,
    sensor_id LowCardinality(String),
    zone_id LowCardinality(String),
    tenant_id LowCardinality(String),
    value Float64,
    quality Float32,
    status_flag UInt8
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(fromUnixTimestamp64Nano(timestamp_ns))
ORDER BY (tenant_id, zone_id, sensor_id, timestamp_ns)
TTL fromUnixTimestamp64Nano(timestamp_ns) + INTERVAL 180 DAY
SETTINGS index_granularity = 8192;
"""

# 品質異常ログテーブルの DDL
_QUALITY_ANOMALY_DDL = """
CREATE TABLE IF NOT EXISTS {database}.quality_anomalies
(
    detected_at DateTime64(3, 'UTC'),
    line_id LowCardinality(String),
    tenant_id LowCardinality(String),
    anomaly_type LowCardinality(String),
    severity UInt8,
    value Float64,
    threshold Float64,
    description String,
    resolved_at Nullable(DateTime64(3, 'UTC'))
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(detected_at)
ORDER BY (tenant_id, line_id, detected_at)
SETTINGS index_granularity = 8192;
"""


# ===========================================================================
# AnalyticsClient クラス
# ===========================================================================


class AnalyticsClient:
    """ClickHouse 分析クライアントクラス。

    製造業プラットフォームの時系列メトリクス・センサーデータ・品質異常を
    ClickHouse HTTP インターフェース経由で操作する。
    """

    def __init__(self, config: Optional[ClickHouseConfig] = None) -> None:
        """AnalyticsClient を設定で初期化する。

        Args:
            config: ClickHouse 接続設定（省略時は環境変数から読み込む）
        """
        # 接続設定を設定する（省略時は環境変数から生成）
        self._config = config or ClickHouseConfig.from_env()
        # セッションヘッダーを初期化する
        self._default_params: Dict[str, str] = {
            "database": self._config.database,
            "user": self._config.user,
        }

    def _execute_query(
        self, query: str, data: Optional[bytes] = None
    ) -> str:
        """ClickHouse HTTP インターフェースに SQL クエリを送信して結果を返す。

        Args:
            query: 実行する SQL テキスト
            data: POST ボディデータ（INSERT 時に使用）

        Returns:
            レスポンスのテキスト文字列

        Raises:
            RuntimeError: クエリ実行に失敗した場合
        """
        # クエリパラメータを構築する
        params = dict(self._default_params)
        # query パラメータを設定する（data がない場合は URL パラメータとして送信）
        if data is None:
            # クエリを URL パラメータとして設定する
            params["query"] = query
        # パラメータを URL エンコードする
        encoded_params = urlencode(params)
        # リクエスト URL を構築する
        url = self._config.base_url() + "?" + encoded_params
        # リクエストオブジェクトを生成する
        req = urllib.request.Request(url, data=data, method="POST" if data else "GET")
        # 認証ヘッダーを設定する
        if self._config.password:
            # パスワードが設定されている場合は X-ClickHouse-Key ヘッダーを設定する
            req.add_header("X-ClickHouse-Key", self._config.password)
        # Content-Type ヘッダーを設定する
        if data:
            # バイナリデータの場合は Content-Type を設定する
            req.add_header("Content-Type", "application/octet-stream")
        # クエリを実行する
        try:
            # HTTP リクエストを送信する
            with urllib.request.urlopen(req, timeout=self._config.timeout) as resp:
                # レスポンスを読み込んで返す
                return resp.read().decode("utf-8")
        except urllib.error.HTTPError as exc:
            # HTTP エラーをログに記録する
            error_body = exc.read().decode("utf-8", errors="ignore")
            # RuntimeError を送出する
            raise RuntimeError(
                f"ClickHouse クエリエラー (HTTP {exc.code}): {error_body[:500]}"
            ) from exc
        except urllib.error.URLError as exc:
            # 接続エラーをログに記録する
            raise RuntimeError(
                f"ClickHouse 接続エラー: {exc.reason}"
            ) from exc

    def ping(self) -> bool:
        """ClickHouse サーバーへの接続を確認して True/False を返す。"""
        # ping エンドポイントにアクセスする
        try:
            # ping URL を構築する
            url = self._config.base_url() + "ping"
            # HTTP リクエストを送信する
            req = urllib.request.Request(url)
            # レスポンスを確認する
            with urllib.request.urlopen(req, timeout=5) as resp:
                # 200 以外はエラーとみなす
                return resp.status == 200
        except Exception as exc:
            # 接続エラーをログに記録する
            logger.debug("ClickHouse ping 失敗: %s", exc)
            # False を返す
            return False

    def initialize_tables(self) -> None:
        """製造業プラットフォーム用のテーブルを初期化する。"""
        # テーブル DDL のリストを定義する
        ddl_list = [
            _MANUFACTURING_METRICS_DDL,
            _SENSOR_READINGS_DDL,
            _QUALITY_ANOMALY_DDL,
        ]
        # 各 DDL を実行する
        for ddl in ddl_list:
            # データベース名を埋め込む
            formatted_ddl = ddl.format(database=self._config.database)
            # DDL を実行する
            try:
                # クエリを実行する
                self._execute_query(formatted_ddl)
                # DDL 実行成功をログに記録する
                logger.debug("DDL 実行成功")
            except RuntimeError as exc:
                # DDL 実行エラーをログに記録する
                logger.error("DDL 実行エラー: %s", exc)

    def insert_metrics_batch(self, metrics: List[ManufacturingMetric]) -> int:
        """製造メトリクスをバッチ挿入して挿入した件数を返す。

        Args:
            metrics: 挿入する ManufacturingMetric のリスト

        Returns:
            挿入した件数
        """
        # メトリクスが空の場合は 0 を返す
        if not metrics:
            # 挿入対象がないため 0 を返す
            return 0
        # INSERT クエリのプレフィックスを構築する
        insert_prefix = (
            f"INSERT INTO {self._config.database}.manufacturing_metrics "
            "(recorded_at, line_id, tenant_id, metric_type, value, unit, tags) "
            "FORMAT TabSeparated"
        )
        # バッチサイズに分割して挿入する
        total_inserted = 0
        # バッチを生成する
        for batch_start in range(0, len(metrics), _DEFAULT_BATCH_SIZE):
            # バッチを切り出す
            batch = metrics[batch_start: batch_start + _DEFAULT_BATCH_SIZE]
            # タブ区切り形式のデータを構築する
            rows: List[str] = []
            # 各メトリクスを行に変換する
            for metric in batch:
                # タプル形式に変換する
                row_tuple = metric.to_row()
                # タブ区切り文字列に変換する
                row_str = "\t".join(str(v) for v in row_tuple)
                # 行リストに追加する
                rows.append(row_str)
            # データ文字列を構築する
            data_str = "\n".join(rows) + "\n"
            # バイト列に変換する
            data_bytes = data_str.encode("utf-8")
            # INSERT クエリを実行する
            try:
                # クエリとデータを送信する
                self._execute_query(insert_prefix, data_bytes)
                # 挿入件数を更新する
                total_inserted += len(batch)
                # 挿入成功をログに記録する
                logger.debug("メトリクスバッチ挿入: %d 件", len(batch))
            except RuntimeError as exc:
                # 挿入エラーをログに記録する
                logger.error("メトリクス挿入エラー: %s", exc)
                # エラーが発生した場合は途中で中断しない（ベストエフォート）
        # 挿入した件数を返す
        return total_inserted

    def insert_sensor_readings_batch(self, readings: List[SensorReading]) -> int:
        """センサー読み取り値をバッチ挿入して挿入した件数を返す。

        Args:
            readings: 挿入する SensorReading のリスト

        Returns:
            挿入した件数
        """
        # 読み取り値が空の場合は 0 を返す
        if not readings:
            # 挿入対象がないため 0 を返す
            return 0
        # INSERT クエリのプレフィックスを構築する
        insert_prefix = (
            f"INSERT INTO {self._config.database}.sensor_readings "
            "(timestamp_ns, sensor_id, zone_id, tenant_id, value, quality, status_flag) "
            "FORMAT TabSeparated"
        )
        # 全読み取り値のタブ区切りデータを構築する
        rows: List[str] = []
        # 各読み取り値を行に変換する
        for reading in readings:
            # タブ区切り文字列を生成する
            row_str = "\t".join([
                str(reading.timestamp_ns),
                reading.sensor_id,
                reading.zone_id,
                reading.tenant_id,
                str(reading.value),
                str(reading.quality),
                str(reading.status_flag),
            ])
            # 行リストに追加する
            rows.append(row_str)
        # データ文字列を構築する
        data_str = "\n".join(rows) + "\n"
        # バイト列に変換する
        data_bytes = data_str.encode("utf-8")
        # INSERT クエリを実行する
        try:
            # クエリとデータを送信する
            self._execute_query(insert_prefix, data_bytes)
            # 挿入成功をログに記録する
            logger.debug("センサーデータバッチ挿入: %d 件", len(readings))
            # 挿入した件数を返す
            return len(readings)
        except RuntimeError as exc:
            # 挿入エラーをログに記録する
            logger.error("センサーデータ挿入エラー: %s", exc)
            # エラーの場合は 0 を返す
            return 0

    def query_metrics_summary(
        self,
        tenant_id: str,
        line_id: Optional[str] = None,
        from_dt: Optional[datetime] = None,
        to_dt: Optional[datetime] = None,
    ) -> QueryResult:
        """製造メトリクスのサマリーをクエリして返す。

        Args:
            tenant_id: クエリ対象のテナント ID
            line_id: 特定のライン ID に絞り込む（省略時は全ライン）
            from_dt: クエリ開始時刻（省略時は 24 時間前）
            to_dt: クエリ終了時刻（省略時は現在時刻）

        Returns:
            クエリ結果を含む QueryResult インスタンス
        """
        # デフォルトの時間範囲を設定する
        if to_dt is None:
            # 現在時刻を終了時刻とする
            to_dt = datetime.now(timezone.utc)
        # 開始時刻のデフォルトを設定する
        if from_dt is None:
            # 24 時間前を開始時刻とする
            from_dt = to_dt - timedelta(hours=24)
        # WHERE 句の条件リストを初期化する
        conditions: List[str] = []
        # テナント ID 条件を追加する
        conditions.append(f"tenant_id = '{tenant_id}'")
        # ライン ID 条件を追加する（指定された場合）
        if line_id:
            # ライン ID の絞り込み条件を追加する
            conditions.append(f"line_id = '{line_id}'")
        # 時間範囲条件を追加する
        conditions.append(
            f"recorded_at BETWEEN '{from_dt.strftime('%Y-%m-%d %H:%M:%S')}' "
            f"AND '{to_dt.strftime('%Y-%m-%d %H:%M:%S')}'"
        )
        # WHERE 句を構築する
        where_clause = " AND ".join(conditions)
        # クエリを構築する
        query = f"""
            SELECT
                metric_type,
                count() AS count,
                avg(value) AS avg_value,
                min(value) AS min_value,
                max(value) AS max_value,
                quantile(0.95)(value) AS p95_value
            FROM {self._config.database}.manufacturing_metrics
            WHERE {where_clause}
            GROUP BY metric_type
            ORDER BY metric_type
            FORMAT JSONEachRow
        """
        # クエリを実行する
        try:
            # ClickHouse にクエリを送信する
            raw_result = self._execute_query(query)
            # 結果行を解析する
            rows: List[List[Any]] = []
            # カラム名のリストを定義する
            column_names = ["metric_type", "count", "avg_value", "min_value", "max_value", "p95_value"]
            # 各行を JSON デコードする
            for line in raw_result.strip().splitlines():
                # 空行はスキップする
                if not line.strip():
                    # 空行をスキップする
                    continue
                # JSON をパースする
                row_dict = json.loads(line)
                # 値のリストを構築する
                row_values = [row_dict.get(col) for col in column_names]
                # 行リストに追加する
                rows.append(row_values)
            # QueryResult を生成して返す
            return QueryResult(
                query=query.strip(),
                rows=rows,
                column_names=column_names,
            )
        except (RuntimeError, json.JSONDecodeError) as exc:
            # クエリエラーをログに記録する
            logger.error("メトリクスサマリークエリエラー: %s", exc)
            # 空の QueryResult を返す
            return QueryResult(query=query.strip(), rows=[], column_names=[])

    def query_anomalies(
        self,
        tenant_id: str,
        since: Optional[datetime] = None,
        severity_min: int = 0,
    ) -> QueryResult:
        """品質異常ログをクエリして返す。

        Args:
            tenant_id: クエリ対象のテナント ID
            since: この時刻以降の異常を取得する（省略時は 7 日前）
            severity_min: 最低重大度（0〜255）

        Returns:
            クエリ結果を含む QueryResult インスタンス
        """
        # デフォルトの開始時刻を設定する
        if since is None:
            # 7 日前を開始時刻とする
            since = datetime.now(timezone.utc) - timedelta(days=7)
        # クエリを構築する
        query = f"""
            SELECT
                detected_at,
                line_id,
                anomaly_type,
                severity,
                value,
                threshold,
                description,
                resolved_at
            FROM {self._config.database}.quality_anomalies
            WHERE tenant_id = '{tenant_id}'
              AND detected_at >= '{since.strftime('%Y-%m-%d %H:%M:%S')}'
              AND severity >= {severity_min}
            ORDER BY detected_at DESC
            LIMIT 1000
            FORMAT JSONEachRow
        """
        # カラム名のリストを定義する
        column_names = [
            "detected_at", "line_id", "anomaly_type", "severity",
            "value", "threshold", "description", "resolved_at",
        ]
        # クエリを実行する
        try:
            # ClickHouse にクエリを送信する
            raw_result = self._execute_query(query)
            # 結果行を解析する
            rows: List[List[Any]] = []
            # 各行を JSON デコードする
            for line in raw_result.strip().splitlines():
                # 空行はスキップする
                if not line.strip():
                    # 空行をスキップする
                    continue
                # JSON をパースする
                row_dict = json.loads(line)
                # 値のリストを構築する
                row_values = [row_dict.get(col) for col in column_names]
                # 行リストに追加する
                rows.append(row_values)
            # QueryResult を生成して返す
            return QueryResult(
                query=query.strip(),
                rows=rows,
                column_names=column_names,
            )
        except (RuntimeError, json.JSONDecodeError) as exc:
            # クエリエラーをログに記録する
            logger.error("品質異常クエリエラー: %s", exc)
            # 空の QueryResult を返す
            return QueryResult(query=query.strip(), rows=[], column_names=[])

    def get_table_stats(self) -> Dict[str, Any]:
        """データベース内の全テーブルの統計情報を取得して辞書で返す。"""
        # system.tables からテーブル統計を取得するクエリ
        query = f"""
            SELECT
                name,
                total_rows,
                total_bytes,
                engine,
                partition_key,
                sorting_key
            FROM system.tables
            WHERE database = '{self._config.database}'
            FORMAT JSONEachRow
        """
        # クエリを実行する
        try:
            # ClickHouse にクエリを送信する
            raw_result = self._execute_query(query)
            # テーブル統計の辞書を初期化する
            stats: Dict[str, Any] = {}
            # 各行を JSON デコードする
            for line in raw_result.strip().splitlines():
                # 空行はスキップする
                if not line.strip():
                    # 空行をスキップする
                    continue
                # JSON をパースする
                table_info = json.loads(line)
                # テーブル名を取得する
                table_name = table_info.get("name", "")
                # テーブル統計を辞書に追加する
                stats[table_name] = table_info
            # テーブル統計を返す
            return stats
        except (RuntimeError, json.JSONDecodeError) as exc:
            # クエリエラーをログに記録する
            logger.error("テーブル統計取得エラー: %s", exc)
            # 空辞書を返す
            return {}

    def execute_raw(self, sql: str) -> QueryResult:
        """任意の SQL クエリを実行して QueryResult を返す。

        Args:
            sql: 実行する SQL テキスト（FORMAT JSONEachRow を含む場合は結果をパースする）

        Returns:
            クエリ結果を含む QueryResult インスタンス
        """
        # クエリを実行する
        try:
            # ClickHouse にクエリを送信する
            raw_result = self._execute_query(sql)
            # FORMAT JSONEachRow の場合は結果をパースする
            if "FORMAT JSONEachRow" in sql.upper():
                # 結果行を解析する
                rows: List[List[Any]] = []
                # カラム名のリストを初期化する
                column_names: List[str] = []
                # 各行を JSON デコードする
                for line in raw_result.strip().splitlines():
                    # 空行はスキップする
                    if not line.strip():
                        # 空行をスキップする
                        continue
                    # JSON をパースする
                    row_dict = json.loads(line)
                    # カラム名が未設定の場合は最初の行から取得する
                    if not column_names:
                        # カラム名をソートして設定する
                        column_names = sorted(row_dict.keys())
                    # 値のリストを構築する
                    row_values = [row_dict.get(col) for col in column_names]
                    # 行リストに追加する
                    rows.append(row_values)
                # QueryResult を生成して返す
                return QueryResult(query=sql, rows=rows, column_names=column_names)
            # FORMAT が指定されていない場合は生テキストを返す
            return QueryResult(
                query=sql,
                rows=[[raw_result]],
                column_names=["result"],
            )
        except RuntimeError as exc:
            # クエリエラーをログに記録する
            logger.error("生クエリエラー: %s", exc)
            # 空の QueryResult を返す
            return QueryResult(query=sql, rows=[], column_names=[])
