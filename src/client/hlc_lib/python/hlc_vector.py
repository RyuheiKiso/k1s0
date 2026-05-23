#!/usr/bin/env python3
"""
hlc_vector.py — k1s0-hlc: HLC ベクタークロックおよびシリアライズ拡張

製造業プラットフォーム k1s0 において複数ノードの HLC タイムスタンプを束ねた
ベクタークロック（VectorClock）と、gRPC/HTTP2 メタデータ伝搬用のシリアライズ
ヘルパーを提供する。

本モジュールは hlc.py の HLCTimestamp / HLCClock を基盤として、以下を追加する:

  VectorClock      : {node_id -> HLCTimestamp} の辞書で全ノードの HLC を追跡する。
                     Fidge/Mattern のベクタークロックを HLC で拡張したもの。
  merge_vector     : 2 つの VectorClock を component-wise max でマージする。
  HLCSerializer    : HLCTimestamp を様々な形式（compact str / b64 / bytes / dict）に変換する。
  HLCDeserializer  : 各形式から HLCTimestamp を復元する。
  HLCIntervalCheck : 2 つの HLCTimestamp の間隔（ms）が許容範囲内かどうかを検証する。
  DriftGuard       : 受信 HLC と自ローカル HLC のドリフトを監視する Guard クラス。
  HLCSequencer     : 単調増加するシーケンス番号を HLC と組み合わせて発行するクラス。
  HLCEventLog      : インメモリで HLC タイムスタンプ付きイベントを記録するログクラス。

依存関係:
  本モジュールは hlc.py にのみ依存し、外部ライブラリへの依存はない。
"""

from __future__ import annotations

# base64: HLC タイムスタンプのシリアライズに使用する
import base64
# collections: OrderedDict を VectorClock の内部ストレージに使用する
import collections
# dataclasses: 値オブジェクトの定義に使用する
import dataclasses
# struct: バイナリシリアライズに使用する
import struct
# threading: VectorClock / DriftGuard / HLCSequencer のスレッドセーフ化に使用する
import threading
# typing: 型ヒントに使用する
from typing import Dict, Iterator, List, Optional, OrderedDict, Tuple

# 同一パッケージの hlc モジュールから必要なクラス/関数をインポートする
from .hlc import (
    EPOCH,
    HLCClock,
    HLCTimestamp,
    _STRUCT_FORMAT,
    _STRUCT_SIZE,
    _U16_MAX,
    pack_b64,
    unpack_b64,
)

# ベクタークロックのバイナリシリアライズ用マジックバイト（識別子）を定義する
_VECTOR_MAGIC: bytes = b"HLCVec1\x00"
# マジックバイトのサイズを定義する
_VECTOR_MAGIC_SIZE: int = len(_VECTOR_MAGIC)
# ベクタークロックのエントリヘッダフォーマット（node_id u16 + reserved u16 + 1 HLC = 16 bytes）
_VEC_ENTRY_FORMAT: str = ">HH" + _STRUCT_FORMAT[1:]
# ベクタークロックのエントリサイズを計算する
_VEC_ENTRY_SIZE: int = struct.calcsize(_VEC_ENTRY_FORMAT)


