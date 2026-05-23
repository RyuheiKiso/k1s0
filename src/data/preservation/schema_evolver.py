"""src/data/preservation/schema_evolver.py

スキーマ進化管理モジュール。
製造業プラットフォームの PostgreSQL スキーママイグレーションを
expand-contract パターンで管理し、破壊的変更を検出する。
仕様: docs/04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# 列挙型に使用する enum をインポートする
import enum
# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# 正規表現に使用する re をインポートする
import re
# システム操作に使用する sys をインポートする
import sys
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Set, Tuple

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

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)


# ===========================================================================
# 列挙型定義
# ===========================================================================


class CompatibilityRule(enum.Enum):
    """スキーマ互換性ルールを表す列挙型。"""

    # 後方互換性のみ（新しいリーダーが古いデータを読める）
    BACKWARD = "BACKWARD"
    # 前方互換性のみ（古いリーダーが新しいデータを読める）
    FORWARD = "FORWARD"
    # 完全互換性（双方向で互換性あり）
    FULL = "FULL"
    # 互換性なし（破壊的変更を含む）
    NONE = "NONE"


class BreakingChangeType(enum.Enum):
    """破壊的変更の種別を表す列挙型。"""

    # カラムの削除
    COLUMN_REMOVED = "column_removed"
    # カラムの型変更（narrowing）
    TYPE_NARROWED = "type_narrowed"
    # 既存カラムへの NOT NULL 制約追加
    CONSTRAINT_ADDED = "constraint_added"
    # テーブルの削除
    TABLE_REMOVED = "table_removed"
    # インデックスの削除
    INDEX_REMOVED = "index_removed"
    # 外部キー制約の追加
    FOREIGN_KEY_ADDED = "foreign_key_added"
    # プライマリキーの変更
    PRIMARY_KEY_CHANGED = "primary_key_changed"


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class SchemaVersion:
    """スキーマのバージョンを表すデータクラス。"""

    # バージョン識別子（例: "V0001" または "2024_01_01_add_tenant_id"）
    version_id: str
    # マイグレーション適用時刻（ISO 8601 形式）
    applied_at: str
    # マイグレーション SQL ファイルの SHA-256 ハッシュ
    migration_hash: str
    # 適用（up）SQL テキスト
    up_sql: str
    # ロールバック（down）SQL テキスト
    down_sql: str
    # マイグレーションの説明
    description: str = ""
    # マイグレーションが適用済みかどうか
    is_applied: bool = False

    def compute_hash(self) -> str:
        """up_sql と down_sql から SHA-256 ハッシュを計算して返す。"""
        # up_sql と down_sql を結合してハッシュを計算する
        combined = self.up_sql + self.down_sql
        # SHA-256 ハッシュを返す
        return hashlib.sha256(combined.encode("utf-8")).hexdigest()

    def verify_hash(self) -> bool:
        """保存済みハッシュが実際の SQL と一致するかを確認して返す。"""
        # migration_hash が空の場合は False を返す
        if not self.migration_hash:
            # 空のハッシュは検証不可
            return False
        # 計算したハッシュと比較する
        return self.compute_hash() == self.migration_hash


@dataclass
class BreakingChange:
    """破壊的変更の検出結果を表すデータクラス。"""

    # 変更の種別
    change_type: BreakingChangeType
    # 変更が発生したテーブル名
    table_name: str
    # 変更が発生したカラム名（テーブルレベルの変更の場合は空文字）
    column_name: str
    # 変更前の状態の説明
    before_description: str
    # 変更後の状態の説明
    after_description: str
    # この変更が互換性に与える影響の説明
    impact: str = ""

    def summary(self) -> str:
        """破壊的変更のサマリー文字列を返す。"""
        # 変更種別・テーブル・カラムをフォーマットして返す
        return (
            f"BreakingChange[{self.change_type.value}] "
            f"table={self.table_name} column={self.column_name}: "
            f"{self.before_description} → {self.after_description}"
        )


@dataclass
class CompatibilityReport:
    """スキーマ互換性チェックの結果レポートを表すデータクラス。"""

    # 適用される互換性ルール
    rule: CompatibilityRule
    # 検出された破壊的変更のリスト
    breaking_changes: List[BreakingChange] = field(default_factory=list)
    # 警告メッセージのリスト（破壊的ではないが注意が必要な変更）
    warnings: List[str] = field(default_factory=list)
    # レポート生成時刻
    generated_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def is_compatible(self) -> bool:
        """互換性ルールが NONE でなければ互換性ありとみなして返す。"""
        # NONE 以外は互換性ありとみなす
        return self.rule != CompatibilityRule.NONE

    def summary(self) -> str:
        """互換性レポートのサマリー文字列を返す。"""
        # 互換性状態のラベルを決定する
        compat_status = "COMPATIBLE" if self.is_compatible else "INCOMPATIBLE"
        # サマリー文字列をフォーマットして返す
        return (
            f"CompatibilityReport[{compat_status}] "
            f"rule={self.rule.value} "
            f"breaking_changes={len(self.breaking_changes)} "
            f"warnings={len(self.warnings)}"
        )


# ===========================================================================
# SQL パーサーユーティリティ
# ===========================================================================


def _extract_columns_from_create_table(sql: str) -> Dict[str, Dict[str, Any]]:
    """CREATE TABLE SQL からカラム定義を抽出して辞書で返す。

    Args:
        sql: CREATE TABLE SQL テキスト

    Returns:
        カラム名 → {"type": str, "not_null": bool, "has_default": bool} の辞書
    """
    # 結果辞書を初期化する
    columns: Dict[str, Dict[str, Any]] = {}
    # CREATE TABLE ... (...) のカラム定義部分を抽出する正規表現
    table_match = re.search(
        r"CREATE\s+TABLE\s+\S+\s*\((.*)\)",
        sql,
        re.DOTALL | re.IGNORECASE,
    )
    # テーブル定義が見つからない場合は空辞書を返す
    if not table_match:
        # テーブル定義が見つからない場合は空辞書を返す
        return columns
    # カラム定義部分を取得する
    columns_text = table_match.group(1)
    # カンマで分割する（括弧内のカンマは無視）
    column_defs = _split_column_defs(columns_text)
    # 各カラム定義を解析する
    for col_def in column_defs:
        # 前後の空白を除去する
        col_def = col_def.strip()
        # 空文字またはテーブル制約はスキップする
        if not col_def:
            # 空の定義はスキップする
            continue
        # PRIMARY KEY / UNIQUE / CHECK / FOREIGN KEY 制約行はスキップする
        if re.match(
            r"^\s*(PRIMARY\s+KEY|UNIQUE|CHECK|FOREIGN\s+KEY|CONSTRAINT)",
            col_def,
            re.IGNORECASE,
        ):
            # テーブル制約はスキップする
            continue
        # カラム名を抽出する（先頭のトークン）
        col_name_match = re.match(r'^\s*"?(\w+)"?\s+', col_def)
        # カラム名が見つからない場合はスキップする
        if not col_name_match:
            # カラム名が抽出できない場合はスキップする
            continue
        # カラム名を取得する
        col_name = col_name_match.group(1)
        # データ型を抽出する（カラム名の後のトークン）
        type_match = re.match(
            r'^\s*"?\w+"?\s+(\w+(?:\s*\([^)]*\))?)',
            col_def,
            re.IGNORECASE,
        )
        # データ型を取得する（見つからない場合は "unknown"）
        col_type = type_match.group(1).strip() if type_match else "unknown"
        # NOT NULL 制約が存在するかを確認する
        not_null = bool(re.search(r"\bNOT\s+NULL\b", col_def, re.IGNORECASE))
        # DEFAULT 値が存在するかを確認する
        has_default = bool(re.search(r"\bDEFAULT\b", col_def, re.IGNORECASE))
        # カラム情報を辞書に追加する
        columns[col_name] = {
            "type": col_type.upper(),
            "not_null": not_null,
            "has_default": has_default,
        }
    # 抽出したカラム辞書を返す
    return columns


def _split_column_defs(columns_text: str) -> List[str]:
    """カラム定義テキストをカンマで分割して個別定義のリストを返す。

    括弧内のカンマは分割対象としない（例: DECIMAL(10,2)）。
    """
    # 分割結果リストを初期化する
    defs: List[str] = []
    # 現在のトークンバッファを初期化する
    current = []
    # 括弧の深さを追跡する
    depth = 0
    # 文字を1つずつ処理する
    for char in columns_text:
        # 開き括弧の場合は深さをインクリメントする
        if char == "(":
            # 深さをインクリメントする
            depth += 1
            # 文字をバッファに追加する
            current.append(char)
        # 閉じ括弧の場合は深さをデクリメントする
        elif char == ")":
            # 深さをデクリメントする
            depth -= 1
            # 文字をバッファに追加する
            current.append(char)
        # カンマかつ深さが 0 の場合は分割する
        elif char == "," and depth == 0:
            # 現在のバッファを結合してリストに追加する
            defs.append("".join(current).strip())
            # バッファをリセットする
            current = []
        else:
            # その他の文字はバッファに追加する
            current.append(char)
    # 最後のトークンをリストに追加する
    if current:
        # 最後のバッファを追加する
        defs.append("".join(current).strip())
    # 分割結果リストを返す
    return defs


def _extract_table_names_from_sql(sql: str) -> Set[str]:
    """SQL テキストから CREATE TABLE で定義されたテーブル名のセットを返す。"""
    # CREATE TABLE ステートメントからテーブル名を抽出する
    matches = re.findall(
        r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)",
        sql,
        re.IGNORECASE,
    )
    # テーブル名を小文字化してセットに変換する
    return {m.strip('"').lower() for m in matches}


def _normalize_type(type_str: str) -> str:
    """PostgreSQL のデータ型を正規化した文字列を返す。"""
    # 型文字列を大文字に変換する
    upper = type_str.upper().strip()
    # 括弧内のパラメータを除去する
    base = re.sub(r"\s*\([^)]*\)", "", upper).strip()
    # 型エイリアスを正規化する
    type_aliases: Dict[str, str] = {
        # 整数型のエイリアス
        "INT": "INTEGER",
        # 短整数型のエイリアス
        "INT2": "SMALLINT",
        # 長整数型のエイリアス
        "INT4": "INTEGER",
        # 64 ビット整数型のエイリアス
        "INT8": "BIGINT",
        # 文字型のエイリアス
        "CHARACTER VARYING": "VARCHAR",
        # 文字型のエイリアス
        "CHARACTER": "CHAR",
        # ブール型のエイリアス
        "BOOL": "BOOLEAN",
        # 浮動小数点型のエイリアス
        "FLOAT4": "REAL",
        # 倍精度浮動小数点型のエイリアス
        "FLOAT8": "DOUBLE PRECISION",
    }
    # エイリアスが存在する場合は正規化した型を返す
    return type_aliases.get(base, base)


def _is_type_narrowing(old_type: str, new_type: str) -> bool:
    """古い型から新しい型への変更が型のナロウイング（縮小）かどうかを返す。"""
    # 型を正規化する
    old_norm = _normalize_type(old_type)
    # 新しい型を正規化する
    new_norm = _normalize_type(new_type)
    # 型が同じであれば narrowing ではない
    if old_norm == new_norm:
        # 同じ型なので narrowing ではない
        return False
    # ナロウイングの組み合わせを定義する（古い型 → 縮小される新しい型のセット）
    narrowing_map: Dict[str, Set[str]] = {
        # BIGINT から INTEGER・SMALLINT への縮小
        "BIGINT": {"INTEGER", "SMALLINT", "REAL", "FLOAT"},
        # INTEGER から SMALLINT への縮小
        "INTEGER": {"SMALLINT", "REAL"},
        # TEXT から VARCHAR・CHAR への縮小（長さ不明だが一般的に縮小）
        "TEXT": {"VARCHAR", "CHAR"},
        # DOUBLE PRECISION から REAL への縮小
        "DOUBLE PRECISION": {"REAL", "FLOAT"},
        # NUMERIC/DECIMAL から INTEGER への縮小
        "NUMERIC": {"INTEGER", "SMALLINT", "BIGINT"},
    }
    # 古い型に対応するナロウイング先のセットを取得する
    narrowing_targets = narrowing_map.get(old_norm, set())
    # 新しい型がナロウイング先に含まれているかを確認する
    return new_norm in narrowing_targets


# ===========================================================================
# SchemaEvolver クラス
# ===========================================================================


class SchemaEvolver:
    """スキーマ進化を管理するクラス。

    マイグレーションファイルの読み込み・互換性チェック・
    破壊的変更の検出・マイグレーション適用・ロールバックを提供する。
    """

    def __init__(self, migrations_dir: Optional[Path] = None) -> None:
        """SchemaEvolver をマイグレーションディレクトリで初期化する。

        Args:
            migrations_dir: マイグレーションファイルが格納されているディレクトリ
        """
        # マイグレーションディレクトリを設定する
        self._migrations_dir = migrations_dir
        # 読み込んだマイグレーションのキャッシュを初期化する
        self._migrations: Optional[List[SchemaVersion]] = None

    def load_migrations(self, migrations_dir: Path) -> List[SchemaVersion]:
        """マイグレーションディレクトリからマイグレーションファイルを読み込んで返す。

        マイグレーションファイルの命名規則:
        - V{番号}__{説明}.sql または
        - {番号}_{説明}.up.sql + {番号}_{説明}.down.sql

        Args:
            migrations_dir: マイグレーションファイルが格納されているディレクトリ

        Returns:
            読み込んだ SchemaVersion のリスト（バージョン番号の昇順）
        """
        # ディレクトリが存在するかどうかを確認する
        if not migrations_dir.exists():
            # ディレクトリが存在しない場合は空リストを返す
            logger.warning("マイグレーションディレクトリが存在しません: %s", migrations_dir)
            # 空リストを返す
            return []
        # マイグレーションディレクトリを保存する
        self._migrations_dir = migrations_dir
        # マイグレーションファイルのリストを初期化する
        migrations: List[SchemaVersion] = []
        # .sql ファイルを検索する
        sql_files = sorted(migrations_dir.glob("*.sql"))
        # SQL ファイルが存在しない場合はログに記録する
        if not sql_files:
            # SQL ファイルが見つからない旨をログに記録する
            logger.warning(
                "マイグレーションディレクトリに SQL ファイルが見つかりません: %s",
                migrations_dir,
            )
        # 各 SQL ファイルを読み込む
        for sql_file in sql_files:
            # ファイル名からバージョン ID を抽出する
            version_id = self._extract_version_id(sql_file.name)
            # SQL ファイルを読み込む
            try:
                # ファイルを UTF-8 で読み込む
                up_sql = sql_file.read_text(encoding="utf-8")
            except OSError as exc:
                # 読み込みエラーをログに記録してスキップする
                logger.warning(
                    "マイグレーションファイル読み込みエラー: %s (%s)",
                    sql_file,
                    exc,
                )
                # 次のファイルへ進む
                continue
            # 対応する down.sql ファイルを探す
            down_sql_path = sql_file.with_suffix("").with_suffix(".down.sql")
            # down.sql が存在する場合は読み込む
            down_sql = ""
            # down.sql ファイルが存在するかどうかを確認する
            if down_sql_path.exists():
                # down.sql ファイルを読み込む
                try:
                    # ファイルを UTF-8 で読み込む
                    down_sql = down_sql_path.read_text(encoding="utf-8")
                except OSError:
                    # 読み込みエラーは警告だけにする
                    pass
            # マイグレーションのハッシュを計算する
            migration_hash = hashlib.sha256(
                (up_sql + down_sql).encode("utf-8")
            ).hexdigest()
            # SchemaVersion を生成してリストに追加する
            version = SchemaVersion(
                version_id=version_id,
                applied_at="",
                migration_hash=migration_hash,
                up_sql=up_sql,
                down_sql=down_sql,
                description=self._extract_description(sql_file.name),
            )
            # マイグレーションリストに追加する
            migrations.append(version)
        # マイグレーションをキャッシュに保存する
        self._migrations = migrations
        # 読み込み完了をログに記録する
        logger.info("マイグレーション読み込み完了: %d 件 (%s)", len(migrations), migrations_dir)
        # マイグレーションリストを返す
        return migrations

    @staticmethod
    def _extract_version_id(filename: str) -> str:
        """ファイル名からバージョン ID を抽出して返す。"""
        # V{番号}__{説明}.sql 形式のファイル名を処理する
        match = re.match(r"^(V\d+)__", filename, re.IGNORECASE)
        # パターンが一致した場合はバージョン ID を返す
        if match:
            # バージョン ID を返す
            return match.group(1).upper()
        # {番号}_{説明}.sql 形式のファイル名を処理する
        num_match = re.match(r"^(\d+)_", filename)
        # パターンが一致した場合はバージョン ID を生成して返す
        if num_match:
            # バージョン ID を "V" + ゼロパディング番号で生成する
            return f"V{num_match.group(1).zfill(4)}"
        # パターンが一致しない場合はファイル名から拡張子を除いたものを返す
        return Path(filename).stem

    @staticmethod
    def _extract_description(filename: str) -> str:
        """ファイル名からマイグレーションの説明を抽出して返す。"""
        # V{番号}__{説明}.sql 形式のファイル名を処理する
        match = re.match(r"^V\d+__(.*?)\.sql$", filename, re.IGNORECASE)
        # パターンが一致した場合は説明を返す
        if match:
            # アンダースコアをスペースに変換して返す
            return match.group(1).replace("_", " ")
        # {番号}_{説明}.sql 形式のファイル名を処理する
        num_match = re.match(r"^\d+_(.*?)\.sql$", filename)
        # パターンが一致した場合は説明を返す
        if num_match:
            # アンダースコアをスペースに変換して返す
            return num_match.group(1).replace("_", " ")
        # パターンが一致しない場合はファイル名をそのまま返す
        return filename

    def check_compatibility(
        self, from_ver: SchemaVersion, to_ver: SchemaVersion
    ) -> CompatibilityRule:
        """2 つのスキーマバージョン間の互換性ルールを判定して返す。

        Args:
            from_ver: 移行元のスキーマバージョン
            to_ver: 移行先のスキーマバージョン

        Returns:
            検出された CompatibilityRule 値
        """
        # 破壊的変更を検出する
        breaking_changes = self.detect_breaking_changes(from_ver.up_sql, to_ver.up_sql)
        # 破壊的変更が検出された場合は NONE を返す
        if breaking_changes:
            # 破壊的変更の種別ログを記録する
            logger.warning(
                "破壊的変更を検出: %d 件 (%s → %s)",
                len(breaking_changes),
                from_ver.version_id,
                to_ver.version_id,
            )
            # 互換性なしを返す
            return CompatibilityRule.NONE
        # down_sql が存在する場合は前方互換性も確認する
        if to_ver.down_sql:
            # 逆方向の破壊的変更を確認する
            reverse_breaking = self.detect_breaking_changes(
                to_ver.up_sql, from_ver.up_sql
            )
            # 逆方向も破壊的変更がない場合は完全互換性を返す
            if not reverse_breaking:
                # 完全互換性を返す
                return CompatibilityRule.FULL
            # 正方向のみ破壊的変更がない場合は後方互換性を返す
            return CompatibilityRule.BACKWARD
        # down_sql がない場合は後方互換性のみとみなす
        return CompatibilityRule.BACKWARD

    def detect_breaking_changes(
        self, old_sql: str, new_sql: str
    ) -> List[BreakingChange]:
        """古い SQL と新しい SQL の差分から破壊的変更を検出して返す。

        Args:
            old_sql: 変更前の CREATE TABLE SQL
            new_sql: 変更後の CREATE TABLE SQL

        Returns:
            検出された BreakingChange のリスト
        """
        # 破壊的変更のリストを初期化する
        breaking_changes: List[BreakingChange] = []
        # 古い SQL から テーブル名を抽出する
        old_tables = _extract_table_names_from_sql(old_sql)
        # 新しい SQL からテーブル名を抽出する
        new_tables = _extract_table_names_from_sql(new_sql)
        # 削除されたテーブルを検出する
        removed_tables = old_tables - new_tables
        # 削除されたテーブルを破壊的変更として記録する
        for table in removed_tables:
            # テーブル削除の破壊的変更を追加する
            breaking_changes.append(
                BreakingChange(
                    change_type=BreakingChangeType.TABLE_REMOVED,
                    table_name=table,
                    column_name="",
                    before_description=f"テーブル {table} が存在する",
                    after_description=f"テーブル {table} が削除された",
                    impact="このテーブルを参照する全てのクライアントが影響を受ける",
                )
            )
        # 共通テーブルのカラム変更を検出する
        common_tables = old_tables & new_tables
        # 共通テーブルの各カラムを比較する
        for table in common_tables:
            # 古い SQL からテーブル定義を抽出する（テーブル名でフィルタ）
            old_table_sql = self._extract_table_sql(old_sql, table)
            # 新しい SQL からテーブル定義を抽出する
            new_table_sql = self._extract_table_sql(new_sql, table)
            # テーブル定義が取得できなかった場合はスキップする
            if not old_table_sql or not new_table_sql:
                # テーブル定義が取得できない場合はスキップする
                continue
            # 古い SQL からカラム定義を抽出する
            old_columns = _extract_columns_from_create_table(old_table_sql)
            # 新しい SQL からカラム定義を抽出する
            new_columns = _extract_columns_from_create_table(new_table_sql)
            # 削除されたカラムを検出する
            removed_columns = set(old_columns.keys()) - set(new_columns.keys())
            # 削除されたカラムを破壊的変更として記録する
            for col in removed_columns:
                # カラム削除の破壊的変更を追加する
                breaking_changes.append(
                    BreakingChange(
                        change_type=BreakingChangeType.COLUMN_REMOVED,
                        table_name=table,
                        column_name=col,
                        before_description=f"カラム {col} が存在する",
                        after_description=f"カラム {col} が削除された",
                        impact="このカラムを参照するクエリが失敗する",
                    )
                )
            # 共通カラムの型変更・制約変更を検出する
            common_columns = set(old_columns.keys()) & set(new_columns.keys())
            # 各共通カラムを比較する
            for col in common_columns:
                # 古いカラム情報を取得する
                old_col = old_columns[col]
                # 新しいカラム情報を取得する
                new_col = new_columns[col]
                # 型のナロウイングを検出する
                if _is_type_narrowing(old_col["type"], new_col["type"]):
                    # 型ナロウイングの破壊的変更を追加する
                    breaking_changes.append(
                        BreakingChange(
                            change_type=BreakingChangeType.TYPE_NARROWED,
                            table_name=table,
                            column_name=col,
                            before_description=f"型 {old_col['type']}",
                            after_description=f"型 {new_col['type']}（縮小）",
                            impact="既存データが新しい型に収まらない可能性がある",
                        )
                    )
                # NOT NULL 制約の追加を検出する（DEFAULT なしの場合のみ破壊的）
                if (
                    not old_col["not_null"]
                    and new_col["not_null"]
                    and not new_col["has_default"]
                ):
                    # NOT NULL 制約追加の破壊的変更を追加する
                    breaking_changes.append(
                        BreakingChange(
                            change_type=BreakingChangeType.CONSTRAINT_ADDED,
                            table_name=table,
                            column_name=col,
                            before_description="NULL 許可",
                            after_description="NOT NULL（DEFAULT なし）",
                            impact="既存の NULL 値が制約違反になる",
                        )
                    )
        # 検出した破壊的変更リストを返す
        return breaking_changes

    @staticmethod
    def _extract_table_sql(sql: str, table_name: str) -> str:
        """指定したテーブル名の CREATE TABLE SQL を抽出して返す。"""
        # テーブル名に対応する CREATE TABLE 定義を抽出する正規表現
        pattern = rf"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?[\"']?{re.escape(table_name)}[\"']?\s*\(.*?\);"
        # 正規表現で検索する
        match = re.search(pattern, sql, re.DOTALL | re.IGNORECASE)
        # 一致した場合はその文字列を返す
        if match:
            # 一致した CREATE TABLE 文を返す
            return match.group(0)
        # 一致しない場合は空文字を返す
        return ""

    def apply_migration(self, conn: Any, migration: SchemaVersion) -> bool:
        """指定したマイグレーションをデータベースに適用して成否を返す。

        Args:
            conn: psycopg2 の接続オブジェクト
            migration: 適用する SchemaVersion

        Returns:
            適用が成功すれば True
        """
        # psycopg2 が利用可能かどうかを確認する
        if not _HAS_PSYCOPG2:
            # psycopg2 が存在しない場合はエラーをログに記録して False を返す
            logger.error("psycopg2 が必要です")
            # False を返す
            return False
        # up_sql が空の場合はスキップする
        if not migration.up_sql.strip():
            # 空の SQL はスキップする
            logger.warning("マイグレーション SQL が空です: %s", migration.version_id)
            # False を返す
            return False
        # マイグレーションを実行する
        try:
            # カーソルを取得する
            with conn.cursor() as cur:
                # up_sql を実行する
                cur.execute(migration.up_sql)
                # schema_version テーブルにマイグレーション記録を挿入する
                cur.execute(
                    """
                    INSERT INTO schema_version (version_id, applied_at, migration_hash)
                    VALUES (%s, %s, %s)
                    ON CONFLICT (version_id) DO UPDATE
                    SET applied_at = EXCLUDED.applied_at
                    """,
                    (
                        migration.version_id,
                        datetime.now(timezone.utc).isoformat(),
                        migration.migration_hash,
                    ),
                )
            # コミットする
            conn.commit()
            # マイグレーション適用済みフラグを設定する
            migration.is_applied = True
            # 適用時刻を設定する
            migration.applied_at = datetime.now(timezone.utc).isoformat()
            # 適用成功をログに記録する
            logger.info("マイグレーション適用完了: %s", migration.version_id)
            # 成功を返す
            return True
        except Exception as exc:
            # エラーをログに記録する
            logger.error(
                "マイグレーション適用失敗: %s (%s)",
                migration.version_id,
                exc,
            )
            # ロールバックする
            conn.rollback()
            # 失敗を返す
            return False

    def rollback_to(self, conn: Any, target_version: str) -> bool:
        """指定したバージョンにロールバックして成否を返す。

        Args:
            conn: psycopg2 の接続オブジェクト
            target_version: ロールバック先のバージョン ID

        Returns:
            ロールバックが成功すれば True
        """
        # psycopg2 が利用可能かどうかを確認する
        if not _HAS_PSYCOPG2:
            # psycopg2 が存在しない場合はエラーをログに記録して False を返す
            logger.error("psycopg2 が必要です")
            # False を返す
            return False
        # キャッシュされたマイグレーションが存在するかを確認する
        if not self._migrations:
            # マイグレーションが読み込まれていない場合はエラーをログに記録する
            logger.error("マイグレーションが読み込まれていません。load_migrations() を呼び出してください。")
            # False を返す
            return False
        # ターゲットバージョンよりも後のマイグレーションを逆順で取得する
        migrations_to_rollback = [
            m for m in reversed(self._migrations)
            if m.version_id > target_version and m.is_applied
        ]
        # ロールバック対象が存在しない場合はスキップする
        if not migrations_to_rollback:
            # ロールバック対象なしをログに記録する
            logger.info(
                "ロールバック対象なし: target_version=%s", target_version
            )
            # 成功（何もしない）を返す
            return True
        # 各マイグレーションをロールバックする
        for migration in migrations_to_rollback:
            # down_sql が空の場合はスキップする
            if not migration.down_sql.strip():
                # ロールバック SQL がない場合は警告をログに記録する
                logger.warning(
                    "ロールバック SQL がありません: %s（スキップ）",
                    migration.version_id,
                )
                # 次のマイグレーションへ進む
                continue
            # ロールバックを実行する
            try:
                # カーソルを取得する
                with conn.cursor() as cur:
                    # down_sql を実行する
                    cur.execute(migration.down_sql)
                    # schema_version テーブルからマイグレーション記録を削除する
                    cur.execute(
                        "DELETE FROM schema_version WHERE version_id = %s",
                        (migration.version_id,),
                    )
                # コミットする
                conn.commit()
                # 適用済みフラグをリセットする
                migration.is_applied = False
                # ロールバック成功をログに記録する
                logger.info("ロールバック完了: %s", migration.version_id)
            except Exception as exc:
                # エラーをログに記録する
                logger.error(
                    "ロールバック失敗: %s (%s)",
                    migration.version_id,
                    exc,
                )
                # ロールバックする
                conn.rollback()
                # 失敗を返す
                return False
        # 全ロールバックが成功した場合は True を返す
        return True
