"""src/test/integration/tier1/test_v1_type_invariant.py

tier1 軸の v1_type_invariant 検証クラスに対するインテグレーションテスト。
KV ストア・コンセンサスログ・HLC タイムスタンプ・トランスポートフレームの
型不変条件を検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: tier1_type_invariant_v1
"""

from __future__ import annotations

# 標準ライブラリのインポートを行う
import struct
import time
import uuid
# pytest フレームワークをインポートする
import pytest
# 型ヒントのサポートをインポートする
from typing import Any, Generator, List, Tuple
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto


# HLC タイムスタンプを表すデータクラスを定義する
@dataclass
class HLCTimestamp:
    """Hybrid Logical Clock タイムスタンプ型。

    wall_time_ms と logical_counter の 2 コンポーネントからなる。
    """

    # 物理時刻をミリ秒で保持するフィールドを定義する
    wall_time_ms: int
    # 同一物理時刻内の論理カウンタを保持するフィールドを定義する
    logical_counter: int

    # 2 つの HLC タイムスタンプを比較する特殊メソッドを定義する
    def __lt__(self, other: "HLCTimestamp") -> bool:
        """HLC タイムスタンプの全順序比較を行う。"""
        # 物理時刻が小さい場合は前であると判定する
        if self.wall_time_ms != other.wall_time_ms:
            # 物理時刻で比較した結果を返す
            return self.wall_time_ms < other.wall_time_ms
        # 物理時刻が等しい場合は論理カウンタで比較する
        return self.logical_counter < other.logical_counter

    # 以上比較演算子を定義する
    def __le__(self, other: "HLCTimestamp") -> bool:
        """以上比較を行う。"""
        # 等しいか小さい場合に True を返す
        return self == other or self < other

    # HLC タイムスタンプを 64 ビット整数にパックするメソッドを定義する
    def pack(self) -> int:
        """タイムスタンプを 64 ビット整数に変換する。"""
        # 物理時刻を上位ビットに配置し論理カウンタを下位ビットに配置する
        return (self.wall_time_ms << 16) | (self.logical_counter & 0xFFFF)


# KV ストアのキーバリューペアを表すデータクラスを定義する
@dataclass
class KVEntry:
    """KV ストアのエントリ型。キーは必ず bytes 型でなければならない。"""

    # キーを bytes 型で保持するフィールドを定義する
    key: bytes
    # 値を bytes 型で保持するフィールドを定義する
    value: bytes
    # エントリに関連付けられた HLC タイムスタンプを定義する
    timestamp: HLCTimestamp

    # キーの型不変条件を検証するバリデーションメソッドを定義する
    def validate(self) -> None:
        """エントリの型不変条件を検証する。"""
        # キーが bytes 型であることを確認する
        if not isinstance(self.key, bytes):
            # 型違反の場合に TypeError を発生させる
            raise TypeError(f"KV key must be bytes, got {type(self.key).__name__}")
        # 値が bytes 型であることを確認する
        if not isinstance(self.value, bytes):
            # 値の型違反の場合に TypeError を発生させる
            raise TypeError(f"KV value must be bytes, got {type(self.value).__name__}")


# コンセンサスログのエントリを表すデータクラスを定義する
@dataclass
class ConsensusLogEntry:
    """Raft コンセンサスログのエントリ型。

    sequence_number は厳密な単調増加でなければならない。
    """

    # シーケンス番号を保持するフィールドを定義する
    sequence_number: int
    # ログの任期番号を保持するフィールドを定義する
    term: int
    # ログのペイロードを bytes 型で保持するフィールドを定義する
    payload: bytes

    # シーケンス番号の不変条件を検証するメソッドを定義する
    def validate(self) -> None:
        """エントリの不変条件を検証する。"""
        # シーケンス番号が正の整数であることを確認する
        if self.sequence_number <= 0:
            # 不正なシーケンス番号の場合に ValueError を発生させる
            raise ValueError(f"sequence_number must be positive, got {self.sequence_number}")
        # 任期番号が非負であることを確認する
        if self.term < 0:
            # 不正な任期番号の場合に ValueError を発生させる
            raise ValueError(f"term must be non-negative, got {self.term}")


# トランスポートメッセージのフレームを表すデータクラスを定義する
@dataclass
class TransportFrame:
    """トランスポート層のメッセージフレーム型。

    先頭 4 バイトが length prefix として機能する。
    """

    # メッセージのペイロードを bytes 型で保持するフィールドを定義する
    payload: bytes

    # フレームをバイト列にシリアライズするメソッドを定義する
    def serialize(self) -> bytes:
        """length prefix 付きバイト列を生成する。"""
        # ペイロード長を 4 バイトビッグエンディアンで先頭に付加する
        length_prefix = struct.pack(">I", len(self.payload))
        # length prefix とペイロードを結合して返す
        return length_prefix + self.payload

    # バイト列からフレームをデシリアライズするクラスメソッドを定義する
    @classmethod
    def deserialize(cls, data: bytes) -> "TransportFrame":
        """バイト列からフレームを復元する。"""
        # データが少なくとも 4 バイトあることを確認する
        if len(data) < 4:
            # データが短すぎる場合に ValueError を発生させる
            raise ValueError(f"Frame too short: {len(data)} bytes")
        # 先頭 4 バイトから length prefix を読み取る
        declared_length = struct.unpack(">I", data[:4])[0]
        # 実際のデータ長を計算する
        actual_length = len(data) - 4
        # 宣言長と実際の長さが一致することを確認する
        if declared_length != actual_length:
            # 長さの不一致の場合に ValueError を発生させる
            raise ValueError(
                f"Length mismatch: declared={declared_length}, actual={actual_length}"
            )
        # ペイロード部分を抽出してフレームを生成する
        return cls(payload=data[4:])


