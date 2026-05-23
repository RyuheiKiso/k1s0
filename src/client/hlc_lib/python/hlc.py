#!/usr/bin/env python3
"""
hlc.py — k1s0-hlc: Hybrid Logical Clock (HLC) の Python 実装

Kulkarni et al. (2014) "Logical Physical Clocks" のアルゴリズムを Python で実装する。
wall-clock 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、TTL / deadline の
比較は HLC elapsed で管理し、time.time() を呼び出し元コードから直接参照しない。
本モジュールが time.time() を扱う唯一の許可された場所であり、他モジュールは本クラスを経由する。

Rust 実装（src/client/hlc_lib/rust/src/lib.rs）および Go 実装（hlc.go）と
同等の API を提供し、wall_ms / logical / node_id の 3 フィールド構造を維持する。

製造業プラットフォーム k1s0 では HLC を用いて以下を実現する:
  - 分散ノード間のイベント因果順序の一意な全順序化
  - gRPC ストリーミングメッセージのカウザリティトラッキング
  - multi-tenant RLS 境界をまたぐ操作のタイムスタンプ検証
  - ops/infra/data 各軸のイベントログの drift 検出（crosscutting c11）
"""

from __future__ import annotations

# base64: HLC タイムスタンプをバイナリエンコードして安全な文字列に変換するために使用する
import base64
# dataclasses: HLCTimestamp の不変値オブジェクトを簡潔に定義するために使用する
import dataclasses
# struct: HLC タイムスタンプのバイナリパック/アンパックに使用する
import struct
# threading: HLCClock の状態を Mutex（Lock）で保護するために使用する
import threading
# time: wall-clock の wall_ms 取得（HLCClock 内部のみ許可）
import time
# typing: 型ヒントに使用する
from typing import Optional, Tuple

# _STRUCT_FORMAT: HLC タイムスタンプのバイナリフォーマット（big-endian: u64 + u16 + u16 = 12 bytes）
_STRUCT_FORMAT: str = ">QHH"
# _STRUCT_SIZE: pack 後のバイト数（12 バイト固定）
_STRUCT_SIZE: int = struct.calcsize(_STRUCT_FORMAT)
# _U16_MAX: u16 の最大値（logical counter overflow 検出に使用する）
_U16_MAX: int = 65535
# _MS_PER_SEC: 1秒あたりのミリ秒数（time.time() を ms に変換するために使用する）
_MS_PER_SEC: int = 1000


