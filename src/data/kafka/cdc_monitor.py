"""src/data/kafka/cdc_monitor.py

Debezium CDC（Change Data Capture）モニタリングモジュール。
製造業プラットフォームの PostgreSQL → ClickHouse パイプラインの
遅延・ラグ・整合性を監視してアラートを生成する。
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
from typing import Any, Dict, List, Optional, Tuple

# PyYAML ライブラリのインポートを試みる
try:
    # YAML ファイルの読み込みに使用する yaml をインポートする
    import yaml
    # yaml が利用可能であることを示すフラグを設定する
    _HAS_YAML = True
except ImportError:
    # yaml がインストールされていない場合はフラグを False に設定する
    _HAS_YAML = False

# HTTP リクエストに使用する urllib をインポートする
import urllib.request
# HTTP エラー処理に使用する urllib.error をインポートする
import urllib.error

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)

# Kafka Connect REST API のデフォルトポート
_DEFAULT_CONNECT_PORT = 8083
# Debezium コネクタ状態の正常値
_CONNECTOR_STATE_RUNNING = "RUNNING"
# CDC ラグのアラート閾値（秒）
_LAG_ALERT_THRESHOLD_SECONDS = 60
# 環境変数からの Kafka Connect ホスト取得キー
_ENV_KAFKA_CONNECT_HOST = "KAFKA_CONNECT_HOST"


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class ConnectorStatus:
    """Debezium コネクタの状態を表すデータクラス。"""

    # コネクタの名称
    connector_name: str
    # コネクタの状態（RUNNING/PAUSED/FAILED/UNASSIGNED）
    state: str
    # コネクタが稼働しているワーカーのホスト
    worker_id: str
    # タスクの状態リスト（タスク ID → 状態）
    task_states: Dict[int, str] = field(default_factory=dict)
    # エラーメッセージ（FAILED 状態の場合に設定）
    error_trace: str = ""
    # 状態確認時刻
    checked_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def is_healthy(self) -> bool:
        """コネクタが正常稼働中かどうかを返す。"""
        # コネクタが RUNNING 状態であることを確認する
        if self.state != _CONNECTOR_STATE_RUNNING:
            # コネクタが RUNNING でない場合は不健全とみなす
            return False
        # 全タスクが RUNNING 状態であることを確認する
        return all(s == _CONNECTOR_STATE_RUNNING for s in self.task_states.values())

    def unhealthy_tasks(self) -> List[int]:
        """RUNNING でないタスク ID のリストを返す。"""
        # RUNNING でないタスク ID をフィルタリングして返す
        return [
            tid for tid, state in self.task_states.items()
            if state != _CONNECTOR_STATE_RUNNING
        ]


@dataclass
class CdcLagInfo:
    """CDC パイプラインのラグ情報を表すデータクラス。"""

    # コネクタ名
    connector_name: str
    # 対象テーブル名
    table_name: str
    # 最新の LSN（Log Sequence Number）
    latest_lsn: str
    # コネクタが処理済みの LSN
    processed_lsn: str
    # 推定ラグ（秒）
    estimated_lag_seconds: float
    # ラグ測定時刻
    measured_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def is_lagging(self) -> bool:
        """ラグがアラート閾値を超えているかどうかを返す。"""
        # ラグがアラート閾値以上の場合は lagging とみなす
        return self.estimated_lag_seconds >= _LAG_ALERT_THRESHOLD_SECONDS


@dataclass
class PipelineHealthReport:
    """CDC パイプライン全体の健全性レポートを表すデータクラス。"""

    # 確認したコネクタの状態リスト
    connector_statuses: List[ConnectorStatus] = field(default_factory=list)
    # テーブル別 CDC ラグ情報リスト
    lag_infos: List[CdcLagInfo] = field(default_factory=list)
    # 検出したアラートメッセージのリスト
    alerts: List[str] = field(default_factory=list)
    # レポート生成時刻
    generated_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def is_healthy(self) -> bool:
        """パイプライン全体が健全かどうかを返す。"""
        # アラートが存在しない場合のみ健全とみなす
        if self.alerts:
            # アラートが存在する場合は不健全とみなす
            return False
        # 全コネクタが健全であることを確認する
        return all(c.is_healthy for c in self.connector_statuses)

    def summary(self) -> str:
        """レポートのサマリー文字列を返す。"""
        # 健全状態のラベルを決定する
        status = "HEALTHY" if self.is_healthy else "DEGRADED"
        # サマリー文字列をフォーマットして返す
        return (
            f"PipelineHealthReport[{status}] "
            f"connectors={len(self.connector_statuses)} "
            f"lagging_tables={sum(1 for l in self.lag_infos if l.is_lagging)} "
            f"alerts={len(self.alerts)}"
        )


# ===========================================================================
# CDCMonitor クラス
# ===========================================================================


class CDCMonitor:
    """Debezium CDC パイプラインを監視するクラス。

    Kafka Connect REST API を使ってコネクタの状態を確認し、
    ラグ・エラー・整合性の問題を検出する。
    """

    def __init__(
        self,
        connect_host: Optional[str] = None,
        connect_port: int = _DEFAULT_CONNECT_PORT,
        timeout: int = 30,
    ) -> None:
        """CDCMonitor を初期化する。

        Args:
            connect_host: Kafka Connect のホスト名（省略時は環境変数を使用）
            connect_port: Kafka Connect の REST API ポート
            timeout: HTTP タイムアウト（秒）
        """
        # ホスト名を設定する（引数 → 環境変数 → デフォルトの順）
        self._host = connect_host or os.environ.get(
            _ENV_KAFKA_CONNECT_HOST, "localhost"
        )
        # ポート番号を設定する
        self._port = connect_port
        # タイムアウトを設定する
        self._timeout = timeout
        # Base URL を構築する
        self._base_url = f"http://{self._host}:{self._port}"

    def _get_json(self, path: str) -> Any:
        """Kafka Connect REST API に GET リクエストを送信して JSON を返す。"""
        # リクエスト URL を構築する
        url = f"{self._base_url}{path}"
        # HTTP GET リクエストを送信する
        try:
            # リクエストを生成する
            req = urllib.request.Request(url)
            # Content-Type ヘッダーを設定する
            req.add_header("Accept", "application/json")
            # レスポンスを取得する
            with urllib.request.urlopen(req, timeout=self._timeout) as resp:
                # レスポンスを JSON デコードして返す
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            # HTTP エラーをログに記録する
            logger.error("Kafka Connect API エラー: %s (%s)", url, exc)
            # None を返す
            return None
        except urllib.error.URLError as exc:
            # 接続エラーをログに記録する
            logger.error("Kafka Connect 接続エラー: %s (%s)", url, exc)
            # None を返す
            return None

    def list_connectors(self) -> List[str]:
        """Kafka Connect に登録されている全コネクタ名のリストを返す。"""
        # コネクタ一覧 API にアクセスする
        result = self._get_json("/connectors")
        # 結果がリストでない場合は空リストを返す
        if not isinstance(result, list):
            # 空リストを返す
            return []
        # コネクタ名のリストを返す
        return result

    def get_connector_status(self, connector_name: str) -> Optional[ConnectorStatus]:
        """指定したコネクタの状態を取得して ConnectorStatus を返す。

        Args:
            connector_name: 状態を確認するコネクタ名

        Returns:
            ConnectorStatus インスタンス、取得失敗時は None
        """
        # コネクタ状態 API にアクセスする
        status_data = self._get_json(f"/connectors/{connector_name}/status")
        # 取得失敗の場合は None を返す
        if not status_data:
            # None を返す
            return None
        # コネクタ状態を取得する
        connector_info = status_data.get("connector", {})
        # コネクタの状態を取得する
        state = connector_info.get("state", "UNKNOWN")
        # ワーカー ID を取得する
        worker_id = connector_info.get("worker_id", "")
        # タスク状態を取得する
        tasks_data = status_data.get("tasks", [])
        # タスク ID → 状態の辞書を構築する
        task_states: Dict[int, str] = {}
        # 各タスクをイテレートして状態を取得する
        for task in tasks_data:
            # タスク ID を取得する
            task_id = int(task.get("id", 0))
            # タスクの状態を取得する
            task_state = task.get("state", "UNKNOWN")
            # タスク状態を辞書に追加する
            task_states[task_id] = task_state
        # エラートレースを取得する（FAILED 状態の場合）
        error_trace = ""
        # コネクタが FAILED 状態の場合はエラートレースを取得する
        if state == "FAILED":
            # エラートレースを取得する
            error_trace = connector_info.get("trace", "")
        # ConnectorStatus を生成して返す
        return ConnectorStatus(
            connector_name=connector_name,
            state=state,
            worker_id=worker_id,
            task_states=task_states,
            error_trace=error_trace,
        )

    def check_all_connectors(self) -> List[ConnectorStatus]:
        """全コネクタの状態を確認してリストで返す。"""
        # コネクタ一覧を取得する
        connector_names = self.list_connectors()
        # 状態リストを初期化する
        statuses: List[ConnectorStatus] = []
        # 各コネクタの状態を確認する
        for name in connector_names:
            # コネクタの状態を取得する
            status = self.get_connector_status(name)
            # 取得成功の場合はリストに追加する
            if status:
                # 状態リストに追加する
                statuses.append(status)
        # 状態リストを返す
        return statuses

    def generate_health_report(self) -> PipelineHealthReport:
        """CDC パイプライン全体の健全性レポートを生成して返す。"""
        # レポートを初期化する
        report = PipelineHealthReport()
        # 全コネクタの状態を確認する
        report.connector_statuses = self.check_all_connectors()
        # 不健全なコネクタのアラートを生成する
        for status in report.connector_statuses:
            # コネクタが不健全な場合はアラートを追加する
            if not status.is_healthy:
                # アラートメッセージを生成する
                alert = (
                    f"コネクタ不健全: {status.connector_name} "
                    f"(state={status.state})"
                )
                # アラートリストに追加する
                report.alerts.append(alert)
                # 不健全なタスクのアラートを追加する
                for tid in status.unhealthy_tasks():
                    # タスクのアラートメッセージを生成する
                    task_alert = (
                        f"タスク不健全: {status.connector_name}[{tid}] "
                        f"(state={status.task_states.get(tid, 'UNKNOWN')})"
                    )
                    # タスクアラートをリストに追加する
                    report.alerts.append(task_alert)
        # レポートの健全性をログに記録する
        logger.info("CDC パイプライン健全性レポート: %s", report.summary())
        # レポートを返す
        return report

    def restart_failed_tasks(self, connector_name: str) -> bool:
        """指定したコネクタの失敗したタスクを再起動して成否を返す。

        Args:
            connector_name: 再起動するコネクタ名

        Returns:
            再起動が成功すれば True
        """
        # コネクタの現在の状態を取得する
        status = self.get_connector_status(connector_name)
        # 状態が取得できない場合は False を返す
        if not status:
            # 状態が取得できないため False を返す
            logger.warning("コネクタ状態取得失敗: %s", connector_name)
            # False を返す
            return False
        # 失敗したタスクのリストを取得する
        failed_tasks = status.unhealthy_tasks()
        # 失敗したタスクが存在しない場合は True を返す
        if not failed_tasks:
            # 再起動不要の場合は True を返す
            logger.info("再起動が必要なタスクはありません: %s", connector_name)
            # True を返す
            return True
        # 各失敗タスクを再起動する
        all_restarted = True
        # 失敗タスクをイテレートする
        for task_id in failed_tasks:
            # タスク再起動 API を呼び出す
            restart_url = (
                f"{self._base_url}/connectors/{connector_name}/tasks/{task_id}/restart"
            )
            # POST リクエストを送信する
            try:
                # タスク再起動リクエストを送信する
                req = urllib.request.Request(restart_url, data=b"", method="POST")
                # レスポンスを取得する
                with urllib.request.urlopen(req, timeout=self._timeout) as resp:
                    # 200 台のステータスコードで成功とみなす
                    if not (200 <= resp.status < 300):
                        # 再起動失敗をログに記録する
                        logger.warning(
                            "タスク再起動失敗: %s[%d] (HTTP %d)",
                            connector_name,
                            task_id,
                            resp.status,
                        )
                        # 全体成功フラグを False に設定する
                        all_restarted = False
                    else:
                        # 再起動成功をログに記録する
                        logger.info(
                            "タスク再起動成功: %s[%d]", connector_name, task_id
                        )
            except Exception as exc:
                # 再起動エラーをログに記録する
                logger.error(
                    "タスク再起動エラー: %s[%d] (%s)",
                    connector_name,
                    task_id,
                    exc,
                )
                # 全体成功フラグを False に設定する
                all_restarted = False
        # 全体の再起動結果を返す
        return all_restarted