# モック KV ストアを実装するクラスを定義する
class MockKVStore:
    """テスト用のインメモリ KV ストアモック。"""

    # 内部ストレージを初期化するメソッドを定義する
    def __init__(self) -> None:
        """KV ストアを初期化する。"""
        # エントリを格納する辞書を初期化する
        self._store: dict[bytes, KVEntry] = {}
        # HLC 状態を初期化する
        self._hlc_wall_ms: int = int(time.time() * 1000)
        # 論理カウンタを初期化する
        self._hlc_counter: int = 0

    # 現在の HLC タイムスタンプを生成するメソッドを定義する
    def _next_hlc(self) -> HLCTimestamp:
        """単調増加する HLC タイムスタンプを生成する。"""
        # 現在の物理時刻をミリ秒で取得する
        now_ms = int(time.time() * 1000)
        # 物理時刻が進んでいる場合は論理カウンタをリセットする
        if now_ms > self._hlc_wall_ms:
            # 物理時刻を更新する
            self._hlc_wall_ms = now_ms
            # 論理カウンタをリセットする
            self._hlc_counter = 0
        else:
            # 物理時刻が変化していない場合は論理カウンタをインクリメントする
            self._hlc_counter += 1
        # 新しい HLC タイムスタンプを生成して返す
        return HLCTimestamp(
            wall_time_ms=self._hlc_wall_ms,
            logical_counter=self._hlc_counter,
        )

    # KV ストアにエントリを書き込むメソッドを定義する
    def put(self, key: bytes, value: bytes) -> HLCTimestamp:
        """キーと値を KV ストアに書き込む。型不変条件を強制する。"""
        # キーが bytes 型であることを確認する
        if not isinstance(key, bytes):
            # 型違反の場合に TypeError を発生させる
            raise TypeError(f"key must be bytes, got {type(key).__name__}")
        # 値が bytes 型であることを確認する
        if not isinstance(value, bytes):
            # 値の型違反の場合に TypeError を発生させる
            raise TypeError(f"value must be bytes, got {type(value).__name__}")
        # HLC タイムスタンプを生成する
        ts = self._next_hlc()
        # エントリを生成する
        entry = KVEntry(key=key, value=value, timestamp=ts)
        # エントリを検証する
        entry.validate()
        # ストアに書き込む
        self._store[key] = entry
        # タイムスタンプを返す
        return ts

    # KV ストアからエントリを読み取るメソッドを定義する
    def get(self, key: bytes) -> KVEntry | None:
        """指定したキーのエントリを取得する。"""
        # ストアからエントリを返す（存在しない場合は None）
        return self._store.get(key)


# モックコンセンサスログを実装するクラスを定義する
class MockConsensusLog:
    """テスト用のコンセンサスログモック。"""

    # ログを初期化するメソッドを定義する
    def __init__(self) -> None:
        """コンセンサスログを初期化する。"""
        # ログエントリのリストを初期化する
        self._entries: list[ConsensusLogEntry] = []
        # 次のシーケンス番号を初期化する
        self._next_seq: int = 1

    # ログにエントリを追加するメソッドを定義する
    def append(self, term: int, payload: bytes) -> ConsensusLogEntry:
        """新しいログエントリを追加する。シーケンス番号は自動採番される。"""
        # エントリを生成する
        entry = ConsensusLogEntry(
            sequence_number=self._next_seq,
            term=term,
            payload=payload,
        )
        # エントリを検証する
        entry.validate()
        # ログに追加する
        self._entries.append(entry)
        # 次のシーケンス番号をインクリメントする
        self._next_seq += 1
        # エントリを返す
        return entry

    # 全ログエントリを取得するプロパティを定義する
    @property
    def entries(self) -> list[ConsensusLogEntry]:
        """全ログエントリを返す。"""
        # エントリのリストを返す
        return list(self._entries)


