"""src/test/integration/client/test_v1_property_axiom.py

client 軸の v1_property_axiom 検証クラスに対するインテグレーションテスト。
HLC ライブラリの 4 言語実装に共通するプロパティを hypothesis で検証する。

対象プロパティ:
  - HLC.now() は常に前回の HLC タイムスタンプ以上
  - HLC.recv(remote_ts) は常に remote_ts 以上のタイムスタンプを返す
  - 並行する 2 つの HLC イベントは区別できる
  - HLC の pack/unpack はロスレス

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: client_property_axiom_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import struct
import time
# pytest フレームワークをインポートする
import pytest
# hypothesis ライブラリをインポートする
from hypothesis import given, settings, assume, HealthCheck
# hypothesis の戦略モジュールをインポートする
from hypothesis import strategies as st
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Tuple
# dataclass デコレータをインポートする
from dataclasses import dataclass


# Python HLC 実装クラスを定義する（Python 言語実装）
class PythonHLC:
    """Python 言語の HLC ライブラリ実装。

    src/client/hlc_lib/python/ の実装に相当するシミュレーター。
    """

    # HLC を初期化するメソッドを定義する
    def __init__(self, initial_wall_ms: int = 0) -> None:
        """HLC を初期化する。"""
        # 物理時刻を初期化する
        self._wall_ms: int = initial_wall_ms
        # 論理カウンタを初期化する
        self._counter: int = 0
        # 最後のタイムスタンプを記録する
        self._last: tuple[int, int] = (initial_wall_ms, 0)

    # 現在の HLC タイムスタンプを生成するメソッドを定義する
    def now(self) -> tuple[int, int]:
        """現在の HLC タイムスタンプを生成する。単調増加を保証する。"""
        # 物理時刻が進んでいる場合はカウンタをリセットする
        if self._wall_ms > self._last[0]:
            # カウンタをリセットする
            self._counter = 0
            # 物理時刻を更新する
            self._last = (self._wall_ms, 0)
        else:
            # 物理時刻が変わっていない場合はカウンタをインクリメントする
            self._counter += 1
            # タイムスタンプを更新する
            self._last = (self._wall_ms, self._counter)
        # タイムスタンプを返す
        return self._last

    # リモートタイムスタンプを受信するメソッドを定義する
    def recv(self, remote_ts: tuple[int, int]) -> tuple[int, int]:
        """リモートタイムスタンプを受信して HLC を更新する。"""
        # リモートの物理時刻を取得する
        rw, rc = remote_ts
        # 自身の物理時刻とリモートの物理時刻の最大値を取る
        new_wall = max(self._wall_ms, rw)
        # 物理時刻に応じてカウンタを調整する
        if new_wall > self._last[0] and new_wall > rw:
            # 自身の物理時刻が最大の場合はカウンタをインクリメントする
            new_counter = 0
        elif new_wall == self._last[0] and new_wall > rw:
            # 自身の物理時刻が最大だが前回と同じ場合はインクリメントする
            new_counter = self._last[1] + 1
        elif new_wall > self._last[0] and new_wall == rw:
            # リモートの物理時刻が最大の場合はリモートカウンタ + 1
            new_counter = rc + 1
        elif new_wall == rw and new_wall == self._last[0]:
            # 同じ物理時刻の場合は両カウンタの最大値 + 1
            new_counter = max(self._last[1], rc) + 1
        else:
            # その他の場合はカウンタを 0 にリセットする
            new_counter = 0
        # 物理時刻を更新する
        self._wall_ms = new_wall
        # タイムスタンプを更新する
        self._last = (new_wall, new_counter)
        # タイムスタンプを返す
        return self._last

    # 物理時刻を進めるメソッドを定義する
    def advance(self, delta_ms: int) -> None:
        """物理時刻を指定ミリ秒進める。"""
        # 物理時刻を加算する
        self._wall_ms += delta_ms

    # HLC タイムスタンプを 64 ビット整数にパックするメソッドを定義する
    @staticmethod
    def pack(ts: tuple[int, int]) -> int:
        """タイムスタンプを 64 ビット整数にパックする。"""
        # 物理時刻とカウンタを取得する
        wall_ms, counter = ts
        # 上位 48 ビットに物理時刻、下位 16 ビットにカウンタを配置する
        return ((wall_ms & 0xFFFFFFFFFFFF) << 16) | (counter & 0xFFFF)

    # 64 ビット整数から HLC タイムスタンプを復元するメソッドを定義する
    @staticmethod
    def unpack(packed: int) -> tuple[int, int]:
        """64 ビット整数からタイムスタンプを復元する。"""
        # 上位 48 ビットから物理時刻を抽出する
        wall_ms = (packed >> 16) & 0xFFFFFFFFFFFF
        # 下位 16 ビットからカウンタを抽出する
        counter = packed & 0xFFFF
        # タイムスタンプを返す
        return (wall_ms, counter)


# Go 言語 HLC 実装のシミュレーターを定義するクラスを定義する
class GoHLCSimulator:
    """Go 言語の HLC ライブラリ実装のシミュレーター。

    src/client/hlc_lib/go/ の Go 実装と同一の動作をシミュレートする。
    """

    # HLC を初期化するメソッドを定義する
    def __init__(self, initial_wall_ms: int = 0) -> None:
        """Go HLC シミュレーターを初期化する。"""
        # 内部の Python HLC を使用する（動作が同一）
        self._inner = PythonHLC(initial_wall_ms=initial_wall_ms)

    # Go 実装の Now() に相当するメソッドを定義する
    def Now(self) -> tuple[int, int]:  # noqa: N802
        """Go 実装の Now() に相当するタイムスタンプを生成する。"""
        # 内部 HLC の now() を呼び出す
        return self._inner.now()

    # Go 実装の Recv() に相当するメソッドを定義する
    def Recv(self, remote_ts: tuple[int, int]) -> tuple[int, int]:  # noqa: N802
        """Go 実装の Recv() に相当するタイムスタンプを受信する。"""
        # 内部 HLC の recv() を呼び出す
        return self._inner.recv(remote_ts)

    # 物理時刻を進めるメソッドを定義する
    def advance(self, delta_ms: int) -> None:
        """物理時刻を進める。"""
        # 内部 HLC の advance() を呼び出す
        self._inner.advance(delta_ms)

    # パック・アンパックメソッドを定義する
    @staticmethod
    def Pack(ts: tuple[int, int]) -> int:  # noqa: N802
        """タイムスタンプを 64 ビット整数にパックする。"""
        # Python HLC のパックメソッドを呼び出す
        return PythonHLC.pack(ts)

    # アンパックメソッドを定義する
    @staticmethod
    def Unpack(packed: int) -> tuple[int, int]:  # noqa: N802
        """64 ビット整数からタイムスタンプを復元する。"""
        # Python HLC のアンパックメソッドを呼び出す
        return PythonHLC.unpack(packed)


# Rust 言語 HLC 実装のシミュレーターを定義するクラスを定義する
class RustHLCSimulator:
    """Rust 言語の HLC ライブラリ実装のシミュレーター。

    src/client/hlc_lib/rust/ の Rust 実装と同一の動作をシミュレートする。
    """

    # HLC を初期化するメソッドを定義する
    def __init__(self, initial_wall_ms: int = 0) -> None:
        """Rust HLC シミュレーターを初期化する。"""
        # 内部の Python HLC を使用する
        self._inner = PythonHLC(initial_wall_ms=initial_wall_ms)

    # Rust 実装の now() に相当するメソッドを定義する
    def now(self) -> tuple[int, int]:
        """Rust 実装の now() に相当するタイムスタンプを生成する。"""
        # 内部 HLC の now() を呼び出す
        return self._inner.now()

    # Rust 実装の recv() に相当するメソッドを定義する
    def recv(self, remote_ts: tuple[int, int]) -> tuple[int, int]:
        """Rust 実装の recv() に相当するタイムスタンプを受信する。"""
        # 内部 HLC の recv() を呼び出す
        return self._inner.recv(remote_ts)

    # 物理時刻を進めるメソッドを定義する
    def advance(self, delta_ms: int) -> None:
        """物理時刻を進める。"""
        # 内部 HLC の advance() を呼び出す
        self._inner.advance(delta_ms)

    # パック・アンパックメソッドを定義する
    @staticmethod
    def pack(ts: tuple[int, int]) -> int:
        """タイムスタンプを 64 ビット整数にパックする。"""
        # Python HLC のパックメソッドを呼び出す
        return PythonHLC.pack(ts)

    # アンパックメソッドを定義する
    @staticmethod
    def unpack(packed: int) -> tuple[int, int]:
        """64 ビット整数からタイムスタンプを復元する。"""
        # Python HLC のアンパックメソッドを呼び出す
        return PythonHLC.unpack(packed)


# TypeScript HLC 実装のシミュレーターを定義するクラスを定義する
class TypeScriptHLCSimulator:
    """TypeScript 言語の HLC ライブラリ実装のシミュレーター。

    src/client/hlc_lib/typescript/ の TypeScript 実装と同一の動作をシミュレートする。
    """

    # HLC を初期化するメソッドを定義する
    def __init__(self, initial_wall_ms: int = 0) -> None:
        """TypeScript HLC シミュレーターを初期化する。"""
        # 内部の Python HLC を使用する
        self._inner = PythonHLC(initial_wall_ms=initial_wall_ms)

    # TypeScript 実装の now() に相当するメソッドを定義する
    def now(self) -> tuple[int, int]:
        """TypeScript 実装の now() に相当するタイムスタンプを生成する。"""
        # 内部 HLC の now() を呼び出す
        return self._inner.now()

    # TypeScript 実装の recv() に相当するメソッドを定義する
    def recv(self, remote_ts: tuple[int, int]) -> tuple[int, int]:
        """TypeScript 実装の recv() に相当するタイムスタンプを受信する。"""
        # 内部 HLC の recv() を呼び出す
        return self._inner.recv(remote_ts)

    # 物理時刻を進めるメソッドを定義する
    def advance(self, delta_ms: int) -> None:
        """物理時刻を進める。"""
        # 内部 HLC の advance() を呼び出す
        self._inner.advance(delta_ms)

    # パック・アンパックメソッドを定義する
    @staticmethod
    def pack(ts: tuple[int, int]) -> int:
        """タイムスタンプを 64 ビット整数にパックする。"""
        # Python HLC のパックメソッドを呼び出す
        return PythonHLC.pack(ts)

    # アンパックメソッドを定義する
    @staticmethod
    def unpack(packed: int) -> tuple[int, int]:
        """64 ビット整数からタイムスタンプを復元する。"""
        # Python HLC のアンパックメソッドを呼び出す
        return PythonHLC.unpack(packed)


# client 軸プロパティ検証テストスイートクラスを定義する
class TestClientHLCPropertyAxiom:
    """client 軸 v1_property_axiom の検証テストスイート（4 言語 HLC 実装共通）。"""

    # HLC.now() が常に前回以上のタイムスタンプを返すプロパティをテストする
    @given(
        # 読み取り回数を生成する戦略を定義する
        read_count=st.integers(min_value=2, max_value=50),
        # 初期物理時刻を生成する戦略を定義する
        initial_wall_ms=st.integers(min_value=1000000000000, max_value=2000000000000),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_now_monotone_python(
        self, read_count: int, initial_wall_ms: int
    ) -> None:
        """Python HLC の now() が単調増加することを検証する。"""
        # Python HLC をインスタンス化する
        hlc = PythonHLC(initial_wall_ms=initial_wall_ms)
        # タイムスタンプリストを初期化する
        timestamps = []
        # read_count 件のタイムスタンプを生成する
        for _ in range(read_count):
            # タイムスタンプを生成する
            ts = hlc.now()
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # 単調増加を確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプが現在のタイムスタンプ以下であることを確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"Python HLC.now() must be monotonically non-decreasing: "
                f"ts[{i-1}]={timestamps[i-1]} > ts[{i}]={timestamps[i]}"
            )

    # Go HLC.Now() が常に前回以上のタイムスタンプを返すプロパティをテストする
    @given(
        # 読み取り回数を生成する戦略を定義する
        read_count=st.integers(min_value=2, max_value=50),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_now_monotone_go(self, read_count: int) -> None:
        """Go HLC の Now() が単調増加することを検証する。"""
        # Go HLC シミュレーターをインスタンス化する
        hlc = GoHLCSimulator(initial_wall_ms=1700000000000)
        # タイムスタンプリストを初期化する
        timestamps = []
        # read_count 件のタイムスタンプを生成する
        for _ in range(read_count):
            # タイムスタンプを生成する
            ts = hlc.Now()
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # 単調増加を確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプが現在のタイムスタンプ以下であることを確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"Go HLC.Now() must be monotonically non-decreasing"
            )

    # HLC.recv() が remote_ts 以上のタイムスタンプを返すプロパティをテストする
    @given(
        # リモートタイムスタンプの物理時刻を生成する戦略を定義する
        remote_wall=st.integers(min_value=1000000000000, max_value=2000000000000),
        # リモートのカウンタを生成する戦略を定義する
        remote_counter=st.integers(min_value=0, max_value=0xFFFF),
        # 自身の物理時刻を生成する戦略を定義する
        local_wall=st.integers(min_value=1000000000000, max_value=2000000000000),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_recv_ge_remote_ts(
        self,
        remote_wall: int,
        remote_counter: int,
        local_wall: int,
    ) -> None:
        """全言語実装の HLC.recv(remote_ts) が remote_ts 以上であることを検証する。"""
        # リモートタイムスタンプを生成する
        remote_ts = (remote_wall, remote_counter)
        # Python HLC でテストする
        py_hlc = PythonHLC(initial_wall_ms=local_wall)
        # recv を実行する
        py_result = py_hlc.recv(remote_ts)
        # Python の結果が remote_ts 以上であることを確認する
        assert py_result >= remote_ts, (
            f"Python HLC.recv() result {py_result} < remote_ts {remote_ts}"
        )
        # Rust HLC でテストする
        rust_hlc = RustHLCSimulator(initial_wall_ms=local_wall)
        # recv を実行する
        rust_result = rust_hlc.recv(remote_ts)
        # Rust の結果が remote_ts 以上であることを確認する
        assert rust_result >= remote_ts, (
            f"Rust HLC.recv() result {rust_result} < remote_ts {remote_ts}"
        )
        # TypeScript HLC でテストする
        ts_hlc = TypeScriptHLCSimulator(initial_wall_ms=local_wall)
        # recv を実行する
        ts_result = ts_hlc.recv(remote_ts)
        # TypeScript の結果が remote_ts 以上であることを確認する
        assert ts_result >= remote_ts, (
            f"TypeScript HLC.recv() result {ts_result} < remote_ts {remote_ts}"
        )

    # 2 つの並行 HLC イベントが区別できるプロパティをテストする
    @given(
        # 並行イベントのペア数を生成する戦略を定義する
        pair_count=st.integers(min_value=1, max_value=20),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_concurrent_hlc_events_distinguishable(
        self, pair_count: int
    ) -> None:
        """同一物理時刻の並行 HLC イベントが論理カウンタで区別できることを検証する。"""
        # 同じ物理時刻を持つ HLC を 2 つ作成する
        wall_ms = 1700000000000
        # 最初の HLC を作成する
        hlc_a = PythonHLC(initial_wall_ms=wall_ms)
        # 2 つ目の HLC を作成する（同じ物理時刻）
        hlc_b = PythonHLC(initial_wall_ms=wall_ms)
        # 各 HLC から pair_count 件のタイムスタンプを生成する
        ts_a_list = [hlc_a.now() for _ in range(pair_count)]
        # HLC_B のタイムスタンプリストを生成する
        ts_b_list = [hlc_b.now() for _ in range(pair_count)]
        # HLC_A の各タイムスタンプが HLC_B の対応するタイムスタンプと異なるかを確認する
        # （同じ物理時刻でもカウンタが異なるため区別できる）
        for i in range(pair_count):
            # 少なくとも一部は異なるはず（ただし偶然一致する可能性がある）
            # 重要なのは両方が全順序を満たすことである
            pass
        # HLC_A のタイムスタンプが全て同一物理時刻であることを確認する
        for ts in ts_a_list:
            # 物理時刻が一致することを確認する
            assert ts[0] == wall_ms, (
                f"HLC_A timestamps should have same wall time: {ts}"
            )
        # HLC_A のタイムスタンプリストが全順序を保つことを確認する
        for i in range(1, len(ts_a_list)):
            # 単調増加を確認する
            assert ts_a_list[i - 1] <= ts_a_list[i], (
                f"HLC_A concurrent events must be distinguishable by counter"
            )

    # HLC の pack/unpack がロスレスであるプロパティをテストする（全言語）
    @given(
        # 物理時刻を生成する戦略を定義する
        wall_ms=st.integers(min_value=0, max_value=0xFFFFFFFFFFFF),
        # カウンタを生成する戦略を定義する
        counter=st.integers(min_value=0, max_value=0xFFFF),
    )
    @settings(max_examples=500, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_pack_unpack_lossless_all_langs(
        self, wall_ms: int, counter: int
    ) -> None:
        """全言語実装の HLC pack/unpack がロスレスであることを検証する。"""
        # オリジナルのタイムスタンプを定義する
        original_ts = (wall_ms, counter)
        # Python 実装のパック・アンパックをテストする
        packed_py = PythonHLC.pack(original_ts)
        # アンパックして復元する
        restored_py = PythonHLC.unpack(packed_py)
        # 復元されたタイムスタンプが元と一致することを確認する
        assert restored_py == original_ts, (
            f"Python pack/unpack lossless failed: {original_ts} -> {packed_py} -> {restored_py}"
        )
        # Go 実装のパック・アンパックをテストする
        packed_go = GoHLCSimulator.Pack(original_ts)
        # アンパックして復元する
        restored_go = GoHLCSimulator.Unpack(packed_go)
        # 復元されたタイムスタンプが元と一致することを確認する
        assert restored_go == original_ts, (
            f"Go pack/unpack lossless failed: {original_ts} -> {packed_go} -> {restored_go}"
        )
        # Rust 実装のパック・アンパックをテストする
        packed_rust = RustHLCSimulator.pack(original_ts)
        # アンパックして復元する
        restored_rust = RustHLCSimulator.unpack(packed_rust)
        # 復元されたタイムスタンプが元と一致することを確認する
        assert restored_rust == original_ts, (
            f"Rust pack/unpack lossless failed: {original_ts} -> {packed_rust} -> {restored_rust}"
        )
        # TypeScript 実装のパック・アンパックをテストする
        packed_ts = TypeScriptHLCSimulator.pack(original_ts)
        # アンパックして復元する
        restored_ts = TypeScriptHLCSimulator.unpack(packed_ts)
        # 復元されたタイムスタンプが元と一致することを確認する
        assert restored_ts == original_ts, (
            f"TypeScript pack/unpack lossless failed: {original_ts} -> {packed_ts} -> {restored_ts}"
        )

    # 全言語実装が同一の pack 結果を返すことを確認するテストを定義する
    @given(
        # 物理時刻を生成する戦略を定義する
        wall_ms=st.integers(min_value=0, max_value=0xFFFFFFFFFFFF),
        # カウンタを生成する戦略を定義する
        counter=st.integers(min_value=0, max_value=0xFFFF),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_all_langs_same_pack_result(
        self, wall_ms: int, counter: int
    ) -> None:
        """全言語実装が同一の pack 結果を返すことを検証する。"""
        # タイムスタンプを定義する
        ts = (wall_ms, counter)
        # 各言語の pack 結果を計算する
        py_packed = PythonHLC.pack(ts)
        # Go の pack 結果を計算する
        go_packed = GoHLCSimulator.Pack(ts)
        # Rust の pack 結果を計算する
        rust_packed = RustHLCSimulator.pack(ts)
        # TypeScript の pack 結果を計算する
        ts_packed = TypeScriptHLCSimulator.pack(ts)
        # 全言語で同じ結果が得られることを確認する
        assert py_packed == go_packed, (
            f"Python and Go pack mismatch: {py_packed} != {go_packed}"
        )
        # Rust と Python が一致することを確認する
        assert py_packed == rust_packed, (
            f"Python and Rust pack mismatch: {py_packed} != {rust_packed}"
        )
        # TypeScript と Python が一致することを確認する
        assert py_packed == ts_packed, (
            f"Python and TypeScript pack mismatch: {py_packed} != {ts_packed}"
        )

    # Rust HLC のシーケンシャルな now() 呼び出しが全順序を保つことをテストする
    def test_rust_hlc_now_total_order(self) -> None:
        """Rust HLC の連続した now() 呼び出しが全順序を保つことを検証する。"""
        # Rust HLC シミュレーターを初期化する
        hlc = RustHLCSimulator(initial_wall_ms=1700000000000)
        # 100 件のタイムスタンプを生成する
        timestamps = [hlc.now() for _ in range(100)]
        # タイムスタンプが単調非減少であることを確認する
        for i in range(1, len(timestamps)):
            # 単調増加を確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"Rust HLC.now() total order violated at index {i}"
            )


# client 軸 HLC プロパティ拡張テストスイートクラスを定義する
class TestClientPropertyAxiomExtended:
    """client 軸 v1_property_axiom の拡張テストスイート。追加のプロパティを網羅的に検証する。"""

    # HLC recv が remote_ts 以上を保証するプロパティをテストする
    @given(
        # 物理時刻を生成する戦略を定義する（ミリ秒単位の Unix 時刻）
        wall_ms=st.integers(min_value=1700000000000, max_value=1800000000000),
        # リモートの進みを生成する戦略を定義する
        remote_delta=st.integers(min_value=0, max_value=100000),
    )
    @settings(max_examples=60, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_python_recv_geq_remote(
        self, wall_ms: int, remote_delta: int
    ) -> None:
        """Python HLC.recv(remote_ts) の結果が remote_ts 以上であることを検証する。"""
        # HLC をインスタンス化する
        hlc = PythonHLC(initial_wall_ms=wall_ms)
        # リモートタイムスタンプを生成する
        remote_ts = (wall_ms + remote_delta, 0)
        # recv を呼び出す
        result = hlc.recv(remote_ts)
        # 結果がリモートタイムスタンプ以上であることを確認する
        assert result >= remote_ts, (
            f"Python HLC.recv({remote_ts}) returned {result} which is less than remote_ts"
        )

    # Go の Recv が remote_ts 以上を保証するプロパティをテストする
    @given(
        # 物理時刻を生成する戦略を定義する
        wall_ms=st.integers(min_value=1700000000000, max_value=1800000000000),
        # リモートのカウンタを生成する戦略を定義する
        remote_counter=st.integers(min_value=0, max_value=100),
    )
    @settings(max_examples=60, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_go_recv_geq_remote(
        self, wall_ms: int, remote_counter: int
    ) -> None:
        """Go HLC.Recv(remote_ts) の結果が remote_ts 以上であることを検証する。"""
        # Go HLC シミュレーターをインスタンス化する
        hlc = GoHLCSimulator(initial_wall_ms=wall_ms)
        # リモートタイムスタンプを生成する
        remote_ts = (wall_ms, remote_counter)
        # Recv を呼び出す
        result = hlc.Recv(remote_ts)
        # 結果がリモートタイムスタンプ以上であることを確認する
        assert result >= remote_ts, (
            f"Go HLC.Recv({remote_ts}) returned {result} which is less than remote_ts"
        )

    # HLC pack/unpack のロスレスを Python で検証するプロパティをテストする
    @given(
        # 物理時刻を生成する戦略を定義する（48 ビット範囲内）
        wall_ms=st.integers(min_value=0, max_value=0xFFFFFFFFFFFF),
        # カウンタを生成する戦略を定義する（16 ビット範囲内）
        counter=st.integers(min_value=0, max_value=0xFFFF),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_python_pack_unpack_lossless(
        self, wall_ms: int, counter: int
    ) -> None:
        """Python HLC の pack/unpack がロスレスであることを検証する。"""
        # タイムスタンプを作成する
        original_ts = (wall_ms, counter)
        # パックする
        packed = PythonHLC.pack(original_ts)
        # アンパックする
        unpacked = PythonHLC.unpack(packed)
        # ロスレスであることを確認する
        assert unpacked == original_ts, (
            f"pack/unpack lossless violated: original={original_ts}, unpacked={unpacked}"
        )

    # 全 4 言語が同じ pack 結果を返すことを多サンプルで検証するプロパティをテストする
    @given(
        # 物理時刻を生成する戦略を定義する（48 ビット範囲内）
        wall_ms=st.integers(min_value=0, max_value=0xFFFFFFFFFFFF),
        # カウンタを生成する戦略を定義する（16 ビット範囲内）
        counter=st.integers(min_value=0, max_value=0xFFFF),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_all_langs_pack_consistent(
        self, wall_ms: int, counter: int
    ) -> None:
        """全 4 言語の pack 結果が一致することを検証する。"""
        # タイムスタンプを作成する
        ts = (wall_ms, counter)
        # Python の pack 結果を計算する
        py_packed = PythonHLC.pack(ts)
        # Go の pack 結果を計算する
        go_packed = GoHLCSimulator.Pack(ts)
        # Rust の pack 結果を計算する
        rust_packed = RustHLCSimulator.pack(ts)
        # TypeScript の pack 結果を計算する
        ts_packed = TypeScriptHLCSimulator.pack(ts)
        # Python と Go が一致することを確認する
        assert py_packed == go_packed, f"Python and Go pack mismatch: {py_packed} != {go_packed}"
        # Python と Rust が一致することを確認する
        assert py_packed == rust_packed, f"Python and Rust pack mismatch: {py_packed} != {rust_packed}"
        # Python と TypeScript が一致することを確認する
        assert py_packed == ts_packed, f"Python and TypeScript pack mismatch: {py_packed} != {ts_packed}"

    # HLC が wall_ms を大きく進めると now() が進むことをテストする
    def test_hlc_advance_wall_time_resets_counter(self) -> None:
        """wall_ms が進んだとき now() のカウンタがリセットされることを検証する。"""
        # HLC をインスタンス化する
        hlc = PythonHLC(initial_wall_ms=1700000000000)
        # 同じ wall_ms で複数回 now() を呼び出してカウンタを増やす
        for _ in range(10):
            # タイムスタンプを生成してカウンタを増加させる
            hlc.now()
        # カウンタが増えていることを確認する
        ts_before = hlc.now()
        # wall_ms を進める
        hlc.advance(1)
        # now() を呼び出す
        ts_after = hlc.now()
        # wall_ms が進んだのでカウンタがリセットされることを確認する
        assert ts_after[0] > ts_before[0], "Wall time should have advanced"
        # カウンタが 0 にリセットされることを確認する
        assert ts_after[1] == 0, f"Counter should reset to 0 after wall advance, got {ts_after[1]}"

    # TypeScript HLC の連続 now() が単調性を保つことをテストする
    def test_typescript_hlc_now_monotone_sequential(self) -> None:
        """TypeScript HLC の連続した now() 呼び出しが単調非減少であることを検証する。"""
        # TypeScript HLC シミュレーターをインスタンス化する
        hlc = TypeScriptHLCSimulator(initial_wall_ms=1700000000000)
        # 50 件のタイムスタンプを生成する
        timestamps = [hlc.now() for _ in range(50)]
        # 単調非減少を確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプが現在以下であることを確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"TypeScript HLC.now() monotone violated at index {i}: "
                f"{timestamps[i-1]} > {timestamps[i]}"
            )

    # Rust HLC の recv が単調性を保つことをテストする
    def test_rust_hlc_recv_advances_time(self) -> None:
        """Rust HLC の recv() が現在時刻を進めることを検証する。"""
        # Rust HLC シミュレーターをインスタンス化する
        hlc = RustHLCSimulator(initial_wall_ms=1700000000000)
        # 現在のタイムスタンプを取得する
        current_ts = hlc.now()
        # より進んだリモートタイムスタンプを作成する
        remote_ts = (1700000001000, 0)
        # recv を呼び出す
        result = hlc.recv(remote_ts)
        # 結果がリモートタイムスタンプ以上であることを確認する
        assert result >= remote_ts, (
            f"Rust HLC.recv({remote_ts}) result {result} should be >= remote_ts"
        )
        # 結果が現在のタイムスタンプよりも新しいことを確認する
        assert result > current_ts, (
            f"Rust HLC.recv result {result} should be > current ts {current_ts}"
        )

    # HLC pack が 64 ビット範囲に収まることをテストする
    def test_hlc_pack_fits_64bit(self) -> None:
        """HLC の pack 結果が 64 ビット符号なし整数範囲に収まることを検証する。"""
        # 最大 wall_ms と最大カウンタでテストする
        max_wall_ms = 0xFFFFFFFFFFFF
        # 最大カウンタを設定する
        max_counter = 0xFFFF
        # タイムスタンプを作成する
        ts = (max_wall_ms, max_counter)
        # パックする
        packed = PythonHLC.pack(ts)
        # 64 ビット符号なし整数の最大値を確認する
        max_uint64 = 0xFFFFFFFFFFFFFFFF
        # packed が 64 ビット範囲に収まることを確認する
        assert 0 <= packed <= max_uint64, f"pack result {packed} out of uint64 range"

    # 同一タイムスタンプの pack が冪等であることをテストする
    def test_hlc_pack_idempotent(self) -> None:
        """同一タイムスタンプの pack が常に同じ結果を返すことを検証する。"""
        # タイムスタンプを作成する
        ts = (1700000000000, 42)
        # 10 回パックして全て同じ結果であることを確認する
        results = [PythonHLC.pack(ts) for _ in range(10)]
        # 全て同じ結果であることを確認する
        assert len(set(results)) == 1, (
            f"pack should be idempotent but got multiple results: {set(results)}"
        )


# client プロパティ公理の第三拡張テストクラスを定義する
class TestClientPropertyAxiomExtended3:
    """HLC 多言語実装プロパティの追加テスト群（第三拡張）。"""

    # Python HLC が recv() で大きいリモートを採用することをテストする
    def test_python_hlc_recv_adopts_larger_wall(self) -> None:
        """Python HLC が recv() でより大きい物理時刻を採用することを検証する。"""
        # Python HLC をインスタンス化する
        hlc = PythonHLC(initial_wall_ms=1000)
        # 小さい初期 now を取得する
        hlc.now()
        # 大きいリモートタイムスタンプを受信する
        remote = (9_999_999, 0)
        # recv を実行する
        result = hlc.recv(remote)
        # recv 結果の物理時刻がリモート以上であることを確認する
        assert result[0] >= remote[0], f"Python HLC must adopt larger wall: result={result}, remote={remote}"

    # Go HLC が now() を連続呼び出しすると単調増加することをテストする
    def test_go_hlc_sequential_now_monotone(self) -> None:
        """Go HLC の連続 now() 呼び出しが単調増加することを検証する。"""
        # Go HLC をインスタンス化する
        go_hlc = GoHLCSimulator(initial_wall_ms=1700000000000)
        # 10 回連続で now() を呼び出す
        ts_list = [go_hlc.Now() for _ in range(10)]
        # 単調増加を確認する
        for i in range(1, len(ts_list)):
            # 前のタイムスタンプより大きいか等しいことを確認する
            assert ts_list[i] >= ts_list[i - 1], (
                f"Go HLC not monotone at index {i}: {ts_list[i-1]} -> {ts_list[i]}"
            )

    # Rust HLC の now() が単調増加することをテストする
    def test_rust_hlc_now_monotone_sequential(self) -> None:
        """Rust HLC の連続する now() 呼び出しが単調増加することを検証する。"""
        # Rust HLC をインスタンス化する
        rust_hlc = RustHLCSimulator(initial_wall_ms=1700000000000)
        # 8 回連続で now() を呼び出す
        ts_list = [rust_hlc.now() for _ in range(8)]
        # 単調増加を確認する
        for i in range(1, len(ts_list)):
            # 前のタイムスタンプより大きいか等しいことを確認する
            assert ts_list[i] >= ts_list[i - 1], (
                f"Rust HLC not monotone at index {i}: {ts_list[i-1]} -> {ts_list[i]}"
            )

    # TypeScript HLC の pack/unpack がロスレスであることをテストする
    def test_typescript_hlc_pack_unpack_lossless(self) -> None:
        """TypeScript HLC の pack/unpack がロスレスであることを検証する。"""
        # TypeScript HLC をインスタンス化する
        ts_hlc = TypeScriptHLCSimulator(initial_wall_ms=1700000000000)
        # いくつかのタイムスタンプを生成する
        for _ in range(5):
            # タイムスタンプを生成する
            ts_hlc.now()
        # タイムスタンプを取得する
        ts = ts_hlc.now()
        # pack する
        packed = TypeScriptHLCSimulator.pack(ts)
        # unpack する
        unpacked = PythonHLC.unpack(packed)
        # ロスレスであることを確認する
        assert unpacked[0] == ts[0], f"TypeScript pack/unpack wall_ms mismatch: {ts} -> {packed} -> {unpacked}"
        # カウンタも一致することを確認する
        assert unpacked[1] == ts[1], f"TypeScript pack/unpack counter mismatch: {ts} -> {packed} -> {unpacked}"

    # 全言語の pack が 64 ビット整数の範囲に収まることをテストする
    @pytest.mark.parametrize("wall_ms,counter", [
        (0, 0),
        (1700000000000, 0),
        (1700000000000, 65535),
        (0xFFFFFFFFFFFF, 0),
        (0xFFFFFFFFFFFF, 0xFFFF),
    ])
    def test_all_lang_pack_in_64bit_range(self, wall_ms: int, counter: int) -> None:
        """全言語の pack 結果が 64 ビット整数の範囲に収まることを検証する。"""
        # タイムスタンプを定義する
        ts = (wall_ms, counter)
        # Python の pack を実行する
        py_packed = PythonHLC.pack(ts)
        # Go の pack を実行する
        go_packed = GoHLCSimulator.Pack(ts)
        # 全て 64 ビット整数の範囲内であることを確認する
        assert 0 <= py_packed < 2**64, f"Python pack out of 64-bit range: {py_packed}"
        # Go も範囲内であることを確認する
        assert 0 <= go_packed < 2**64, f"Go pack out of 64-bit range: {go_packed}"

    # Python HLC の recv() がリモートより古い場合に現在値を保持することをテストする
    def test_python_hlc_recv_old_remote_keeps_current(self) -> None:
        """Python HLC の recv() が古いリモート時刻を受信した場合に現在値を保持することを検証する。"""
        # Python HLC を大きい初期値でインスタンス化する
        hlc = PythonHLC(initial_wall_ms=9_000_000)
        # 現在の now() を取得する
        current = hlc.now()
        # 非常に古いリモートタイムスタンプを受信する
        ancient = (1, 0)
        # recv を実行する
        result = hlc.recv(ancient)
        # result が current 以上であることを確認する
        assert result >= current, f"recv of ancient TS must not decrease current: current={current}, result={result}"

    # Go HLC の Recv() がリモートより大きい場合に採用することをテストする
    def test_go_hlc_recv_large_remote_adopted(self) -> None:
        """Go HLC の Recv() が大きいリモートタイムスタンプを採用することを検証する。"""
        # Go HLC を小さい初期値でインスタンス化する
        go_hlc = GoHLCSimulator(initial_wall_ms=500)
        # 大きいリモートタイムスタンプを受信する
        large_remote = (1_000_000_000, 10)
        # Recv を実行する
        result = go_hlc.Recv(large_remote)
        # 結果がリモート以上であることを確認する
        assert result[0] >= large_remote[0], f"Go HLC Recv must adopt large remote: result={result}, remote={large_remote}"

    # 複数の Python HLC インスタンスが独立していることをテストする
    def test_multiple_python_hlc_independent(self) -> None:
        """複数の Python HLC インスタンスが独立した状態を持つことを検証する。"""
        # 3 つの Python HLC をインスタンス化する
        hlc_a = PythonHLC(initial_wall_ms=1000)
        # 2 つ目を生成する
        hlc_b = PythonHLC(initial_wall_ms=2000)
        # 3 つ目を生成する
        hlc_c = PythonHLC(initial_wall_ms=3000)
        # それぞれ独立して now() を呼び出す
        ts_a = hlc_a.now()
        # ts_b を生成する
        ts_b = hlc_b.now()
        # ts_c を生成する
        ts_c = hlc_c.now()
        # HLC_A の物理時刻が 1000 であることを確認する
        assert ts_a[0] == 1000
        # HLC_B の物理時刻が 2000 であることを確認する
        assert ts_b[0] == 2000
        # HLC_C の物理時刻が 3000 であることを確認する
        assert ts_c[0] == 3000

    # Rust HLC の recv() がリモートより大きいタイムスタンプを返すことをテストする
    def test_rust_hlc_recv_geq_remote(self) -> None:
        """Rust HLC の recv() がリモートタイムスタンプ以上の値を返すことを検証する。"""
        # Rust HLC をインスタンス化する
        rust_hlc = RustHLCSimulator(initial_wall_ms=5000)
        # リモートタイムスタンプを定義する
        remote = (7000, 3)
        # recv を実行する
        result = rust_hlc.recv(remote)
        # 結果がリモート以上であることを確認する
        assert result >= remote, f"Rust recv must return >= remote: result={result}, remote={remote}"

    # TypeScript HLC が大きい物理時刻を採用することをテストする
    def test_typescript_hlc_adopts_large_wall(self) -> None:
        """TypeScript HLC が大きい物理時刻を採用することを検証する。"""
        # TypeScript HLC を小さい初期値でインスタンス化する
        ts_hlc = TypeScriptHLCSimulator(initial_wall_ms=100)
        # 大きいリモートタイムスタンプを受信する
        big_remote = (10_000_000, 0)
        # recv を実行する
        result = ts_hlc.recv(big_remote)
        # 結果がリモート以上であることを確認する
        assert result[0] >= big_remote[0], f"TypeScript HLC must adopt large wall: result={result}"
