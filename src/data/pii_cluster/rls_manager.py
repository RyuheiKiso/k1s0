"""src/data/pii_cluster/rls_manager.py

PostgreSQL Row Level Security (RLS) 管理モジュール。
製造業プラットフォームの PII 専用クラスタで、テナント分離を
RLS FORCE ポリシーを用いて強制する。
仕様: docs/04_詳細設計/01_適合仕様/14_データ保全適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# ログ出力に使用する logging をインポートする
import logging
# OS 環境変数取得に使用する os をインポートする
import os
# システム操作に使用する sys をインポートする
import sys
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# 列挙型に使用する enum をインポートする
import enum
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Tuple

# psycopg2 のインポートを試みる（非同期版 asyncpg はオプション）
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

# asyncpg の非同期クライアントのインポートを試みる
try:
    # 非同期 PostgreSQL クライアントをインポートする
    import asyncpg
    # asyncpg が利用可能であることを示すフラグを設定する
    _HAS_ASYNCPG = True
except ImportError:
    # asyncpg が存在しない場合はフラグを False に設定する
    _HAS_ASYNCPG = False

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)


# ===========================================================================
# データクラス・列挙型定義
# ===========================================================================


@dataclass
class RLSPolicy:
    """PostgreSQL RLS ポリシーを表すデータクラス。"""

    # ポリシーが適用されるテーブル名
    table_name: str
    # ポリシーの名称（例: "tenant_isolation_policy"）
    policy_name: str
    # PERMISSIVE（許可型）か RESTRICTIVE（制限型）かを示すフラグ
    permissive: bool
    # ポリシーが適用されるロールのリスト（空リストは全ロールに適用）
    roles: List[str]
    # SELECT/UPDATE/DELETE 時の行フィルタ式
    using_expr: str
    # INSERT/UPDATE 時の行チェック式
    check_expr: str = ""
    # ポリシーが有効化されているかどうか
    is_enabled: bool = True
    # ポリシーの説明
    description: str = ""

    def to_create_sql(self) -> str:
        """CREATE POLICY SQL 文を生成して返す。"""
        # ポリシータイプ文字列を決定する
        policy_type = "PERMISSIVE" if self.permissive else "RESTRICTIVE"
        # ロール指定部分を構築する（空リストの場合は PUBLIC）
        roles_clause = (
            f"TO {', '.join(self.roles)}" if self.roles else "TO PUBLIC"
        )
        # USING 句を構築する
        using_clause = f"USING ({self.using_expr})"
        # WITH CHECK 句を構築する（check_expr が存在する場合のみ）
        check_clause = (
            f"WITH CHECK ({self.check_expr})" if self.check_expr else ""
        )
        # CREATE POLICY SQL 文を構築して返す
        return (
            f"CREATE POLICY {self.policy_name} ON {self.table_name} "
            f"AS {policy_type} {roles_clause} "
            f"{using_clause} {check_clause}".strip()
        )

    def to_drop_sql(self) -> str:
        """DROP POLICY SQL 文を生成して返す。"""
        # DROP POLICY SQL 文を構築して返す
        return f"DROP POLICY IF EXISTS {self.policy_name} ON {self.table_name}"


@dataclass
class IsolationResult:
    """テナント分離テストの結果を表すデータクラス。"""

    # テスト対象のテーブル名
    table_name: str
    # テナント A の識別子
    tenant_a: str
    # テナント B の識別子
    tenant_b: str
    # テナント A のセッションでのテナント A 行のアクセス成功かどうか
    tenant_a_sees_own_rows: bool
    # テナント A のセッションでのテナント B 行が見えないかどうか（True = 見えない = 安全）
    tenant_a_cannot_see_tenant_b: bool
    # テナント B のセッションでのテナント B 行のアクセス成功かどうか
    tenant_b_sees_own_rows: bool
    # テナント B のセッションでのテナント A 行が見えないかどうか（True = 見えない = 安全）
    tenant_b_cannot_see_tenant_a: bool
    # エラーメッセージ（成功時は空文字）
    error_message: str = ""
    # テスト実行時刻
    tested_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def is_isolated(self) -> bool:
        """テナント分離が正しく機能しているかどうかを返す。"""
        # 全ての分離条件が満たされている場合のみ True を返す
        return (
            self.tenant_a_sees_own_rows
            and self.tenant_a_cannot_see_tenant_b
            and self.tenant_b_sees_own_rows
            and self.tenant_b_cannot_see_tenant_a
        )

    def summary(self) -> str:
        """分離テスト結果のサマリー文字列を返す。"""
        # 分離状態の文字列を決定する
        status = "ISOLATED" if self.is_isolated else "BREACH_DETECTED"
        # サマリー文字列をフォーマットして返す
        return (
            f"IsolationResult[{status}] table={self.table_name} "
            f"a_own={self.tenant_a_sees_own_rows} "
            f"a_no_b={self.tenant_a_cannot_see_tenant_b} "
            f"b_own={self.tenant_b_sees_own_rows} "
            f"b_no_a={self.tenant_b_cannot_see_tenant_a}"
        )


# ===========================================================================
# SQL テンプレート定義
# ===========================================================================

# RLS を有効化する SQL テンプレート（FORCE オプション付き）
_SQL_ENABLE_RLS = "ALTER TABLE {table} ENABLE ROW LEVEL SECURITY"
# RLS を FORCE 適用する SQL テンプレート
_SQL_FORCE_RLS = "ALTER TABLE {table} FORCE ROW LEVEL SECURITY"
# RLS の現在状態を確認する SQL
_SQL_CHECK_RLS = """
    SELECT relrowsecurity, relforcerowsecurity
    FROM pg_class
    WHERE relname = %s AND relkind = 'r'