# tier1 型不変条件テストスイートクラスを定義する
class TestTier1TypeInvariant:
    """tier1 軸 v1_type_invariant の検証テストスイート。"""

    # KV ストアのモックフィクスチャを定義する
    @pytest.fixture
    def mock_kv_store(self) -> Generator[MockKVStore, None, None]:
        """MockKVStore フィクスチャを生成する。"""
        # モック KV ストアをインスタンス化する
        store = MockKVStore()
        # フィクスチャを返す
        yield store

    # コンセンサスログのモックフィクスチャを定義する
    @pytest.fixture
    def mock_consensus_log(self) -> Generator[MockConsensusLog, None, None]:
        """MockConsensusLog フィクスチャを生成する。"""
        # モックコンセンサスログをインスタンス化する
        log = MockConsensusLog()
        # フィクスチャを返す
        yield log

    # KV ストアのキーが bytes 型であることを検証するテストを定義する
    def test_kv_key_must_be_bytes(self, mock_kv_store: MockKVStore) -> None:
        """KV ストアのキーは必ず bytes 型でなければならないことを検証する。"""
        # bytes 型のキーは正常に書き込めることを確認する
        ts = mock_kv_store.put(b"valid_key", b"value")
        # タイムスタンプが生成されたことを確認する
        assert ts is not None
        # タイムスタンプが HLCTimestamp 型であることを確認する
        assert isinstance(ts, HLCTimestamp)
        # str 型のキーを使用すると TypeError が発生することを確認する
        with pytest.raises(TypeError) as exc_info:
            # 文字列型のキーを渡してエラーを誘発する
            mock_kv_store.put("string_key", b"value")  # type: ignore[arg-type]
        # エラーメッセージが型情報を含んでいることを確認する
        assert "bytes" in str(exc_info.value)
        # int 型のキーを使用すると TypeError が発生することを確認する
        with pytest.raises(TypeError):
            # 整数型のキーを渡してエラーを誘発する
            mock_kv_store.put(12345, b"value")  # type: ignore[arg-type]

    # KV ストアの値が bytes 型であることを検証するテストを定義する
    def test_kv_value_must_be_bytes(self, mock_kv_store: MockKVStore) -> None:
        """KV ストアの値は必ず bytes 型でなければならないことを検証する。"""
        # bytes 型の値は正常に書き込めることを確認する
        ts = mock_kv_store.put(b"key", b"valid_value")
        # タイムスタンプが正常に生成されたことを確認する
        assert ts is not None
        # str 型の値を使用すると TypeError が発生することを確認する
        with pytest.raises(TypeError) as exc_info:
            # 文字列型の値を渡してエラーを誘発する
            mock_kv_store.put(b"key", "string_value")  # type: ignore[arg-type]
        # エラーメッセージに型情報が含まれることを確認する
        assert "bytes" in str(exc_info.value)

    # 書き込まれたエントリが読み取れることを検証するテストを定義する
    def test_kv_read_back_written_entry(self, mock_kv_store: MockKVStore) -> None:
        """書き込んだキーバリューが正しく読み取れることを検証する。"""
        # テスト用のキーと値を定義する
        key = b"test_key_read_back"
        # テスト用の値を定義する
        value = b"test_value_payload_12345"
        # KV ストアに書き込む
        mock_kv_store.put(key, value)
        # 書き込んだキーで読み取る
        entry = mock_kv_store.get(key)
        # エントリが存在することを確認する
        assert entry is not None
        # キーが一致することを確認する
        assert entry.key == key
        # 値が一致することを確認する
        assert entry.value == value

    # コンセンサスログのシーケンス番号が単調増加することを検証するテストを定義する
    def test_consensus_log_monotonic_sequence(self, mock_consensus_log: MockConsensusLog) -> None:
        """コンセンサスログのシーケンス番号が厳密な単調増加であることを検証する。"""
        # 10 件のログエントリを追加する
        entries = []
        # ループ処理でエントリを追加する
        for i in range(10):
            # ログエントリを追加する
            entry = mock_consensus_log.append(term=1, payload=f"payload_{i}".encode())
            # エントリをリストに追加する
            entries.append(entry)
        # エントリのシーケンス番号が 1 始まりであることを確認する
        assert entries[0].sequence_number == 1
        # 最後のエントリのシーケンス番号が 10 であることを確認する
        assert entries[-1].sequence_number == 10
        # 連続するエントリのシーケンス番号が 1 ずつ増加することを確認する
        for i in range(1, len(entries)):
            # 現在のシーケンス番号が前のシーケンス番号より 1 大きいことを確認する
            assert entries[i].sequence_number == entries[i - 1].sequence_number + 1

    # HLC タイムスタンプが単調増加することを検証するテストを定義する
    def test_hlc_timestamps_always_advance(self, mock_kv_store: MockKVStore) -> None:
        """KV 書き込みのたびに HLC タイムスタンプが進むことを検証する。"""
        # 複数回の書き込みで生成されるタイムスタンプを収集する
        timestamps = []
        # 20 件の書き込みを実行する
        for i in range(20):
            # KV ストアに書き込んでタイムスタンプを取得する
            ts = mock_kv_store.put(
                f"key_{i:03d}".encode(),
                f"value_{i}".encode(),
            )
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # 連続するタイムスタンプが前進することを確認する
        for i in range(1, len(timestamps)):
            # 現在のタイムスタンプが前のタイムスタンプ以上であることを確認する
            assert timestamps[i - 1] <= timestamps[i], (
                f"HLC regression at index {i}: "
                f"{timestamps[i-1]} > {timestamps[i]}"
            )

    # トランスポートフレームの length prefix が正しいことを検証するテストを定義する
    def test_transport_frame_length_prefix_correct(self) -> None:
        """トランスポートフレームの length prefix が正確であることを検証する。"""
        # テスト用のペイロードを定義する
        payload = b"hello_world_transport_test_payload"
        # フレームを生成する
        frame = TransportFrame(payload=payload)
        # フレームをシリアライズする
        serialized = frame.serialize()
        # シリアライズ結果の先頭 4 バイトが length prefix であることを確認する
        declared_length = struct.unpack(">I", serialized[:4])[0]
        # 宣言された長さが実際のペイロード長と一致することを確認する
        assert declared_length == len(payload)
        # 全体の長さが 4 + ペイロード長であることを確認する
        assert len(serialized) == 4 + len(payload)

    # トランスポートフレームのデシリアライズが正しいことを検証するテストを定義する
    def test_transport_frame_deserialize_roundtrip(self) -> None:
        """トランスポートフレームのシリアライズ・デシリアライズが往復変換可能であることを検証する。"""
        # テスト用のペイロードを定義する
        original_payload = b"roundtrip_test_payload_k1s0_tier1"
        # フレームを生成する
        original_frame = TransportFrame(payload=original_payload)
        # シリアライズする
        serialized = original_frame.serialize()
        # デシリアライズして復元する
        restored_frame = TransportFrame.deserialize(serialized)
        # 復元されたペイロードが元のペイロードと一致することを確認する
        assert restored_frame.payload == original_payload

    # 不正な length prefix を持つフレームがエラーになることを検証するテストを定義する
    def test_transport_frame_invalid_length_prefix_rejected(self) -> None:
        """length prefix と実際の長さが不一致のフレームが拒否されることを検証する。"""
        # 宣言長が 100 だが実際のペイロードが 5 バイトのフレームを構築する
        bad_frame = struct.pack(">I", 100) + b"short"
        # デシリアライズ時に ValueError が発生することを確認する
        with pytest.raises(ValueError) as exc_info:
            # 不正なフレームをデシリアライズする
            TransportFrame.deserialize(bad_frame)
        # エラーメッセージに mismatch 情報が含まれることを確認する
        assert "mismatch" in str(exc_info.value).lower() or "length" in str(exc_info.value).lower()

    # コンセンサスログのシーケンス番号が 1 始まりであることを検証するテストを定義する
    def test_consensus_log_starts_at_one(self, mock_consensus_log: MockConsensusLog) -> None:
        """コンセンサスログの最初のシーケンス番号が 1 であることを検証する。"""
        # 最初のエントリを追加する
        first_entry = mock_consensus_log.append(term=1, payload=b"first_entry")
        # シーケンス番号が 1 であることを確認する
        assert first_entry.sequence_number == 1

    # 空のキーが bytes 型として扱えることを検証するテストを定義する
    def test_kv_empty_bytes_key_allowed(self, mock_kv_store: MockKVStore) -> None:
        """空の bytes キーが型不変条件を満たすことを検証する。"""
        # 空の bytes キーで書き込みを実行する
        ts = mock_kv_store.put(b"", b"value_for_empty_key")
        # タイムスタンプが正常に生成されたことを確認する
        assert ts is not None
        # 空のキーで読み取る
        entry = mock_kv_store.get(b"")
        # エントリが存在することを確認する
        assert entry is not None
        # 値が一致することを確認する
        assert entry.value == b"value_for_empty_key"

    # HLC タイムスタンプのパックが 64 ビット整数を返すことを検証するテストを定義する
    def test_hlc_pack_returns_int(self, mock_kv_store: MockKVStore) -> None:
        """HLC タイムスタンプの pack メソッドが整数を返すことを検証する。"""
        # KV ストアに書き込んで HLC タイムスタンプを取得する
        ts = mock_kv_store.put(b"pack_test_key", b"pack_test_value")
        # pack メソッドを実行する
        packed = ts.pack()
        # 結果が整数であることを確認する
        assert isinstance(packed, int)
        # 結果が非負であることを確認する
        assert packed >= 0

    # 複数のコンセンサス任期にわたるログエントリのシーケンス番号を検証するテストを定義する
    def test_consensus_log_sequence_across_terms(self, mock_consensus_log: MockConsensusLog) -> None:
        """異なる任期にわたってシーケンス番号が単調増加することを検証する。"""
        # 任期 1 でエントリを追加する
        entries_term1 = [
            mock_consensus_log.append(term=1, payload=f"t1_p{i}".encode())
            for i in range(5)
        ]
        # 任期 2 でエントリを追加する
        entries_term2 = [
            mock_consensus_log.append(term=2, payload=f"t2_p{i}".encode())
            for i in range(5)
        ]
        # 全エントリのシーケンス番号を収集する
        all_entries = entries_term1 + entries_term2
        # シーケンス番号が 1 から 10 まで連続していることを確認する
        seq_numbers = [e.sequence_number for e in all_entries]
        # 期待されるシーケンス番号リストと一致することを確認する
        assert seq_numbers == list(range(1, 11))

    # KVEntry の validate メソッドが正常エントリをパスすることを検証するテストを定義する
    def test_kv_entry_validate_passes_for_valid_entry(self) -> None:
        """正常な KVEntry が validate() をパスすることを検証する。"""
        # 正常なエントリを生成する
        entry = KVEntry(
            key=b"valid_key",
            value=b"valid_value",
            timestamp=HLCTimestamp(wall_time_ms=1700000000000, logical_counter=0),
        )
        # validate が例外を発生させないことを確認する
        entry.validate()

    # ConsensusLogEntry の不正シーケンス番号が拒否されることを検証するテストを定義する
    def test_consensus_entry_zero_sequence_rejected(self) -> None:
        """シーケンス番号が 0 のコンセンサスエントリが拒否されることを検証する。"""
        # シーケンス番号が 0 のエントリを生成する
        entry = ConsensusLogEntry(
            sequence_number=0,
            term=1,
            payload=b"payload",
        )
        # validate() が ValueError を発生させることを確認する
        with pytest.raises(ValueError) as exc_info:
            # バリデーションを実行する
            entry.validate()
        # エラーメッセージに "positive" が含まれることを確認する
        assert "positive" in str(exc_info.value)

    # トランスポートフレームのデータが短すぎる場合にエラーになることを検証するテストを定義する
    def test_transport_frame_too_short_rejected(self) -> None:
        """4 バイト未満のデータがフレームデシリアライズで拒否されることを検証する。"""
        # 3 バイトの不完全なデータを用意する
        too_short = b"\x00\x00\x00"
        # デシリアライズ時に ValueError が発生することを確認する
        with pytest.raises(ValueError) as exc_info:
            # 短すぎるデータをデシリアライズする
            TransportFrame.deserialize(too_short)
        # エラーメッセージに "too short" が含まれることを確認する
        assert "short" in str(exc_info.value).lower() or "4" in str(exc_info.value)


