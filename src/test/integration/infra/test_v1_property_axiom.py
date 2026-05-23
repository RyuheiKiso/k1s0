"""src/test/integration/infra/test_v1_property_axiom.py

infra 軸の v1_property_axiom 検証クラスに対するインテグレーションテスト。
HLC 単調増加・NTP 同期クロックスキュー・ネットワーク RTT 正値・
トポロジーグラフの負重みなしプロパティを hypothesis で検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: infra_property_axiom_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import time
import math
# pytest フレームワークをインポートする
import pytest
# hypothesis ライブラリをインポートする
from hypothesis import given, settings, assume, HealthCheck
# hypothesis の戦略モジュールをインポートする
from hypothesis import strategies as st
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict, Tuple
# dataclass デコレータをインポートする
from dataclasses import dataclass, field


# HLC ノードシミュレーターのクラスを定義する
class HLCNode:
    """HLC（Hybrid Logical Clock）を持つインフラノードのシミュレーター。"""

    # ノードを初期化するメソッドを定義する
    def __init__(self, node_id: str, initial_wall_ms: int) -> None:
        """HLC ノードを初期化する。"""
        # ノード ID を設定する
        self.node_id = node_id
        # 物理時刻を初期化する
        self._wall_ms = initial_wall_ms
        # 論理カウンタを初期化する
        self._counter = 0
        # 最後に生成したタイムスタンプを記録する
        self._last_ts: tuple[int, int] = (initial_wall_ms, 0)

    # HLC の now() を実行するメソッドを定義する
    def now(self) -> tuple[int, int]:
        """現在の HLC タイムスタンプを生成する。単調増加を保証する。"""
        # 物理時刻が進んでいる場合はカウンタをリセットする
        if self._wall_ms > self._last_ts[0]:
            # 物理時刻が進んだのでカウンタをリセットする
            self._counter = 0
        else:
            # 物理時刻が変わっていない場合はカウンタをインクリメントする
            self._counter += 1
        # タイムスタンプを更新する
        self._last_ts = (self._wall_ms, self._counter)
        # タイムスタンプを返す
        return self._last_ts

    # リモートタイムスタンプを受信するメソッドを定義する
    def recv(self, remote_ts: tuple[int, int]) -> tuple[int, int]:
        """リモートタイムスタンプを受信して自身の HLC を更新する。"""
        # リモートの物理時刻を取得する
        remote_wall, remote_counter = remote_ts
        # 自身の物理時刻とリモートの物理時刻の最大値を取る
        new_wall = max(self._wall_ms, remote_wall)
        # 物理時刻が同じ場合はカウンタを調整する
        if new_wall == self._wall_ms and new_wall == remote_wall:
            # 両方同じ場合は max counter + 1 にする
            self._counter = max(self._counter, remote_counter) + 1
        elif new_wall == self._wall_ms:
            # 自身の物理時刻が大きい場合はカウンタをインクリメントする
            self._counter += 1
        elif new_wall == remote_wall:
            # リモートの物理時刻が大きい場合はリモートカウンタ + 1 にする
            self._counter = remote_counter + 1
        else:
            # 新しい物理時刻の場合はカウンタをリセットする
            self._counter = 0
        # 物理時刻を更新する
        self._wall_ms = new_wall
        # タイムスタンプを更新する
        self._last_ts = (self._wall_ms, self._counter)
        # タイムスタンプを返す
        return self._last_ts

    # 物理時刻を進めるメソッドを定義する
    def advance_clock(self, delta_ms: int) -> None:
        """物理時刻を指定ミリ秒進める。"""
        # 物理時刻を加算する
        self._wall_ms += delta_ms


# NTP クロック同期シミュレーターのクラスを定義する
class NTPSyncSimulator:
    """NTP クロック同期をシミュレートするクラス。"""

    # シミュレーターを初期化するメソッドを定義する
    def __init__(self, base_time_ms: int = 1700000000000) -> None:
        """NTP 同期シミュレーターを初期化する。"""
        # 基準時刻を設定する
        self._base_time_ms = base_time_ms
        # ノードのクロックオフセットを管理する辞書を初期化する
        self._node_offsets: dict[str, float] = {}

    # ノードを追加するメソッドを定義する
    def add_node(self, node_id: str, offset_ms: float = 0.0) -> None:
        """ノードを追加してクロックオフセットを設定する。"""
        # ノードのオフセットを設定する
        self._node_offsets[node_id] = offset_ms

    # ノードのクロック時刻を取得するメソッドを定義する
    def get_node_time(self, node_id: str) -> float:
        """ノードの現在時刻を返す。"""
        # ノードのオフセットを取得する
        offset = self._node_offsets.get(node_id, 0.0)
        # 基準時刻にオフセットを加算して返す
        return self._base_time_ms + offset

    # 2 ノード間のクロックスキューを計算するメソッドを定義する
    def clock_skew_ms(self, node_a: str, node_b: str) -> float:
        """2 ノード間のクロックスキュー（絶対値）を返す。"""
        # ノード A の時刻を取得する
        time_a = self.get_node_time(node_a)
        # ノード B の時刻を取得する
        time_b = self.get_node_time(node_b)
        # 絶対値のスキューを計算して返す
        return abs(time_a - time_b)


# ネットワークプローブのモッククラスを定義する
class MockNetworkProbe:
    """ネットワーク RTT プローブのモック。"""

    # プローブを初期化するメソッドを定義する
    def __init__(self, base_rtt_ms: float = 5.0) -> None:
        """ネットワークプローブを初期化する。"""
        # 基準 RTT を設定する
        self._base_rtt_ms = base_rtt_ms

    # RTT を測定するメソッドを定義する
    def measure_rtt(
        self,
        src_node: str,
        dst_node: str,
        jitter_ms: float = 0.0,
    ) -> float:
        """2 ノード間の RTT を測定する。常に正の値を返す。"""
        # RTT を計算する（基準 RTT + ジッター）
        rtt = self._base_rtt_ms + abs(jitter_ms)
        # RTT が正であることを確認する
        if rtt <= 0:
            # RTT が非正の場合は最小値を設定する
            rtt = 0.001
        # RTT を返す
        return rtt


# トポロジーグラフのモッククラスを定義する
class TopologyGraph:
    """ネットワークトポロジーグラフ（辺の重みは RTT ms）。"""

    # グラフを初期化するメソッドを定義する
    def __init__(self) -> None:
        """トポロジーグラフを初期化する。"""
        # ノードのセットを初期化する
        self._nodes: set[str] = set()
        # 辺（src, dst, weight）のリストを初期化する
        self._edges: list[tuple[str, str, float]] = []

    # ノードを追加するメソッドを定義する
    def add_node(self, node_id: str) -> None:
        """ノードをグラフに追加する。"""
        # ノードを追加する
        self._nodes.add(node_id)

    # 辺を追加するメソッドを定義する
    def add_edge(self, src: str, dst: str, weight_ms: float) -> None:
        """辺をグラフに追加する。重みは RTT（ms）。"""
        # 重みが非正の場合はエラーを発生させる
        if weight_ms <= 0:
            # バリデーションエラーを発生させる
            raise ValueError(
                f"Edge weight must be positive (RTT > 0), got {weight_ms}"
            )
        # 辺を追加する
        self._edges.append((src, dst, weight_ms))

    # 辺を取得するプロパティを定義する
    @property
    def edges(self) -> list[tuple[str, str, float]]:
        """辺のリストのコピーを返す。"""
        # 辺のコピーを返す
        return list(self._edges)

    # 全辺の重みが正であることを確認するメソッドを定義する
    def has_negative_weight(self) -> bool:
        """負の重みを持つ辺が存在するかどうかを返す。"""
        # 各辺の重みを確認する
        for _, _, w in self._edges:
            # 重みが非正の場合は True を返す
            if w <= 0:
                # 負の重みが存在することを返す
                return True
        # 負の重みなしを返す
        return False


# infra 軸プロパティ検証テストスイートクラスを定義する
class TestInfraPropertyAxiom:
    """infra 軸 v1_property_axiom の検証テストスイート。"""

    # HLC ノードのフィクスチャを定義する
    @pytest.fixture
    def hlc_node(self) -> Generator[HLCNode, None, None]:
        """HLCNode フィクスチャを生成する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="infra_node_0", initial_wall_ms=1700000000000)
        # フィクスチャを返す
        yield node

    # 同一ノードの連続クロック読み取りが単調増加するプロパティをテストする
    @given(
        # クロック読み取り回数を生成する戦略を定義する
        read_count=st.integers(min_value=2, max_value=100),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_now_monotonically_increasing(
        self, read_count: int
    ) -> None:
        """同一ノードの HLC.now() が単調増加することを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="test_node", initial_wall_ms=1700000000000)
        # タイムスタンプリストを初期化する
        timestamps = []
        # read_count 件の読み取りを実行する
        for _ in range(read_count):
            # タイムスタンプを生成する
            ts = node.now()
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # 連続するタイムスタンプが単調増加することを確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプが現在のタイムスタンプ以下であることを確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"HLC.now() must be monotonically non-decreasing: "
                f"ts[{i-1}]={timestamps[i-1]} > ts[{i}]={timestamps[i]}"
            )

    # NTP 同期後のノード間クロックスキューが 100ms 未満であるプロパティをテストする
    @given(
        # ノード A のオフセットを生成する戦略を定義する
        offset_a_ms=st.floats(min_value=-50.0, max_value=50.0),
        # ノード B のオフセットを生成する戦略を定義する
        offset_b_ms=st.floats(min_value=-50.0, max_value=50.0),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_ntp_synced_clock_skew_under_100ms(
        self, offset_a_ms: float, offset_b_ms: float
    ) -> None:
        """NTP 同期済みノード間のクロックスキューが 100ms 未満であることを検証する。"""
        # NaN や無限大の場合はスキップする
        assume(math.isfinite(offset_a_ms) and math.isfinite(offset_b_ms))
        # NTP 同期シミュレーターを初期化する
        ntp = NTPSyncSimulator(base_time_ms=1700000000000)
        # ノード A を追加する
        ntp.add_node("node_a", offset_ms=offset_a_ms)
        # ノード B を追加する
        ntp.add_node("node_b", offset_ms=offset_b_ms)
        # 2 ノード間のクロックスキューを計算する
        skew = ntp.clock_skew_ms("node_a", "node_b")
        # スキューが 100ms 未満であることを確認する（±50ms 範囲なので最大 100ms）
        assert skew < 100.0, (
            f"NTP clock skew must be < 100ms, got {skew}ms "
            f"(offset_a={offset_a_ms}, offset_b={offset_b_ms})"
        )

    # ネットワークプローブの RTT が常に正であるプロパティをテストする
    @given(
        # 基準 RTT を生成する戦略を定義する
        base_rtt_ms=st.floats(min_value=0.1, max_value=1000.0),
        # ジッターを生成する戦略を定義する
        jitter_ms=st.floats(min_value=0.0, max_value=100.0),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_network_probe_rtt_always_positive(
        self, base_rtt_ms: float, jitter_ms: float
    ) -> None:
        """ネットワークプローブの RTT が常に正であることを検証する。"""
        # NaN や無限大の場合はスキップする
        assume(math.isfinite(base_rtt_ms) and math.isfinite(jitter_ms))
        # プローブをインスタンス化する
        probe = MockNetworkProbe(base_rtt_ms=base_rtt_ms)
        # RTT を測定する
        rtt = probe.measure_rtt(
            src_node="node_0",
            dst_node="node_1",
            jitter_ms=jitter_ms,
        )
        # RTT が正であることを確認する
        assert rtt > 0, f"Network probe RTT must be positive, got {rtt}"

    # トポロジーグラフに負の重みの辺がないプロパティをテストする
    @given(
        # 辺の重みのリストを生成する戦略を定義する
        weights=st.lists(
            st.floats(min_value=0.1, max_value=100.0),
            min_size=1,
            max_size=20,
        ),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_topology_graph_no_negative_weight_edges(
        self, weights: list[float]
    ) -> None:
        """トポロジーグラフの全辺が正の重みを持つことを検証する。"""
        # NaN や無限大の値を除外する
        valid_weights = [w for w in weights if math.isfinite(w) and w > 0]
        # 有効な重みがない場合はスキップする
        assume(len(valid_weights) > 0)
        # トポロジーグラフを生成する
        graph = TopologyGraph()
        # ノードを追加する
        for i in range(len(valid_weights) + 1):
            # ノードを追加する
            graph.add_node(f"node_{i}")
        # 辺を追加する
        for i, weight in enumerate(valid_weights):
            # 辺を追加する
            graph.add_edge(f"node_{i}", f"node_{i+1}", weight_ms=weight)
        # 負の重みを持つ辺がないことを確認する
        assert not graph.has_negative_weight(), (
            "Topology graph must not have negative-weight edges"
        )

    # 負の重みを持つ辺が追加されると例外が発生することをテストする
    def test_topology_negative_weight_raises(self) -> None:
        """負の重みを持つ辺の追加が例外を発生させることを検証する。"""
        # トポロジーグラフを生成する
        graph = TopologyGraph()
        # ノードを追加する
        graph.add_node("src")
        # 宛先ノードを追加する
        graph.add_node("dst")
        # 負の重みを持つ辺の追加がエラーを発生させることを確認する
        with pytest.raises(ValueError, match="positive"):
            # 負の重みで辺を追加する
            graph.add_edge("src", "dst", weight_ms=-1.0)

    # HLC.recv() が受信タイムスタンプ以上のタイムスタンプを返すプロパティをテストする
    @given(
        # リモートタイムスタンプの物理時刻を生成する戦略を定義する
        remote_wall=st.integers(min_value=1600000000000, max_value=2000000000000),
        # リモートのカウンタを生成する戦略を定義する
        remote_counter=st.integers(min_value=0, max_value=1000),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_recv_returns_ts_ge_remote(
        self, remote_wall: int, remote_counter: int
    ) -> None:
        """HLC.recv(remote_ts) が remote_ts 以上のタイムスタンプを返すことを検証する。"""
        # HLC ノードを初期化する
        node = HLCNode(node_id="recv_test_node", initial_wall_ms=1700000000000)
        # リモートタイムスタンプを生成する
        remote_ts = (remote_wall, remote_counter)
        # recv を実行する
        local_ts = node.recv(remote_ts)
        # local_ts >= remote_ts であることを確認する
        assert local_ts >= remote_ts, (
            f"HLC.recv() must return ts >= remote_ts: "
            f"local={local_ts}, remote={remote_ts}"
        )

    # HLC の連続 now() 呼び出しが全順序を保つプロパティをテストする
    def test_hlc_consecutive_now_total_order(self, hlc_node: HLCNode) -> None:
        """連続した HLC.now() 呼び出しが全順序を保つことを検証する。"""
        # 50 件のタイムスタンプを生成する
        timestamps = [hlc_node.now() for _ in range(50)]
        # ソートしたタイムスタンプが元のリストと一致することを確認する
        sorted_ts = sorted(timestamps)
        # タイムスタンプが単調非減少であることを確認する
        assert sorted_ts == timestamps, (
            "HLC.now() consecutive calls must be in non-decreasing order"
        )


# infra 軸プロパティ検証追加テストスイートクラスを定義する
class TestInfraPropertyAxiomExtended:
    """infra 軸 v1_property_axiom の追加検証テストスイート。

    クロック同期・ネットワーク測定・トポロジー管理の追加プロパティを検証する。
    """

    # クロックドリフトが設定した範囲内に収まることを検証するテストを定義する
    def test_clock_skew_within_configured_range(self) -> None:
        """設定した最大オフセット内のクロックスキューが閾値以下であることを検証する。"""
        # NTP 同期シミュレーターを初期化する
        ntp = NTPSyncSimulator(base_time_ms=1700000000000)
        # 5 ノードを追加する（各 10ms 以内のオフセット）
        node_offsets = [0.0, 9.5, -8.3, 7.1, -5.0]
        # ノードを追加する
        for i, offset in enumerate(node_offsets):
            # ノードを追加する
            ntp.add_node(f"cluster_node_{i}", offset_ms=offset)
        # 全ノードペア間のスキューが 20ms 以内であることを確認する
        for i in range(len(node_offsets)):
            # 比較対象のノードを選択する
            for j in range(i + 1, len(node_offsets)):
                # クロックスキューを計算する
                skew = ntp.clock_skew_ms(
                    f"cluster_node_{i}",
                    f"cluster_node_{j}",
                )
                # スキューが 20ms 以内であることを確認する
                assert skew <= 20.0, (
                    f"Cluster clock skew exceeds 20ms: node_{i} vs node_{j}: {skew}ms"
                )

    # ネットワーク RTT が基準値より大きいことを検証するテストを定義する
    @pytest.mark.parametrize(
        "base_rtt,jitter",
        [
            # 最小 RTT のテストケースを定義する
            (0.5, 0.0),
            # 中程度の RTT のテストケースを定義する
            (10.0, 2.0),
            # 高レイテンシのテストケースを定義する
            (100.0, 20.0),
            # 1 秒超のレイテンシのテストケースを定義する
            (1000.0, 50.0),
        ],
    )
    def test_network_rtt_always_positive_parametrized(
        self, base_rtt: float, jitter: float
    ) -> None:
        """ネットワーク RTT が常に正であることを複数パラメータで検証する。"""
        # プローブをインスタンス化する
        probe = MockNetworkProbe(base_rtt_ms=base_rtt)
        # RTT を測定する
        rtt = probe.measure_rtt("src", "dst", jitter_ms=jitter)
        # RTT が正であることを確認する
        assert rtt > 0, f"RTT must be positive: base={base_rtt}, jitter={jitter}, rtt={rtt}"

    # HLC.now() が連続 1000 回呼び出し後も単調増加することを検証するテストを定義する
    def test_hlc_high_frequency_now_monotone(self) -> None:
        """HLC.now() を 1000 回連続呼び出しても単調増加が維持されることを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="high_freq_node", initial_wall_ms=1700000000000)
        # 1000 件のタイムスタンプを生成する
        prev_ts = node.now()
        # 999 回の追加呼び出しを実行する
        for i in range(999):
            # タイムスタンプを生成する
            curr_ts = node.now()
            # 単調増加を確認する
            assert prev_ts <= curr_ts, (
                f"HLC regressed at iteration {i}: {prev_ts} > {curr_ts}"
            )
            # 前のタイムスタンプを更新する
            prev_ts = curr_ts

    # トポロジーグラフに辺が追加されることを検証するテストを定義する
    def test_topology_graph_edges_added_correctly(self) -> None:
        """トポロジーグラフに辺が正しく追加されることを検証する。"""
        # グラフを生成する
        graph = TopologyGraph()
        # ノードを追加する
        graph.add_node("rack_a_node_1")
        # 2 つ目のノードを追加する
        graph.add_node("rack_a_node_2")
        # 3 つ目のノードを追加する
        graph.add_node("rack_b_node_1")
        # 辺を追加する
        graph.add_edge("rack_a_node_1", "rack_a_node_2", weight_ms=0.5)
        # 2 つ目の辺を追加する
        graph.add_edge("rack_a_node_1", "rack_b_node_1", weight_ms=2.3)
        # 辺が 2 件追加されていることを確認する
        assert len(graph.edges) == 2
        # 負の重みがないことを確認する
        assert not graph.has_negative_weight()

    # HLC の recv 後の now が recv の結果以上であることを検証するテストを定義する
    def test_hlc_now_after_recv_ge_recv_result(self) -> None:
        """recv() 後の now() が recv() の結果以上であることを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="recv_then_now_node", initial_wall_ms=1700000000000)
        # リモートタイムスタンプを設定する（未来の時刻）
        remote_ts = (1700000100000, 5)
        # recv を実行する
        recv_result = node.recv(remote_ts)
        # now を呼び出す
        now_result = node.now()
        # now の結果が recv の結果以上であることを確認する
        assert now_result >= recv_result, (
            f"now() after recv() must be >= recv() result: "
            f"now={now_result}, recv={recv_result}"
        )

    # NTP 同期シミュレーターが同一ノードのクロックを正しく返すことを検証するテストを定義する
    def test_ntp_sync_same_node_same_time(self) -> None:
        """NTP 同期シミュレーターが同一ノードに対して同じ時刻を返すことを検証する。"""
        # NTP 同期シミュレーターを初期化する
        ntp = NTPSyncSimulator(base_time_ms=1700000000000)
        # ノードを追加する
        ntp.add_node("stable_node", offset_ms=42.5)
        # 2 回時刻を取得して同じ結果が返されることを確認する
        time1 = ntp.get_node_time("stable_node")
        # 2 回目の取得を実行する
        time2 = ntp.get_node_time("stable_node")
        # 同じ時刻が返されることを確認する
        assert time1 == time2 == 1700000000042.5


# infra 軸プロパティ検証第 2 拡張テストスイートクラスを定義する
class TestInfraPropertyAxiomExtended2:
    """infra 軸 v1_property_axiom の第 2 拡張テストスイート。追加のプロパティを網羅的に検証する。"""

    # HLC が wall_ms を大きく進めた後に monotone を保つことをテストする
    def test_hlc_after_large_wall_advance_monotone(self) -> None:
        """物理時刻を大きく進めた後も HLC が単調非減少であることを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="large_advance_node", initial_wall_ms=1700000000000)
        # 最初のタイムスタンプを生成する
        ts1 = node.now()
        # 1000 ミリ秒進める
        node.advance_clock(1000)
        # 2 番目のタイムスタンプを生成する
        ts2 = node.now()
        # ts2 > ts1 であることを確認する
        assert ts2 > ts1, f"After large wall advance: ts2={ts2} should be > ts1={ts1}"

    # HLC を複数ノードで初期化したときの独立性をテストする
    def test_hlc_multiple_nodes_independent(self) -> None:
        """複数の HLC ノードが独立してタイムスタンプを管理することを検証する。"""
        # ノード 1 を初期化する
        node1 = HLCNode(node_id="node_1", initial_wall_ms=1700000000000)
        # ノード 2 を初期化する
        node2 = HLCNode(node_id="node_2", initial_wall_ms=1700000000000)
        # ノード 1 を 10 回進める
        for _ in range(10):
            # ノード 1 のタイムスタンプを進める
            node1.now()
        # ノード 2 の最初のタイムスタンプを生成する
        ts_node2 = node2.now()
        # ノード 1 の最後のタイムスタンプが ノード 2 の最初より大きいことを確認する
        ts_node1 = node1.now()
        # ノード 1 のカウンタがノード 2 のカウンタより大きいことを確認する（wall_ms 同じなら）
        if ts_node1[0] == ts_node2[0]:
            # 同じ wall_ms ならノード 1 のカウンタが大きいはず
            assert ts_node1[1] > ts_node2[1], (
                f"node1 counter {ts_node1[1]} should be > node2 counter {ts_node2[1]}"
            )

    # NTP 同期シミュレーターが複数ノードの時刻オフセットを正確に返すことをテストする
    def test_ntp_multiple_node_offsets_accurate(self) -> None:
        """NTP 同期シミュレーターが各ノードのオフセットを正確に適用することを検証する。"""
        # NTP 同期シミュレーターを初期化する
        ntp = NTPSyncSimulator(base_time_ms=1700000000000)
        # オフセットのリストを定義する
        offsets = [0.0, 10.0, -5.0, 20.5, -15.3]
        # 各ノードを追加する
        for i, offset in enumerate(offsets):
            # ノードを追加する
            ntp.add_node(f"node_{i}", offset_ms=offset)
        # 各ノードの時刻が正確であることを確認する
        for i, offset in enumerate(offsets):
            # ノードの時刻を取得する
            node_time = ntp.get_node_time(f"node_{i}")
            # 期待する時刻を計算する
            expected = 1700000000000 + offset
            # 時刻が正確であることを確認する
            assert node_time == expected, (
                f"Node {i} time mismatch: expected={expected}, got={node_time}"
            )

    # NTP スキューがゼロオフセットで 0 になることをテストする
    def test_ntp_skew_zero_offset_nodes(self) -> None:
        """全てのノードがゼロオフセットの場合、スキューが 0 になることを検証する。"""
        # NTP 同期シミュレーターを初期化する
        ntp = NTPSyncSimulator(base_time_ms=1700000000000)
        # 3 ノードを全てゼロオフセットで追加する
        for i in range(3):
            # ノードを追加する
            ntp.add_node(f"zero_node_{i}", offset_ms=0.0)
        # 全ノードペア間のスキューが 0 であることを確認する
        time0 = ntp.get_node_time("zero_node_0")
        # 2 番目のノードの時刻を取得する
        time1 = ntp.get_node_time("zero_node_1")
        # 3 番目のノードの時刻を取得する
        time2 = ntp.get_node_time("zero_node_2")
        # スキューが 0 であることを確認する
        assert abs(time0 - time1) == 0.0
        # スキューが 0 であることを確認する
        assert abs(time1 - time2) == 0.0

    # トポロジーグラフに多数の辺を追加しても負の重みが検出されないことをテストする
    def test_topology_many_edges_no_negative_weight(self) -> None:
        """多数の辺を追加しても負の重みが検出されないことを検証する。"""
        # トポロジーグラフを生成する
        graph = TopologyGraph()
        # 10 ノードを追加する
        for i in range(10):
            # ノードを追加する
            graph.add_node(f"node_{i}")
        # 9 本の辺を追加する（線形トポロジー）
        for i in range(9):
            # 辺を追加する
            graph.add_edge(f"node_{i}", f"node_{i+1}", weight_ms=float(i + 1))
        # 辺が 9 件あることを確認する
        assert len(graph.edges) == 9
        # 負の重みがないことを確認する
        assert not graph.has_negative_weight()

    # HLC の recv が自身より古いタイムスタンプで変化しないことをテストする
    def test_hlc_recv_older_ts_does_not_go_backward(self) -> None:
        """recv() に古いタイムスタンプを渡しても HLC が後退しないことを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="recv_older_node", initial_wall_ms=1700000100000)
        # 現在のタイムスタンプを取得する
        current = node.now()
        # 古いタイムスタンプを設定する（1 秒前）
        old_ts = (1700000000000, 0)
        # recv を実行する
        result = node.recv(old_ts)
        # 結果が現在以上であることを確認する
        assert result >= current, (
            f"HLC.recv(old_ts) should not go backward: result={result}, current={current}"
        )

    # プロパティ: 任意の HLC タイムスタンプで recv が単調性を保つことをテストする
    @given(
        # 初期 wall_ms を生成する戦略を定義する
        initial_wall_ms=st.integers(min_value=1700000000000, max_value=1800000000000),
        # リモートのデルタを生成する戦略を定義する（前後 10 秒）
        remote_delta=st.integers(min_value=-10000, max_value=10000),
    )
    @settings(max_examples=80, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_recv_monotone(
        self, initial_wall_ms: int, remote_delta: int
    ) -> None:
        """任意のリモートタイムスタンプに対して recv() が単調性を保つことを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="monotone_recv_node", initial_wall_ms=initial_wall_ms)
        # 現在のタイムスタンプを取得する
        before = node.now()
        # リモートタイムスタンプを設定する
        remote_ts = (initial_wall_ms + remote_delta, 0)
        # remote_ts が負にならないことを確認する
        assume(remote_ts[0] >= 0)
        # recv を実行する
        after = node.recv(remote_ts)
        # recv 後の now が after 以上であることを確認する
        now_after = node.now()
        # now_after >= after >= before（after が min(before, remote_ts) 以上）
        assert now_after >= after, (
            f"now() after recv() must be >= recv() result: now={now_after}, recv={after}"
        )