"""
# テーブルに設定されている全 RLS ポリシーを取得する SQL
_SQL_LIST_POLICIES = """
    SELECT
        polname AS policy_name,
        polpermissive,
        polroles,
        pg_get_expr(polqual, polrelid) AS using_expr,
        pg_get_expr(polwithcheck, polrelid) AS check_expr,
        polcmd
    FROM pg_policy
    WHERE polrelid = %s::regclass
"""
# テナント設定 GUC を使ったセッション切り替え SQL
_SQL_SET_TENANT = "SET app.current_tenant = %s"
# RLS ポリシー用の current_setting 式テンプレート
_RLS_TENANT_EXPR = "tenant_id = current_setting('app.current_tenant', TRUE)"


# ===========================================================================
# RLSManager クラス
# ===========================================================================


class RLSManager:
    """PostgreSQL RLS ポリシーを管理するクラス。

    テナント分離のための RLS FORCE ポリシーを作成・有効化・検証し、
    テナント間分離が正しく機能しているかをテストする。
    """

    def __init__(self) -> None:
        """RLSManager を空の接続状態で初期化する。"""
        # データベース接続オブジェクトを初期化する
        self._conn: Any = None
        # 接続先の DSN を保存する
        self._dsn: str = ""

    def connect(self, dsn: str) -> None:
        """PostgreSQL に接続する。

        Args:
            dsn: PostgreSQL 接続 DSN 文字列
                 例: "postgresql://user:pass@localhost:5432/pii_db"

        Raises:
            RuntimeError: psycopg2 が利用できない場合
            Exception: 接続に失敗した場合
        """
        # psycopg2 が利用可能かどうかを確認する
        if not _HAS_PSYCOPG2:
            # psycopg2 が存在しない場合は RuntimeError を送出する
            raise RuntimeError(
                "psycopg2 が必要です。pip install psycopg2-binary を実行してください。"
            )
        # 既存の接続がある場合はクローズする
        if self._conn is not None:
            # 既存接続をクローズしてから新規接続する
            try:
                # 接続をクローズする
                self._conn.close()
            except Exception:
                # クローズ中のエラーは無視する
                pass
        # PostgreSQL に接続する
        self._conn = psycopg2.connect(dsn)
        # 接続を autocommit モードに設定する
        self._conn.autocommit = False
        # DSN を保存する（パスワード部分は非表示）
        self._dsn = dsn
        # 接続成功をログに記録する
        logger.info("PostgreSQL 接続完了: %s", self._mask_dsn(dsn))

    @staticmethod
    def _mask_dsn(dsn: str) -> str:
        """DSN のパスワード部分をマスクして返す。"""
        # パスワードパターンを検出してマスクする
        import re
        # PostgreSQL DSN のパスワード部分を正規表現でマスクする
        masked = re.sub(r"(://[^:]+:)[^@]+(@)", r"\1****\2", dsn)
        # マスクした DSN を返す
        return masked

    def _require_connection(self) -> None:
        """接続が確立されているかどうかを確認する。未接続の場合は RuntimeError を送出する。"""
        # 接続オブジェクトが None の場合は RuntimeError を送出する
        if self._conn is None:
            # 未接続の場合はエラーを送出する
            raise RuntimeError(
                "データベースに接続されていません。先に connect() を呼び出してください。"
            )

    def enable_rls(self, table: str, force: bool = True) -> None:
        """指定したテーブルに RLS を有効化する。

        Args:
            table: RLS を有効化するテーブル名
            force: True の場合は FORCE RLS も設定する（スーパーユーザー含む全ロールに適用）

        Raises:
            RuntimeError: 接続が確立されていない場合
            Exception: SQL 実行に失敗した場合
        """
        # 接続が確立されているかを確認する
        self._require_connection()
        # カーソルを取得する
        with self._conn.cursor() as cur:
            # RLS を有効化する SQL を実行する
            cur.execute(_SQL_ENABLE_RLS.format(table=table))
            # force フラグが True の場合は FORCE RLS も設定する
            if force:
                # FORCE RLS を設定する SQL を実行する
                cur.execute(_SQL_FORCE_RLS.format(table=table))
        # トランザクションをコミットする
        self._conn.commit()
        # RLS 有効化をログに記録する
        logger.info("RLS 有効化完了: table=%s force=%s", table, force)

    def create_policy(self, policy: RLSPolicy) -> None:
        """指定した RLS ポリシーをテーブルに作成する。

        Args:
            policy: 作成する RLSPolicy データクラス

        Raises:
            RuntimeError: 接続が確立されていない場合
            Exception: SQL 実行に失敗した場合
        """
        # 接続が確立されているかを確認する
        self._require_connection()
        # DROP→CREATE の順で冪等に適用する
        with self._conn.cursor() as cur:
            # 既存のポリシーを削除する（存在しない場合は何もしない）
            cur.execute(policy.to_drop_sql())
            # 新しいポリシーを作成する
            cur.execute(policy.to_create_sql())
        # トランザクションをコミットする
        self._conn.commit()
        # ポリシー作成をログに記録する
        logger.info(
            "RLS ポリシー作成完了: table=%s policy=%s",
            policy.table_name,
            policy.policy_name,
        )

    def create_tenant_isolation_policy(self, table: str) -> RLSPolicy:
        """標準的なテナント分離 RLS ポリシーを作成して返す。

        app.current_tenant GUC と tenant_id カラムを使ったテナント分離ポリシーを
        生成して適用する。

        Args:
            table: ポリシーを適用するテーブル名

        Returns:
            作成した RLSPolicy インスタンス
        """
        # テナント分離ポリシーのデータクラスを生成する
        policy = RLSPolicy(
            table_name=table,
            policy_name=f"{table}_tenant_isolation",
            permissive=True,
            roles=[],
            using_expr=_RLS_TENANT_EXPR,
            check_expr=_RLS_TENANT_EXPR,
            description=f"{table} テーブルのテナント分離 RLS ポリシー",
        )
        # ポリシーを作成する
        self.create_policy(policy)
        # 作成したポリシーを返す
        return policy

    def verify_policy_active(self, table: str) -> bool:
        """指定したテーブルで RLS が有効かつ FORCE が設定されているかを返す。

        Args:
            table: 確認するテーブル名

        Returns:
            RLS が有効かつ FORCE されていれば True
        """
        # 接続が確立されているかを確認する
        self._require_connection()
        # カーソルを取得する
        with self._conn.cursor(cursor_factory=psycopg2.extras.DictCursor) as cur:
            # pg_class から RLS の状態を確認する
            cur.execute(_SQL_CHECK_RLS, (table,))
            # クエリ結果を取得する
            row = cur.fetchone()
            # テーブルが存在しない場合は False を返す
            if row is None:
                # テーブルが見つからない旨をログに記録する
                logger.warning("テーブルが見つかりません: %s", table)
                # False を返す
                return False
            # RLS が有効かどうかを確認する
            rls_enabled = bool(row["relrowsecurity"])
            # FORCE RLS が設定されているかを確認する
            rls_forced = bool(row["relforcerowsecurity"])
            # RLS の状態をログに記録する
            logger.debug(
                "RLS 状態確認: table=%s enabled=%s forced=%s",
                table,
                rls_enabled,
                rls_forced,
            )
            # RLS が有効かつ FORCE されている場合のみ True を返す
            return rls_enabled and rls_forced

    def test_isolation(
        self, table: str, tenant_a: str, tenant_b: str
    ) -> IsolationResult:
        """テナント分離が正しく機能しているかどうかをテストして返す。

        Args:
            table: テスト対象のテーブル名
            tenant_a: テナント A の識別子
            tenant_b: テナント B の識別子

        Returns:
            分離テスト結果を含む IsolationResult インスタンス
        """
        # 接続が確立されているかを確認する
        self._require_connection()
        # テスト結果の変数を初期化する
        a_sees_own = False
        # テナント A がテナント B の行を見えないかのフラグを初期化する
        a_no_b = True
        # テナント B が自分の行を見えるかのフラグを初期化する
        b_sees_own = False
        # テナント B がテナント A の行を見えないかのフラグを初期化する
        b_no_a = True
        # エラーメッセージを初期化する
        error_msg = ""
        # テスト用のデータを挿入して確認する
        try:
            # テスト用テナント A のデータを挿入する
            with self._conn.cursor() as cur:
                # テスト行を挿入する（テナント A 用）
                cur.execute(
                    f"INSERT INTO {table} (tenant_id, test_marker) "
                    "VALUES (%s, %s) ON CONFLICT DO NOTHING",
                    (tenant_a, "rls_test_a"),
                )
                # テスト行を挿入する（テナント B 用）
                cur.execute(
                    f"INSERT INTO {table} (tenant_id, test_marker) "
                    "VALUES (%s, %s) ON CONFLICT DO NOTHING",
                    (tenant_b, "rls_test_b"),
                )
            # コミットしてデータを確定する
            self._conn.commit()
            # テナント A のセッションを設定する
            with self._conn.cursor() as cur:
                # テナント A のセッションを設定する
                cur.execute(_SQL_SET_TENANT, (tenant_a,))
                # テナント A 自身の行が見えるかを確認する
                cur.execute(
                    f"SELECT COUNT(*) FROM {table} WHERE tenant_id = %s",
                    (tenant_a,),
                )
                # カウントを取得する
                count_a_own = cur.fetchone()[0]
                # テナント A が自分の行を見えるかを確認する
                a_sees_own = count_a_own > 0
                # テナント A からテナント B の行が見えないかを確認する
                cur.execute(
                    f"SELECT COUNT(*) FROM {table} WHERE tenant_id = %s",
                    (tenant_b,),
                )
                # テナント B の行数を取得する（RLS が効いていれば 0 になる）
                count_b_from_a = cur.fetchone()[0]
                # テナント A からテナント B が見えない場合は True を設定する
                a_no_b = count_b_from_a == 0
            # テナント B のセッションに切り替える
            with self._conn.cursor() as cur:
                # テナント B のセッションを設定する
                cur.execute(_SQL_SET_TENANT, (tenant_b,))
                # テナント B 自身の行が見えるかを確認する
                cur.execute(
                    f"SELECT COUNT(*) FROM {table} WHERE tenant_id = %s",
                    (tenant_b,),
                )
                # カウントを取得する
                count_b_own = cur.fetchone()[0]
                # テナント B が自分の行を見えるかを確認する
                b_sees_own = count_b_own > 0
                # テナント B からテナント A の行が見えないかを確認する
                cur.execute(
                    f"SELECT COUNT(*) FROM {table} WHERE tenant_id = %s",
                    (tenant_a,),
                )
                # テナント A の行数を取得する（RLS が効いていれば 0 になる）
                count_a_from_b = cur.fetchone()[0]
                # テナント B からテナント A が見えない場合は True を設定する
                b_no_a = count_a_from_b == 0
        except Exception as exc:
            # テスト実行中のエラーをログに記録する
            logger.error("分離テスト中にエラーが発生しました: %s", exc)
            # エラーメッセージを保存する
            error_msg = str(exc)
            # ロールバックする
            self._conn.rollback()
        # IsolationResult を生成して返す
        result = IsolationResult(
            table_name=table,
            tenant_a=tenant_a,
            tenant_b=tenant_b,
            tenant_a_sees_own_rows=a_sees_own,
            tenant_a_cannot_see_tenant_b=a_no_b,
            tenant_b_sees_own_rows=b_sees_own,
            tenant_b_cannot_see_tenant_a=b_no_a,
            error_message=error_msg,
        )
        # テスト結果をログに記録する
        logger.info("テナント分離テスト完了: %s", result.summary())
        # テスト結果を返す
        return result

    def audit_policies(self) -> List[RLSPolicy]:
        """データベース内の全 RLS ポリシーを取得してリストで返す。

        Returns:
            データベースに存在する全 RLSPolicy のリスト
        """
        # 接続が確立されているかを確認する
        self._require_connection()
        # ポリシーリストを初期化する
        policies: List[RLSPolicy] = []
        # pg_policy カタログから全ポリシーを取得する
        query = """
            SELECT
                c.relname AS table_name,
                p.polname AS policy_name,
                p.polpermissive,
                p.polroles,
                pg_get_expr(p.polqual, p.polrelid) AS using_expr,
                pg_get_expr(p.polwithcheck, p.polrelid) AS check_expr
            FROM pg_policy p
            JOIN pg_class c ON c.oid = p.polrelid
            WHERE c.relkind = 'r'
            ORDER BY c.relname, p.polname
        """
        # カーソルを取得してクエリを実行する
        with self._conn.cursor(cursor_factory=psycopg2.extras.DictCursor) as cur:
            # ポリシー一覧取得クエリを実行する
            cur.execute(query)
            # 全行を取得する
            rows = cur.fetchall()
            # 各行を RLSPolicy に変換する
            for row in rows:
                # polroles からロールリストを取得する（OID の配列）
                roles_oids = row["polroles"] or []
                # RLSPolicy を生成してリストに追加する
                policy = RLSPolicy(
                    table_name=row["table_name"],
                    policy_name=row["policy_name"],
                    permissive=bool(row["polpermissive"]),
                    roles=[str(oid) for oid in roles_oids],
                    using_expr=row["using_expr"] or "",
                    check_expr=row["check_expr"] or "",
                )
                # ポリシーリストに追加する
                policies.append(policy)
        # ポリシー一覧取得をログに記録する
        logger.info("RLS ポリシー監査完了: %d 件", len(policies))
        # ポリシーリストを返す
        return policies

    def close(self) -> None:
        """データベース接続をクローズする。"""
        # 接続が存在する場合はクローズする
        if self._conn is not None:
            # 接続をクローズする
            try:
                # データベース接続をクローズする
                self._conn.close()
                # 接続オブジェクトを None に設定する
                self._conn = None
                # クローズをログに記録する
                logger.info("PostgreSQL 接続をクローズしました")
            except Exception as exc:
                # クローズ中のエラーをログに記録する
                logger.warning("接続クローズ中にエラーが発生しました: %s", exc)

    def __enter__(self) -> "RLSManager":
        """コンテキストマネージャーのエントリポイントを返す。"""
        # self を返す
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        """コンテキストマネージャーの終了時に接続をクローズする。"""
        # コンテキスト終了時に接続をクローズする
        self.close()