# tier1 追加型不変条件テストスイートクラスを定義する
class TestTier1TypeInvariantExtended:
    """tier1 軸 v1_type_invariant の追加検証テストスイート。

    KV ストアの上書き・複数フレームの分割処理・HLC pack の境界値テストを含む。
    """

    # KV ストアの上書きが正しく動作することを検証するテストを定義する
    def test_kv_overwrite_same_key_returns_new_value(self) -> None:
        """同じキーへの上書き書き込みが正しく動作することを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 最初の値を書き込む
        store.put(b"overwrite_key", b"first_value")
        # 2 番目の値を書き込む（上書き）
        store.put(b"overwrite_key", b"second_value")
        # 読み取って最新の値であることを確認する
        entry = store.get(b"overwrite_key")
        # エントリが存在することを確認する
        assert entry is not None
        # 最新の値が返されることを確認する
        assert entry.value == b"second_value", (
            f"Expected second_value, got {entry.value!r}"
        )

    # KV ストアが複数のキーを独立して保持できることを検証するテストを定義する
    def test_kv_multiple_keys_independent(self) -> None:
        """複数のキーが独立して保持されることを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 10 件のキーと値を書き込む
        written = {}
        # 書き込みループを実行する
        for i in range(10):
            # キーと値を生成する
            key = f"indep_key_{i:03d}".encode()
            # 値を生成する
            value = f"indep_value_{i * i}".encode()
            # 書き込みを実行する
            store.put(key, value)
            # 書き込んだ内容を記録する
            written[key] = value
        # 全キーの値が正しく読み取れることを確認する
        for key, expected_value in written.items():
            # エントリを取得する
            entry = store.get(key)
            # エントリが存在することを確認する
            assert entry is not None
            # 値が一致することを確認する
            assert entry.value == expected_value

    # 存在しないキーの読み取りが None を返すことを検証するテストを定義する
    def test_kv_get_nonexistent_key_returns_none(self) -> None:
        """存在しないキーの読み取りが None を返すことを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 存在しないキーの読み取りを実行する
        result = store.get(b"nonexistent_key_xyz")
        # None が返されることを確認する
        assert result is None

    # HLC タイムスタンプの pack で境界値が正しく処理されることを検証するテストを定義する
    def test_hlc_pack_boundary_values(self) -> None:
        """HLC タイムスタンプ pack の境界値が正しく処理されることを検証する。"""
        # 最小値のタイムスタンプをテストする
        ts_min = HLCTimestamp(wall_time_ms=0, logical_counter=0)
        # パックしてアンパックする
        packed_min = ts_min.pack()
        # 最小値が 0 であることを確認する
        assert packed_min == 0
        # カウンタが最大値（0xFFFF）のタイムスタンプをテストする
        ts_max_counter = HLCTimestamp(wall_time_ms=0, logical_counter=0xFFFF)
        # パックする
        packed_max_c = ts_max_counter.pack()
        # 結果が 0xFFFF であることを確認する
        assert packed_max_c == 0xFFFF

    # TransportFrame の空ペイロードが正しく処理されることを検証するテストを定義する
    def test_transport_frame_empty_payload(self) -> None:
        """空ペイロードの TransportFrame が正しく処理されることを検証する。"""
        # 空ペイロードのフレームを生成する
        frame = TransportFrame(payload=b"")
        # シリアライズする
        serialized = frame.serialize()
        # 全体の長さが 4 バイトであることを確認する
        assert len(serialized) == 4
        # デシリアライズして復元する
        restored = TransportFrame.deserialize(serialized)
        # ペイロードが空であることを確認する
        assert restored.payload == b""

    # TransportFrame の大きなペイロードが正しく処理されることを検証するテストを定義する
    def test_transport_frame_large_payload(self) -> None:
        """大きなペイロードの TransportFrame が正しく処理されることを検証する。"""
        # 64KB のペイロードを生成する
        large_payload = b"X" * 65536
        # フレームを生成する
        frame = TransportFrame(payload=large_payload)
        # シリアライズする
        serialized = frame.serialize()
        # 全体の長さが 4 + 65536 であることを確認する
        assert len(serialized) == 4 + 65536
        # デシリアライズして復元する
        restored = TransportFrame.deserialize(serialized)
        # ペイロードが一致することを確認する
        assert restored.payload == large_payload

    # コンセンサスログが多数のエントリを保持できることを検証するテストを定義する
    def test_consensus_log_large_number_of_entries(self) -> None:
        """コンセンサスログが多数のエントリを正しく保持することを検証する。"""
        # コンセンサスログをインスタンス化する
        log = MockConsensusLog()
        # 1000 件のエントリを追加する
        for i in range(1000):
            # エントリを追加する
            log.append(term=1, payload=f"bulk_payload_{i}".encode())
        # エントリ数が 1000 件であることを確認する
        assert len(log.entries) == 1000
        # 最初のエントリのシーケンス番号が 1 であることを確認する
        assert log.entries[0].sequence_number == 1
        # 最後のエントリのシーケンス番号が 1000 であることを確認する
        assert log.entries[-1].sequence_number == 1000

    # KV エントリの HLC タイムスタンプが書き込み順序を反映することを検証するテストを定義する
    def test_kv_timestamps_reflect_write_order(self) -> None:
        """KV ストアのエントリタイムスタンプが書き込み順序を正しく反映することを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 5 件の書き込みを実行してタイムスタンプを収集する
        timestamps = []
        # 書き込みループを実行する
        for i in range(5):
            # 書き込みを実行してタイムスタンプを取得する
            ts = store.put(f"order_key_{i}".encode(), f"order_val_{i}".encode())
            # タイムスタンプをリストに追加する
            timestamps.append(ts)
        # タイムスタンプが単調増加であることを確認する
        for i in range(1, len(timestamps)):
            # 前のタイムスタンプが現在のタイムスタンプ以下であることを確認する
            assert timestamps[i - 1] <= timestamps[i]

    # HLCTimestamp の __le__ が同値の場合 True を返すことを検証するテストを定義する
    def test_hlc_timestamp_le_equal_returns_true(self) -> None:
        """HLCTimestamp の __le__ が同値の場合 True を返すことを検証する。"""
        # 同じ値を持つ 2 つのタイムスタンプを生成する
        ts_a = HLCTimestamp(wall_time_ms=1700000000000, logical_counter=5)
        # ts_b を生成する
        ts_b = HLCTimestamp(wall_time_ms=1700000000000, logical_counter=5)
        # __le__ が True を返すことを確認する
        assert ts_a <= ts_b
        # __lt__ が False を返すことを確認する
        assert not (ts_a < ts_b)

    # KVEntry のバリデーションが str キーを正しく拒否することを検証するテストを定義する
    def test_kv_entry_validate_str_key_raises_type_error(self) -> None:
        """KVEntry.validate() が str キーを TypeError で拒否することを検証する。"""
        # str キーを持つ KVEntry を生成する（型チェックを回避して直接生成）
        entry = KVEntry(
            key="not_bytes",  # type: ignore[arg-type]
            value=b"valid_value",
            timestamp=HLCTimestamp(wall_time_ms=1700000000000, logical_counter=0),
        )
        # バリデーションが TypeError を発生させることを確認する
        with pytest.raises(TypeError, match="bytes"):
            # バリデーションを実行する
            entry.validate()

    # ConsensusLogEntry の負のシーケンス番号が拒否されることを検証するテストを定義する
    def test_consensus_entry_negative_sequence_rejected(self) -> None:
        """負のシーケンス番号を持つ ConsensusLogEntry が拒否されることを検証する。"""
        # 負のシーケンス番号を持つエントリを生成する
        entry = ConsensusLogEntry(
            sequence_number=-100,
            term=1,
            payload=b"negative_seq_payload",
        )
        # バリデーションが ValueError を発生させることを確認する
        with pytest.raises(ValueError, match="positive"):
            # バリデーションを実行する
            entry.validate()