@dataclasses.dataclass(frozen=True, order=False)
class HLCTimestamp:
    """
    HLC のタイムスタンプ（wall_ms + logical + node_id の 3 tuple）。

    全順序（wall_ms > logical > node_id の辞書順）で比較可能。
    frozen=True により不変オブジェクトとして扱い、dict key や set 要素として使用できる。
    """

    # wall_ms: UNIX epoch からの経過ミリ秒（wall-clock 部分、記録目的のみ）
    wall_ms: int
    # logical: 同一 wall_ms 内の単調カウンタ（0 以上 65535 以下）
    logical: int
    # node_id: ノード識別子（複数インスタンスでの衝突回避、環境変数 HLC_NODE_ID で指定）
    node_id: int

    def __post_init__(self) -> None:
        # wall_ms が 0 以上の整数であることを検証する
        if self.wall_ms < 0:
            raise ValueError(f"wall_ms は 0 以上でなければならない: {self.wall_ms}")
        # logical が 0 以上 65535 以下であることを検証する
        if not (0 <= self.logical <= _U16_MAX):
            raise ValueError(f"logical は 0 から {_U16_MAX} の範囲でなければならない: {self.logical}")
        # node_id が 0 以上 65535 以下であることを検証する
        if not (0 <= self.node_id <= _U16_MAX):
            raise ValueError(f"node_id は 0 から {_U16_MAX} の範囲でなければならない: {self.node_id}")

    def _key(self) -> Tuple[int, int, int]:
        """全順序比較のためのキータプルを返す（内部使用）。"""
        # (wall_ms, logical, node_id) の辞書順でキーを構築する
        return (self.wall_ms, self.logical, self.node_id)

    def __lt__(self, other: object) -> bool:
        # other が HLCTimestamp でない場合は NotImplemented を返す
        if not isinstance(other, HLCTimestamp):
            return NotImplemented
        # キータプルで辞書順比較を行う
        return self._key() < other._key()

    def __le__(self, other: object) -> bool:
        # other が HLCTimestamp でない場合は NotImplemented を返す
        if not isinstance(other, HLCTimestamp):
            return NotImplemented
        # キータプルで辞書順比較（以下）を行う
        return self._key() <= other._key()

    def __gt__(self, other: object) -> bool:
        # other が HLCTimestamp でない場合は NotImplemented を返す
        if not isinstance(other, HLCTimestamp):
            return NotImplemented
        # キータプルで辞書順比較（より大きい）を行う
        return self._key() > other._key()

    def __ge__(self, other: object) -> bool:
        # other が HLCTimestamp でない場合は NotImplemented を返す
        if not isinstance(other, HLCTimestamp):
            return NotImplemented
        # キータプルで辞書順比較（以上）を行う
        return self._key() >= other._key()

    def __eq__(self, other: object) -> bool:
        # other が HLCTimestamp でない場合は NotImplemented を返す
        if not isinstance(other, HLCTimestamp):
            return NotImplemented
        # 全フィールドが同一の場合のみ等値とする
        return self._key() == other._key()

    def __hash__(self) -> int:
        # frozen dataclass として hash を定義する（dict key として使用可能にする）
        return hash(self._key())

    def format_compact(self) -> str:
        """
        HLCTimestamp を compact 文字列に変換する。

        形式: "{wall_ms_hex_16}-{logical_04x}-{node_04x}"
        Rust/Go 実装と互換性のある文字列表現を返す。
        """
        # 16 桁 hex + 4 桁 hex + 4 桁 hex の形式でフォーマットする
        return f"{self.wall_ms:016x}-{self.logical:04x}-{self.node_id:04x}"

    @classmethod
    def parse_compact(cls, s: str) -> Optional["HLCTimestamp"]:
        """
        compact 文字列から HLCTimestamp を復元する。

        parse に失敗した場合は None を返す（不正入力を呼び出し元で処理させる）。
        """
        # 入力文字列の長さが期待値（26 文字）と一致することを確認する
        if len(s) != 26:
            return None
        # ハイフン区切りで 3 パートに分割する
        parts = s.split("-")
        # パートが 3 つでない場合は None を返す
        if len(parts) != 3:
            return None
        # 各パートを16進数整数にパースする
        try:
            # wall_ms パート: 16 桁 hex → int
            wall_ms = int(parts[0], 16)
            # logical パート: 4 桁 hex → int
            logical = int(parts[1], 16)
            # node_id パート: 4 桁 hex → int
            node_id = int(parts[2], 16)
        except ValueError:
            # 16進数パースに失敗した場合は None を返す
            return None
        # パースに成功した場合は HLCTimestamp を生成して返す
        return cls(wall_ms=wall_ms, logical=logical, node_id=node_id)

    def add_ms(self, duration_ms: int) -> "HLCTimestamp":
        """
        self に duration_ms を加算した deadline 用 HLCTimestamp を返す。

        wall-clock の直接使用を禁止するため、deadline 表現はこのメソッドを経由する。
        """
        # duration_ms が負の場合はエラーを raise する
        if duration_ms < 0:
            raise ValueError(f"duration_ms は 0 以上でなければならない: {duration_ms}")
        # wall_ms に duration_ms を加算する（オーバーフローは Python の任意精度整数で扱う）
        new_wall = self.wall_ms + duration_ms
        # deadline の先頭イベントを表すため logical を 0 にリセットする
        # node_id は引き継ぐ（deadline の発行者を追跡する）
        return HLCTimestamp(wall_ms=new_wall, logical=0, node_id=self.node_id)

    def elapsed_ms_since(self, reference: "HLCTimestamp") -> int:
        """
        reference から self までの経過ミリ秒を返す。

        self が reference より前の場合は 0 を返す（負の elapsed は表現しない）。
        deadline との差分比較（expired 判定）に使用する。
        """
        # wall_ms の差分を計算する（負になる場合は 0 に飽和する）
        diff = self.wall_ms - reference.wall_ms
        # 差分が負の場合は 0 に飽和させる
        return max(0, diff)

    def is_expired_at(self, current: "HLCTimestamp") -> bool:
        """
        deadline（self）と比較して current が期限切れかどうかを返す。

        True = current が self（deadline）を超えた = 期限切れ。
        """
        # current が self 以上（後または同時）なら期限切れ
        return current >= self


# EPOCH: wall_ms=0, logical=0, node_id=0 の最小タイムスタンプ（未初期化判定に使用する）
EPOCH: HLCTimestamp = HLCTimestamp(wall_ms=0, logical=0, node_id=0)


def before(a: HLCTimestamp, b: HLCTimestamp) -> bool:
    """
    a が b より前（happened-before）かどうかを返す。

    HLC では a < b が happened-before 関係（a が b の原因になりうる）を表す。
    """
    # a のキーが b のキーより小さい場合は happened-before 関係が成立する
    return a < b


