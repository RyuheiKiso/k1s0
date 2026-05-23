"""src/data/postgresql/tenant_context.py

テナントコンテキスト管理モジュール。
PostgreSQL セッションの app.current_tenant / app.current_actor GUC を
安全に設定・検証し、RLS ポリシーが正しく機能する前提条件を保証する。
仕様: docs/04_詳細設計/01_適合仕様/14_データ保全適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# ログ出力に使用する logging をインポートする
import logging
# 正規表現に使用する re をインポートする
import re
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)

# テナント ID の有効なパターン（英数字・ハイフン・アンダースコア）
_VALID_TENANT_ID_PATTERN = re.compile(r"^[a-zA-Z0-9_\-]{1,64}$")
# アクター ID の有効なパターン
_VALID_ACTOR_ID_PATTERN = re.compile(r"^[a-zA-Z0-9_\-@.]{1,128}$")


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class TenantContext:
    """セッションのテナントコンテキストを表すデータクラス。"""

    # テナントの識別子
    tenant_id: str
    # 操作を実行するアクターの識別子
    actor_id: str
    # コンテキストが設定された時刻
    set_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    # コンテキストの有効期限（ISO 8601 形式、空文字は無期限）
    expires_at: str = ""

    def validate(self) -> bool:
        """テナントコンテキストが有効かどうかを検証して返す。"""
        # tenant_id が有効なパターンに一致するかを確認する
        if not _VALID_TENANT_ID_PATTERN.match(self.tenant_id):
            # 無効なテナント ID の場合は False を返す
            logger.warning("無効なテナント ID: %r", self.tenant_id)
            # False を返す
            return False
        # actor_id が有効なパターンに一致するかを確認する
        if not _VALID_ACTOR_ID_PATTERN.match(self.actor_id):
            # 無効なアクター ID の場合は False を返す
            logger.warning("無効なアクター ID: %r", self.actor_id)
            # False を返す
            return False
        # 全検証が通過した場合は True を返す
        return True


@dataclass
class ContextSetResult:
    """テナントコンテキスト設定の結果を表すデータクラス。"""

    # 設定が成功したかどうか
    success: bool
    # 設定したテナントコンテキスト
    context: Optional[TenantContext] = None
    # 設定失敗の理由（成功時は空文字）
    error_message: str = ""


# ===========================================================================
# TenantContextManager クラス
# ===========================================================================


class TenantContextManager:
    """PostgreSQL セッションのテナントコンテキストを管理するクラス。

    app.current_tenant / app.current_actor GUC を設定し、
    RLS ポリシーが正しく機能する前提条件を保証する。
    """

    # テナント GUC キー
    _TENANT_GUC_KEY = "app.current_tenant"
    # アクター GUC キー
    _ACTOR_GUC_KEY = "app.current_actor"
    # GUC 設定クエリテンプレート
    _SET_GUC_QUERY = "SELECT set_config(%s, %s, TRUE)"
    # GUC 取得クエリテンプレート
    _GET_GUC_QUERY = "SELECT current_setting(%s, TRUE)"

    def __init__(self, conn: Any) -> None:
        """TenantContextManager を接続オブジェクトで初期化する。"""
        # 接続オブジェクトを保存する
        self._conn = conn
        # 現在のコンテキストを初期化する
        self._current_context: Optional[TenantContext] = None

    def set_context(self, context: TenantContext) -> ContextSetResult:
        """セッションにテナントコンテキストを設定して結果を返す。

        Args:
            context: 設定するテナントコンテキスト

        Returns:
            コンテキスト設定の結果
        """
        # コンテキストの有効性を確認する
        if not context.validate():
            # 無効なコンテキストの場合は失敗結果を返す
            return ContextSetResult(
                success=False,
                error_message=f"無効なコンテキスト: tenant={context.tenant_id} actor={context.actor_id}",
            )
        # PostgreSQL セッションに GUC を設定する
        try:
            # カーソルを取得する
            with self._conn.cursor() as cur:
                # テナント ID の GUC を設定する
                cur.execute(self._SET_GUC_QUERY, (self._TENANT_GUC_KEY, context.tenant_id))
                # アクター ID の GUC を設定する
                cur.execute(self._SET_GUC_QUERY, (self._ACTOR_GUC_KEY, context.actor_id))
            # 現在のコンテキストを更新する
            self._current_context = context
            # 設定成功をログに記録する
            logger.debug(
                "テナントコンテキスト設定: tenant=%s actor=%s",
                context.tenant_id,
                context.actor_id,
            )
            # 成功結果を返す
            return ContextSetResult(success=True, context=context)
        except Exception as exc:
            # 設定エラーをログに記録する
            logger.error("テナントコンテキスト設定エラー: %s", exc)
            # 失敗結果を返す
            return ContextSetResult(
                success=False,
                error_message=str(exc),
            )

    def get_current_context(self) -> Optional[TenantContext]:
        """セッションの現在のテナントコンテキストを取得して返す。

        Returns:
            現在の TenantContext インスタンス、未設定の場合は None
        """
        # データベースから現在の GUC 値を取得する
        try:
            # カーソルを取得する
            with self._conn.cursor() as cur:
                # テナント ID の GUC を取得する
                cur.execute(self._GET_GUC_QUERY, (self._TENANT_GUC_KEY,))
                # 結果を取得する
                tenant_row = cur.fetchone()
                # アクター ID の GUC を取得する
                cur.execute(self._GET_GUC_QUERY, (self._ACTOR_GUC_KEY,))
                # 結果を取得する
                actor_row = cur.fetchone()
            # テナント ID を取得する（None の場合は空文字）
            tenant_id = tenant_row[0] if tenant_row and tenant_row[0] else ""
            # アクター ID を取得する（None の場合は空文字）
            actor_id = actor_row[0] if actor_row and actor_row[0] else ""
            # テナント ID が空の場合は None を返す
            if not tenant_id:
                # テナントコンテキストが未設定の場合は None を返す
                return None
            # TenantContext を生成して返す
            return TenantContext(tenant_id=tenant_id, actor_id=actor_id)
        except Exception as exc:
            # 取得エラーをログに記録する
            logger.error("テナントコンテキスト取得エラー: %s", exc)
            # None を返す
            return None

    def clear_context(self) -> None:
        """セッションのテナントコンテキストをクリアする。"""
        # GUC を空文字に設定してクリアする
        try:
            # カーソルを取得する
            with self._conn.cursor() as cur:
                # テナント ID の GUC を空文字に設定する
                cur.execute(self._SET_GUC_QUERY, (self._TENANT_GUC_KEY, ""))
                # アクター ID の GUC を空文字に設定する
                cur.execute(self._SET_GUC_QUERY, (self._ACTOR_GUC_KEY, ""))
            # 現在のコンテキストを None に設定する
            self._current_context = None
            # クリア成功をログに記録する
            logger.debug("テナントコンテキストをクリアしました")
        except Exception as exc:
            # クリアエラーをログに記録する
            logger.error("テナントコンテキストクリアエラー: %s", exc)

    def verify_context_active(self) -> bool:
        """セッションにテナントコンテキストが設定されているかを確認して返す。"""
        # 現在のコンテキストを取得する
        ctx = self.get_current_context()
        # コンテキストが存在し、テナント ID が空でない場合は True を返す
        return ctx is not None and bool(ctx.tenant_id)

    def __enter__(self) -> "TenantContextManager":
        """コンテキストマネージャーのエントリポイントを返す。"""
        # self を返す
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        """コンテキストマネージャー終了時にコンテキストをクリアする。"""
        # コンテキストをクリアする
        self.clear_context()
