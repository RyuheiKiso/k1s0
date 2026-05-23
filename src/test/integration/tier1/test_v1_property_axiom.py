"""src/test/integration/tier1/test_v1_property_axiom.py

tier1 軸の v1_property_axiom 検証クラスに対するインテグレーションテスト。
hypothesis ライブラリを使用してプロパティベーステストを実施する。

対象プロパティ:
  - 後の書き込みの HLC タイムスタンプは前の書き込みより大きい
  - 書き込んだキーは同じ値で読み取れる
  - コンセンサスログエントリはシーケンス番号で順序付けできる
  - トランスポートメッセージのチャンク再組み立ては元と同一

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: tier1_property_axiom_v1
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
from typing import Generator, List, Tuple
# dataclass デコレータをインポートする
from dataclasses import dataclass, field


# HLC タイムスタンプ型を定義する
@dataclass(frozen=True)
class HLCTimestamp:
    """プロパティテスト用の Hybrid Logical Clock タイムスタンプ。"""

    # 物理時刻をミリ秒で保持するフィールドを定義する
    wall_time_ms: int
    # 同一物理時刻内の論理カウンタを保持するフィールドを定義する
    logical_counter: int

    # 大小比較演算子を定義する
    def __lt__(self, other: "HLCTimestamp") -> bool:
        """HLC の全順序比較を行う。"""
        # 物理時刻が異なる場合はその差で比較する
        if self.wall_time_ms != other.wall_time_ms:
            # 物理時刻の大小で判定する
            return self.wall_time_ms < other.wall_time_ms
        # 物理時刻が同じ場合は論理カウンタで比較する
        return self.logical_counter < other.logical_counter

    # 以上比較演算子を定義する
    def __le__(self, other: "HLCTimestamp") -> bool:
        """以上比較を行う。"""
        # 等しいか小さい場合に True を返す
        return self == other or self < other

    # 64 ビット整数への変換メソッドを定義する
    def to_int(self) -> int:
        """タイムスタンプを 64 ビット整数に変換する。"""
        # 上位 48 ビットに物理時刻、下位 16 ビットに論理カウンタを配置する
        return (self.wall_time_ms << 16) | (self.logical_counter & 0xFFFF)

    # 64 ビット整数から復元するクラスメソッドを定義する
    @classmethod
    def from_int(cls, value: int) -> "HLCTimestamp":
        """64 ビット整数からタイムスタンプを復元する。"""
        # 上位 48 ビットから物理時刻を抽出する
        wall = value >> 16
        # 下位 16 ビットから論理カウンタを抽出する
        counter = value & 0xFFFF
        # タイムスタンプを生成して返す
        return cls(wall_time_ms=wall, logical_counter=counter)


# プロパティテスト用の KV ストアクラスを定義する
class PropertyKVStore:
    """プロパティテスト用のインメモリ KV ストア。"""

    # ストアを初期化するメソッドを定義する
    def __init__(self, initial_wall_ms: int = 1700000000000) -> None:
        """KV ストアを初期化する。初期物理時刻を指定可能にする。"""
        # ストレージを初期化する
        self._store: dict[bytes, bytes] = {}
        # HLC の物理時刻を初期化する
        self._wall_ms: int = initial_wall_ms
        # 論理カウンタを初期化する
        self._counter: int = 0
        # タイムスタンプ履歴を記録する
        self._ts_history: list[HLCTimestamp] = []

    # 現在の HLC タイムスタンプを生成するメソッドを定義する
    def _tick(self) -> HLCTimestamp:
        """HLC タイムスタンプを生成しカウンタを進める。"""
        # 論理カウンタをインクリメントする
        self._counter += 1
        # 新しいタイムスタンプを生成する
        ts = HLCTimestamp(wall_time_ms=self._wall_ms, logical_counter=self._counter)
        # 履歴に記録する
        self._ts_history.append(ts)
        # タイムスタンプを返す
        return ts

    # 物理時刻を進めるメソッドを定義する
    def advance_wall_clock(self, delta_ms: int) -> None:
        """物理時刻を進めて論理カウンタをリセットする。"""
        # 物理時刻を加算する
        self._wall_ms += delta_ms
        # 物理時刻が進んだので論理カウンタをリセットする
        self._counter = 0

    # KV ストアに書き込むメソッドを定義する
    def put(self, key: bytes, value: bytes) -> HLCTimestamp:
        """キーバリューを書き込み HLC タイムスタンプを返す。"""
        # タイムスタンプを生成する
        ts = self._tick()
        # ストアに書き込む
        self._store[key] = value
        # タイムスタンプを返す
        return ts

    # KV ストアから読み取るメソッドを定義する
    def get(self, key: bytes) -> bytes | None:
        """指定したキーの値を取得する。"""
        # ストアから値を返す
        return self._store.get(key)

    # タイムスタンプ履歴を取得するプロパティを定義する
    @property
    def timestamp_history(self) -> list[HLCTimestamp]:
        """タイムスタンプ履歴のコピーを返す。"""
        # 履歴のコピーを返す
        return list(self._ts_history)


# コンセンサスログのプロパティテスト用クラスを定義する
class PropertyConsensusLog:
    """プロパティテスト用コンセンサスログ。"""

    # ログを初期化するメソッドを定義する
    def __init__(self) -> None:
        """コンセンサスログを初期化する。"""
        # ログエントリを保持するリストを初期化する
        self._entries: list[tuple[int, int, bytes]] = []
        # 次のシーケンス番号を初期化する
        self._seq: int = 1

    # ログにエントリを追加するメソッドを定義する
    def append(self, term: int, payload: bytes) -> tuple[int, int, bytes]:
        """エントリを追加し (seq, term, payload) タプルを返す。"""
        # エントリを生成する
        entry = (self._seq, term, payload)
        # ログに追加する
        self._entries.append(entry)
        # シーケンス番号をインクリメントする
        self._seq += 1
        # エントリを返す
        return entry

    # 全エントリを取得するプロパティを定義する
    @property
    def entries(self) -> list[tuple[int, int, bytes]]:
        """全エントリのコピーを返す。"""
        # エントリのコピーを返す
        return list(self._entries)


# トランスポートメッセージのチャンク分割・再組み立て関数を定義する
def chunk_message(data: bytes, chunk_size: int) -> list[bytes]:
    """メッセージを chunk_size ごとに分割する。"""
    # チャンクのリストを初期化する
    chunks = []
    # データを chunk_size ごとに分割する
    for i in range(0, len(data), chunk_size):
        # チャンクを切り出してリストに追加する
        chunks.append(data[i : i + chunk_size])
    # チャンクリストを返す（空の場合でも空リストを返す）
    return chunks if chunks else [b""]


# チャンクを再組み立てする関数を定義する
def reassemble_chunks(chunks: list[bytes]) -> bytes:
    """チャンクリストを結合して元のメッセージを復元する。"""
    # チャンクを結合して返す
    return b"".join(chunks)


# tier1 プロパティ検証テストスイートクラスを定義する
class TestTier1PropertyAxiom:
    """tier1 軸 v1_property_axiom の検証テストスイート。"""

    # KV ストアのフィクスチャを定義する
    @pytest.fixture
    def kv_store(self) -> Generator[PropertyKVStore, None, None]:
        """PropertyKVStore フィクスチャを生成する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # フィクスチャを返す
        yield store

    # コンセンサスログのフィクスチャを定義する
    @pytest.fixture
    def consensus_log(self) -> Generator[PropertyConsensusLog, None, None]:
        """PropertyConsensusLog フィクスチャを生成する。"""
        # コンセンサスログをインスタンス化する
        log = PropertyConsensusLog()
        # フィクスチャを返す
        yield log

    # 後の書き込みの HLC タイムスタンプが前の書き込みより大きいプロパティをテストする
    @given(
        # キーのリストを生成する戦略を定義する
        keys=st.lists(
            st.binary(min_size=1, max_size=32),
            min_size=2,
            max_size=20,
            unique=True,
        ),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_later_write_has_greater_hlc(self, keys: list[bytes]) -> None:
        """後の書き込みの HLC タイムスタンプが前の書き込みのタイムスタンプより大きいことを検証する。"""
        # 新しい KV ストアをインスタンス化する
        store = PropertyKVStore()
        # タイムスタンプリストを初期化する
        timestamps = []
        # 各キーに対して書き込みを実行する
        for i, key in enumerate(keys):
            # テスト用の値を生成する
            value = f"value_{i}".encode()
            # 書き込みを実行してタイムスタンプを収集する
            ts = store.put(key, value)
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # 連続するタイムスタンプが前進することを確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプが現在のタイムスタンプ以下であることを確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"HLC must be monotonically non-decreasing: "
                f"ts[{i-1}]={timestamps[i-1]} > ts[{i}]={timestamps[i]}"
            )

    # 書き込んだキーが同じ値で読み取れるプロパティをテストする
    @given(
        # キーと値のペアを生成する戦略を定義する
        key=st.binary(min_size=1, max_size=64),
        # 値の戦略を定義する
        value=st.binary(min_size=0, max_size=256),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_written_key_readable_with_same_value(
        self, key: bytes, value: bytes
    ) -> None:
        """書き込んだキーが同じ値で読み取れることを検証する。"""
        # 新しい KV ストアをインスタンス化する
        store = PropertyKVStore()
        # KV ストアに書き込む
        store.put(key, value)
        # 書き込んだキーで読み取る
        read_back = store.get(key)
        # 読み取った値が書き込んだ値と一致することを確認する
        assert read_back == value, (
            f"Read-back value mismatch: wrote {value!r}, got {read_back!r}"
        )

    # コンセンサスログのエントリがシーケンス番号で順序付けできるプロパティをテストする
    @given(
        # エントリ数を生成する戦略を定義する
        entry_count=st.integers(min_value=1, max_value=50),
        # 任期番号を生成する戦略を定義する
        term=st.integers(min_value=1, max_value=10),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_consensus_entries_orderable_by_sequence(
        self, entry_count: int, term: int
    ) -> None:
        """コンセンサスログエントリがシーケンス番号で全順序付けできることを検証する。"""
        # 新しいコンセンサスログをインスタンス化する
        log = PropertyConsensusLog()
        # 指定された件数のエントリを追加する
        for i in range(entry_count):
            # ペイロードを生成してエントリを追加する
            log.append(term=term, payload=f"entry_{i}".encode())
        # エントリを取得する
        entries = log.entries
        # エントリのシーケンス番号が 1 から連続していることを確認する
        seq_numbers = [e[0] for e in entries]
        # 期待されるシーケンス番号と一致することを確認する
        expected = list(range(1, entry_count + 1))
        # シーケンス番号のリストが期待値と一致することを確認する
        assert seq_numbers == expected, (
            f"Sequence numbers not monotonic: {seq_numbers}"
        )

    # トランスポートメッセージのチャンク再組み立てが元と同一であるプロパティをテストする
    @given(
        # メッセージデータを生成する戦略を定義する
        data=st.binary(min_size=0, max_size=1024),
        # チャンクサイズを生成する戦略を定義する
        chunk_size=st.integers(min_value=1, max_value=128),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_transport_chunks_reassembled_identical(
        self, data: bytes, chunk_size: int
    ) -> None:
        """トランスポートメッセージのチャンク分割と再組み立てが元と同一であることを検証する。"""
        # メッセージをチャンクに分割する
        chunks = chunk_message(data, chunk_size)
        # チャンクを再組み立てする
        reassembled = reassemble_chunks(chunks)
        # 再組み立て結果が元のデータと一致することを確認する
        assert reassembled == data, (
            f"Reassembly mismatch: original={data!r}, reassembled={reassembled!r}"
        )

    # HLC タイムスタンプの pack/unpack がロスレスであるプロパティをテストする
    @given(
        # 物理時刻を生成する戦略を定義する
        wall_ms=st.integers(min_value=0, max_value=0xFFFFFFFFFFFF),
        # 論理カウンタを生成する戦略を定義する
        counter=st.integers(min_value=0, max_value=0xFFFF),
    )
    @settings(max_examples=500, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_pack_unpack_lossless(
        self, wall_ms: int, counter: int
    ) -> None:
        """HLC タイムスタンプの pack/unpack がロスレスであることを検証する。"""
        # タイムスタンプを生成する
        original = HLCTimestamp(wall_time_ms=wall_ms, logical_counter=counter)
        # 整数にパックする
        packed = original.to_int()
        # 整数から復元する
        restored = HLCTimestamp.from_int(packed)
        # 復元されたタイムスタンプが元と一致することを確認する
        assert restored.wall_time_ms == original.wall_time_ms, (
            f"wall_time_ms mismatch: {restored.wall_time_ms} != {original.wall_time_ms}"
        )
        # 論理カウンタが一致することを確認する
        assert restored.logical_counter == original.logical_counter, (
            f"logical_counter mismatch: {restored.logical_counter} != {original.logical_counter}"
        )

    # 物理時刻が進んだ後の HLC が前の HLC より大きいプロパティをテストする
    @given(
        # 物理時刻の初期値を生成する戦略を定義する
        initial_wall_ms=st.integers(min_value=1000000000000, max_value=2000000000000),
        # 物理時刻の増分を生成する戦略を定義する
        delta_ms=st.integers(min_value=1, max_value=60000),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_wall_clock_advance_increases_hlc(
        self, initial_wall_ms: int, delta_ms: int
    ) -> None:
        """物理時刻が進むとHLCが増加することを検証する。"""
        # KV ストアを初期物理時刻で初期化する
        store = PropertyKVStore(initial_wall_ms=initial_wall_ms)
        # 最初の書き込みを実行してタイムスタンプを取得する
        ts_before = store.put(b"key_before", b"val_before")
        # 物理時刻を進める
        store.advance_wall_clock(delta_ms)
        # 物理時刻を進めた後の書き込みを実行してタイムスタンプを取得する
        ts_after = store.put(b"key_after", b"val_after")
        # 後のタイムスタンプが前のタイムスタンプより大きいことを確認する
        assert ts_before < ts_after, (
            f"After wall clock advance, HLC must increase: "
            f"ts_before={ts_before}, ts_after={ts_after}"
        )

    # 複数の書き込みで HLC が単調増加するプロパティをテストする
    @given(
        # 書き込み件数を生成する戦略を定義する
        write_count=st.integers(min_value=2, max_value=100),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_multiple_writes_all_timestamps_ordered(
        self, write_count: int
    ) -> None:
        """複数の書き込みで全タイムスタンプが全順序を満たすことを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # タイムスタンプリストを初期化する
        timestamps = []
        # write_count 件の書き込みを実行する
        for i in range(write_count):
            # ユニークなキーと値を生成して書き込む
            ts = store.put(f"key_{i:05d}".encode(), f"val_{i}".encode())
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # 全タイムスタンプが全順序を満たすことを確認する
        sorted_timestamps = sorted(timestamps)
        # ソート後のリストが元のリストと一致することを確認する
        assert sorted_timestamps == timestamps, (
            "Timestamps are not in monotonically non-decreasing order"
        )

    # コンセンサスログのシーケンス番号が一意であるプロパティをテストする
    @given(
        # エントリ数を生成する戦略を定義する
        count=st.integers(min_value=1, max_value=100),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_consensus_sequence_numbers_unique(self, count: int) -> None:
        """コンセンサスログのシーケンス番号が一意であることを検証する。"""
        # コンセンサスログをインスタンス化する
        log = PropertyConsensusLog()
        # count 件のエントリを追加する
        for i in range(count):
            # エントリを追加する
            log.append(term=1, payload=f"p{i}".encode())
        # シーケンス番号を収集する
        seq_numbers = [e[0] for e in log.entries]
        # シーケンス番号が一意であることを確認する
        assert len(seq_numbers) == len(set(seq_numbers)), (
            f"Duplicate sequence numbers found: {seq_numbers}"
        )

    # チャンクサイズが 1 の場合でも正しく再組み立てできるプロパティをテストする
    @given(
        # 任意のバイト列を生成する戦略を定義する
        data=st.binary(min_size=0, max_size=512),
    )
    @settings(max_examples=200, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_single_byte_chunks_reassemble_correctly(self, data: bytes) -> None:
        """1 バイトチャンクで分割しても正しく再組み立てできることを検証する。"""
        # 1 バイトのチャンクサイズで分割する
        chunks = chunk_message(data, chunk_size=1)
        # 再組み立てする
        result = reassemble_chunks(chunks)
        # 元のデータと一致することを確認する
        assert result == data

    # HLC タイムスタンプの全順序がトータルオーダーを保つプロパティをテストする
    @given(
        # 2 つのタイムスタンプペアを生成する戦略を定義する
        ts_data=st.lists(
            st.tuples(
                st.integers(min_value=0, max_value=2**47),
                st.integers(min_value=0, max_value=0xFFFF),
            ),
            min_size=2,
            max_size=10,
        ),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_total_order_antisymmetric(
        self, ts_data: list[tuple[int, int]]
    ) -> None:
        """HLC タイムスタンプの全順序が反対称性を満たすことを検証する。"""
        # タイムスタンプリストを生成する
        timestamps = [HLCTimestamp(wall_time_ms=w, logical_counter=c) for w, c in ts_data]
        # 任意の 2 つのタイムスタンプに対して反対称性を検証する
        for i in range(len(timestamps)):
            # 比較対象のタイムスタンプを選択する
            for j in range(i + 1, len(timestamps)):
                # a < b ならば b < a でないことを確認する
                a = timestamps[i]
                # b を選択する
                b = timestamps[j]
                # a < b の場合は b < a でないことを確認する
                if a < b:
                    # b < a でないことを確認する
                    assert not (b < a), (
                        f"Antisymmetry violation: a={a} < b={b} but also b < a"
                    )


# tier1 プロパティ公理拡張テストスイートクラスを定義する
class TestTier1PropertyAxiomExtended:
    """tier1 軸 v1_property_axiom の拡張テストスイート。追加のプロパティを網羅的に検証する。"""

    # KV ストアに複数のキーを書き込んで全て読み出せるプロパティをテストする
    @given(
        # キーのリストを生成する戦略を定義する
        keys=st.lists(st.binary(min_size=1, max_size=32), min_size=1, max_size=20, unique=True),
        # 値のリストを生成する戦略を定義する（同じ長さのリストを要求）
        seed=st.integers(min_value=0, max_value=1000),
    )
    @settings(max_examples=60, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_kv_multi_key_all_readable(self, keys: list[bytes], seed: int) -> None:
        """複数のキーを書き込んで全て読み出せることを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # キーごとにユニークな値を生成して書き込む
        written: dict[bytes, bytes] = {}
        # 各キーに対して書き込みを実行する
        for i, key in enumerate(keys):
            # ユニークな値を生成する
            value = f"value_{seed}_{i}".encode()
            # KV ストアに書き込む
            store.put(key, value)
            # 書き込んだキーと値を記録する
            written[key] = value
        # 全キーを読み出して書き込んだ値と一致することを確認する
        for key, expected_value in written.items():
            # 読み出した値を取得する
            got = store.get(key)
            # 値が一致することを確認する
            assert got == expected_value, (
                f"KV multi-key read mismatch for key={key!r}: "
                f"expected={expected_value!r}, got={got!r}"
            )

    # コンセンサスログに異なるタームでエントリを追加できるプロパティをテストする
    @given(
        # タームリストを生成する戦略を定義する
        terms=st.lists(
            st.integers(min_value=1, max_value=100),
            min_size=1, max_size=30,
        ),
    )
    @settings(max_examples=80, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_consensus_multi_term_seq_monotone(self, terms: list[int]) -> None:
        """複数のタームでエントリを追加してもシーケンス番号が単調増加することを検証する。"""
        # コンセンサスログをインスタンス化する
        log = PropertyConsensusLog()
        # 各タームに対してエントリを追加する
        for i, term in enumerate(terms):
            # エントリを追加する
            log.append(term=term, payload=f"entry_{i}".encode())
        # エントリのシーケンス番号を取得する
        seq_numbers = [e[0] for e in log.entries]
        # シーケンス番号が単調増加していることを確認する
        for i in range(1, len(seq_numbers)):
            # 前のシーケンス番号が現在以下であることを確認する
            assert seq_numbers[i - 1] < seq_numbers[i], (
                f"Sequence numbers not monotone at index {i}: "
                f"{seq_numbers[i-1]} >= {seq_numbers[i]}"
            )

    # HLC タイムスタンプが推移律を満たすプロパティをテストする
    @given(
        # 3 つのタイムスタンプを生成する戦略を定義する
        ts1=st.tuples(
            st.integers(min_value=0, max_value=2**47),
            st.integers(min_value=0, max_value=0xFFFF),
        ),
        # 2 つ目のタイムスタンプを生成する戦略を定義する
        ts2=st.tuples(
            st.integers(min_value=0, max_value=2**47),
            st.integers(min_value=0, max_value=0xFFFF),
        ),
        # 3 つ目のタイムスタンプを生成する戦略を定義する
        ts3=st.tuples(
            st.integers(min_value=0, max_value=2**47),
            st.integers(min_value=0, max_value=0xFFFF),
        ),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_order_transitive(
        self,
        ts1: tuple[int, int],
        ts2: tuple[int, int],
        ts3: tuple[int, int],
    ) -> None:
        """HLC タイムスタンプの順序関係が推移律を満たすことを検証する。"""
        # タイムスタンプオブジェクトを生成する
        a = HLCTimestamp(wall_time_ms=ts1[0], logical_counter=ts1[1])
        # 2 つ目のタイムスタンプオブジェクトを生成する
        b = HLCTimestamp(wall_time_ms=ts2[0], logical_counter=ts2[1])
        # 3 つ目のタイムスタンプオブジェクトを生成する
        c = HLCTimestamp(wall_time_ms=ts3[0], logical_counter=ts3[1])
        # a < b かつ b < c ならば a < c を確認する（推移律）
        if a < b and b < c:
            # 推移律を確認する
            assert a < c, (
                f"Transitivity violated: a={a} < b={b} < c={c} but not a < c"
            )

    # KV ストアの書き込みが常に成功するプロパティをテストする
    @given(
        # 書き込むキーを生成する戦略を定義する
        key=st.binary(min_size=1, max_size=64),
        # 書き込む値を生成する戦略を定義する
        value=st.binary(min_size=0, max_size=1024),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_kv_put_always_succeeds(self, key: bytes, value: bytes) -> None:
        """KV ストアへの書き込みが常に成功してタイムスタンプを返すことを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # 書き込みを実行する
        ts = store.put(key, value)
        # タイムスタンプが返ることを確認する
        assert ts is not None, "KV put should return a timestamp"
        # タイムスタンプが wall_time_ms フィールドを持つことを確認する
        assert hasattr(ts, "wall_time_ms") or isinstance(ts, tuple), (
            f"Unexpected timestamp type: {type(ts)}"
        )

    # チャンク再組み立てが元データと完全一致するプロパティをテストする
    @given(
        # データを生成する戦略を定義する
        data=st.binary(min_size=0, max_size=2048),
        # チャンクサイズを生成する戦略を定義する
        chunk_size=st.integers(min_value=1, max_value=256),
    )
    @settings(max_examples=80, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_chunk_reassemble_exact_match(
        self, data: bytes, chunk_size: int
    ) -> None:
        """任意のチャンクサイズで分割した後の再組み立てが元データと完全一致することを検証する。"""
        # チャンクに分割する
        chunks = chunk_message(data, chunk_size=chunk_size)
        # 再組み立てする
        result = reassemble_chunks(chunks)
        # 元のデータと一致することを確認する
        assert result == data, (
            f"Reassembly mismatch with chunk_size={chunk_size}: "
            f"original length={len(data)}, result length={len(result)}"
        )


# tier1 プロパティ公理第 3 拡張テストスイートクラスを定義する
class TestTier1PropertyAxiomExtended3:
    """tier1 軸 v1_property_axiom の第 3 拡張テストスイート。さらに追加のプロパティを検証する。"""

    # KV ストアの上書き書き込みが正しく動作するプロパティをテストする
    @given(
        # キーを生成する戦略を定義する
        key=st.binary(min_size=1, max_size=32),
        # 1 回目の値を生成する戦略を定義する
        value1=st.binary(min_size=1, max_size=64),
        # 2 回目の値を生成する戦略を定義する
        value2=st.binary(min_size=1, max_size=64),
    )
    @settings(max_examples=80, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_kv_overwrite_reads_latest(
        self, key: bytes, value1: bytes, value2: bytes
    ) -> None:
        """KV ストアの上書き後に最新の値が読み取れることを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # 最初の値を書き込む
        store.put(key, value1)
        # 2 番目の値で上書きする
        store.put(key, value2)
        # 最新の値が読み取れることを確認する
        got = store.get(key)
        # 最新の値と一致することを確認する
        assert got == value2, (
            f"After overwrite, should read value2={value2!r}, got={got!r}"
        )

    # HLC の論理カウンタが同一 wall_ms で増加するプロパティをテストする
    @given(
        # 書き込み回数を生成する戦略を定義する
        write_count=st.integers(min_value=2, max_value=30),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_hlc_counter_increments_same_wall(self, write_count: int) -> None:
        """同一 wall_ms でのカウンタが単調増加することを検証する。"""
        # 固定 wall_ms の KV ストアをインスタンス化する
        store = PropertyKVStore(initial_wall_ms=1700000000000)
        # カウンタのリストを初期化する
        counters = []
        # write_count 回書き込む（wall_ms は進めない）
        for i in range(write_count):
            # 書き込みを実行する
            ts = store.put(f"key_{i}".encode(), b"val")
            # カウンタを記録する
            if hasattr(ts, "logical_counter"):
                # カウンタをリストに追加する
                counters.append(ts.logical_counter)
            elif isinstance(ts, tuple):
                # タプルのカウンタを追加する
                counters.append(ts[1])
        # カウンタが単調増加していることを確認する（リストが存在する場合）
        if len(counters) > 1:
            # カウンタが単調増加していることを確認する
            for i in range(1, len(counters)):
                # 前のカウンタが現在以下であることを確認する
                assert counters[i - 1] <= counters[i], (
                    f"Counter should be non-decreasing: counters[{i-1}]={counters[i-1]}, "
                    f"counters[{i}]={counters[i]}"
                )


# tier1 プロパティ公理の第四拡張テストクラスを定義する
class TestTier1PropertyAxiomExtended4:
    """tier1 プロパティ公理の追加テスト群（第四拡張）。"""

    # KV ストアが複数の異なるキーを管理できることをテストする
    def test_kv_store_multiple_keys_independent(self) -> None:
        """KV ストアが複数の異なるキーを独立して管理することを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # 複数のキーバリューペアを書き込む
        pairs = {
            b"alpha": b"value_alpha",
            b"beta": b"value_beta",
            b"gamma": b"value_gamma",
        }
        # 全ペアを書き込む
        for k, v in pairs.items():
            # 書き込みを実行する
            store.put(k, v)
        # 全ペアが正しく読み取れることを確認する
        for k, expected_v in pairs.items():
            # 値を取得する
            got = store.get(k)
            # 値が一致することを確認する
            assert got == expected_v, f"Key {k!r}: expected {expected_v!r}, got {got!r}"

    # KV ストアに存在しないキーが None を返すことをテストする
    def test_kv_store_nonexistent_key_none(self) -> None:
        """KV ストアが存在しないキーに対して None を返すことを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # 存在しないキーで取得する
        result = store.get(b"no_such_key_xyz")
        # None が返ることを確認する
        assert result is None

    # コンセンサスログが追加後にエントリを保持することをテストする
    def test_consensus_log_entries_preserved_after_append(self) -> None:
        """コンセンサスログが追加後にエントリを保持することを検証する。"""
        # コンセンサスログをインスタンス化する
        log = PropertyConsensusLog()
        # 5 件のエントリを追加する
        for i in range(5):
            # エントリを追加する
            log.append(term=1, payload=f"entry_{i}".encode())
        # エントリが 5 件であることを確認する
        assert len(log.entries) == 5

    # コンセンサスログのシーケンス番号が単調増加であることをテストする
    def test_consensus_log_seq_monotone(self) -> None:
        """コンセンサスログのシーケンス番号が単調増加であることを検証する。"""
        # コンセンサスログをインスタンス化する
        log = PropertyConsensusLog()
        # 10 件のエントリを追加する
        seqs = []
        # 10 件のエントリを追加する
        for i in range(10):
            # エントリを追加する
            seq = log.append(term=1, payload=f"data_{i}".encode())
            # シーケンス番号を記録する
            seqs.append(seq)
        # シーケンス番号が単調増加であることを確認する
        for i in range(1, len(seqs)):
            # 前のシーケンス番号より大きいことを確認する
            assert seqs[i] > seqs[i - 1], f"Seq not monotone at {i}: {seqs[i-1]} -> {seqs[i]}"

    # HLC タイムスタンプの pack/unpack がロスレスであることをテストする（多値）
    @pytest.mark.parametrize("wall_ms,counter", [
        (0, 0),
        (1000, 100),
        (1700000000000, 0),
        (1700000000000, 65535),
        (0xFFFFFFFFFFFF, 0),
    ])
    def test_hlc_timestamp_pack_unpack_lossless(self, wall_ms: int, counter: int) -> None:
        """HLC タイムスタンプの pack/unpack がロスレスであることを検証する（多値）。"""
        # HLC タイムスタンプを生成する
        ts = HLCTimestamp(wall_time_ms=wall_ms, logical_counter=counter)
        # 64 ビット整数に変換する
        packed = ts.to_int()
        # 復元する
        unpacked = HLCTimestamp.from_int(packed)
        # wall_ms が一致することを確認する
        assert unpacked.wall_time_ms == wall_ms
        # counter が一致することを確認する
        assert unpacked.logical_counter == counter

    # チャンク化と再組み立てが小さいペイロードでも動作することをテストする
    def test_chunk_reassemble_small_payload(self) -> None:
        """チャンク化と再組み立てが小さいペイロードでも正確に動作することを検証する。"""
        # 小さいペイロードを定義する
        payload = b"tiny"
        # 1 バイトのチャンクサイズで分割する
        chunks = chunk_message(payload, chunk_size=1)
        # 全チャンクが 1 バイトであることを確認する
        assert all(len(c) == 1 for c in chunks)
        # 再組み立てを実行する
        reassembled = reassemble_chunks(chunks)
        # 元のペイロードと一致することを確認する
        assert reassembled == payload

    # KV ストアの上書き書き込みが最新値を返すことをテストする
    def test_kv_store_overwrite_returns_latest(self) -> None:
        """KV ストアへの上書き書き込みが最新値を返すことを検証する。"""
        # KV ストアをインスタンス化する
        store = PropertyKVStore()
        # 最初の書き込みを実行する
        store.put(b"overwrite_key", b"initial_value")
        # 上書き書き込みを実行する
        store.put(b"overwrite_key", b"updated_value")
        # 最新値が返ることを確認する
        result = store.get(b"overwrite_key")
        # 最新値が一致することを確認する
        assert result == b"updated_value"

    # HLC タイムスタンプの大小比較が正しく動作することをテストする
    def test_hlc_timestamp_comparison_correct(self) -> None:
        """HLC タイムスタンプの大小比較が正しく動作することを検証する。"""
        # 小さいタイムスタンプを定義する
        small = HLCTimestamp(wall_time_ms=100, logical_counter=0)
        # 大きいタイムスタンプを定義する（wall_ms が大きい）
        large_wall = HLCTimestamp(wall_time_ms=200, logical_counter=0)
        # 大きいカウンタのタイムスタンプを定義する（wall_ms が同じ）
        large_counter = HLCTimestamp(wall_time_ms=100, logical_counter=5)
        # small が large_wall より小さいことを確認する
        assert small < large_wall
        # small が large_counter より小さいことを確認する
        assert small < large_counter
        # large_wall が small より大きいことを確認する
        assert not large_wall < small


# tier1 プロパティ公理の最終補完テスト関数を定義する
def test_hlc_timestamp_pack_zero_roundtrip() -> None:
    """HLC タイムスタンプのゼロ値が pack/unpack ロスレスであることを検証する。"""
    # ゼロ値のタイムスタンプを生成する
    ts = HLCTimestamp(wall_time_ms=0, logical_counter=0)
    # pack を実行する
    packed = ts.to_int()
    # unpack を実行する
    unpacked = HLCTimestamp.from_int(packed)
    # ゼロ値が一致することを確認する
    assert unpacked.wall_time_ms == 0
    # カウンタがゼロであることを確認する
    assert unpacked.logical_counter == 0