def concurrent(a: HLCTimestamp, b: HLCTimestamp) -> bool:
    """
    a と b が同時（concurrent）かどうかを返す。

    HLC では a == b の場合のみ concurrent と見なす（node_id が違えば別イベント）。
    注意: 実際の分散システムでは node_id まで含めて等値になることは稀であり、
    ほとんどの場合 a < b または a > b になる。
    """
    # 全フィールドが同一の場合のみ concurrent と判定する
    return a == b


def pack_b64(ts: HLCTimestamp) -> str:
    """
    HLCTimestamp を base64 エンコードされた文字列に変換する。

    gRPC メタデータや HTTP ヘッダーでの伝搬に使用する。
    エンコード形式: big-endian u64(wall_ms) + u16(logical) + u16(node_id) = 12 bytes を base64url で表現する。
    """
    # struct.pack で 12 バイトのバイナリに変換する
    raw = struct.pack(_STRUCT_FORMAT, ts.wall_ms, ts.logical, ts.node_id)
    # base64url エンコードして文字列に変換する（padding なし）
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def unpack_b64(s: str) -> Optional[HLCTimestamp]:
    """
    base64 エンコードされた文字列から HLCTimestamp を復元する。

    pack_b64 の逆操作。デコードに失敗した場合は None を返す。
    """
    # base64url デコードのために padding を補完する
    padded = s + "=" * (4 - len(s) % 4 if len(s) % 4 else 0)
    # base64url デコードを試みる
    try:
        raw = base64.urlsafe_b64decode(padded)
    except Exception:
        # デコードに失敗した場合は None を返す
        return None
    # バイト列の長さが期待値（12 バイト）と一致することを確認する
    if len(raw) != _STRUCT_SIZE:
        return None
    # struct.unpack で 3 フィールドに分解する
    try:
        wall_ms, logical, node_id = struct.unpack(_STRUCT_FORMAT, raw)
    except struct.error:
        # アンパックに失敗した場合は None を返す
        return None
    # HLCTimestamp を生成して返す
    try:
        return HLCTimestamp(wall_ms=wall_ms, logical=logical, node_id=node_id)
    except ValueError:
        # バリデーションエラーが発生した場合は None を返す
        return None


def _wall_ms_now() -> int:
    """現在の UNIX epoch からの経過ミリ秒を返す（モジュール内部専用）。"""
    # time.time() を呼び出す（本モジュール内でのみ許可、呼び出し元では禁止）
    raw_s = time.time()
    # 秒をミリ秒に変換して整数に丸める
    ms = int(raw_s * _MS_PER_SEC)
    # 負値（理論上発生しないが安全策）は 0 に飽和させる
    return max(0, ms)