class VectorClock:
    """
    HLC ベクタークロック: 複数ノードの HLCTimestamp を {node_id -> HLCTimestamp} で管理する。

    各ノードのクロックを独立して追跡し、因果関係の比較に使用する。
    Fidge/Mattern ベクタークロックを HLC で拡張した実装。

    スレッドセーフ: threading.Lock で内部状態を保護する。
    """

    def __init__(self) -> None:
        """空のベクタークロックを初期化する。"""
        # _clocks: node_id -> HLCTimestamp のマッピングを OrderedDict で保持する
        self._clocks: OrderedDict[int, HLCTimestamp] = collections.OrderedDict()
        # _lock: スレッドセーフな状態更新のための Mutex
        self._lock: threading.Lock = threading.Lock()

    def update(self, node_id: int, ts: HLCTimestamp) -> None:
        """指定した node_id の HLCTimestamp を更新する（現在値より後の場合のみ更新する）。"""
        # Lock を取得してスレッドセーフに更新する
        with self._lock:
            # 現在の HLCTimestamp を取得する（存在しない場合は EPOCH を使用する）
            current = self._clocks.get(node_id, EPOCH)
            # 新しいタイムスタンプが現在より後の場合のみ更新する（単調増加を保証する）
            if ts > current:
                self._clocks[node_id] = ts

    def get(self, node_id: int) -> HLCTimestamp:
        """指定した node_id の HLCTimestamp を返す（存在しない場合は EPOCH を返す）。"""
        # Lock を取得してスレッドセーフに読み取る
        with self._lock:
            # node_id に対応する HLCTimestamp を返す（存在しない場合は EPOCH）
            return self._clocks.get(node_id, EPOCH)

    def all_nodes(self) -> List[int]:
        """このベクタークロックが追跡している全 node_id のリストを返す（ソート済み）。"""
        # Lock を取得してスレッドセーフに読み取る
        with self._lock:
            # node_id をソートして返す
            return sorted(self._clocks.keys())

    def snapshot(self) -> Dict[int, HLCTimestamp]:
        """現在のベクタークロックのスナップショットを辞書として返す（コピー）。"""
        # Lock を取得してスレッドセーフに読み取る
        with self._lock:
            # 辞書のコピーを返す（内部状態の直接変更を防ぐ）
            return dict(self._clocks)

    def dominates(self, other: "VectorClock") -> bool:
        """
        self が other を dominate（全ノードで self >= other）するかどうかを返す。

        self >= other かつ少なくとも 1 ノードで self > other の場合に True。
        因果関係の判定: self が other の後に発生したことの証明に使用する。
        """
        # 全ノードの和集合を取得する
        with self._lock:
            self_snap = dict(self._clocks)
        with other._lock:
            other_snap = dict(other._clocks)
        # 全ノードの union を計算する
        all_nodes = set(self_snap.keys()) | set(other_snap.keys())
        # すべてのノードで self >= other を確認する
        at_least_one_greater = False
        for node_id in all_nodes:
            self_ts = self_snap.get(node_id, EPOCH)
            other_ts = other_snap.get(node_id, EPOCH)
            # self が other より前になっている場合は dominate しない
            if self_ts < other_ts:
                return False
            # 少なくとも 1 ノードで self > other かどうかを記録する
            if self_ts > other_ts:
                at_least_one_greater = True
        # 全ノードで self >= other かつ 1 ノード以上で self > other の場合に True
        return at_least_one_greater

    def to_bytes(self) -> bytes:
        """ベクタークロックをバイナリ形式にシリアライズする。"""
        # マジックバイトで始まるバイト列を構築する
        with self._lock:
            entries = list(self._clocks.items())
        # エントリ数をヘッダに含めるためにバッファを構築する
        buf = bytearray()
        # マジックバイトを書き込む
        buf.extend(_VECTOR_MAGIC)
        # エントリ数を書き込む（u16, big-endian）
        buf.extend(struct.pack(">H", len(entries)))
        # 各エントリをシリアライズする
        for node_id, ts in sorted(entries):
            # node_id (u16) + reserved (u16=0) + wall_ms (u64) + logical (u16) + hlc_node_id (u16)
            buf.extend(
                struct.pack(">HH", node_id, 0)
            )
            # HLC タイムスタンプをパックする
            buf.extend(struct.pack(_STRUCT_FORMAT, ts.wall_ms, ts.logical, ts.node_id))
        # バイト列を返す
        return bytes(buf)

    @classmethod
    def from_bytes(cls, data: bytes) -> Optional["VectorClock"]:
        """バイナリ形式からベクタークロックを復元する。"""
        # マジックバイトを確認する
        if len(data) < _VECTOR_MAGIC_SIZE + 2:
            return None
        # マジックバイトが一致しない場合は None を返す
        if data[:_VECTOR_MAGIC_SIZE] != _VECTOR_MAGIC:
            return None
        # エントリ数を読み取る
        offset = _VECTOR_MAGIC_SIZE
        (entry_count,) = struct.unpack_from(">H", data, offset)
        # オフセットを更新する
        offset += 2
        # 新しい VectorClock を生成する
        vc = cls()
        # 各エントリをデシリアライズする
        for _ in range(entry_count):
            # データが不足している場合は None を返す
            if offset + 2 + _STRUCT_SIZE > len(data):
                return None
            # node_id と reserved を読み取る
            (node_id, _reserved) = struct.unpack_from(">HH", data, offset)
            # オフセットを更新する
            offset += 4
            # HLC タイムスタンプを読み取る
            wall_ms, logical, hlc_node_id = struct.unpack_from(_STRUCT_FORMAT, data, offset)
            # オフセットを更新する
            offset += _STRUCT_SIZE
            # HLCTimestamp を生成する
            try:
                ts = HLCTimestamp(wall_ms=wall_ms, logical=logical, node_id=hlc_node_id)
            except ValueError:
                # バリデーションエラーが発生した場合は None を返す
                return None
            # ベクタークロックに追加する
            vc._clocks[node_id] = ts
        # 復元した VectorClock を返す
        return vc


