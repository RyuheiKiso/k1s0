"""src/data/audit_chain/compaction.py

監査チェーンのコンパクション（圧縮）モジュール。
大量の監査イベントを Merkle ツリーのチェックポイントに集約し、
ストレージを削減しながら整合性を保証する。
仕様: docs/04_詳細設計/01_適合仕様/14_データ保全適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# 数学関数に使用する math をインポートする
import math
# OS 操作に使用する os をインポートする
import os
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timedelta, timezone
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Tuple

# pii_cluster の AuditEvent をインポートする
from ..pii_cluster.audit_hook import AuditEvent

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)


# ===========================================================================
# Merkle ツリー実装
# ===========================================================================


def _compute_leaf_hash(event: AuditEvent) -> str:
    """監査イベントから Merkle ツリーのリーフハッシュを計算して返す。"""
    # イベントの識別子・タイムスタンプ・操作をシリアライズする
    leaf_data = json.dumps(
        {
            "event_id": event.event_id,
            "timestamp_hlc": event.timestamp_hlc,
            "operation": event.operation,
            "table_name": event.table_name,
            "chain_hash": event.chain_hash,
        },
        sort_keys=True,
    )
    # SHA-256 ハッシュを計算して返す
    return hashlib.sha256(leaf_data.encode("utf-8")).hexdigest()


def _compute_parent_hash(left: str, right: str) -> str:
    """Merkle ツリーの 2 つの子ハッシュから親ハッシュを計算して返す。"""
    # 左右のハッシュを結合して親ハッシュを計算する
    combined = left + right
    # SHA-256 ハッシュを計算して返す
    return hashlib.sha256(combined.encode("utf-8")).hexdigest()


def compute_merkle_root(events: List[AuditEvent]) -> str:
    """監査イベントリストから Merkle ツリーのルートハッシュを計算して返す。

    Args:
        events: Merkle ツリーを計算する AuditEvent リスト

    Returns:
        Merkle ルートハッシュの16進数文字列（空リストの場合は空文字）
    """
    # イベントが空の場合は空文字を返す
    if not events:
        # 空リストに対するルートハッシュは空文字
        return ""
    # リーフハッシュのリストを計算する
    hashes: List[str] = [_compute_leaf_hash(e) for e in events]
    # ハッシュリストが 1 件の場合はそのまま返す
    if len(hashes) == 1:
        # 単一イベントのルートはそのリーフハッシュ
        return hashes[0]
    # Merkle ツリーを構築する（ボトムアップ）
    while len(hashes) > 1:
        # 奇数個の場合は最後のハッシュを複製してペアを揃える
        if len(hashes) % 2 != 0:
            # 最後のハッシュを複製して偶数個にする
            hashes.append(hashes[-1])
        # 親ハッシュのリストを生成する
        parent_hashes: List[str] = []
        # 2 個ずつペアにして親ハッシュを計算する
        for idx in range(0, len(hashes), 2):
            # 左右の子ハッシュから親ハッシュを計算する
            parent = _compute_parent_hash(hashes[idx], hashes[idx + 1])
            # 親ハッシュリストに追加する
            parent_hashes.append(parent)
        # ハッシュリストを親ハッシュリストで置き換える
        hashes = parent_hashes
    # ルートハッシュを返す
    return hashes[0]


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class CompactionPolicy:
    """コンパクションのポリシーを表すデータクラス。"""

    # コンパクション対象イベントの最大経過日数
    max_age_days: int
    # 1 バッチあたりの最大イベント件数
    batch_size: int
    # チェックポイントの生成間隔（イベント件数単位）
    checkpoint_interval: int
    # コンパクション後に元のイベントを削除するかどうか
    delete_after_compaction: bool = False
    # Merkle ツリーの深さの上限
    max_merkle_depth: int = 20

    def validate(self) -> bool:
        """ポリシーが有効かどうかを検証して返す。"""
        # max_age_days が 1 以上であることを確認する
        if self.max_age_days < 1:
            # 無効な値の場合は False を返す
            return False
        # batch_size が 1 以上であることを確認する
        if self.batch_size < 1:
            # 無効な値の場合は False を返す
            return False
        # checkpoint_interval が 1 以上であることを確認する
        if self.checkpoint_interval < 1:
            # 無効な値の場合は False を返す
            return False
        # 全検証通過の場合は True を返す
        return True


@dataclass
class Checkpoint:
    """監査チェーンのチェックポイントを表すデータクラス。"""

    # チェックポイントの一意識別子
    checkpoint_id: str
    # チェックポイントが対象とするイベントの開始時刻
    from_timestamp: str
    # チェックポイントが対象とするイベントの終了時刻
    to_timestamp: str
    # Merkle ツリーのルートハッシュ
    merkle_root: str
    # チェックポイントに含まれるイベント件数
    event_count: int
    # チェックポイント生成時刻
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    # チェックポイントのメタデータ辞書
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """チェックポイントを辞書形式に変換して返す。"""
        # 辞書形式に変換して返す
        return {
            "checkpoint_id": self.checkpoint_id,
            "from_timestamp": self.from_timestamp,
            "to_timestamp": self.to_timestamp,
            "merkle_root": self.merkle_root,
            "event_count": self.event_count,
            "created_at": self.created_at,
            "metadata": self.metadata,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Checkpoint":
        """辞書から Checkpoint を生成して返す。"""
        # 辞書から Checkpoint インスタンスを生成する
        return cls(
            checkpoint_id=data["checkpoint_id"],
            from_timestamp=data["from_timestamp"],
            to_timestamp=data["to_timestamp"],
            merkle_root=data["merkle_root"],
            event_count=int(data["event_count"]),
            created_at=data.get("created_at", ""),
            metadata=dict(data.get("metadata", {})),
        )


@dataclass
class CompactedBatch:
    """コンパクション済みのイベントバッチを表すデータクラス。"""

    # バッチの識別子
    batch_id: str
    # バッチに含まれる元のイベント件数
    original_event_count: int
    # バッチの Merkle ルートハッシュ
    merkle_root: str
    # バッチの対象期間（開始時刻）
    from_timestamp: str
    # バッチの対象期間（終了時刻）
    to_timestamp: str
    # 圧縮後のデータサイズ（バイト）
    compressed_size_bytes: int = 0
    # バッチ生成時刻
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    # バッチの整合性が検証済みかどうか
    integrity_verified: bool = False

    def compression_ratio(self, original_size_bytes: int) -> float:
        """圧縮率（0.0〜1.0）を計算して返す。"""
        # 元のサイズが 0 の場合は 0.0 を返す
        if original_size_bytes == 0:
            # ゼロ除算を防ぐため 0.0 を返す
            return 0.0
        # 圧縮後サイズ / 元のサイズを返す
        return self.compressed_size_bytes / original_size_bytes


# ===========================================================================
# ChainCompactor クラス
# ===========================================================================


class ChainCompactor:
    """監査チェーンのコンパクションを管理するクラス。

    大量の監査イベントを Merkle ツリーのチェックポイントに集約し、
    ストレージ使用量を削減しながら整合性を保証する。
    """

    def __init__(self, policy: Optional[CompactionPolicy] = None) -> None:
        """ChainCompactor をポリシーで初期化する。

        Args:
            policy: コンパクションポリシー（省略時はデフォルト値を使用）
        """
        # ポリシーを設定する（省略時はデフォルト値を使用）
        self._policy = policy or CompactionPolicy(
            max_age_days=90,
            batch_size=10000,
            checkpoint_interval=1000,
        )
        # チェックポイントの保存先ディレクトリを初期化する
        self._checkpoint_dir: Optional[Path] = None

    def set_checkpoint_dir(self, directory: Path) -> None:
        """チェックポイントファイルの保存先ディレクトリを設定する。"""
        # ディレクトリが存在しない場合は作成する
        directory.mkdir(parents=True, exist_ok=True)
        # チェックポイントディレクトリを設定する
        self._checkpoint_dir = directory
        # 設定をログに記録する
        logger.info("チェックポイントディレクトリ設定: %s", directory)

    def checkpoint(
        self, events: List[AuditEvent], up_to: datetime
    ) -> Checkpoint:
        """指定時刻までのイベントからチェックポイントを生成して返す。

        Args:
            events: チェックポイント対象の AuditEvent リスト
            up_to: この時刻以前のイベントのみを対象とする

        Returns:
            生成した Checkpoint インスタンス
        """
        # up_to 以前のイベントのみを対象とする
        target_events = [
            e for e in events
            if e.timestamp_hlc <= up_to.isoformat()
        ]
        # 対象イベントが空の場合は空のチェックポイントを返す
        if not target_events:
            # 空のチェックポイントを生成して返す
            return Checkpoint(
                checkpoint_id=self._generate_checkpoint_id(),
                from_timestamp="",
                to_timestamp=up_to.isoformat(),
                merkle_root="",
                event_count=0,
            )
        # Merkle ルートハッシュを計算する
        merkle_root = compute_merkle_root(target_events)
        # 対象イベントの開始時刻を取得する
        from_ts = target_events[0].timestamp_hlc
        # 対象イベントの終了時刻を取得する
        to_ts = target_events[-1].timestamp_hlc
        # チェックポイント ID を生成する
        checkpoint_id = self._generate_checkpoint_id()
        # Checkpoint を生成する
        cp = Checkpoint(
            checkpoint_id=checkpoint_id,
            from_timestamp=from_ts,
            to_timestamp=to_ts,
            merkle_root=merkle_root,
            event_count=len(target_events),
            metadata={
                "policy_max_age_days": self._policy.max_age_days,
                "policy_batch_size": self._policy.batch_size,
            },
        )
        # チェックポイントをファイルに保存する（保存先が設定されている場合）
        if self._checkpoint_dir is not None:
            # チェックポイントをファイルに保存する
            self._save_checkpoint_to_file(cp)
        # チェックポイント生成をログに記録する
        logger.info(
            "チェックポイント生成完了: id=%s events=%d merkle=%s",
            checkpoint_id,
            len(target_events),
            merkle_root[:16],
        )
        # チェックポイントを返す
        return cp

    def compact_batch(self, events: List[AuditEvent]) -> CompactedBatch:
        """イベントリストをコンパクション済みバッチに圧縮して返す。

        Args:
            events: 圧縮対象の AuditEvent リスト

        Returns:
            圧縮された CompactedBatch インスタンス
        """
        # イベントが空の場合は空のバッチを返す
        if not events:
            # 空のバッチを返す
            return CompactedBatch(
                batch_id=self._generate_batch_id(),
                original_event_count=0,
                merkle_root="",
                from_timestamp="",
                to_timestamp="",
            )
        # バッチサイズの制限を適用する
        if len(events) > self._policy.batch_size:
            # バッチサイズを超える場合は最初の batch_size 件に切り詰める
            logger.warning(
                "バッチサイズ超過: %d > %d、最初の %d 件のみ処理します",
                len(events),
                self._policy.batch_size,
                self._policy.batch_size,
            )
            # バッチサイズ分だけ切り取る
            events = events[: self._policy.batch_size]
        # Merkle ルートハッシュを計算する
        merkle_root = compute_merkle_root(events)
        # バッチの開始・終了時刻を取得する
        from_ts = events[0].timestamp_hlc
        # バッチの終了時刻を取得する
        to_ts = events[-1].timestamp_hlc
        # バッチ ID を生成する
        batch_id = self._generate_batch_id()
        # イベントリストの JSON サイズを計算する（元のサイズの近似値）
        json_size = len(
            json.dumps(
                [
                    {
                        "event_id": e.event_id,
                        "timestamp_hlc": e.timestamp_hlc,
                        "operation": e.operation,
                        "chain_hash": e.chain_hash,
                    }
                    for e in events
                ]
            ).encode("utf-8")
        )
        # CompactedBatch を生成して返す
        batch = CompactedBatch(
            batch_id=batch_id,
            original_event_count=len(events),
            merkle_root=merkle_root,
            from_timestamp=from_ts,
            to_timestamp=to_ts,
            compressed_size_bytes=json_size,
        )
        # バッチ生成をログに記録する
        logger.info(
            "バッチコンパクション完了: id=%s events=%d merkle=%s",
            batch_id,
            len(events),
            merkle_root[:16],
        )
        # コンパクション済みバッチを返す
        return batch

    def merge_checkpoints(self, old: Checkpoint, new: Checkpoint) -> Checkpoint:
        """2 つのチェックポイントをマージして新しいチェックポイントを返す。

        Args:
            old: 古いチェックポイント
            new: 新しいチェックポイント

        Returns:
            マージされた新しい Checkpoint インスタンス
        """
        # 両方のチェックポイントの Merkle ルートを結合してマージハッシュを計算する
        merged_merkle = _compute_parent_hash(old.merkle_root, new.merkle_root)
        # イベント件数を合計する
        total_events = old.event_count + new.event_count
        # from_timestamp は古い方を使用する
        from_ts = old.from_timestamp if old.from_timestamp else new.from_timestamp
        # to_timestamp は新しい方を使用する
        to_ts = new.to_timestamp if new.to_timestamp else old.to_timestamp
        # マージされたチェックポイント ID を生成する
        merged_id = self._generate_checkpoint_id()
        # マージチェックポイントのメタデータを構築する
        metadata = {
            "merged_from": [old.checkpoint_id, new.checkpoint_id],
            "old_merkle": old.merkle_root,
            "new_merkle": new.merkle_root,
        }
        # マージされた Checkpoint を生成して返す
        merged = Checkpoint(
            checkpoint_id=merged_id,
            from_timestamp=from_ts,
            to_timestamp=to_ts,
            merkle_root=merged_merkle,
            event_count=total_events,
            metadata=metadata,
        )
        # マージ完了をログに記録する
        logger.info(
            "チェックポイントマージ完了: id=%s events=%d",
            merged_id,
            total_events,
        )
        # マージされたチェックポイントを返す
        return merged

    def verify_compacted_integrity(self, batch: CompactedBatch) -> bool:
        """コンパクション済みバッチの整合性を検証して返す。

        Merkle ルートハッシュが存在し、バッチの基本構造が正しいかを確認する。

        Args:
            batch: 検証対象の CompactedBatch インスタンス

        Returns:
            整合性が確認できれば True
        """
        # Merkle ルートハッシュが存在するかを確認する
        if not batch.merkle_root:
            # Merkle ルートが空の場合は無効とする
            logger.warning("整合性検証失敗: merkle_root が空です (batch=%s)", batch.batch_id)
            # False を返す
            return False
        # Merkle ルートハッシュが有効な16進数文字列かを確認する
        try:
            # 16進数として解釈できるかを確認する
            int(batch.merkle_root, 16)
        except ValueError:
            # 無効な16進数の場合はログに記録して False を返す
            logger.warning(
                "整合性検証失敗: merkle_root が無効な16進数です (batch=%s)",
                batch.batch_id,
            )
            # False を返す
            return False
        # Merkle ルートの長さを確認する（SHA-256 = 64 文字）
        if len(batch.merkle_root) != 64:
            # 長さが 64 文字でない場合は無効とする
            logger.warning(
                "整合性検証失敗: merkle_root の長さが不正です (batch=%s len=%d)",
                batch.batch_id,
                len(batch.merkle_root),
            )
            # False を返す
            return False
        # イベント件数が 0 より大きいことを確認する
        if batch.original_event_count <= 0:
            # イベント件数が 0 以下の場合は無効とする
            logger.warning(
                "整合性検証失敗: event_count が不正です (batch=%s count=%d)",
                batch.batch_id,
                batch.original_event_count,
            )
            # False を返す
            return False
        # 全検証通過の場合は True を返す
        return True

    def restore_from_checkpoint(self, cp: Checkpoint) -> List[AuditEvent]:
        """チェックポイントからイベントリストを復元して返す。

        チェックポイントはメタデータとハッシュのみを保持するため、
        実際のイベントデータはデータベースから再取得する必要がある。
        この実装では、チェックポイントのメタデータに基づいて
        ダミーの AuditEvent リストを返す（実運用はデータベースから取得）。

        Args:
            cp: 復元元のチェックポイント

        Returns:
            復元された AuditEvent のリスト（チェックポイントの情報のみ）
        """
        # チェックポイントの情報をログに記録する
        logger.info(
            "チェックポイントから復元: id=%s events=%d from=%s to=%s",
            cp.checkpoint_id,
            cp.event_count,
            cp.from_timestamp,
            cp.to_timestamp,
        )
        # チェックポイントのディレクトリからイベントファイルを検索する
        if self._checkpoint_dir is not None:
            # チェックポイントのイベントファイルパスを構築する
            events_file = self._checkpoint_dir / f"{cp.checkpoint_id}_events.json"
            # ファイルが存在する場合は読み込む
            if events_file.exists():
                # イベントファイルを読み込む
                try:
                    # JSON ファイルを読み込む
                    raw = events_file.read_text(encoding="utf-8")
                    # JSON をパースする
                    events_data = json.loads(raw)
                    # AuditEvent のリストを生成する
                    restored_events: List[AuditEvent] = []
                    # 各イベントデータを AuditEvent に変換する
                    for ev_data in events_data:
                        # AuditEvent を生成する
                        event = AuditEvent(
                            event_id=ev_data.get("event_id", ""),
                            timestamp_hlc=ev_data.get("timestamp_hlc", ""),
                            actor_id=ev_data.get("actor_id", ""),
                            tenant_id=ev_data.get("tenant_id", ""),
                            operation=ev_data.get("operation", ""),
                            table_name=ev_data.get("table_name", ""),
                            row_keys=ev_data.get("row_keys", {}),
                            old_hash=ev_data.get("old_hash", ""),
                            new_hash=ev_data.get("new_hash", ""),
                            prev_chain_hash=ev_data.get("prev_chain_hash", ""),
                            chain_hash=ev_data.get("chain_hash", ""),
                        )
                        # イベントリストに追加する
                        restored_events.append(event)
                    # 復元したイベントリストを返す
                    return restored_events
                except (json.JSONDecodeError, KeyError) as exc:
                    # 読み込みエラーをログに記録する
                    logger.error(
                        "チェックポイントイベントファイル読み込みエラー: %s (%s)",
                        events_file,
                        exc,
                    )
        # ファイルが存在しない場合は空リストを返す
        logger.warning(
            "チェックポイントのイベントファイルが見つかりません: id=%s",
            cp.checkpoint_id,
        )
        # 空リストを返す
        return []

    def _save_checkpoint_to_file(self, cp: Checkpoint) -> None:
        """チェックポイントをファイルに保存する。"""
        # チェックポイントディレクトリが設定されていない場合はスキップする
        if self._checkpoint_dir is None:
            # ディレクトリが未設定なのでスキップする
            return
        # チェックポイントファイルのパスを構築する
        cp_file = self._checkpoint_dir / f"{cp.checkpoint_id}.json"
        # チェックポイントを JSON に変換して保存する
        try:
            # JSON 形式でファイルに書き込む
            cp_file.write_text(
                json.dumps(cp.to_dict(), indent=2, ensure_ascii=False),
                encoding="utf-8",
            )
            # 保存成功をログに記録する
            logger.debug("チェックポイントファイル保存: %s", cp_file)
        except OSError as exc:
            # 保存エラーをログに記録する
            logger.error("チェックポイントファイル保存エラー: %s (%s)", cp_file, exc)

    @staticmethod
    def _generate_checkpoint_id() -> str:
        """チェックポイントの一意 ID を生成して返す。"""
        # タイムスタンプベースの一意 ID を生成する
        import time
        # ナノ秒タイムスタンプを SHA-256 でハッシュ化する
        ts = str(time.time_ns()).encode("utf-8")
        # ハッシュの先頭 16 文字を ID として返す
        return "cp_" + hashlib.sha256(ts).hexdigest()[:16]

    @staticmethod
    def _generate_batch_id() -> str:
        """バッチの一意 ID を生成して返す。"""
        # タイムスタンプベースの一意 ID を生成する
        import time
        # ナノ秒タイムスタンプを SHA-256 でハッシュ化する
        ts = str(time.time_ns() + 1).encode("utf-8")
        # ハッシュの先頭 16 文字を ID として返す
        return "batch_" + hashlib.sha256(ts).hexdigest()[:16]

    def filter_events_by_age(
        self, events: List[AuditEvent], max_age_days: Optional[int] = None
    ) -> Tuple[List[AuditEvent], List[AuditEvent]]:
        """イベントリストを経過日数でフィルタリングして (古い, 新しい) のタプルを返す。

        Args:
            events: フィルタリング対象のイベントリスト
            max_age_days: 最大経過日数（省略時はポリシーの値を使用）

        Returns:
            (コンパクション対象の古いイベント, 保持する新しいイベント) のタプル
        """
        # max_age_days が省略された場合はポリシーの値を使用する
        age_limit = max_age_days if max_age_days is not None else self._policy.max_age_days
        # 境界時刻を計算する
        cutoff = datetime.now(timezone.utc) - timedelta(days=age_limit)
        # cutoff の ISO 文字列を生成する
        cutoff_iso = cutoff.isoformat()
        # 古いイベント（コンパクション対象）をフィルタリングする
        old_events = [e for e in events if e.timestamp_hlc <= cutoff_iso]
        # 新しいイベント（保持対象）をフィルタリングする
        new_events = [e for e in events if e.timestamp_hlc > cutoff_iso]
        # フィルタリング結果をログに記録する
        logger.debug(
            "イベントフィルタリング: 古い=%d 新しい=%d cutoff=%s",
            len(old_events),
            len(new_events),
            cutoff_iso,
        )
        # (古いイベント, 新しいイベント) のタプルを返す
        return old_events, new_events
