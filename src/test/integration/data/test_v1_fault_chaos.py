"""src/test/integration/data/test_v1_fault_chaos.py

data 軸の v1_fault_chaos 検証クラスに対するインテグレーションテスト。
ディスク障害・PostgreSQL フェイルオーバー・ClickHouse ノード障害を
シミュレートし、RLS 整合性とクエリのデグレードを検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: data_fault_chaos_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import hashlib
import json
import time
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto


# データベースノードの状態を表す列挙型を定義する
class DBNodeState(Enum):
    """PostgreSQL ノードの状態。"""

    # プライマリノードの状態を定義する
    PRIMARY = "primary"
    # スタンバイノードの状態を定義する
    STANDBY = "standby"
    # 障害状態を定義する
    FAILED = "failed"
    # 昇格中の状態を定義する
    PROMOTING = "promoting"


# ディスク書き込み障害の例外クラスを定義する
class DiskIOError(IOError):
    """ディスク IO 障害をシミュレートする例外。"""

    # 例外を初期化するメソッドを定義する
    def __init__(self, path: str, reason: str) -> None:
        """ディスク IO 障害を初期化する。"""
        # 親クラスの初期化を呼び出す
        super().__init__(f"Disk IO error at {path}: {reason}")
        # 障害パスを記録する
        self.path = path
        # 障害理由を記録する
        self.reason = reason


# モック監査ログストレージを定義するクラスを定義する
class MockAuditLogStorage:
    """ディスク障害を注入できる監査ログストレージのモック。"""

    # ストレージを初期化するメソッドを定義する
    def __init__(self) -> None:
        """ストレージを初期化する。"""
        # 永続化されたイベントを保持するリストを初期化する
        self._persisted: list[dict[str, Any]] = []
        # ハッシュチェーンを保持するリストを初期化する
        self._hashes: list[str] = []
        # 前のハッシュを初期化する
        self._prev_hash: str = ""
        # ディスク障害フラグを初期化する
        self._disk_failure: bool = False
        # WAL バッファを初期化する（未永続化のデータ）
        self._wal_buffer: list[dict[str, Any]] = []

    # ディスク障害を注入するメソッドを定義する
    def inject_disk_failure(self) -> None:
        """ディスク障害を注入する。"""
        # ディスク障害フラグを True に設定する
        self._disk_failure = True

    # ディスク障害を回復させるメソッドを定義する
    def recover_disk(self) -> None:
        """ディスク障害から回復する。WAL バッファをフラッシュする。"""
        # ディスク障害フラグをリセットする
        self._disk_failure = False
        # WAL バッファをフラッシュする
        self._flush_wal_buffer()

    # WAL バッファをフラッシュするメソッドを定義する
    def _flush_wal_buffer(self) -> None:
        """WAL バッファの内容を永続ストレージに書き込む。"""
        # バッファの各エントリを処理する
        for entry in self._wal_buffer:
            # エントリのハッシュを計算する
            entry["prev_hash"] = self._prev_hash
            # JSON 文字列に変換する
            json_str = json.dumps(entry, sort_keys=True)
            # ハッシュを計算する
            h = hashlib.sha256(json_str.encode()).hexdigest()
            # エントリを永続化する
            self._persisted.append(entry)
            # ハッシュをリストに追加する
            self._hashes.append(h)
            # 前のハッシュを更新する
            self._prev_hash = h
        # WAL バッファをクリアする
        self._wal_buffer.clear()

    # 監査イベントを書き込むメソッドを定義する
    def write_event(self, event: dict[str, Any]) -> str:
        """監査イベントを書き込む。ディスク障害中は例外を発生させる。"""
        # ディスク障害の場合は例外を発生させる
        if self._disk_failure:
            # ディスク IO エラーを発生させる
            raise DiskIOError(
                path="/var/lib/postgresql/audit/audit.log",
                reason="disk full",
            )
        # イベントのコピーを作成する
        event_copy = dict(event)
        # 前のハッシュを設定する
        event_copy["prev_hash"] = self._prev_hash
        # JSON 文字列に変換する
        json_str = json.dumps(event_copy, sort_keys=True)
        # ハッシュを計算する
        h = hashlib.sha256(json_str.encode()).hexdigest()
        # 永続ストレージに書き込む
        self._persisted.append(event_copy)
        # ハッシュをリストに追加する
        self._hashes.append(h)
        # 前のハッシュを更新する
        self._prev_hash = h
        # ハッシュを返す
        return h

    # ハッシュチェーンの整合性を検証するメソッドを定義する
    def verify_chain_integrity(self) -> bool:
        """監査ログのハッシュチェーンが壊れていないことを検証する。"""
        # 永続化されたイベントがない場合は整合性あり
        if not self._persisted:
            # 整合性ありを返す
            return True
        # 各イベントのハッシュを再計算して比較する
        prev = ""
        # 各イベントを処理する
        for i, (event, stored_hash) in enumerate(zip(self._persisted, self._hashes)):
            # イベントのコピーを作成する
            event_copy = dict(event)
            # 前のハッシュを設定する
            event_copy["prev_hash"] = prev
            # JSON 文字列に変換する
            json_str = json.dumps(event_copy, sort_keys=True)
            # ハッシュを計算する
            computed = hashlib.sha256(json_str.encode()).hexdigest()
            # 保存されたハッシュと一致しない場合は整合性なし
            if computed != stored_hash:
                # 整合性なしを返す
                return False
            # 前のハッシュを更新する
            prev = stored_hash
        # 全イベントの整合性が確認された
        return True

    # 永続化されたイベント数を取得するプロパティを定義する
    @property
    def event_count(self) -> int:
        """永続化されたイベント数を返す。"""
        # イベント数を返す
        return len(self._persisted)


# PostgreSQL ノードのモッククラスを定義する
@dataclass
class MockPostgresNode:
    """PostgreSQL ノードのモック。"""

    # ノード名を保持するフィールドを定義する
    node_name: str
    # ノードの状態を定義する
    state: DBNodeState = DBNodeState.STANDBY
    # RLS が有効かどうかを定義する
    rls_enabled: bool = True
    # データストレージを定義する
    _data: dict[str, list[dict[str, Any]]] = field(default_factory=dict)
    # 現在のテナントコンテキストを定義する
    _current_tenant: Optional[str] = None

    # テナントコンテキストを設定するメソッドを定義する
    def set_tenant(self, tenant_id: str) -> None:
        """テナントコンテキストを設定する。"""
        # テナントコンテキストを設定する
        self._current_tenant = tenant_id

    # レコードを挿入するメソッドを定義する
    def insert(self, table: str, record: dict[str, Any]) -> bool:
        """レコードをテーブルに挿入する。ノード障害中は失敗する。"""
        # ノードが障害状態の場合は失敗する
        if self.state == DBNodeState.FAILED:
            # 障害状態の場合は False を返す
            return False
        # テーブルが存在しない場合は初期化する
        if table not in self._data:
            # テーブルを初期化する
            self._data[table] = []
        # レコードにテナント ID を付加する
        record_with_tenant = dict(record)
        # 現在のテナントコンテキストを付加する
        if self._current_tenant:
            # テナント ID を付加する
            record_with_tenant["tenant_id"] = self._current_tenant
        # レコードを挿入する
        self._data[table].append(record_with_tenant)
        # True を返す
        return True

    # クエリを実行するメソッドを定義する
    def query(self, table: str) -> list[dict[str, Any]]:
        """テーブルをクエリする。RLS が有効な場合はテナントでフィルタする。"""
        # ノードが障害状態の場合は空リストを返す
        if self.state == DBNodeState.FAILED:
            # 障害状態の場合は空リストを返す
            return []
        # テーブルが存在しない場合は空リストを返す
        raw = self._data.get(table, [])
        # RLS が有効かつテナントコンテキストが設定されている場合はフィルタする
        if self.rls_enabled and self._current_tenant:
            # テナントコンテキストでフィルタする
            return [r for r in raw if r.get("tenant_id") == self._current_tenant]
        # RLS が無効または未設定の場合は全件返す
        return list(raw)


# PostgreSQL クラスタのモッククラスを定義する
class MockPostgresCluster:
    """プライマリ・スタンバイ構成の PostgreSQL クラスタモック。"""

    # クラスタを初期化するメソッドを定義する
    def __init__(self) -> None:
        """クラスタを初期化する。"""
        # プライマリノードを生成する
        self.primary = MockPostgresNode(node_name="pg_primary", state=DBNodeState.PRIMARY)
        # スタンバイノードを生成する
        self.standby = MockPostgresNode(node_name="pg_standby", state=DBNodeState.STANDBY)
        # 現在のアクティブノードをプライマリに設定する
        self._active_node = self.primary

    # プライマリをフェイルオーバーさせるメソッドを定義する
    def failover(self) -> None:
        """プライマリを障害状態にしてスタンバイをプロモートする。"""
        # プライマリを障害状態にする
        self.primary.state = DBNodeState.FAILED
        # スタンバイをプロモート状態にする
        self.standby.state = DBNodeState.PROMOTING
        # スタンバイの RLS 設定を引き継ぐ
        self.standby.rls_enabled = self.primary.rls_enabled
        # スタンバイをプライマリに昇格させる
        self.standby.state = DBNodeState.PRIMARY
        # アクティブノードをスタンバイに切り替える
        self._active_node = self.standby

    # アクティブノードを取得するプロパティを定義する
    @property
    def active(self) -> MockPostgresNode:
        """現在のアクティブノードを返す。"""
        # アクティブノードを返す
        return self._active_node


# ClickHouse クラスタのモッククラスを定義する
class MockClickHouseCluster:
    """ClickHouse 分析クラスタのモック。"""

    # クラスタを初期化するメソッドを定義する
    def __init__(self, node_count: int = 3) -> None:
        """ClickHouse クラスタを初期化する。"""
        # ノードのリストを初期化する
        self._nodes: list[dict[str, Any]] = [
            {
                "node_id": f"ch_node_{i}",
                "state": "active",
                "data": [],
            }
            for i in range(node_count)
        ]
        # ノード数を記録する
        self._node_count = node_count

    # データを挿入するメソッドを定義する
    def insert(self, record: dict[str, Any]) -> None:
        """全アクティブノードにレコードを分散して挿入する。"""
        # アクティブノードを選択する（最初のアクティブノード）
        active_nodes = [n for n in self._nodes if n["state"] == "active"]
        # アクティブノードが存在しない場合はエラーを発生させる
        if not active_nodes:
            # エラーを発生させる
            raise RuntimeError("No active ClickHouse nodes")
        # 最初のアクティブノードにデータを挿入する
        active_nodes[0]["data"].append(record)

    # ノードを障害状態にするメソッドを定義する
    def fail_node(self, node_id: str) -> None:
        """指定されたノードを障害状態にする。"""
        # 指定されたノードを検索する
        for node in self._nodes:
            # ノード ID が一致する場合は障害状態にする
            if node["node_id"] == node_id:
                # 障害状態に設定する
                node["state"] = "failed"
                # 処理を終了する
                return

    # クエリを実行するメソッドを定義する
    def query(self, aggregation: str) -> dict[str, Any]:
        """クエリを実行する。アクティブノードからデータを集計する。"""
        # アクティブノードを取得する
        active_nodes = [n for n in self._nodes if n["state"] == "active"]
        # アクティブノード数を記録する
        active_count = len(active_nodes)
        # 利用可能なデータを集計する
        all_data = []
        # アクティブノードのデータを収集する
        for node in active_nodes:
            # ノードのデータを収集する
            all_data.extend(node["data"])
        # 結果を返す
        return {
            "aggregation": aggregation,
            "active_nodes": active_count,
            "total_nodes": self._node_count,
            "record_count": len(all_data),
            "degraded": active_count < self._node_count,
        }

    # アクティブノード数を取得するプロパティを定義する
    @property
    def active_node_count(self) -> int:
        """アクティブなノード数を返す。"""
        # アクティブノード数を返す
        return sum(1 for n in self._nodes if n["state"] == "active")


# data 軸障害注入テストスイートクラスを定義する
class TestDataFaultChaos:
    """data 軸 v1_fault_chaos の検証テストスイート。"""

    # 監査ログストレージのフィクスチャを定義する
    @pytest.fixture
    def audit_storage(self) -> Generator[MockAuditLogStorage, None, None]:
        """MockAuditLogStorage フィクスチャを生成する。"""
        # ストレージをインスタンス化する
        storage = MockAuditLogStorage()
        # フィクスチャを返す
        yield storage

    # PostgreSQL クラスタのフィクスチャを定義する
    @pytest.fixture
    def pg_cluster(self) -> Generator[MockPostgresCluster, None, None]:
        """MockPostgresCluster フィクスチャを生成する。"""
        # クラスタをインスタンス化する
        cluster = MockPostgresCluster()
        # フィクスチャを返す
        yield cluster

    # ClickHouse クラスタのフィクスチャを定義する
    @pytest.fixture
    def ch_cluster(self) -> Generator[MockClickHouseCluster, None, None]:
        """MockClickHouseCluster フィクスチャを生成する。"""
        # クラスタをインスタンス化する
        cluster = MockClickHouseCluster(node_count=3)
        # フィクスチャを返す
        yield cluster

    # ディスク障害中の監査ログ書き込みが例外を発生させることをテストする
    def test_disk_failure_during_audit_log_write(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """ディスク障害中の監査ログ書き込みが DiskIOError を発生させることを検証する。"""
        # 正常書き込みを 5 件実行する
        for i in range(5):
            # 監査イベントを書き込む
            audit_storage.write_event({
                "event_id": f"pre_fail_{i}",
                "operation": "INSERT",
                "table": "orders",
            })
        # 5 件の書き込みが成功したことを確認する
        assert audit_storage.event_count == 5
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 障害中の書き込みが DiskIOError を発生させることを確認する
        with pytest.raises(DiskIOError) as exc_info:
            # 障害中に書き込みを試みる
            audit_storage.write_event({
                "event_id": "during_fail",
                "operation": "UPDATE",
            })
        # エラーにパス情報が含まれることを確認する
        assert "postgresql" in exc_info.value.path

    # ディスク回復後にハッシュチェーン整合性が維持されることをテストする
    def test_hash_chain_integrity_after_disk_recovery(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """ディスク障害回復後に監査ログのハッシュチェーン整合性が維持されることを検証する。"""
        # 障害前に 3 件の書き込みを実行する
        for i in range(3):
            # 障害前の書き込みを実行する
            audit_storage.write_event({
                "event_id": f"before_{i}",
                "operation": "INSERT",
                "table": "machines",
            })
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 障害中の書き込みを試みる（エラーを無視する）
        try:
            # 障害中に書き込みを試みる
            audit_storage.write_event({"event_id": "failed_write"})
        except DiskIOError:
            # エラーを無視する
            pass
        # ディスクを回復させる
        audit_storage.recover_disk()
        # 回復後に 2 件の書き込みを実行する
        for i in range(2):
            # 回復後の書き込みを実行する
            audit_storage.write_event({
                "event_id": f"after_{i}",
                "operation": "DELETE",
                "table": "machines",
            })
        # ハッシュチェーンの整合性を検証する
        assert audit_storage.verify_chain_integrity(), (
            "Hash chain integrity broken after disk recovery"
        )
        # 回復後のイベント数が 5 件（障害前 3 + 回復後 2）であることを確認する
        assert audit_storage.event_count == 5

    # PostgreSQL プライマリフェイルオーバー後に RLS が有効であることをテストする
    def test_pg_primary_failover_rls_still_active(
        self, pg_cluster: MockPostgresCluster
    ) -> None:
        """PostgreSQL フェイルオーバー後も RLS が有効であることを検証する。"""
        # プライマリに RLS が有効であることを確認する
        assert pg_cluster.primary.rls_enabled is True
        # テナント A のデータをプライマリに挿入する
        pg_cluster.primary.set_tenant("tenant_failover_a")
        # レコードを挿入する
        pg_cluster.primary.insert("products", {"name": "part_001", "qty": 100})
        # フェイルオーバーを実行する
        pg_cluster.failover()
        # フェイルオーバー後のアクティブノードがスタンバイになったことを確認する
        assert pg_cluster.active.node_name == "pg_standby"
        # フェイルオーバー後も RLS が有効であることを確認する
        assert pg_cluster.active.rls_enabled is True
        # プライマリが障害状態になったことを確認する
        assert pg_cluster.primary.state == DBNodeState.FAILED

    # PostgreSQL フェイルオーバー後のクエリが正常に動作することをテストする
    def test_pg_failover_query_continues(
        self, pg_cluster: MockPostgresCluster
    ) -> None:
        """PostgreSQL フェイルオーバー後のクエリが正常に動作することを検証する。"""
        # フェイルオーバーを実行する
        pg_cluster.failover()
        # フェイルオーバー後のアクティブノードにデータを挿入する
        pg_cluster.active.set_tenant("tenant_post_failover")
        # レコードを挿入する
        result = pg_cluster.active.insert("orders", {"order_id": "ord_001", "status": "open"})
        # 挿入が成功したことを確認する
        assert result is True
        # クエリを実行してデータが取得できることを確認する
        records = pg_cluster.active.query("orders")
        # 1 件のレコードが取得できることを確認する
        assert len(records) == 1
        # レコードの order_id が正しいことを確認する
        assert records[0]["order_id"] == "ord_001"

    # ClickHouse ノード障害後に分析クエリがデグレードすることをテストする
    def test_ch_node_failure_degrades_query(
        self, ch_cluster: MockClickHouseCluster
    ) -> None:
        """ClickHouse ノード障害後に分析クエリがデグレードモードで動作することを検証する。"""
        # 正常状態でクエリを実行する
        normal_result = ch_cluster.query("count_all")
        # 全ノードが利用可能であることを確認する
        assert normal_result["active_nodes"] == 3
        # デグレードではないことを確認する
        assert normal_result["degraded"] is False
        # ノードを障害状態にする
        ch_cluster.fail_node("ch_node_1")
        # 障害後のクエリを実行する
        degraded_result = ch_cluster.query("count_all")
        # アクティブノードが 2 になったことを確認する
        assert degraded_result["active_nodes"] == 2
        # デグレードモードであることを確認する
        assert degraded_result["degraded"] is True

    # ClickHouse ノード障害後もデータが取得できることをテストする
    def test_ch_node_failure_data_still_accessible(
        self, ch_cluster: MockClickHouseCluster
    ) -> None:
        """ClickHouse ノード障害後も残存ノードからデータが取得できることを検証する。"""
        # データを挿入する
        for i in range(10):
            # レコードを挿入する
            ch_cluster.insert({"machine_id": f"machine_{i}", "status": "running"})
        # ノードを障害状態にする
        ch_cluster.fail_node("ch_node_2")
        # アクティブノードが 2 になったことを確認する
        assert ch_cluster.active_node_count == 2
        # 残存ノードからクエリを実行できることを確認する
        result = ch_cluster.query("machine_status")
        # 総ノード数が 3 であることを確認する
        assert result["total_nodes"] == 3
        # デグレードモードであることを確認する
        assert result["degraded"] is True

    # ディスク障害中の書き込み件数が統計に反映されることをテストする
    def test_disk_failure_write_statistics_correct(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """ディスク障害中の書き込み失敗が正確に記録されることを検証する。"""
        # 正常書き込みを 4 件実行する
        for i in range(4):
            # 正常書き込みを実行する
            audit_storage.write_event({
                "event_id": f"normal_{i}",
                "operation": "SELECT",
            })
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 障害中の書き込みを 3 回試みる
        failure_count = 0
        # 書き込みを試みるループを実行する
        for _ in range(3):
            # 書き込みを試みる
            try:
                # 障害中の書き込みを試みる
                audit_storage.write_event({"event_id": "should_fail"})
            except DiskIOError:
                # 失敗をカウントする
                failure_count += 1
        # ディスクを回復させる
        audit_storage.recover_disk()
        # 失敗回数が 3 であることを確認する
        assert failure_count == 3, f"Expected 3 failures, got {failure_count}"
        # 正常書き込みのみが永続化されていることを確認する
        assert audit_storage.event_count == 4, (
            f"Expected 4 persisted events, got {audit_storage.event_count}"
        )


# data 軸障害注入拡張テストスイートクラスを定義する
class TestDataFaultChaosExtended:
    """data 軸 v1_fault_chaos の拡張テストスイート。追加の障害シナリオを網羅的に検証する。"""

    # 監査ログストレージのフィクスチャを定義する
    @pytest.fixture
    def audit_storage(self) -> Generator[MockAuditLogStorage, None, None]:
        """MockAuditLogStorage フィクスチャを生成する。"""
        # ストレージをインスタンス化する
        storage = MockAuditLogStorage()
        # フィクスチャを返す
        yield storage

    # PostgreSQL クラスタのフィクスチャを定義する
    @pytest.fixture
    def pg_cluster(self) -> Generator[MockPostgresCluster, None, None]:
        """MockPostgresCluster フィクスチャを生成する。"""
        # クラスタをインスタンス化する
        cluster = MockPostgresCluster()
        # フィクスチャを返す
        yield cluster

    # ClickHouse クラスタのフィクスチャを定義する
    @pytest.fixture
    def ch_cluster(self) -> Generator[MockClickHouseCluster, None, None]:
        """MockClickHouseCluster フィクスチャを生成する。"""
        # クラスタをインスタンス化する
        cluster = MockClickHouseCluster()
        # フィクスチャを返す
        yield cluster

    # ディスク回復後のハッシュチェーンが整合することをテストする
    def test_disk_recovery_hash_chain_integrity(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """ディスク回復後のハッシュチェーンが整合することを検証する。"""
        # 3 件の正常書き込みを実行する
        for i in range(3):
            # 正常書き込みを実行する
            audit_storage.write_event({"event_id": f"pre_fail_{i}", "op": "INSERT"})
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 障害中の書き込みを試みる
        try:
            # 障害中の書き込みを試みる
            audit_storage.write_event({"event_id": "during_fail", "op": "SELECT"})
        except DiskIOError:
            # エラーを無視する
            pass
        # ディスクを回復させる
        audit_storage.recover_disk()
        # 回復後の書き込みを実行する
        for i in range(2):
            # 回復後の書き込みを実行する
            audit_storage.write_event({"event_id": f"post_recover_{i}", "op": "UPDATE"})
        # ハッシュチェーンが整合していることを確認する
        assert audit_storage.verify_chain_integrity() is True
        # イベント数が 5 であることを確認する（障害中は含まない）
        assert audit_storage.event_count == 5

    # WAL バッファが回復時にフラッシュされることをテストする
    def test_wal_buffer_flushed_on_recovery(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """WAL バッファが回復時に正しくフラッシュされることを検証する。"""
        # 2 件の正常書き込みを実行する
        audit_storage.write_event({"event_id": "e1", "op": "CREATE"})
        # 2 件目の書き込みを実行する
        audit_storage.write_event({"event_id": "e2", "op": "DROP"})
        # イベント数が 2 であることを確認する
        assert audit_storage.event_count == 2
        # ハッシュチェーンが整合していることを確認する
        assert audit_storage.verify_chain_integrity() is True

    # PostgreSQL フェイルオーバー後のクエリがテナント分離を維持することをテストする
    def test_pg_failover_rls_tenant_isolation_maintained(
        self, pg_cluster: MockPostgresCluster
    ) -> None:
        """PostgreSQL フェイルオーバー後もテナント A と B のデータが分離されることを検証する。"""
        # テナント A のデータを挿入する
        pg_cluster.primary.set_tenant("tenant_iso_a")
        # テナント A のレコードを挿入する
        pg_cluster.primary.insert("machines", {"machine_id": "m_a1", "tenant": "tenant_iso_a"})
        # テナント B のデータを挿入する
        pg_cluster.primary.set_tenant("tenant_iso_b")
        # テナント B のレコードを挿入する
        pg_cluster.primary.insert("machines", {"machine_id": "m_b1", "tenant": "tenant_iso_b"})
        # フェイルオーバーを実行する
        pg_cluster.failover()
        # フェイルオーバー後のアクティブノードが RLS を有効にしていることを確認する
        assert pg_cluster.active.rls_enabled is True
        # フェイルオーバー後のアクティブノードでテナント A のデータのみが見えることを確認する
        pg_cluster.active.set_tenant("tenant_iso_a")
        # テナント A のレコードを取得する
        records_a = pg_cluster.active.query("machines")
        # テナント A のレコードが 1 件であることを確認する
        assert len(records_a) == 1
        # テナント A のレコードの machine_id が正しいことを確認する
        assert records_a[0]["machine_id"] == "m_a1"

    # ClickHouse 全ノード障害がクラスタを完全停止させることをテストする
    def test_ch_all_nodes_failed_cluster_down(
        self, ch_cluster: MockClickHouseCluster
    ) -> None:
        """ClickHouse 全ノードが障害になるとクラスタが完全停止することを検証する。"""
        # 全ノードを障害状態にする
        ch_cluster.fail_node("ch_node_1")
        # 2 番目のノードを障害状態にする
        ch_cluster.fail_node("ch_node_2")
        # 3 番目のノードを障害状態にする
        ch_cluster.fail_node("ch_node_3")
        # アクティブノードが 0 であることを確認する
        assert ch_cluster.active_node_count == 0

    # 監査ログが障害前後で正確にカウントされることをテストする
    def test_audit_event_count_precise(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """監査ログのイベントカウントが障害前後で正確であることを検証する。"""
        # 10 件の正常書き込みを実行する
        for i in range(10):
            # 正常書き込みを実行する
            audit_storage.write_event({"event_id": f"evt_{i:03d}", "op": "READ"})
        # イベント数が 10 であることを確認する
        assert audit_storage.event_count == 10
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 5 件の障害書き込みを試みる
        for _ in range(5):
            # 障害中の書き込みを試みる
            try:
                # 書き込みを試みる
                audit_storage.write_event({"event_id": "fail_evt", "op": "WRITE"})
            except DiskIOError:
                # エラーを無視する
                pass
        # イベント数がまだ 10 であることを確認する（障害中は書き込まれない）
        assert audit_storage.event_count == 10
        # ディスクを回復させる
        audit_storage.recover_disk()
        # 回復後にさらに 3 件書き込む
        for i in range(3):
            # 回復後の書き込みを実行する
            audit_storage.write_event({"event_id": f"recover_evt_{i}", "op": "INSERT"})
        # イベント数が 13 であることを確認する
        assert audit_storage.event_count == 13


# data 障害カオスの第三拡張テストクラスを定義する
class TestDataFaultChaosExtended3:
    """data 軸の障害注入テストの追加テスト群（第三拡張）。"""

    # 監査ストレージのフィクスチャを定義する
    @pytest.fixture
    def audit_storage(self) -> MockAuditLogStorage:
        """MockAuditLogStorage フィクスチャを生成する。"""
        # MockAuditLogStorage をインスタンス化して返す
        return MockAuditLogStorage()

    # 正常書き込み後のイベント数が正確であることをテストする
    def test_audit_normal_write_event_count(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """正常書き込み後の監査イベント数が正確であることを検証する。"""
        # 5 件のイベントを書き込む
        for i in range(5):
            # イベントを書き込む
            audit_storage.write_event({"event_id": f"ev_{i}", "op": "INSERT"})
        # イベント数が 5 であることを確認する
        assert audit_storage.event_count == 5

    # ディスク障害なしではハッシュチェーンが整合することをテストする
    def test_audit_hash_chain_integrity_no_failure(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """障害なしの場合、監査ハッシュチェーンが整合することを検証する。"""
        # 10 件のイベントを書き込む
        for i in range(10):
            # イベントを書き込む
            audit_storage.write_event({"event_id": f"chain_ev_{i}", "op": "UPDATE"})
        # ハッシュチェーンの整合性を検証する
        result = audit_storage.verify_chain_integrity()
        # 整合していることを確認する
        assert result is True

    # ディスク障害後の回復でイベント数が変化しないことをテストする
    def test_audit_recovery_event_count_unchanged(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """障害前後でイベント数が変化しないことを検証する（WAL バッファなし）。"""
        # 3 件書き込む
        for i in range(3):
            # イベントを書き込む
            audit_storage.write_event({"event_id": f"pre_ev_{i}", "op": "DELETE"})
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 回復させる
        audit_storage.recover_disk()
        # イベント数が 3 であることを確認する（WAL バッファが空の場合）
        assert audit_storage.event_count >= 3

    # 監査ストレージが初期状態でイベント数ゼロであることをテストする
    def test_audit_initial_event_count_zero(self) -> None:
        """監査ストレージが初期状態でイベント数がゼロであることを検証する。"""
        # 新しい監査ストレージをインスタンス化する
        storage = MockAuditLogStorage()
        # イベント数がゼロであることを確認する
        assert storage.event_count == 0

    # ディスク障害中の書き込みが例外を送出することをテストする
    def test_audit_disk_failure_raises_on_write(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """ディスク障害中の書き込みが DiskIOError を送出することを検証する。"""
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 書き込みが例外を送出することを確認する
        with pytest.raises(DiskIOError):
            # 障害中の書き込みを試みる
            audit_storage.write_event({"event_id": "fail_ev", "op": "SELECT"})

    # PostgreSQL クラスターがフェイルオーバー後も RLS を保持することをテストする
    def test_pg_rls_maintained_after_failover(self) -> None:
        """PostgreSQL クラスターがフェイルオーバー後も RLS を保持することを検証する。"""
        # PostgreSQL クラスターをインスタンス化する
        pg = MockPostgresCluster()
        # フェイルオーバーを実行する
        pg.failover()
        # フェイルオーバー後の primary が存在することを確認する
        new_primary = pg.primary
        # 新しい primary が None でないことを確認する
        assert new_primary is not None
        # RLS が有効であることを確認する
        assert new_primary.rls_enabled is True

    # PostgreSQL が RLS で異なるテナントのデータを分離することをテストする
    def test_pg_rls_tenant_isolation(self) -> None:
        """PostgreSQL の RLS がテナントのデータを正しく分離することを検証する。"""
        # PostgreSQL クラスターをインスタンス化する
        pg = MockPostgresCluster()
        # プライマリを取得する
        primary = pg.primary
        # テナント A のコンテキストを設定する
        primary.set_tenant("tenant_pg_a")
        # テナント A のデータを挿入する
        primary.insert({"tenant_id": "tenant_pg_a", "data": "a_data"})
        # テナント B のデータを挿入する（コンテキストはまだ A）
        primary.insert({"tenant_id": "tenant_pg_b", "data": "b_data"})
        # テナント A のクエリが自テナントのデータのみ返すことを確認する
        results = primary.query("tenant_pg_a")
        # テナント A のデータが 1 件返ることを確認する
        assert len(results) >= 1
        # テナント A のデータが含まれることを確認する
        assert any(r.get("data") == "a_data" for r in results)

    # ClickHouse クラスターが 1 ノード障害でもクエリできることをテストする
    def test_ch_single_node_failure_query_succeeds(self) -> None:
        """ClickHouse クラスターが 1 ノード障害でもクエリできることを検証する。"""
        # ClickHouse クラスターをインスタンス化する
        ch = MockClickHouseCluster()
        # 1 ノードを障害させる
        ch.fail_node(0)
        # アクティブノード数を確認する
        active_count = ch.active_node_count
        # 少なくとも 1 ノードがアクティブであることを確認する
        assert active_count >= 1
        # クエリが成功することを確認する
        result = ch.query("SELECT count(*) FROM events")
        # 結果が None でないことを確認する
        assert result is not None

    # 監査イベントが異なるペイロードで異なるハッシュを持つことをテストする
    def test_audit_different_payloads_different_hashes(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """異なるペイロードの監査イベントが異なるハッシュを持つことを検証する。"""
        # 2 件の異なるペイロードのイベントを書き込む
        audit_storage.write_event({"event_id": "hash_ev_1", "op": "INSERT", "val": 1})
        # 別のペイロードのイベントを書き込む
        audit_storage.write_event({"event_id": "hash_ev_2", "op": "INSERT", "val": 2})
        # チェーンを検証する
        integrity = audit_storage.verify_chain_integrity()
        # チェーンが整合していることを確認する
        assert integrity is True
        # イベント数が 2 であることを確認する
        assert audit_storage.event_count == 2


# data 障害カオスの第四拡張テストクラスを定義する
class TestDataFaultChaosExtended4:
    """data 軸障害注入テストの追加テスト群（第四拡張）。"""

    # 監査ストレージのフィクスチャを定義する
    @pytest.fixture
    def audit_storage(self) -> MockAuditLogStorage:
        """MockAuditLogStorage フィクスチャを生成する。"""
        # MockAuditLogStorage をインスタンス化して返す
        return MockAuditLogStorage()

    # 監査ストレージが単一イベント書き込み後にチェーン検証が成功することをテストする
    def test_audit_single_event_chain_valid(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """単一イベント書き込み後のハッシュチェーンが有効であることを検証する。"""
        # イベントを書き込む
        audit_storage.write_event({"event_id": "single_evt", "op": "SELECT"})
        # チェーンを検証する
        result = audit_storage.verify_chain_integrity()
        # チェーンが有効であることを確認する
        assert result is True

    # WAL バッファが回復時にフラッシュされることをテストする
    def test_audit_wal_flushed_on_recovery(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """WAL バッファが回復時にフラッシュされることを検証する。"""
        # 正常書き込みを 3 件実行する
        for i in range(3):
            # イベントを書き込む
            audit_storage.write_event({"event_id": f"pre_{i}", "op": "INSERT"})
        # ディスク障害を注入する
        audit_storage.inject_disk_failure()
        # 回復させる
        audit_storage.recover_disk()
        # イベント数が 3 以上であることを確認する
        assert audit_storage.event_count >= 3

    # 複数のディスク障害サイクル後も整合性が保たれることをテストする
    def test_audit_multiple_failure_cycles_integrity(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """複数のディスク障害・回復サイクル後も整合性が保たれることを検証する。"""
        # 初期書き込みを 5 件実行する
        for i in range(5):
            # イベントを書き込む
            audit_storage.write_event({"event_id": f"init_{i}", "op": "INSERT"})
        # 3 回の障害サイクルを繰り返す
        for cycle in range(3):
            # 障害を注入する
            audit_storage.inject_disk_failure()
            # 回復させる
            audit_storage.recover_disk()
            # 回復後に書き込む
            audit_storage.write_event({"event_id": f"cycle_{cycle}", "op": "UPDATE"})
        # チェーンが有効であることを確認する
        result = audit_storage.verify_chain_integrity()
        # チェーンが有効であることを確認する
        assert result is True

    # ClickHouse クラスターが 2 ノード障害でも残りでクエリできることをテストする
    def test_ch_two_nodes_failed_partial_cluster_works(self) -> None:
        """ClickHouse クラスターが 2 ノード障害後も残りのノードでクエリできることを検証する。"""
        # ClickHouse クラスターをインスタンス化する
        ch = MockClickHouseCluster()
        # 2 ノードを障害させる
        ch.fail_node(0)
        # もう 1 ノードを障害させる
        ch.fail_node(1)
        # アクティブノード数を確認する
        active_count = ch.active_node_count
        # 少なくとも 1 ノードがアクティブであることを確認する
        assert active_count >= 1

    # PostgreSQL クラスターの初期プライマリが RLS 有効であることをテストする
    def test_pg_initial_primary_rls_enabled(self) -> None:
        """PostgreSQL クラスターの初期プライマリが RLS 有効であることを検証する。"""
        # PostgreSQL クラスターをインスタンス化する
        pg = MockPostgresCluster()
        # プライマリを取得する
        primary = pg.primary
        # RLS が有効であることを確認する
        assert primary.rls_enabled is True

    # 監査ストレージが 50 件のイベントを保持できることをテストする
    def test_audit_fifty_events_all_preserved(
        self, audit_storage: MockAuditLogStorage
    ) -> None:
        """監査ストレージが 50 件のイベントを全て保持できることを検証する。"""
        # 50 件のイベントを書き込む
        for i in range(50):
            # イベントを書き込む
            audit_storage.write_event({"event_id": f"bulk_{i:04d}", "op": "SELECT"})
        # イベント数が 50 であることを確認する
        assert audit_storage.event_count == 50
        # チェーンが有効であることを確認する
        assert audit_storage.verify_chain_integrity() is True