def merge_vector(a: VectorClock, b: VectorClock) -> VectorClock:
    """
    2 つの VectorClock を component-wise max でマージして新しい VectorClock を返す。

    各 node_id について max(a.get(node_id), b.get(node_id)) を採用する。
    分散システムでのベクタークロック伝搬時に使用する。
    """
    # 新しい VectorClock を生成する
    result = VectorClock()
    # a のスナップショットを取得する
    a_snap = a.snapshot()
    # b のスナップショットを取得する
    b_snap = b.snapshot()
    # 全ノードの union を計算する
    all_nodes = set(a_snap.keys()) | set(b_snap.keys())
    # 各ノードについて max を採用する
    for node_id in all_nodes:
        # a から node_id の HLCTimestamp を取得する（存在しない場合は EPOCH）
        ts_a = a_snap.get(node_id, EPOCH)
        # b から node_id の HLCTimestamp を取得する（存在しない場合は EPOCH）
        ts_b = b_snap.get(node_id, EPOCH)
        # max を採用する
        result.update(node_id, ts_a if ts_a > ts_b else ts_b)
    # マージ結果を返す
    return result


class HLCSerializer:
    """
    HLCTimestamp を各種形式にシリアライズするユーティリティクラス。

    gRPC メタデータ、HTTP ヘッダー、JSON フィールドに埋め込む際に使用する。
    全メソッドはスタティックで状態を持たない（Pure Function）。
    """

    @staticmethod
    def to_compact_str(ts: HLCTimestamp) -> str:
        """HLCTimestamp を compact 16 進数文字列に変換する（Rust/Go と互換）。"""
        # format_compact メソッドに委譲する
        return ts.format_compact()

    @staticmethod
    def to_b64(ts: HLCTimestamp) -> str:
        """HLCTimestamp を base64url エンコードされた文字列に変換する。"""
        # pack_b64 に委譲する
        return pack_b64(ts)

    @staticmethod
    def to_bytes(ts: HLCTimestamp) -> bytes:
        """HLCTimestamp を 12 バイトのビッグエンディアンバイナリに変換する。"""
        # struct.pack でシリアライズする
        return struct.pack(_STRUCT_FORMAT, ts.wall_ms, ts.logical, ts.node_id)

    @staticmethod
    def to_dict(ts: HLCTimestamp) -> Dict[str, int]:
        """HLCTimestamp を JSON 互換の辞書に変換する。"""
        # 3 フィールドを辞書に格納して返す
        return {
            "wall_ms": ts.wall_ms,
            "logical": ts.logical,
            "node_id": ts.node_id,
        }

    @staticmethod
    def to_grpc_metadata_value(ts: HLCTimestamp) -> str:
        """gRPC バイナリメタデータ（-bin サフィックス）用のエンコード文字列を返す。"""
        # バイナリシリアライズして base64 エンコードする
        raw = HLCSerializer.to_bytes(ts)
        # base64 エンコード（padding あり、標準 b64 を使用する）
        return base64.b64encode(raw).decode("ascii")


