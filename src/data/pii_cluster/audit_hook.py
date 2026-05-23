"""src/data/pii_cluster/audit_hook.py

PII データアクセス監査フックモジュール。
PostgreSQL の AFTER トリガーを使って PII テーブルへのアクセスを記録し、
ハッシュチェーンで改ざんを検知する。
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
# OS 操作に使用する os をインポートする
import os
# 秘密鍵生成に使用する secrets をインポートする
import secrets
# システム操作に使用する sys をインポートする
import sys
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Tuple

# psycopg2 のインポートを試みる
try:
    # PostgreSQL 同期クライアントをインポートする
    import psycopg2
    # psycopg2 の型変換モジュールをインポートする
    import psycopg2.extras
    # psycopg2 が利用可能であることを示すフラグを設定する
    _HAS_PSYCOPG2 = True
except ImportError:
    # psycopg2 が存在しない場合はフラグを False に設定する
    _HAS_PSYCOPG2 = False
    # エラーメッセージを出力する
    print("WARNING: psycopg2 がインストールされていません。", file=sys.stderr)

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)


# ===========================================================================
# ハッシュチェーン計算ユーティリティ
# ===========================================================================


def _compute_event_hash(
    prev_hash: str,
    event_id: str,
    timestamp_hlc: str,
    actor_id: str,
    operation: str,
    row_keys_json: str,
) -> str:
    """監査イベントのハッシュを計算して返す。

    Args:
        prev_hash: 直前のイベントのハッシュ（最初のイベントは空文字）
        event_id: イベントの一意識別子
        timestamp_hlc: HLC タイムスタンプ文字列
        actor_id: 操作を実行したアクターの識別子
        operation: 操作種別（INSERT/UPDATE/DELETE/SELECT）
        row_keys_json: 操作対象行のキー辞書の JSON 文字列

    Returns:
        SHA-256 ハッシュの16進数文字列
    """
    # ハッシュ計算の入力文字列を構築する
    hash_input = "|".join([
        prev_hash,
        event_id,
        timestamp_hlc,
        actor_id,
        operation,
        row_keys_json,
    ])
    # SHA-256 ハッシュを計算して返す
    return hashlib.sha256(hash_input.encode("utf-8")).hexdigest()


def _compute_row_keys_hash(row_keys: Dict[str, Any]) -> str:
    """行キー辞書の SHA-256 ハッシュを計算して返す。"""
    # 辞書を JSON にシリアライズする（キーをソートして決定論的にする）
    keys_json = json.dumps(row_keys, sort_keys=True, ensure_ascii=False)
    # SHA-256 ハッシュを計算して返す
    return hashlib.sha256(keys_json.encode("utf-8")).hexdigest()


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class AuditEvent:
    """PII データアクセスの監査イベントを表すデータクラス。"""

    # イベントの一意識別子（UUID 形式）
    event_id: str
    # HLC タイムスタンプ（壁時計との合成ロジカルクロック）
    timestamp_hlc: str
    # 操作を実行したアクターの識別子（ユーザー ID またはサービス名）
    actor_id: str
    # テナントの識別子
    tenant_id: str
    # 操作種別（INSERT/UPDATE/DELETE/SELECT）
    operation: str
    # 操作対象のテーブル名
    table_name: str
    # 操作対象行の主キー辞書
    row_keys: Dict[str, Any]
    # 操作前の行内容のハッシュ（INSERT 時は空文字）
    old_hash: str
    # 操作後の行内容のハッシュ（DELETE 時は空文字）
    new_hash: str
    # ハッシュチェーン上の前のイベントのハッシュ
    prev_chain_hash: str = ""
    # このイベント自身のハッシュ（チェーン上のリンク）
    chain_hash: str = ""

    def compute_chain_hash(self) -> str:
        """このイベントのハッシュチェーンハッシュを計算して返す。"""
        # 行キー辞書の JSON 文字列を生成する
        row_keys_json = json.dumps(self.row_keys, sort_keys=True)
        # ハッシュを計算して返す
        return _compute_event_hash(
            prev_hash=self.prev_chain_hash,
            event_id=self.event_id,
            timestamp_hlc=self.timestamp_hlc,
            actor_id=self.actor_id,
            operation=self.operation,
            row_keys_json=row_keys_json,
        )

    def verify_self_hash(self) -> bool:
        """このイベントのハッシュが chain_hash と一致するかどうかを確認して返す。"""
        # 自身のハッシュが空の場合は False を返す
        if not self.chain_hash:
            # chain_hash が未設定の場合は無効とする
            return False
        # 再計算したハッシュと比較する
        expected = self.compute_chain_hash()
        # 一致する場合は True を返す
        return self.chain_hash == expected


@dataclass
class ChainVerifyResult:
    """ハッシュチェーン検証の結果を表すデータクラス。"""

    # チェーンが有効かどうか（破損なし）
    is_valid: bool
    # 検証したイベントの総数
    total_events: int
    # 検証が成功したイベントの数
    verified_count: int
    # ハッシュが不一致のイベントのインデックスリスト
    broken_indices: List[int] = field(default_factory=list)
    # 改ざんが検出されたイベントのリスト
    tampered_events: List[AuditEvent] = field(default_factory=list)
    # 検証実行時刻
    verified_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def summary(self) -> str:
        """検証結果のサマリー文字列を返す。"""
        # 状態ラベルを決定する
        status = "VALID" if self.is_valid else "TAMPERED"
        # サマリー文字列をフォーマットして返す
        return (
            f"ChainVerifyResult[{status}] "
            f"total={self.total_events} "
            f"verified={self.verified_count} "
            f"broken={len(self.broken_indices)}"
        )


# ===========================================================================
# トリガー SQL テンプレート
# ===========================================================================

# 監査トリガー関数の SQL テンプレート
_TRIGGER_FUNCTION_SQL = """
CREATE OR REPLACE FUNCTION pii_audit_trigger_fn()
RETURNS TRIGGER AS $$
DECLARE
    v_actor_id TEXT;
    v_tenant_id TEXT;
    v_old_hash TEXT;
    v_new_hash TEXT;
    v_event_id TEXT;