class HLCClock:
    """
    スレッドセーフな HLC クロック。

    tick / recv / now の 3 操作でイベント間の因果関係を追跡する。
    複数スレッドから threading.Lock を通じて安全に使用できる。
    Rust 実装（HlcClock）および Go 実装（HlcClock）と同等の API を提供する。
    """

    def __init__(self, node_id: int = 0) -> None:
        """node_id を受け取って HLCClock を初期化する。"""
        # node_id の範囲を検証する
        if not (0 <= node_id <= _U16_MAX):
            raise ValueError(f"node_id は 0 から {_U16_MAX} の範囲でなければならない: {node_id}")
        # node_id をインスタンス変数に保存する
        self._node_id: int = node_id
        # _state_wall: 最後に観測した wall_ms（単調増加を保証するための状態）
        self._state_wall: int = 0
        # _state_logical: 最後に使用した logical カウンタ
        self._state_logical: int = 0
        # _lock: スレッドセーフな状態更新のための Mutex
        self._lock: threading.Lock = threading.Lock()

    @classmethod
    def from_env(cls) -> "HLCClock":
        """
        環境変数 HLC_NODE_ID から node_id を読み込んで HLCClock を生成する。

        HLC_NODE_ID が未設定 / パース失敗時は node_id=0 を使用する。
        """
        # os モジュールをインポートして環境変数を取得する
        import os
        # HLC_NODE_ID 環境変数を文字列として取得する
        node_id_str = os.environ.get("HLC_NODE_ID", "")
        # 環境変数が未設定 / 空の場合は node_id=0 を使用する
        if not node_id_str:
            return cls(node_id=0)
        # 文字列を整数にパースする
        try:
            node_id = int(node_id_str)
        except ValueError:
            # パース失敗の場合は node_id=0 を使用する
            return cls(node_id=0)
        # 範囲外の場合は node_id=0 を使用する
        if not (0 <= node_id <= _U16_MAX):
            return cls(node_id=0)
        # パース成功した node_id を使って HLCClock を初期化する
        return cls(node_id=node_id)

    def tick(self) -> HLCTimestamp:
        """
        send/local event のタイムスタンプを生成する（HLC の "send event"）。

        アルゴリズム:
          l' = max(_state_wall, pt)
          if l' == _state_wall: c' = _state_logical + 1
          else: c' = 0
        """
        # Lock を取得してスレッドセーフに状態を更新する
        with self._lock:
            # pt: 現在の物理クロック（ms）を取得する
            pt = _wall_ms_now()
            # l': max(_state_wall, pt) を計算する（単調増加を保証する）
            new_wall = max(self._state_wall, pt)
            # c': wall_ms が変化したかどうかで logical を更新する
            if new_wall == self._state_wall:
                # wall_ms が変わらなければ logical を +1 する
                new_logical = self._state_logical + 1
                # logical が u16 の最大値を超えた場合はエラーを raise する
                if new_logical > _U16_MAX:
                    raise OverflowError("HLC logical counter overflow (max 65535)")
            else:
                # wall_ms が進んだ場合は logical を 0 にリセットする
                new_logical = 0
            # ステートを更新する（次回の tick/recv の比較基準になる）
            self._state_wall = new_wall
            self._state_logical = new_logical
            # 生成した HLCTimestamp を返す
            return HLCTimestamp(wall_ms=new_wall, logical=new_logical, node_id=self._node_id)

    def recv(self, msg_ts: HLCTimestamp) -> HLCTimestamp:
        """
        受信メッセージのタイムスタンプを踏まえてローカルクロックを更新する（HLC の "receive event"）。

        アルゴリズム（3-way max）:
          l' = max(_state_wall, msg_ts.wall_ms, pt)
          3 者の最大一致に応じて logical を更新する
        """
        # Lock を取得してスレッドセーフに状態を更新する
        with self._lock:
            # pt: 現在の物理クロックを取得する
            pt = _wall_ms_now()
            # l': max(_state_wall, msg_ts.wall_ms, pt) を計算する（3-way max）
            new_wall = max(self._state_wall, msg_ts.wall_ms, pt)
            # c': 3-way の最大一致パターンに応じて logical を更新する
            if new_wall == self._state_wall and new_wall == msg_ts.wall_ms:
                # 3 者の wall_ms が同一: max(local_logical, msg_logical) + 1
                max_logical = max(self._state_logical, msg_ts.logical)
                # logical がオーバーフローする場合はエラーを raise する
                new_logical = max_logical + 1
                if new_logical > _U16_MAX:
                    raise OverflowError("HLC logical counter overflow")
            elif new_wall == self._state_wall:
                # ローカルの wall_ms が最大: local_logical + 1
                new_logical = self._state_logical + 1
                # logical がオーバーフローする場合はエラーを raise する
                if new_logical > _U16_MAX:
                    raise OverflowError("HLC logical counter overflow")
            elif new_wall == msg_ts.wall_ms:
                # 受信メッセージの wall_ms が最大: msg_logical + 1
                new_logical = msg_ts.logical + 1
                # logical がオーバーフローする場合はエラーを raise する
                if new_logical > _U16_MAX:
                    raise OverflowError("HLC logical counter overflow")
            else:
                # pt が最大（物理クロックが両者を上回った）: logical を 0 にリセットする
                new_logical = 0
            # ステートを更新する
            self._state_wall = new_wall
            self._state_logical = new_logical
            # 更新後の HLCTimestamp を返す
            return HLCTimestamp(wall_ms=new_wall, logical=new_logical, node_id=self._node_id)

    def now(self) -> HLCTimestamp:
        """
        現在の HLC タイムスタンプを生成する（tick の alias）。

        キャッシュエントリの cached_at_hlc フィールドへの書き込みに使用する。
        """
        # tick と等価：send event として扱う
        return self.tick()

    def peek_state(self) -> Tuple[int, int]:
        """
        現在の内部状態（wall_ms, logical）を返す（デバッグ・テスト用）。

        production コードでは使用しない。
        """
        # Lock を取得してスレッドセーフに状態を読み取る
        with self._lock:
            # (_state_wall, _state_logical) のタプルを返す
            return (self._state_wall, self._state_logical)

    @property
    def node_id(self) -> int:
        """このクロックの node_id を返す（読み取り専用）。"""
        # _node_id を返す（変更不可）
        return self._node_id