class HLCDeserializer:
    """
    各種形式から HLCTimestamp を復元するユーティリティクラス。

    HLCSerializer の逆操作。全メソッドはスタティックで状態を持たない。
    """

    @staticmethod
    def from_compact_str(s: str) -> Optional[HLCTimestamp]:
        """compact 16 進数文字列から HLCTimestamp を復元する。"""
        # parse_compact に委譲する
        return HLCTimestamp.parse_compact(s)

    @staticmethod
    def from_b64(s: str) -> Optional[HLCTimestamp]:
        """base64url 文字列から HLCTimestamp を復元する。"""
        # unpack_b64 に委譲する
        return unpack_b64(s)

    @staticmethod
    def from_bytes(data: bytes) -> Optional[HLCTimestamp]:
        """12 バイトのバイナリから HLCTimestamp を復元する。"""
        # バイト列の長さが 12 であることを確認する
        if len(data) != _STRUCT_SIZE:
            return None
        # struct.unpack でデシリアライズする
        try:
            wall_ms, logical, node_id = struct.unpack(_STRUCT_FORMAT, data)
        except struct.error:
            # アンパックに失敗した場合は None を返す
            return None
        # HLCTimestamp を生成して返す
        try:
            return HLCTimestamp(wall_ms=wall_ms, logical=logical, node_id=node_id)
        except ValueError:
            # バリデーションエラーが発生した場合は None を返す
            return None

    @staticmethod
    def from_dict(d: Dict[str, int]) -> Optional[HLCTimestamp]:
        """辞書から HLCTimestamp を復元する。"""
        # 必須フィールドを取得する
        try:
            wall_ms = int(d["wall_ms"])
            logical = int(d["logical"])
            node_id = int(d["node_id"])
        except (KeyError, TypeError, ValueError):
            # フィールドが欠けている場合は None を返す
            return None
        # HLCTimestamp を生成して返す
        try:
            return HLCTimestamp(wall_ms=wall_ms, logical=logical, node_id=node_id)
        except ValueError:
            # バリデーションエラーが発生した場合は None を返す
            return None

    @staticmethod
    def from_grpc_metadata_value(s: str) -> Optional[HLCTimestamp]:
        """gRPC バイナリメタデータ値から HLCTimestamp を復元する。"""
        # base64 デコードを試みる
        try:
            raw = base64.b64decode(s)
        except Exception:
            # デコードに失敗した場合は None を返す
            return None
        # from_bytes に委譲する
        return HLCDeserializer.from_bytes(raw)


@dataclasses.dataclass(frozen=True)
class IntervalCheckResult:
    """HLCIntervalCheck の結果を格納する不変データクラス。"""

    # is_within_bounds: 間隔が許容範囲内の場合に True
    is_within_bounds: bool
    # actual_interval_ms: 実際の間隔（ミリ秒）
    actual_interval_ms: int
    # max_allowed_ms: 許容上限（ミリ秒）
    max_allowed_ms: int
    # ts_a: 比較元のタイムスタンプ
    ts_a: HLCTimestamp
    # ts_b: 比較先のタイムスタンプ
    ts_b: HLCTimestamp


class HLCIntervalCheck:
    """
    2 つの HLCTimestamp の間隔が許容範囲内かどうかを検証するクラス。

    製造ラインのリアルタイム制御において、イベント間の時間間隔が
    許容範囲（SLO）を超えていないかを確認するために使用する。
    """

    def __init__(self, max_allowed_ms: int) -> None:
        """許容上限（ミリ秒）を受け取って HLCIntervalCheck を初期化する。"""
        # max_allowed_ms が正の値であることを検証する
        if max_allowed_ms <= 0:
            raise ValueError(f"max_allowed_ms は正の値でなければならない: {max_allowed_ms}")
        # 許容上限をインスタンス変数に保存する
        self._max_allowed_ms: int = max_allowed_ms

    def check(self, ts_a: HLCTimestamp, ts_b: HLCTimestamp) -> IntervalCheckResult:
        """
        ts_a と ts_b の間隔が許容上限内かどうかを検証する。

        ts_a と ts_b の順序に関わらず、wall_ms の差の絶対値で計算する。
        """
        # wall_ms の差を絶対値で計算する
        diff = abs(ts_a.wall_ms - ts_b.wall_ms)
        # 許容範囲内かどうかを判定する
        within_bounds = diff <= self._max_allowed_ms
        # 結果を生成して返す
        return IntervalCheckResult(
            is_within_bounds=within_bounds,
            actual_interval_ms=diff,
            max_allowed_ms=self._max_allowed_ms,
            ts_a=ts_a,
            ts_b=ts_b,
        )

    @property
    def max_allowed_ms(self) -> int:
        """許容上限（ミリ秒）を返す。"""
        # 許容上限を返す
        return self._max_allowed_ms