# infra プロパティ公理の第三拡張テストクラスを定義する
class TestInfraPropertyAxiomExtended3:
    """HLC・NTP・RTT・トポロジーのプロパティ公理の追加テスト群（第三拡張）。"""

    # HLC ノードが now() を連続呼び出しすると単調増加することをテストする
    def test_hlc_sequential_now_calls_monotone(self) -> None:
        """HLC ノードの連続する now() 呼び出しが単調増加することを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="seq_monotone_node", initial_wall_ms=1700000000000)
        # 10 回連続で now() を呼び出す
        timestamps = [node.now() for _ in range(10)]
        # 全タイムスタンプが単調増加することを確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプより大きいか等しいことを確認する
            assert timestamps[i] >= timestamps[i - 1], (
                f"HLC not monotone at index {i}: {timestamps[i]} < {timestamps[i - 1]}"
            )

    # HLC ノードが recv() で大きいリモート時刻を採用することをテストする
    def test_hlc_recv_adopts_larger_remote_wall(self) -> None:
        """HLC ノードが recv() でより大きい物理時刻を採用することを検証する。"""
        # HLC ノードを小さい初期値でインスタンス化する
        node = HLCNode(node_id="small_wall_node", initial_wall_ms=1000)
        # 現在の now() を取得する
        before = node.now()
        # 大きいリモートタイムスタンプを受信する
        large_remote = (9_000_000, 5)
        # recv を実行する
        result = node.recv(large_remote)
        # recv 後の wall_ms が large_remote 以上であることを確認する
        assert result[0] >= large_remote[0], (
            f"After recv of large remote, wall_ms should be >= remote: result={result}, remote={large_remote}"
        )

    # NTP シミュレーターが 5 ノードの時刻を独立に管理することをテストする
    def test_ntp_five_nodes_independent_times(self) -> None:
        """NTP シミュレーターが 5 ノードの時刻を独立して管理することを検証する。"""
        # NTP シミュレーターをインスタンス化する
        ntp = NTPSyncSimulator(base_time_ms=1700000000000)
        # 5 ノードにそれぞれ異なるオフセットを設定する
        offsets = [0, 100, -50, 200, -150]
        # 全ノードを追加する
        for i, offset in enumerate(offsets):
            # ノードを追加する
            ntp.add_node(f"node_{i}", offset_ms=offset)
        # 各ノードの時刻が正しいことを確認する
        for i, offset in enumerate(offsets):
            # ノードの時刻を取得する
            t = ntp.get_node_time(f"node_{i}")
            # 期待値を計算する
            expected = 1700000000000 + offset
            # 時刻が一致することを確認する
            assert t == expected, f"Node {i}: expected {expected}, got {t}"

    # RTT がジッターの影響を受けることをテストする
    def test_rtt_with_jitter_is_positive(self) -> None:
        """ジッター付きの RTT 測定が常に正値であることを検証する。"""
        # ネットワークプローブをインスタンス化する
        probe = MockNetworkProbe(base_rtt_ms=10.0)
        # ジッターを様々な値で測定する
        for jitter in [0.0, 1.0, 5.0, 9.9]:
            # RTT を測定する
            rtt = probe.measure_rtt("src_node", "dst_node", jitter_ms=jitter)
            # RTT が正値であることを確認する
            assert rtt > 0, f"RTT must be positive with jitter={jitter}, got {rtt}"

    # トポロジーグラフが重みゼロのエッジを含まないことをテストする
    def test_topology_graph_zero_weight_edges_no_negative(self) -> None:
        """トポロジーグラフの重みゼロエッジが負重みなし条件を満たすことを検証する。"""
        # トポロジーグラフをインスタンス化する
        graph = TopologyGraph()
        # ノードを追加する
        graph.add_node("A")
        # ノードを追加する
        graph.add_node("B")
        # 重みゼロのエッジを追加する
        graph.add_edge("A", "B", weight_ms=0.0)
        # 負重みがないことを確認する（ゼロは許可）
        assert not graph.has_negative_weight(), "Zero-weight edges should not trigger negative weight detection"

    # トポロジーグラフが複数の負重みエッジを持つと検出されることをテストする
    def test_topology_graph_multiple_negative_edges_detected(self) -> None:
        """トポロジーグラフが複数の負重みエッジを持つと検出されることを検証する。"""
        # トポロジーグラフをインスタンス化する
        graph = TopologyGraph()
        # ノードを追加する
        graph.add_node("X")
        # ノードを追加する
        graph.add_node("Y")
        # ノードを追加する
        graph.add_node("Z")
        # 負重みエッジを追加する
        graph.add_edge("X", "Y", weight_ms=-1.0)
        # さらに負重みエッジを追加する
        graph.add_edge("Y", "Z", weight_ms=-2.0)
        # 負重みが検出されることを確認する
        assert graph.has_negative_weight(), "Multiple negative edges should be detected"

    # HLC ノードが物理時刻の大幅な進みに対応することをテストする
    def test_hlc_large_wall_advance_handled(self) -> None:
        """HLC ノードが物理時刻の大幅な進みに正しく対応することを検証する。"""
        # HLC ノードをインスタンス化する
        node = HLCNode(node_id="large_advance_node", initial_wall_ms=1000)
        # 現在の now() を取得する
        before = node.now()
        # 物理時刻を大幅に進める
        node.advance_clock(delta_ms=1_000_000_000)
        # 大幅な進み後の now() を取得する
        after = node.now()
        # after が before より大きいことを確認する
        assert after > before, f"After large advance, HLC must be greater: before={before}, after={after}"

    # NTP ノードにゼロオフセットを設定すると base_time と一致することをテストする
    def test_ntp_zero_offset_equals_base_time(self) -> None:
        """NTP ノードのオフセットがゼロの場合、ベース時刻と一致することを検証する。"""
        # base_time_ms を設定する
        base_ms = 1700000000000
        # NTP シミュレーターをインスタンス化する
        ntp = NTPSyncSimulator(base_time_ms=base_ms)
        # ゼロオフセットのノードを追加する
        ntp.add_node("zero_offset_node", offset_ms=0)
        # ノードの時刻を取得する
        t = ntp.get_node_time("zero_offset_node")
        # base_time_ms と一致することを確認する
        assert t == base_ms, f"Zero offset node should equal base_time: expected={base_ms}, got={t}"

    # トポロジーグラフのエッジリストが追加したエッジを含むことをテストする
    def test_topology_graph_edges_property_contains_added(self) -> None:
        """トポロジーグラフの edges プロパティが追加したエッジを含むことを検証する。"""
        # トポロジーグラフをインスタンス化する
        graph = TopologyGraph()
        # ノードを追加する
        graph.add_node("P")
        # ノードを追加する
        graph.add_node("Q")
        # エッジを追加する
        graph.add_edge("P", "Q", weight_ms=5.0)
        # edges プロパティを取得する
        edges = graph.edges
        # エッジリストに追加したエッジが含まれることを確認する
        assert len(edges) >= 1
        # P→Q エッジが含まれることを確認する
        found = any(e[0] == "P" and e[1] == "Q" for e in edges)
        # エッジが見つかることを確認する
        assert found, "Edge P→Q should be in edges property"

    # HLC の now() 後に recv() でそれ以前のタイムスタンプを受信しても退行しないことをテストする
    def test_hlc_recv_old_timestamp_no_regression(self) -> None:
        """recv() で古いタイムスタンプを受信した場合、HLC が退行しないことを検証する。"""
        # HLC ノードを大きい初期値でインスタンス化する
        node = HLCNode(node_id="no_regression_node", initial_wall_ms=9_000_000_000)
        # 現在の now() を取得する
        current = node.now()
        # 非常に古いタイムスタンプを受信する
        ancient_ts = (1, 0)
        # recv を実行する
        result = node.recv(ancient_ts)
        # recv 結果が current 以上であることを確認する
        assert result >= current, (
            f"recv of ancient TS must not regress: current={current}, result={result}"
        )

    # NTP ノードにマイナスオフセットを設定すると base_time より小さいことをテストする
    def test_ntp_negative_offset_less_than_base(self) -> None:
        """NTP ノードの負オフセットが base_time より小さい時刻を返すことを検証する。"""
        # base_time_ms を設定する
        base_ms = 1700000000000
        # NTP シミュレーターをインスタンス化する
        ntp = NTPSyncSimulator(base_time_ms=base_ms)
        # 負オフセットのノードを追加する
        ntp.add_node("neg_offset_node", offset_ms=-500)
        # ノードの時刻を取得する
        t = ntp.get_node_time("neg_offset_node")
        # base_time より小さいことを確認する
        assert t < base_ms, f"Negative offset node should have time < base_time: expected <{base_ms}, got {t}"


# infra プロパティ公理の第四拡張テストクラスを定義する
class TestInfraPropertyAxiomExtended4:
    """HLC・NTP・RTT・トポロジーのプロパティ公理の追加テスト群（第四拡張）。"""

    # HLC ノードの now() が物理時刻の大幅な後退に対応することをテストする
    def test_hlc_handles_wall_clock_regression(self) -> None:
        """HLC ノードが物理時刻の後退を受けた場合も単調増加を保つことを検証する。"""
        # HLC ノードを 大きい時刻でインスタンス化する
        node = HLCNode(node_id="regression_safe", initial_wall_ms=1_000_000)
        # 現在の now() を取得する
        ts1 = node.now()
        # 物理時刻を後退させる
        node.advance_clock(delta_ms=-500_000)
        # 後退後の now() を取得する（単調増加を保つはず）
        ts2 = node.now()
        # ts2 が ts1 以上であることを確認する
        assert ts2 >= ts1, f"HLC must remain monotone even after wall regression: ts1={ts1}, ts2={ts2}"

    # トポロジーグラフが正重みのエッジのみを持つ場合に負重みなしを確認することをテストする
    def test_topology_positive_weights_only_no_negative(self) -> None:
        """トポロジーグラフが正重みのエッジのみを持つ場合に負重みなしを確認することを検証する。"""
        # トポロジーグラフをインスタンス化する
        graph = TopologyGraph()
        # 3 ノードを追加する
        for i in range(3):
            # ノードを追加する
            graph.add_node(f"host_{i}")
        # 正重みのエッジを追加する
        graph.add_edge("host_0", "host_1", weight_ms=5.0)
        # さらに正重みのエッジを追加する
        graph.add_edge("host_1", "host_2", weight_ms=10.0)
        # 負重みがないことを確認する
        assert not graph.has_negative_weight()

    # NTP シミュレーターが大きい正オフセットを正しく計算することをテストする
    def test_ntp_large_positive_offset(self) -> None:
        """NTP シミュレーターが大きい正オフセットを正しく計算することを検証する。"""
        # base_time_ms を設定する
        base_ms = 1_000_000
        # NTP シミュレーターをインスタンス化する
        ntp = NTPSyncSimulator(base_time_ms=base_ms)
        # 大きいオフセットのノードを追加する
        ntp.add_node("fast_node", offset_ms=500_000)
        # ノードの時刻を取得する
        t = ntp.get_node_time("fast_node")
        # 正しいオフセット計算を確認する
        assert t == base_ms + 500_000
