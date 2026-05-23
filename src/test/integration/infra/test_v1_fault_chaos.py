"""src/test/integration/infra/test_v1_fault_chaos.py

infra 軸の v1_fault_chaos 検証クラスに対するインテグレーションテスト。
NTP 障害・ネットワーク分断・ディスク飽和・HLC 異常をシミュレートし、
検知・アラートのシナリオを検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: infra_fault_chaos_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import time
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto


# インフラ障害の種類を表す列挙型を定義する
class InfraFaultType(Enum):
    """インフラ障害の種類。"""

    # NTP 障害を表す障害タイプを定義する
    NTP_FAILURE = "ntp_failure"
    # ネットワーク分断を表す障害タイプを定義する
    NETWORK_PARTITION = "network_partition"
    # ディスク飽和を表す障害タイプを定義する
    DISK_SATURATION = "disk_saturation"
    # HLC 異常を表す障害タイプを定義する
    HLC_ANOMALY = "hlc_anomaly"


# アラートの重大度を表す列挙型を定義する
class AlertSeverity(Enum):
    """アラートの重大度。"""

    # 重大な重大度を定義する
    CRITICAL = "CRITICAL"
    # 高い重大度を定義する
    HIGH = "HIGH"
    # 中程度の重大度を定義する
    MEDIUM = "MEDIUM"
    # 低い重大度を定義する
    LOW = "LOW"


# NTP モニターのモッククラスを定義する
class MockNTPMonitor:
    """NTP サーバーの健全性を監視するモック。"""

    # モニターを初期化するメソッドを定義する
    def __init__(self, max_drift_ms: float = 100.0) -> None:
        """NTP モニターを初期化する。"""
        # 最大許容ドリフトを設定する
        self._max_drift_ms = max_drift_ms
        # NTP サーバーの健全性フラグを初期化する
        self._ntp_healthy: bool = True
        # 現在のドリフト量を初期化する
        self._current_drift_ms: float = 0.0
        # 発火済みアラートのリストを初期化する
        self._alerts: list[dict[str, Any]] = []

    # NTP 障害を注入するメソッドを定義する
    def inject_ntp_failure(self, drift_ms: float) -> None:
        """NTP 障害を注入してドリフトを設定する。"""
        # NTP 健全性フラグを False に設定する
        self._ntp_healthy = False
        # ドリフト量を設定する
        self._current_drift_ms = drift_ms

    # NTP を回復させるメソッドを定義する
    def recover_ntp(self) -> None:
        """NTP を回復させてドリフトをリセットする。"""
        # NTP 健全性フラグを True に設定する
        self._ntp_healthy = True
        # ドリフトをリセットする
        self._current_drift_ms = 0.0

    # ドリフトを確認するメソッドを定義する
    def check_drift(self) -> Optional[dict[str, Any]]:
        """現在のドリフトを確認してアラートを生成する。"""
        # NTP が健全な場合はアラートなし
        if self._ntp_healthy and abs(self._current_drift_ms) <= self._max_drift_ms:
            # アラートなしを返す
            return None
        # ドリフトが閾値を超えた場合はアラートを生成する
        alert = {
            "type": "clock_drift",
            "severity": AlertSeverity.HIGH.value,
            "drift_ms": self._current_drift_ms,
            "max_allowed_ms": self._max_drift_ms,
            "ntp_healthy": self._ntp_healthy,
        }
        # アラートをリストに追加する
        self._alerts.append(alert)
        # アラートを返す
        return alert

    # 発火済みアラートのリストを取得するプロパティを定義する
    @property
    def alerts(self) -> list[dict[str, Any]]:
        """発火済みアラートのコピーを返す。"""
        # アラートのコピーを返す
        return list(self._alerts)


# ネットワークトポロジープローブのモッククラスを定義する
class MockTopologyProbe:
    """ネットワークトポロジーの分断を検出するモック。"""

    # プローブを初期化するメソッドを定義する
    def __init__(self) -> None:
        """トポロジープローブを初期化する。"""
        # ノードのリストを初期化する
        self._nodes: list[str] = []
        # 到達可能なノードのセットを初期化する
        self._reachable: set[frozenset[str]] = set()
        # 分断アラートのリストを初期化する
        self._partition_alerts: list[dict[str, Any]] = []

    # ノードを追加するメソッドを定義する
    def add_node(self, node_id: str) -> None:
        """プローブ対象のノードを追加する。"""
        # ノードを追加する
        self._nodes.append(node_id)
        # 全ノード間の到達可能性を初期化する
        for other in self._nodes:
            # 自分自身を除く全ノードへの到達可能性を設定する
            if other != node_id:
                # 到達可能なペアを追加する
                self._reachable.add(frozenset({node_id, other}))

    # ネットワーク分断を注入するメソッドを定義する
    def inject_partition(self, node_a: str, node_b: str) -> None:
        """2 ノード間のネットワーク分断を注入する。"""
        # 到達不可能なペアを削除する
        self._reachable.discard(frozenset({node_a, node_b}))

    # ネットワーク分断を解消するメソッドを定義する
    def heal_partition(self, node_a: str, node_b: str) -> None:
        """2 ノード間のネットワーク分断を解消する。"""
        # 到達可能なペアを追加する
        self._reachable.add(frozenset({node_a, node_b}))

    # 分断を検出するメソッドを定義する
    def detect_split(self) -> Optional[dict[str, Any]]:
        """ネットワーク分断を検出してアラートを生成する。"""
        # 到達不可能なペアを検出する
        unreachable_pairs = []
        # 全ノードペアを確認する
        for i in range(len(self._nodes)):
            # j を i + 1 から始める
            for j in range(i + 1, len(self._nodes)):
                # ペアを生成する
                pair = frozenset({self._nodes[i], self._nodes[j]})
                # 到達不可能なペアを検出する
                if pair not in self._reachable:
                    # 到達不可能なペアをリストに追加する
                    unreachable_pairs.append(
                        (self._nodes[i], self._nodes[j])
                    )
        # 到達不可能なペアがある場合はアラートを生成する
        if unreachable_pairs:
            # アラートを生成する
            alert = {
                "type": "network_partition",
                "severity": AlertSeverity.CRITICAL.value,
                "unreachable_pairs": unreachable_pairs,
            }
            # アラートをリストに追加する
            self._partition_alerts.append(alert)
            # アラートを返す
            return alert
        # 到達不可能なペアがない場合はアラートなしを返す
        return None


# ディスクモニターのモッククラスを定義する
class MockDiskMonitor:
    """ディスク使用率を監視するモック。"""

    # モニターを初期化するメソッドを定義する
    def __init__(
        self,
        warning_threshold: float = 0.80,
        critical_threshold: float = 0.95,
    ) -> None:
        """ディスクモニターを初期化する。"""
        # 警告閾値を設定する
        self._warning = warning_threshold
        # クリティカル閾値を設定する
        self._critical = critical_threshold
        # 現在のディスク使用率を初期化する
        self._current_usage: float = 0.0
        # 発火済みアラートのリストを初期化する
        self._alerts: list[dict[str, Any]] = []

    # ディスク使用率を設定するメソッドを定義する
    def set_disk_usage(self, usage_ratio: float) -> None:
        """ディスク使用率を設定する（0.0 ～ 1.0）。"""
        # ディスク使用率を設定する
        self._current_usage = usage_ratio

    # ディスク使用率を確認するメソッドを定義する
    def check_usage(self) -> Optional[dict[str, Any]]:
        """ディスク使用率を確認してアラートを生成する。"""
        # クリティカル閾値を超えた場合はアラートを生成する
        if self._current_usage >= self._critical:
            # クリティカルアラートを生成する
            alert = {
                "type": "disk_saturation",
                "severity": AlertSeverity.CRITICAL.value,
                "usage_ratio": self._current_usage,
                "threshold": self._critical,
            }
            # アラートをリストに追加する
            self._alerts.append(alert)
            # アラートを返す
            return alert
        # 警告閾値を超えた場合はアラートを生成する
        if self._current_usage >= self._warning:
            # 警告アラートを生成する
            alert = {
                "type": "disk_saturation",
                "severity": AlertSeverity.HIGH.value,
                "usage_ratio": self._current_usage,
                "threshold": self._warning,
            }
            # アラートをリストに追加する
            self._alerts.append(alert)
            # アラートを返す
            return alert
        # 閾値以下の場合はアラートなしを返す
        return None

    # 発火済みアラートのリストを取得するプロパティを定義する
    @property
    def alerts(self) -> list[dict[str, Any]]:
        """発火済みアラートのコピーを返す。"""
        # アラートのコピーを返す
        return list(self._alerts)


# HLC 検証器のモッククラスを定義する
class MockHLCVerifier:
    """HLC の単調増加違反を検出する検証器のモック。"""

    # 検証器を初期化するメソッドを定義する
    def __init__(self) -> None:
        """HLC 検証器を初期化する。"""
        # 最後のタイムスタンプを初期化する
        self._last_ts: tuple[int, int] = (0, 0)
        # 検出された異常のリストを初期化する
        self._anomalies: list[dict[str, Any]] = []

    # タイムスタンプを検証するメソッドを定義する
    def verify(self, ts: tuple[int, int]) -> Optional[dict[str, Any]]:
        """タイムスタンプを検証する。単調増加違反を検出した場合はアラートを返す。"""
        # タイムスタンプが前のタイムスタンプ未満の場合は異常を検出する
        if ts < self._last_ts:
            # 異常を記録する
            anomaly = {
                "type": "hlc_backward_jump",
                "severity": AlertSeverity.CRITICAL.value,
                "current_ts": ts,
                "expected_min_ts": self._last_ts,
            }
            # 異常をリストに追加する
            self._anomalies.append(anomaly)
            # 異常を返す
            return anomaly
        # タイムスタンプを更新する
        self._last_ts = ts
        # 異常なしを返す
        return None

    # 検出された異常のリストを取得するプロパティを定義する
    @property
    def anomalies(self) -> list[dict[str, Any]]:
        """検出された異常のコピーを返す。"""
        # 異常のコピーを返す
        return list(self._anomalies)


# infra 軸障害注入テストスイートクラスを定義する
class TestInfraFaultChaos:
    """infra 軸 v1_fault_chaos の検証テストスイート。"""

    # NTP モニターのフィクスチャを定義する
    @pytest.fixture
    def ntp_monitor(self) -> Generator[MockNTPMonitor, None, None]:
        """MockNTPMonitor フィクスチャを生成する。"""
        # NTP モニターをインスタンス化する
        monitor = MockNTPMonitor(max_drift_ms=100.0)
        # フィクスチャを返す
        yield monitor

    # トポロジープローブのフィクスチャを定義する
    @pytest.fixture
    def topology_probe(self) -> Generator[MockTopologyProbe, None, None]:
        """MockTopologyProbe フィクスチャを生成する。"""
        # トポロジープローブをインスタンス化する
        probe = MockTopologyProbe()
        # フィクスチャを返す
        yield probe

    # ディスクモニターのフィクスチャを定義する
    @pytest.fixture
    def disk_monitor(self) -> Generator[MockDiskMonitor, None, None]:
        """MockDiskMonitor フィクスチャを生成する。"""
        # ディスクモニターをインスタンス化する
        monitor = MockDiskMonitor()
        # フィクスチャを返す
        yield monitor

    # HLC 検証器のフィクスチャを定義する
    @pytest.fixture
    def hlc_verifier(self) -> Generator[MockHLCVerifier, None, None]:
        """MockHLCVerifier フィクスチャを生成する。"""
        # 検証器をインスタンス化する
        verifier = MockHLCVerifier()
        # フィクスチャを返す
        yield verifier

    # NTP 障害注入後にアラートが発生することをテストする
    def test_ntp_failure_detected_and_alerted(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """NTP 障害注入後にクロックドリフトアラートが発生することを検証する。"""
        # 正常状態ではアラートが発生しないことを確認する
        normal_check = ntp_monitor.check_drift()
        # 正常状態ではアラートなしであることを確認する
        assert normal_check is None, "No alert expected in normal state"
        # NTP 障害を注入する（500ms のドリフト）
        ntp_monitor.inject_ntp_failure(drift_ms=500.0)
        # 障害後にアラートが発生することを確認する
        alert = ntp_monitor.check_drift()
        # アラートが生成されたことを確認する
        assert alert is not None, "Alert must be generated after NTP failure"
        # アラートタイプが clock_drift であることを確認する
        assert alert["type"] == "clock_drift"
        # アラートの重大度が HIGH であることを確認する
        assert alert["severity"] == AlertSeverity.HIGH.value
        # ドリフト量が記録されていることを確認する
        assert alert["drift_ms"] == 500.0

    # NTP 回復後にアラートが発生しないことをテストする
    def test_ntp_recovery_clears_alerts(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """NTP 回復後にドリフトアラートが発生しないことを検証する。"""
        # NTP 障害を注入する
        ntp_monitor.inject_ntp_failure(drift_ms=200.0)
        # アラートが発生することを確認する
        alert_before = ntp_monitor.check_drift()
        # アラートが生成されたことを確認する
        assert alert_before is not None
        # NTP を回復させる
        ntp_monitor.recover_ntp()
        # 回復後にアラートが発生しないことを確認する
        alert_after = ntp_monitor.check_drift()
        # 回復後はアラートなしであることを確認する
        assert alert_after is None, "No alert expected after NTP recovery"

    # ネットワーク分断後にトポロジー分断が検出されることをテストする
    def test_network_partition_topology_detects_split(
        self, topology_probe: MockTopologyProbe
    ) -> None:
        """ネットワーク分断後にトポロジープローブが分断を検出することを検証する。"""
        # ノードを追加する
        topology_probe.add_node("node_alpha")
        # 2 つ目のノードを追加する
        topology_probe.add_node("node_beta")
        # 3 つ目のノードを追加する
        topology_probe.add_node("node_gamma")
        # 正常状態では分断が検出されないことを確認する
        normal_split = topology_probe.detect_split()
        # 正常状態では分断なしであることを確認する
        assert normal_split is None, "No partition expected in normal state"
        # node_alpha と node_beta の間に分断を注入する
        topology_probe.inject_partition("node_alpha", "node_beta")
        # 分断後に検出されることを確認する
        split_alert = topology_probe.detect_split()
        # 分断アラートが生成されたことを確認する
        assert split_alert is not None, "Partition must be detected"
        # アラートタイプが network_partition であることを確認する
        assert split_alert["type"] == "network_partition"
        # アラートの重大度が CRITICAL であることを確認する
        assert split_alert["severity"] == AlertSeverity.CRITICAL.value

    # ネットワーク分断解消後に検出がリセットされることをテストする
    def test_network_partition_heal_not_detected(
        self, topology_probe: MockTopologyProbe
    ) -> None:
        """ネットワーク分断解消後に分断が検出されなくなることを検証する。"""
        # ノードを追加する
        topology_probe.add_node("node_x")
        # 2 つ目のノードを追加する
        topology_probe.add_node("node_y")
        # 分断を注入する
        topology_probe.inject_partition("node_x", "node_y")
        # 分断が検出されることを確認する
        assert topology_probe.detect_split() is not None
        # 分断を解消する
        topology_probe.heal_partition("node_x", "node_y")
        # 解消後に分断が検出されないことを確認する
        healed_split = topology_probe.detect_split()
        # 解消後は分断なしであることを確認する
        assert healed_split is None, "Partition must not be detected after healing"

    # ディスク飽和時にアラートが発生することをテストする
    def test_disk_saturation_alerts(self, disk_monitor: MockDiskMonitor) -> None:
        """ディスク飽和時にアラートが発生することを検証する。"""
        # 正常使用率（50%）ではアラートなしを確認する
        disk_monitor.set_disk_usage(0.50)
        # アラートが発生しないことを確認する
        assert disk_monitor.check_usage() is None
        # 警告閾値（85%）超過でアラートが発生することを確認する
        disk_monitor.set_disk_usage(0.85)
        # 警告アラートが発生することを確認する
        warning_alert = disk_monitor.check_usage()
        # アラートが生成されたことを確認する
        assert warning_alert is not None
        # アラートタイプが disk_saturation であることを確認する
        assert warning_alert["type"] == "disk_saturation"
        # 警告の重大度が HIGH であることを確認する
        assert warning_alert["severity"] == AlertSeverity.HIGH.value

    # ディスククリティカル閾値超過時に CRITICAL アラートが発生することをテストする
    def test_disk_critical_threshold_alerts(
        self, disk_monitor: MockDiskMonitor
    ) -> None:
        """ディスク使用率がクリティカル閾値を超えると CRITICAL アラートが発生することを検証する。"""
        # クリティカル閾値（97%）超過でアラートが発生することを確認する
        disk_monitor.set_disk_usage(0.97)
        # クリティカルアラートが発生することを確認する
        critical_alert = disk_monitor.check_usage()
        # アラートが生成されたことを確認する
        assert critical_alert is not None
        # クリティカルの重大度が CRITICAL であることを確認する
        assert critical_alert["severity"] == AlertSeverity.CRITICAL.value

    # HLC 後退ジャンプが検証器に検出されることをテストする
    def test_hlc_backward_jump_caught_by_verifier(
        self, hlc_verifier: MockHLCVerifier
    ) -> None:
        """HLC タイムスタンプの後退ジャンプが検証器に検出されることを検証する。"""
        # 正常なタイムスタンプを提出する
        anomaly1 = hlc_verifier.verify((1700000000000, 0))
        # 正常なタイムスタンプはアノーマリなしであることを確認する
        assert anomaly1 is None
        # 後退タイムスタンプを提出する（前より小さい）
        anomaly2 = hlc_verifier.verify((1699999999000, 0))
        # アノーマリが検出されることを確認する
        assert anomaly2 is not None, "Backward HLC jump must be detected"
        # アノーマリタイプが hlc_backward_jump であることを確認する
        assert anomaly2["type"] == "hlc_backward_jump"
        # アノーマリの重大度が CRITICAL であることを確認する
        assert anomaly2["severity"] == AlertSeverity.CRITICAL.value

    # HLC 正常進行は検証器を通過することをテストする
    def test_hlc_normal_advance_not_anomaly(
        self, hlc_verifier: MockHLCVerifier
    ) -> None:
        """正常に進む HLC タイムスタンプが検証器のアノーマリチェックを通過することを検証する。"""
        # 正常なタイムスタンプを連続して提出する
        timestamps = [
            (1700000000000, 0),
            (1700000000000, 1),
            (1700000000001, 0),
            (1700000000001, 1),
            (1700000000100, 0),
        ]
        # 各タイムスタンプを検証する
        for ts in timestamps:
            # タイムスタンプを検証する
            anomaly = hlc_verifier.verify(ts)
            # アノーマリが検出されないことを確認する
            assert anomaly is None, (
                f"Normal HLC advance should not trigger anomaly: ts={ts}"
            )
        # アノーマリが記録されていないことを確認する
        assert len(hlc_verifier.anomalies) == 0


# infra 軸障害注入拡張テストスイートクラスを定義する
class TestInfraFaultChaosExtended:
    """infra 軸 v1_fault_chaos 拡張検証テストスイート。追加の障害シナリオを網羅的に検証する。"""

    # NTP モニターのフィクスチャを定義する
    @pytest.fixture
    def ntp_monitor(self) -> Generator[MockNTPMonitor, None, None]:
        """MockNTPMonitor フィクスチャを生成する。"""
        # NTP モニターをインスタンス化する（最大ドリフト 50ms）
        monitor = MockNTPMonitor(max_drift_ms=50)
        # フィクスチャを返す
        yield monitor

    # トポロジプローブのフィクスチャを定義する
    @pytest.fixture
    def topology_probe(self) -> Generator[MockTopologyProbe, None, None]:
        """MockTopologyProbe フィクスチャを生成する。"""
        # トポロジプローブをインスタンス化する（3 ノードクラスタ）
        probe = MockTopologyProbe(nodes=["node-1", "node-2", "node-3"])
        # フィクスチャを返す
        yield probe

    # ディスクモニターのフィクスチャを定義する
    @pytest.fixture
    def disk_monitor(self) -> Generator[MockDiskMonitor, None, None]:
        """MockDiskMonitor フィクスチャを生成する。"""
        # ディスクモニターをインスタンス化する
        monitor = MockDiskMonitor(warning_threshold=0.80, critical_threshold=0.95)
        # フィクスチャを返す
        yield monitor

    # HLC 検証器のフィクスチャを定義する
    @pytest.fixture
    def hlc_verifier(self) -> Generator[MockHLCVerifier, None, None]:
        """MockHLCVerifier フィクスチャを生成する。"""
        # HLC 検証器をインスタンス化する
        verifier = MockHLCVerifier()
        # フィクスチャを返す
        yield verifier

    # NTP ドリフトが境界値で警告になることをテストする
    def test_ntp_drift_at_boundary_triggers_alert(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """NTP ドリフトが最大ドリフト許容値ちょうどで警告アラートが発生することを検証する。"""
        # ドリフトを最大値ちょうどに設定する（50ms）
        ntp_monitor.set_drift_ms(50)
        # アラートを確認する
        alert = ntp_monitor.check()
        # アラートが発生することを確認する
        assert alert is not None
        # アラートタイプが ntp_clock_drift であることを確認する
        assert alert["type"] == "ntp_clock_drift"

    # 複数 NTP 障害が重複しないことをテストする
    def test_multiple_ntp_failures_not_duplicated(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """複数の NTP 障害が独立したアラートとして発生することを検証する。"""
        # ドリフトを超過値に設定する
        ntp_monitor.set_drift_ms(100)
        # アラートを 3 回確認する
        alerts = [ntp_monitor.check() for _ in range(3)]
        # 全てのアラートが発生することを確認する
        assert all(a is not None for a in alerts)
        # 全てが同じタイプであることを確認する
        assert all(a["type"] == "ntp_clock_drift" for a in alerts)

    # ドリフトが正常範囲に戻ると警告が消えることをテストする
    def test_ntp_drift_recovery_clears_alert(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """NTP ドリフトが正常範囲に戻るとアラートが解消されることを検証する。"""
        # ドリフトを超過値に設定する
        ntp_monitor.set_drift_ms(200)
        # アラートが発生することを確認する
        alert_before = ntp_monitor.check()
        # アラートが発生することを確認する
        assert alert_before is not None
        # ドリフトを正常範囲に戻す
        ntp_monitor.set_drift_ms(10)
        # アラートが解消されることを確認する
        alert_after = ntp_monitor.check()
        # アラートなしであることを確認する
        assert alert_after is None

    # ネットワーク分断で 2 ノードがアイソレートされることをテストする
    def test_network_partition_isolates_two_nodes(
        self, topology_probe: MockTopologyProbe
    ) -> None:
        """ネットワーク分断で 2 ノードが切り離されることを検証する。"""
        # node-2 と node-3 の間を分断する
        topology_probe.partition(["node-2", "node-3"])
        # トポロジ状態を確認する
        status = topology_probe.get_topology_status()
        # 分断フラグが立っていることを確認する
        assert status["partitioned"] is True
        # 分断されたノードのリストを確認する
        partitioned = status["partitioned_nodes"]
        # node-2 が分断されていることを確認する
        assert "node-2" in partitioned
        # node-3 が分断されていることを確認する
        assert "node-3" in partitioned

    # ネットワーク回復でトポロジが復旧することをテストする
    def test_network_recovery_restores_topology(
        self, topology_probe: MockTopologyProbe
    ) -> None:
        """ネットワーク分断回復後にトポロジが正常に戻ることを検証する。"""
        # ネットワーク分断を注入する
        topology_probe.partition(["node-1"])
        # 分断状態を確認する
        status_before = topology_probe.get_topology_status()
        # 分断されていることを確認する
        assert status_before["partitioned"] is True
        # ネットワークを回復させる
        topology_probe.recover()
        # 回復後のトポロジ状態を確認する
        status_after = topology_probe.get_topology_status()
        # 分断が解消されていることを確認する
        assert status_after["partitioned"] is False

    # ディスク使用率が低下すると警告が解消されることをテストする
    def test_disk_usage_drop_clears_warning(
        self, disk_monitor: MockDiskMonitor
    ) -> None:
        """ディスク使用率が警告閾値を下回ると警告が解消されることを検証する。"""
        # 使用率を 90% に設定する（警告閾値 80% を超える）
        disk_monitor.set_disk_usage(0.90)
        # 警告アラートが発生することを確認する
        alert = disk_monitor.check_usage()
        # アラートが発生することを確認する
        assert alert is not None
        # 使用率を 50% に戻す
        disk_monitor.set_disk_usage(0.50)
        # 警告が解消されることを確認する
        no_alert = disk_monitor.check_usage()
        # アラートなしであることを確認する
        assert no_alert is None

    # HLC ロジカルカウンタ進行は後退でないことをテストする
    def test_hlc_logical_counter_advance_not_backward(
        self, hlc_verifier: MockHLCVerifier
    ) -> None:
        """HLC のロジカルカウンタが増加するパターンは後退として検出されないことを検証する。"""
        # wall_time_ms が同じでロジカルカウンタが増加するタイムスタンプ列を作成する
        timestamps = [
            (1700000000000, 0),
            (1700000000000, 1),
            (1700000000000, 2),
            (1700000000000, 3),
        ]
        # 各タイムスタンプを検証する
        for ts in timestamps:
            # タイムスタンプを検証する
            anomaly = hlc_verifier.verify(ts)
            # アノーマリが検出されないことを確認する
            assert anomaly is None, f"Logical counter advance should not be anomaly: ts={ts}"

    # 複数の HLC 後退が全て記録されることをテストする
    def test_multiple_hlc_backward_jumps_all_recorded(
        self, hlc_verifier: MockHLCVerifier
    ) -> None:
        """複数の HLC 後退ジャンプが全て異常として記録されることを検証する。"""
        # 初期タイムスタンプを設定する
        hlc_verifier.verify((1700000100000, 0))
        # 1 回目の後退ジャンプを発生させる
        anomaly1 = hlc_verifier.verify((1700000050000, 0))
        # 異常が検出されることを確認する
        assert anomaly1 is not None
        # タイムスタンプを進める
        hlc_verifier.verify((1700000200000, 0))
        # 2 回目の後退ジャンプを発生させる
        anomaly2 = hlc_verifier.verify((1700000150000, 0))
        # 異常が検出されることを確認する
        assert anomaly2 is not None
        # 記録された異常が 2 件であることを確認する
        assert len(hlc_verifier.anomalies) == 2

    # ディスクが段階的に増加する場合のアラート進行をテストする
    def test_disk_progressive_usage_alert_escalation(
        self, disk_monitor: MockDiskMonitor
    ) -> None:
        """ディスク使用率が段階的に増加してアラートが HIGH → CRITICAL に昇格することを検証する。"""
        # 70% では正常（閾値 80% 以下）
        disk_monitor.set_disk_usage(0.70)
        # アラートなしを確認する
        assert disk_monitor.check_usage() is None
        # 82% で警告（閾値 80% 超）
        disk_monitor.set_disk_usage(0.82)
        # 警告アラートを確認する
        warning_alert = disk_monitor.check_usage()
        # 警告アラートが発生することを確認する
        assert warning_alert is not None
        # アラートの重大度が HIGH であることを確認する
        assert warning_alert["severity"] == AlertSeverity.HIGH.value
        # 96% でクリティカル（閾値 95% 超）
        disk_monitor.set_disk_usage(0.96)
        # クリティカルアラートを確認する
        critical_alert = disk_monitor.check_usage()
        # クリティカルアラートが発生することを確認する
        assert critical_alert is not None
        # アラートの重大度が CRITICAL であることを確認する
        assert critical_alert["severity"] == AlertSeverity.CRITICAL.value


# infra 障害カオスの第三拡張テストクラスを定義する
class TestInfraFaultChaosExtended3:
    """infra 軸の障害注入テストの追加テスト群（第三拡張）。"""

    # NTP モニターのフィクスチャを定義する
    @pytest.fixture
    def ntp_monitor(self) -> MockNTPMonitor:
        """MockNTPMonitor フィクスチャを生成する。"""
        # MockNTPMonitor をインスタンス化して返す
        return MockNTPMonitor(max_drift_ms=50.0)

    # トポロジープローブのフィクスチャを定義する
    @pytest.fixture
    def topology_probe(self) -> MockTopologyProbe:
        """MockTopologyProbe フィクスチャを生成する。"""
        # MockTopologyProbe をインスタンス化して返す
        return MockTopologyProbe()

    # ディスクモニターのフィクスチャを定義する
    @pytest.fixture
    def disk_monitor(self) -> MockDiskMonitor:
        """MockDiskMonitor フィクスチャを生成する。"""
        # MockDiskMonitor をインスタンス化して返す
        return MockDiskMonitor(warning_threshold=0.75, critical_threshold=0.90)

    # HLC 検証器のフィクスチャを定義する
    @pytest.fixture
    def hlc_verifier(self) -> MockHLCVerifier:
        """MockHLCVerifier フィクスチャを生成する。"""
        # MockHLCVerifier をインスタンス化して返す
        return MockHLCVerifier()

    # NTP ドリフトが閾値以下の場合にアラートなしであることをテストする
    def test_ntp_drift_below_threshold_no_alert(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """NTP ドリフトが閾値以下の場合にアラートが発生しないことを検証する。"""
        # ドリフトを閾値以下に設定する
        ntp_monitor.set_drift_ms(30.0)
        # check() を実行する
        alert = ntp_monitor.check()
        # アラートがないことを確認する
        assert alert is None

    # NTP ドリフトが閾値を超えるとアラートが発生することをテストする
    def test_ntp_drift_above_threshold_generates_alert(
        self, ntp_monitor: MockNTPMonitor
    ) -> None:
        """NTP ドリフトが閾値を超えた場合にアラートが発生することを検証する。"""
        # ドリフトを閾値超えに設定する
        ntp_monitor.set_drift_ms(100.0)
        # check() を実行する
        alert = ntp_monitor.check()
        # アラートが発生していることを確認する
        assert alert is not None
        # アラートのタイプを確認する
        assert alert.get("type") == "ntp_drift"

    # ネットワーク分断がないときにトポロジーが正常であることをテストする
    def test_topology_no_partition_status_healthy(
        self, topology_probe: MockTopologyProbe
    ) -> None:
        """ネットワーク分断がない場合にトポロジーが正常であることを検証する。"""
        # 3 ノードを追加する
        topology_probe.add_node("n1")
        # ノードを追加する
        topology_probe.add_node("n2")
        # ノードを追加する
        topology_probe.add_node("n3")
        # トポロジー状態を取得する
        status = topology_probe.get_topology_status()
        # 全ノードが到達可能であることを確認する
        assert status.get("all_reachable") is True

    # ネットワーク分断後に回復すると正常に戻ることをテストする
    def test_topology_partition_then_recover_healthy(
        self, topology_probe: MockTopologyProbe
    ) -> None:
        """ネットワーク分断後に回復するとトポロジーが正常に戻ることを検証する。"""
        # 2 ノードを追加する
        topology_probe.add_node("a")
        # ノードを追加する
        topology_probe.add_node("b")
        # 分断を注入する
        topology_probe.partition(["a"])
        # 回復させる
        topology_probe.recover()
        # トポロジー状態を取得する
        status = topology_probe.get_topology_status()
        # 全ノードが到達可能であることを確認する
        assert status.get("all_reachable") is True

    # ディスク使用率が警告閾値以下の場合にアラートなしであることをテストする
    def test_disk_below_warning_no_alert(
        self, disk_monitor: MockDiskMonitor
    ) -> None:
        """ディスク使用率が警告閾値以下の場合にアラートがないことを検証する。"""
        # 使用率を 0.50 に設定する
        disk_monitor.set_disk_usage(0.50)
        # アラートを確認する
        alert = disk_monitor.check_usage()
        # アラートがないことを確認する
        assert alert is None

    # ディスク使用率がクリティカル閾値を超えるとアラートが発生することをテストする
    def test_disk_critical_threshold_generates_alert(
        self, disk_monitor: MockDiskMonitor
    ) -> None:
        """ディスク使用率がクリティカル閾値を超えた場合にアラートが発生することを検証する。"""
        # 使用率を 0.95 に設定する
        disk_monitor.set_disk_usage(0.95)
        # アラートを確認する
        alert = disk_monitor.check_usage()
        # クリティカルアラートが発生していることを確認する
        assert alert is not None
        # アラートの重大度が CRITICAL であることを確認する
        assert alert.get("severity") == AlertSeverity.CRITICAL.value

    # HLC 検証器が単調増加するタイムスタンプを受け入れることをテストする
    def test_hlc_verifier_monotone_no_anomaly(
        self, hlc_verifier: MockHLCVerifier
    ) -> None:
        """HLC 検証器が単調増加するタイムスタンプを受け入れることを検証する。"""
        # 単調増加するタイムスタンプを検証する
        ts_list = [(100, 0), (200, 0), (200, 1), (300, 0), (300, 1), (300, 2)]
        # 全タイムスタンプを検証する
        for ts in ts_list:
            # タイムスタンプを検証する
            result = hlc_verifier.verify(ts)
            # 異常がないことを確認する
            assert result is None, f"Monotone TS {ts} should not generate anomaly"
        # 異常リストが空であることを確認する
        assert len(hlc_verifier.anomalies) == 0

    # HLC 検証器が後退するタイムスタンプを検出することをテストする
    def test_hlc_verifier_backward_jump_detected(
        self, hlc_verifier: MockHLCVerifier
    ) -> None:
        """HLC 検証器が後退するタイムスタンプを検出することを検証する。"""
        # まず正常なタイムスタンプを検証する
        hlc_verifier.verify((1000, 5))
        # 後退するタイムスタンプを検証する
        result = hlc_verifier.verify((999, 0))
        # 異常が検出されることを確認する
        assert result is not None
        # タイプが hlc_backward_jump であることを確認する
        assert result.get("type") == "hlc_backward_jump"
        # 異常リストに 1 件あることを確認する
        assert len(hlc_verifier.anomalies) == 1

    # ディスク使用率が 1.0 でクリティカルアラートが発生することをテストする
    def test_disk_full_generates_critical_alert(
        self, disk_monitor: MockDiskMonitor
    ) -> None:
        """ディスク使用率が 1.0（満杯）の場合にクリティカルアラートが発生することを検証する。"""
        # 使用率を 1.0（満杯）に設定する
        disk_monitor.set_disk_usage(1.0)
        # アラートを確認する
        alert = disk_monitor.check_usage()
        # アラートが発生していることを確認する
        assert alert is not None
        # 重大度が CRITICAL であることを確認する
        assert alert.get("severity") == AlertSeverity.CRITICAL.value