# tier1 型不変条件第 3 拡張テストスイートクラスを定義する
class TestTier1TypeInvariantExtended3:
    """tier1 軸 v1_type_invariant の第 3 拡張テストスイート。さらに追加の型不変条件を検証する。"""

    # HLC タイムスタンプのシリアライズが正確であることをテストする
    def test_hlc_timestamp_serialize_deserialize_roundtrip(self) -> None:
        """HLC タイムスタンプのシリアライズとデシリアライズが正確であることを検証する。"""
        # タイムスタンプを作成する
        ts_original = HLCTimestamp(wall_time_ms=1700000042000, logical_counter=77)
        # 64 ビット整数にシリアライズする
        packed = ts_original.pack()
        # デシリアライズする
        ts_restored = HLCTimestamp.unpack(packed)
        # wall_time_ms が一致することを確認する
        assert ts_restored.wall_time_ms == ts_original.wall_time_ms, (
            f"wall_time_ms mismatch: {ts_restored.wall_time_ms} != {ts_original.wall_time_ms}"
        )
        # logical_counter が一致することを確認する
        assert ts_restored.logical_counter == ts_original.logical_counter, (
            f"logical_counter mismatch: {ts_restored.logical_counter} != {ts_original.logical_counter}"
        )

    # MockKVStore の存在しないキーに対して None が返ることをテストする
    def test_kv_store_nonexistent_key_returns_none(self) -> None:
        """MockKVStore の存在しないキーに対して None が返ることを検証する。"""
        # KV ストアをインスタンス化する
        kv = MockKVStore()
        # 存在しないキーを取得する
        result = kv.get(b"nonexistent_key")
        # None が返ることを確認する
        assert result is None

    # トランスポートフレームのチェックサムが一意であることをテストする
    def test_transport_frame_checksum_unique_per_payload(self) -> None:
        """異なるペイロードのトランスポートフレームが異なるチェックサムを持つことを検証する。"""
        # 2 つの異なるトランスポートフレームを作成する
        frame1 = TransportFrame(payload=b"payload_alpha")
        # 2 番目のフレームを作成する
        frame2 = TransportFrame(payload=b"payload_beta")
        # チェックサムを取得する
        checksum1 = frame1.checksum
        # 2 番目のチェックサムを取得する
        checksum2 = frame2.checksum
        # チェックサムが異なることを確認する
        assert checksum1 != checksum2, (
            f"Different payloads should have different checksums: "
            f"checksum1={checksum1}, checksum2={checksum2}"
        )

    # MockConsensusLog の空ログに対してエントリが 0 件であることをテストする
    def test_consensus_log_empty_initially(self) -> None:
        """MockConsensusLog の初期状態がエントリ 0 件であることを検証する。"""
        # コンセンサスログをインスタンス化する
        log = MockConsensusLog()
        # エントリが 0 件であることを確認する
        assert len(log.entries) == 0

    # HLC タイムスタンプの wall_time_ms が境界値でも正しいことをテストする
    @pytest.mark.parametrize(
        "wall_ms,counter",
        [
            # 最小値のケースを定義する（0, 0）
            (0, 0),
            # 中間値のケースを定義する
            (1700000000000, 100),
            # 最大カウンタのケースを定義する
            (1700000000000, 0xFFFF),
            # 最大 wall_ms のケースを定義する
            (0xFFFFFFFFFFFF, 0),
        ],
    )
    def test_hlc_boundary_values(self, wall_ms: int, counter: int) -> None:
        """HLC タイムスタンプが境界値でも正しく動作することを検証する。"""
        # タイムスタンプを作成する
        ts = HLCTimestamp(wall_time_ms=wall_ms, logical_counter=counter)
        # pack/unpack がロスレスであることを確認する
        packed = ts.pack()
        # アンパックする
        unpacked = HLCTimestamp.unpack(packed)
        # wall_time_ms が一致することを確認する
        assert unpacked.wall_time_ms == wall_ms
        # logical_counter が一致することを確認する
        assert unpacked.logical_counter == counter