BEGIN
    v_actor_id := current_setting('app.current_actor', TRUE);
    v_tenant_id := current_setting('app.current_tenant', TRUE);

    IF TG_OP = 'INSERT' THEN
        v_old_hash := '';
        v_new_hash := md5(row_to_json(NEW)::text);
    ELSIF TG_OP = 'UPDATE' THEN
        v_old_hash := md5(row_to_json(OLD)::text);
        v_new_hash := md5(row_to_json(NEW)::text);
    ELSIF TG_OP = 'DELETE' THEN
        v_old_hash := md5(row_to_json(OLD)::text);
        v_new_hash := '';
    END IF;

    v_event_id := gen_random_uuid()::text;

    INSERT INTO pii_audit_log (
        event_id, timestamp_hlc, actor_id, tenant_id,
        operation, table_name, row_keys,
        old_hash, new_hash
    ) VALUES (
        v_event_id,
        clock_timestamp()::text,
        COALESCE(v_actor_id, session_user),
        COALESCE(v_tenant_id, ''),
        TG_OP,
        TG_TABLE_NAME,
        row_to_json(
            CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END
        ),
        v_old_hash,
        v_new_hash
    );

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
"""

# 監査テーブル作成 SQL
_CREATE_AUDIT_TABLE_SQL = """
CREATE TABLE IF NOT EXISTS pii_audit_log (
    event_id TEXT PRIMARY KEY,
    timestamp_hlc TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    table_name TEXT NOT NULL,
    row_keys JSONB NOT NULL DEFAULT '{}',
    old_hash TEXT NOT NULL DEFAULT '',
    new_hash TEXT NOT NULL DEFAULT '',
    prev_chain_hash TEXT NOT NULL DEFAULT '',
    chain_hash TEXT NOT NULL DEFAULT ''
);
"""

# テーブルにトリガーを作成する SQL テンプレート
_CREATE_TRIGGER_SQL = """
CREATE TRIGGER {trigger_name}
AFTER INSERT OR UPDATE OR DELETE ON {table_name}
FOR EACH ROW EXECUTE FUNCTION pii_audit_trigger_fn();
"""


# ===========================================================================
# PgAuditHook クラス
# ===========================================================================


class PgAuditHook:
    """PII データアクセスを記録する PostgreSQL 監査フッククラス。

    AFTER トリガーを使ってテーブルへの操作を監査ログに記録し、
    ハッシュチェーンで改ざんを検知する。
    """

    def __init__(self, conn: Any) -> None:
        """PgAuditHook を接続オブジェクトで初期化する。

        Args:
            conn: psycopg2 の接続オブジェクト
        """
        # 接続オブジェクトを保存する
        self._conn = conn

    def install_trigger(self, table: str) -> None:
        """指定したテーブルに監査トリガーをインストールする。

        Args:
            table: トリガーをインストールするテーブル名

        Raises:
            RuntimeError: psycopg2 が利用できない場合
            Exception: SQL 実行に失敗した場合
        """
        # psycopg2 が利用可能かどうかを確認する
        if not _HAS_PSYCOPG2:
            # psycopg2 が存在しない場合は RuntimeError を送出する
            raise RuntimeError("psycopg2 が必要です。")
        # カーソルを取得する
        with self._conn.cursor() as cur:
            # 監査テーブルを作成する（存在しない場合のみ）
            cur.execute(_CREATE_AUDIT_TABLE_SQL)
            # トリガー関数を作成または置換する
            cur.execute(_TRIGGER_FUNCTION_SQL)
            # トリガー名を生成する
            trigger_name = f"pii_audit_{table}_trigger"
            # 既存トリガーを削除する（冪等化）
            cur.execute(
                f"DROP TRIGGER IF EXISTS {trigger_name} ON {table}"
            )
            # 新しいトリガーを作成する
            cur.execute(
                _CREATE_TRIGGER_SQL.format(
                    trigger_name=trigger_name,
                    table_name=table,
                )
            )
        # トランザクションをコミットする
        self._conn.commit()
        # トリガーインストール成功をログに記録する
        logger.info("監査トリガーインストール完了: table=%s", table)

    def read_audit_log(
        self,
        since: datetime,
        table_name: Optional[str] = None,
        tenant_id: Optional[str] = None,
    ) -> List[AuditEvent]:
        """指定時刻以降の監査ログを読み込んでイベントリストで返す。

        Args:
            since: この時刻以降のイベントを取得する
            table_name: 特定のテーブルのみに絞り込む（省略時は全テーブル）
            tenant_id: 特定のテナントのみに絞り込む（省略時は全テナント）

        Returns:
            AuditEvent のリスト（timestamp_hlc の昇順）
        """
        # 基本クエリを構築する
        query = """
            SELECT
                event_id, timestamp_hlc, actor_id, tenant_id,
                operation, table_name, row_keys,
                old_hash, new_hash, prev_chain_hash, chain_hash
            FROM pii_audit_log
            WHERE timestamp_hlc >= %s
        """
        # クエリパラメータのリストを初期化する
        params: List[Any] = [since.isoformat()]
        # テーブル名フィルタを追加する（指定された場合のみ）
        if table_name:
            # テーブル名の絞り込み条件を追加する
            query += " AND table_name = %s"
            # テーブル名パラメータを追加する
            params.append(table_name)
        # テナント ID フィルタを追加する（指定された場合のみ）
        if tenant_id:
            # テナント ID の絞り込み条件を追加する
            query += " AND tenant_id = %s"
            # テナント ID パラメータを追加する
            params.append(tenant_id)
        # 時刻の昇順でソートする
        query += " ORDER BY timestamp_hlc ASC"
        # カーソルを取得してクエリを実行する
        events: List[AuditEvent] = []
        # DictCursor で結果を取得する
        with self._conn.cursor(cursor_factory=psycopg2.extras.DictCursor) as cur:
            # クエリを実行する
            cur.execute(query, params)
            # 全行を取得する
            rows = cur.fetchall()
            # 各行を AuditEvent に変換する
            for row in rows:
                # row_keys が JSONB の場合は辞書に変換する
                row_keys = row["row_keys"] or {}
                # 文字列の場合は JSON デコードする
                if isinstance(row_keys, str):
                    # JSON 文字列を辞書に変換する
                    try:
                        # JSON デコードを試みる
                        row_keys = json.loads(row_keys)
                    except json.JSONDecodeError:
                        # デコード失敗の場合は空辞書を使用する
                        row_keys = {}
                # AuditEvent を生成してリストに追加する
                event = AuditEvent(
                    event_id=row["event_id"],
                    timestamp_hlc=row["timestamp_hlc"],
                    actor_id=row["actor_id"],
                    tenant_id=row["tenant_id"],
                    operation=row["operation"],
                    table_name=row["table_name"],
                    row_keys=row_keys,
                    old_hash=row["old_hash"] or "",
                    new_hash=row["new_hash"] or "",
                    prev_chain_hash=row["prev_chain_hash"] or "",
                    chain_hash=row["chain_hash"] or "",
                )
                # イベントリストに追加する
                events.append(event)
        # 読み込み完了をログに記録する
        logger.info("監査ログ読み込み完了: %d 件 (since=%s)", len(events), since.isoformat())
        # イベントリストを返す
        return events

    def verify_hash_chain(self, events: List[AuditEvent]) -> ChainVerifyResult:
        """監査イベントのハッシュチェーンを検証して結果を返す。

        Args:
            events: 検証対象の AuditEvent リスト（timestamp_hlc の昇順）

        Returns:
            チェーン検証結果を含む ChainVerifyResult インスタンス
        """
        # イベントが空の場合は空の検証結果を返す
        if not events:
            # 空リストの場合はチェーンが有効（イベントなし）とみなす
            return ChainVerifyResult(
                is_valid=True,
                total_events=0,
                verified_count=0,
            )
        # 破損したインデックスのリストを初期化する
        broken_indices: List[int] = []
        # 改ざんが検出されたイベントのリストを初期化する
        tampered_events: List[AuditEvent] = []
        # 前のイベントのハッシュを追跡する変数を初期化する
        prev_hash = ""
        # 各イベントをイテレートしてチェーンを検証する
        for idx, event in enumerate(events):
            # chain_hash が設定されている場合は検証する
            if event.chain_hash:
                # 期待するハッシュを計算する
                row_keys_json = json.dumps(event.row_keys, sort_keys=True)
                # ハッシュを計算する
                expected_hash = _compute_event_hash(
                    prev_hash=prev_hash,
                    event_id=event.event_id,
                    timestamp_hlc=event.timestamp_hlc,
                    actor_id=event.actor_id,
                    operation=event.operation,
                    row_keys_json=row_keys_json,
                )
                # 計算したハッシュと記録済みハッシュを比較する
                if event.chain_hash != expected_hash:
                    # ハッシュ不一致のインデックスを記録する
                    broken_indices.append(idx)
                    # 改ざんされたイベントをリストに追加する
                    tampered_events.append(event)
                    # ハッシュ不一致をログに記録する
                    logger.warning(
                        "ハッシュ不一致: idx=%d event_id=%s",
                        idx,
                        event.event_id,
                    )
                # 前のハッシュを更新する（不一致でも連鎖を継続する）
                prev_hash = event.chain_hash
            else:
                # chain_hash が未設定の場合は prev_hash をリセットする
                prev_hash = ""
        # 検証済みカウントを計算する
        verified_count = len(events) - len(broken_indices)
        # 検証結果を生成して返す
        result = ChainVerifyResult(
            is_valid=len(broken_indices) == 0,
            total_events=len(events),
            verified_count=verified_count,
            broken_indices=broken_indices,
            tampered_events=tampered_events,
        )
        # 検証結果をログに記録する
        logger.info("ハッシュチェーン検証完了: %s", result.summary())
        # 検証結果を返す
        return result

    def detect_tampering(self, events: List[AuditEvent]) -> List[AuditEvent]:
        """監査イベントリストから改ざんを検出して改ざんイベントのリストを返す。

        Args:
            events: 検証対象の AuditEvent リスト

        Returns:
            改ざんが検出された AuditEvent のリスト
        """
        # ハッシュチェーンを検証する
        result = self.verify_hash_chain(events)
        # 改ざんが検出された場合はログに警告を出力する
        if not result.is_valid:
            # 改ざん検出の警告をログに記録する
            logger.warning(
                "改ざんを検出しました: %d 件のイベントが不正です",
                len(result.tampered_events),
            )
        # 改ざんされたイベントのリストを返す
        return result.tampered_events

    def update_hash_chain(self, events: List[AuditEvent]) -> int:
        """既存の監査イベントにハッシュチェーンを計算して書き込む。

        ハッシュチェーンが未計算のイベントに対して計算・更新を行う。

        Args:
            events: ハッシュチェーンを設定する AuditEvent リスト

        Returns:
            更新したイベントの件数
        """
        # 更新カウントを初期化する
        updated_count = 0
        # 前のハッシュを初期化する
        prev_hash = ""
        # 各イベントをイテレートしてハッシュを計算する
        for event in events:
            # prev_chain_hash を設定する
            event.prev_chain_hash = prev_hash
            # chain_hash を計算して設定する
            event.chain_hash = event.compute_chain_hash()
            # データベースを更新する
            with self._conn.cursor() as cur:
                # 監査ログのハッシュを更新する
                cur.execute(
                    """
                    UPDATE pii_audit_log
                    SET prev_chain_hash = %s, chain_hash = %s
                    WHERE event_id = %s
                    """,
                    (event.prev_chain_hash, event.chain_hash, event.event_id),
                )
            # 前のハッシュを更新する
            prev_hash = event.chain_hash
            # 更新カウントをインクリメントする
            updated_count += 1
        # コミットする
        self._conn.commit()
        # 更新カウントをログに記録する
        logger.info("ハッシュチェーン更新完了: %d 件", updated_count)
        # 更新件数を返す
        return updated_count