class DriftGuard:
    """
    受信 HLC タイムスタンプと自ローカル HLC クロックのドリフトを監視するガードクラス。

    max_drift_ms を超えるドリフトが検出された場合に DriftGuardViolation を raise する。
    製造業のリアルタイム制御において、クロック同期が維持されているかを継続監視するために使用する。
    """

    def __init__(self, clock: HLCClock, max_drift_ms: int) -> None:
        """HLCClock とドリフト許容上限を受け取って DriftGuard を初期化する。"""
        # max_drift_ms が正の値であることを検証する
        if max_drift_ms <= 0:
            raise ValueError(f"max_drift_ms は正の値でなければならない: {max_drift_ms}")
        # clock をインスタンス変数に保存する
        self._clock: HLCClock = clock
        # max_drift_ms をインスタンス変数に保存する
        self._max_drift_ms: int = max_drift_ms
        # 観測された最大ドリフトを追跡するカウンタを初期化する
        self._max_observed_drift: int = 0
        # ドリフト違反の回数を追跡するカウンタを初期化する
        self._violation_count: int = 0
        # Lock で統計情報を保護する
        self._stats_lock: threading.Lock = threading.Lock()

    def check_recv(self, msg_ts: HLCTimestamp) -> HLCTimestamp:
        """
        受信メッセージの HLC タイムスタンプを検証してローカルクロックを更新する。

        ドリフトが max_drift_ms を超えた場合は DriftGuardViolation を raise する。
        問題がなければ recv 後の HLCTimestamp を返す。
        """
        # ローカルクロックの現在値を取得する（tick を呼び出さず peek する）
        local_wall, _ = self._clock.peek_state()
        # msg_ts と local_wall の差を計算する
        drift = abs(int(msg_ts.wall_ms) - int(local_wall))
        # 統計情報を更新する
        with self._stats_lock:
            # 最大ドリフトを更新する
            if drift > self._max_observed_drift:
                self._max_observed_drift = drift
            # ドリフトが許容上限を超えた場合は違反カウントを増加させる
            if drift > self._max_drift_ms:
                self._violation_count += 1
                # DriftGuardViolation を raise する
                raise DriftGuardViolation(
                    msg_ts=msg_ts,
                    local_wall_ms=local_wall,
                    drift_ms=drift,
                    max_drift_ms=self._max_drift_ms,
                )
        # ローカルクロックを recv で更新して返す
        return self._clock.recv(msg_ts)

    @property
    def max_observed_drift_ms(self) -> int:
        """観測された最大ドリフト（ミリ秒）を返す。"""
        # 統計情報 Lock を取得して値を返す
        with self._stats_lock:
            return self._max_observed_drift

    @property
    def violation_count(self) -> int:
        """ドリフト違反の総回数を返す。"""
        # 統計情報 Lock を取得して値を返す
        with self._stats_lock:
            return self._violation_count


class DriftGuardViolation(Exception):
    """DriftGuard がドリフト許容上限を超えたときに raise する例外。"""

    def __init__(
        self,
        msg_ts: HLCTimestamp,
        local_wall_ms: int,
        drift_ms: int,
        max_drift_ms: int,
    ) -> None:
        """ドリフト違反の詳細を受け取って DriftGuardViolation を初期化する。"""
        # msg_ts を保存する
        self.msg_ts = msg_ts
        # local_wall_ms を保存する
        self.local_wall_ms = local_wall_ms
        # drift_ms を保存する
        self.drift_ms = drift_ms
        # max_drift_ms を保存する
        self.max_drift_ms = max_drift_ms
        # 親クラスのコンストラクタを呼び出す
        super().__init__(
            f"HLC ドリフト超過: {drift_ms}ms > {max_drift_ms}ms "
            f"(msg_wall={msg_ts.wall_ms}, local_wall={local_wall_ms})"
        )


@dataclasses.dataclass
class SequencedTimestamp:
    """HLC タイムスタンプとシーケンス番号を組み合わせた値オブジェクト。"""

    # seq: 単調増加するシーケンス番号
    seq: int
    # ts: HLC タイムスタンプ
    ts: HLCTimestamp