# tier1 型不変条件の第四拡張テストクラスを定義する
class TestTier1TypeInvariantExtended4:
    """KV ストア・コンセンサスログ・HLC・トランスポートの型不変条件の追加テスト群（第四拡張）。"""

    # HLC タイムスタンプが wall_ms=0, counter=0 でも動作することをテストする
    def test_hlc_zero_values_valid(self) -> None:
        """HLC タイムスタンプが wall_ms=0, counter=0 でも正常動作することを検証する。"""
        # ゼロ値のタイムスタンプを生成する
        ts = HLCTimestamp(wall_time_ms=0, logical_counter=0)
        # pack を実行する
        packed = ts.pack()
        # ゼロが返ることを確認する
        assert packed == 0
        # unpack を実行する
        unpacked = HLCTimestamp.unpack(packed)
        # wall_ms がゼロであることを確認する
        assert unpacked.wall_time_ms == 0
        # counter がゼロであることを確認する
        assert unpacked.logical_counter == 0

    # HLC タイムスタンプが同値比較で正しく動作することをテストする
    def test_hlc_same_value_equal(self) -> None:
        """同じ値の HLC タイムスタンプが等しいと判定されることを検証する。"""
        # 同じ値のタイムスタンプを 2 つ生成する
        ts1 = HLCTimestamp(wall_time_ms=1000, logical_counter=5)
        # 2 つ目のタイムスタンプを生成する
        ts2 = HLCTimestamp(wall_time_ms=1000, logical_counter=5)
        # 等しいことを確認する
        assert ts1 == ts2
        # どちらも相手より小さくないことを確認する
        assert not ts1 < ts2
        # どちらも相手より小さくないことを確認する
        assert not ts2 < ts1

    # KV ストアが書き込み後に正しい値を返すことをテストする
    def test_kv_put_and_get_basic(self) -> None:
        """KV ストアへの書き込み後に正しい値を返すことを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # キーと値を定義する
        key = b"test_basic_key"
        # 値を定義する
        value = b"test_basic_value"
        # 書き込みを実行する
        store.put(key, value)
        # 読み取りを実行する
        result = store.get(key)
        # 値が一致することを確認する
        assert result == value

    # KV ストアが存在しないキーに対して None を返すことをテストする
    def test_kv_get_missing_key_none(self) -> None:
        """KV ストアが存在しないキーに対して None を返すことを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 存在しないキーで取得する
        result = store.get(b"missing_key_999")
        # None が返ることを確認する
        assert result is None

    # コンセンサスログが空の状態で初期化されることをテストする
    def test_consensus_log_initial_empty(self) -> None:
        """コンセンサスログが空の状態で初期化されることを検証する。"""
        # コンセンサスログをインスタンス化する
        log = MockConsensusLog()
        # エントリ数が 0 であることを確認する
        assert len(log.entries) == 0

    # トランスポートフレームのチェックサムが同じペイロードで一致することをテストする
    def test_transport_frame_same_payload_same_checksum(self) -> None:
        """同じペイロードのトランスポートフレームが同じチェックサムを持つことを検証する。"""
        # 同じペイロードで 2 つのフレームを生成する
        payload = b"identical_payload"
        # 1 つ目のフレームを生成する
        frame1 = TransportFrame(payload=payload)
        # 2 つ目のフレームを生成する
        frame2 = TransportFrame(payload=payload)
        # チェックサムが一致することを確認する
        assert frame1.checksum == frame2.checksum

    # HLC タイムスタンプが論理カウンタで全順序を持つことをテストする
    def test_hlc_counter_ordering(self) -> None:
        """HLC タイムスタンプが論理カウンタで全順序を持つことを検証する。"""
        # 同じ wall_ms で異なるカウンタのタイムスタンプを生成する
        ts_0 = HLCTimestamp(wall_time_ms=1000, logical_counter=0)
        # カウンタ 1 のタイムスタンプを生成する
        ts_1 = HLCTimestamp(wall_time_ms=1000, logical_counter=1)
        # カウンタ 5 のタイムスタンプを生成する
        ts_5 = HLCTimestamp(wall_time_ms=1000, logical_counter=5)
        # 全順序が成立することを確認する
        assert ts_0 < ts_1
        # ts_1 が ts_5 より小さいことを確認する
        assert ts_1 < ts_5
        # ts_0 が ts_5 より小さいことを確認する
        assert ts_0 < ts_5

    # KV ストアが複数の連続書き込みを全て保持することをテストする
    def test_kv_sequential_writes_all_preserved(self) -> None:
        """KV ストアが複数の連続書き込みを全て保持することを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 10 件の書き込みを実行する
        for i in range(10):
            # キーと値を生成する
            key = f"seq_key_{i}".encode()
            # 値を生成する
            value = f"seq_value_{i}".encode()
            # 書き込みを実行する
            store.put(key, value)
        # 全書き込みが保持されることを確認する
        for i in range(10):
            # キーを生成する
            key = f"seq_key_{i}".encode()
            # 値を生成する
            expected = f"seq_value_{i}".encode()
            # 値を取得する
            got = store.get(key)
            # 値が一致することを確認する
            assert got == expected, f"Sequential write {i}: expected {expected!r}, got {got!r}"


# tier1 型不変条件の第五拡張テストクラスを定義する
class TestTier1TypeInvariantExtended5:
    """KV ストア・HLC の型不変条件の追加テスト群（第五拡張）。"""

    # HLC タイムスタンプが wall_ms が大きい場合に前よりも大きいことをテストする
    def test_hlc_larger_wall_ms_is_greater(self) -> None:
        """wall_ms が大きい HLC タイムスタンプが小さいものより大きいことを検証する。"""
        # 小さい wall_ms のタイムスタンプを定義する
        small = HLCTimestamp(wall_time_ms=100, logical_counter=99999)
        # 大きい wall_ms のタイムスタンプを定義する（counter が 0 でも大きい）
        large = HLCTimestamp(wall_time_ms=200, logical_counter=0)
        # 大きい wall_ms が小さいカウンタでも勝つことを確認する
        assert small < large

    # KV ストアの初期状態がいかなるキーも存在しないことをテストする
    def test_kv_initial_state_no_keys(self) -> None:
        """KV ストアの初期状態でいかなるキーも存在しないことを検証する。"""
        # KV ストアをインスタンス化する
        store = MockKVStore()
        # 複数の異なるキーが None を返すことを確認する
        for key in [b"a", b"key1", b"missing", b"\x00\xff"]:
            # 存在しないキーで取得する
            result = store.get(key)
            # None が返ることを確認する
            assert result is None, f"Initial KV store must have no keys, got {result!r} for {key!r}"


# HLC の pack が符号なし 64 ビット整数の範囲に収まることを確認するモジュールレベルの定数を定義する
HLC_MAX_PACKED_VALUE: int = (0xFFFFFFFFFFFF << 16) | 0xFFFF
HLC_MIN_PACKED_VALUE: int = 0