class HLCSequencer:
    """
    単調増加するシーケンス番号を HLC タイムスタンプと組み合わせて発行するクラス。

    製造ラインのコマンドキューにおいて、HLC タイムスタンプだけでは一意性が
    保証できないケースでシーケンス番号を付加して一意性を確保するために使用する。

    スレッドセーフ: threading.Lock で内部状態を保護する。
    """

    def __init__(self, clock: HLCClock, start_seq: int = 0) -> None:
        """HLCClock と開始シーケンス番号を受け取って HLCSequencer を初期化する。"""
        # clock をインスタンス変数に保存する
        self._clock: HLCClock = clock
        # _seq: 現在のシーケンス番号を初期化する
        self._seq: int = start_seq
        # _lock: スレッドセーフな状態更新のための Mutex
        self._lock: threading.Lock = threading.Lock()

    def next(self) -> SequencedTimestamp:
        """次のシーケンス番号と HLC タイムスタンプを組み合わせた SequencedTimestamp を返す。"""
        # Lock を取得してスレッドセーフにシーケンス番号を発行する
        with self._lock:
            # シーケンス番号を発行する（現在値を取得後にインクリメント）
            seq = self._seq
            # シーケンス番号をインクリメントする
            self._seq += 1
        # HLC タイムスタンプを生成する（clock.tick()）
        ts = self._clock.tick()
        # SequencedTimestamp を生成して返す
        return SequencedTimestamp(seq=seq, ts=ts)

    @property
    def current_seq(self) -> int:
        """現在のシーケンス番号（次に発行される番号）を返す。"""
        # Lock を取得してスレッドセーフにシーケンス番号を返す
        with self._lock:
            return self._seq


@dataclasses.dataclass
class HLCLogEntry:
    """HLCEventLog の単一エントリを表すデータクラス。"""

    # seq: イベントのシーケンス番号
    seq: int
    # ts: HLC タイムスタンプ
    ts: HLCTimestamp
    # event_type: イベントの種類を表す文字列
    event_type: str
    # payload: イベントのペイロード（任意の辞書）
    payload: Dict[str, object]


class HLCEventLog:
    """
    インメモリで HLC タイムスタンプ付きイベントを記録するログクラス。

    製造ラインのデバッグ・監査目的でイベントをメモリ上に保持する。
    max_entries を超えた場合は古いエントリを自動削除する（FIFO）。

    スレッドセーフ: threading.Lock で内部状態を保護する。
    """

    def __init__(self, clock: HLCClock, max_entries: int = 10_000) -> None:
        """HLCClock と最大保持エントリ数を受け取って HLCEventLog を初期化する。"""
        # max_entries が正の値であることを検証する
        if max_entries <= 0:
            raise ValueError(f"max_entries は正の値でなければならない: {max_entries}")
        # clock をインスタンス変数に保存する
        self._clock: HLCClock = clock
        # max_entries をインスタンス変数に保存する
        self._max_entries: int = max_entries
        # _entries: イベントログのエントリを格納する deque を初期化する
        self._entries: collections.deque = collections.deque(maxlen=max_entries)
        # _seq: シーケンス番号カウンタを初期化する
        self._seq: int = 0
        # _lock: スレッドセーフな状態更新のための Mutex
        self._lock: threading.Lock = threading.Lock()

    def append(self, event_type: str, payload: Optional[Dict[str, object]] = None) -> HLCLogEntry:
        """新しいイベントをログに追記して HLCLogEntry を返す。"""
        # HLC タイムスタンプを生成する
        ts = self._clock.tick()
        # Lock を取得してスレッドセーフにエントリを追加する
        with self._lock:
            # シーケンス番号を発行する
            seq = self._seq
            # シーケンス番号をインクリメントする
            self._seq += 1
            # HLCLogEntry を生成する
            entry = HLCLogEntry(
                seq=seq,
                ts=ts,
                event_type=event_type,
                payload=payload if payload is not None else {},
            )
            # deque にエントリを追加する（max_entries を超えた場合は古いエントリが自動削除される）
            self._entries.append(entry)
        # 生成したエントリを返す
        return entry

    def entries(self) -> List[HLCLogEntry]:
        """現在のログエントリのリストを返す（新しい順）。"""
        # Lock を取得してスレッドセーフに読み取る
        with self._lock:
            # deque のコピーをリストとして返す
            return list(self._entries)

    def entries_since(self, ts: HLCTimestamp) -> List[HLCLogEntry]:
        """指定した HLCTimestamp 以降のエントリのリストを返す。"""
        # Lock を取得してスレッドセーフに読み取る
        with self._lock:
            # ts 以降のエントリのみをフィルタリングして返す
            return [e for e in self._entries if e.ts >= ts]

    def __iter__(self) -> Iterator[HLCLogEntry]:
        """ログエントリを順にイテレートする。"""
        # Lock を取得してスレッドセーフにイテレータを返す
        with self._lock:
            entries_copy = list(self._entries)
        # コピーしたリストからイテレータを返す
        return iter(entries_copy)

    def __len__(self) -> int:
        """現在のログエントリ数を返す。"""
        # Lock を取得してスレッドセーフに長さを返す
        with self._lock:
            return len(self._entries)
