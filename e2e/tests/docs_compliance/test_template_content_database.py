"""テンプレート仕様-データベース.md の内容準拠テスト。

CLI/templates/database/ のテンプレートファイルの内容が
仕様ドキュメントのコードブロックと一致するかを検証する。
"""

from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
DB = TEMPLATES / "database"


class TestPostgresqlUpContent:
    """テンプレート仕様-データベース.md: PostgreSQL 001_init.up.sql の内容検証。"""

    def setup_method(self) -> None:
        self.content = (DB / "postgresql" / "001_init.up.sql.tera").read_text(encoding="utf-8")

    def test_service_name_variable(self) -> None:
        assert "{{ service_name" in self.content

    def test_uuid_ossp_extension(self) -> None:
        assert 'CREATE EXTENSION IF NOT EXISTS "uuid-ossp"' in self.content

    def test_pgcrypto_extension(self) -> None:
        assert 'CREATE EXTENSION IF NOT EXISTS "pgcrypto"' in self.content

    def test_create_schema(self) -> None:
        assert "CREATE SCHEMA IF NOT EXISTS" in self.content

    def test_examples_table(self) -> None:
        assert "CREATE TABLE" in self.content
        assert "examples" in self.content

    def test_uuid_primary_key(self) -> None:
        assert "UUID PRIMARY KEY" in self.content
        assert "uuid_generate_v4()" in self.content

    def test_timestamptz_columns(self) -> None:
        assert "TIMESTAMPTZ" in self.content

    def test_update_trigger(self) -> None:
        assert "update_updated_at" in self.content
        assert "CREATE OR REPLACE FUNCTION" in self.content
        assert "CREATE TRIGGER" in self.content

    def test_indexes(self) -> None:
        assert "idx_examples_status" in self.content
        assert "idx_examples_created_at" in self.content


class TestPostgresqlDownContent:
    """テンプレート仕様-データベース.md: PostgreSQL 001_init.down.sql の内容検証。"""

    def setup_method(self) -> None:
        self.content = (DB / "postgresql" / "001_init.down.sql.tera").read_text(encoding="utf-8")

    def test_drop_trigger(self) -> None:
        assert "DROP TRIGGER IF EXISTS" in self.content

    def test_drop_function(self) -> None:
        assert "DROP FUNCTION IF EXISTS" in self.content

    def test_drop_table(self) -> None:
        assert "DROP TABLE IF EXISTS" in self.content

    def test_drop_schema(self) -> None:
        assert "DROP SCHEMA IF EXISTS" in self.content

    def test_drop_extensions(self) -> None:
        assert 'DROP EXTENSION IF EXISTS "pgcrypto"' in self.content
        assert 'DROP EXTENSION IF EXISTS "uuid-ossp"' in self.content


class TestMysqlUpContent:
    """テンプレート仕様-データベース.md: MySQL 001_init.up.sql の内容検証。"""

    def setup_method(self) -> None:
        self.content = (DB / "mysql" / "001_init.up.sql.tera").read_text(encoding="utf-8")

    def test_create_database(self) -> None:
        assert "CREATE DATABASE IF NOT EXISTS" in self.content

    def test_utf8mb4(self) -> None:
        """テンプレート仕様-データベース.md: MySQL は utf8mb4 指定必須。"""
        assert "utf8mb4" in self.content

    def test_uuid_function(self) -> None:
        assert "UUID()" in self.content

    def test_datetime6(self) -> None:
        """テンプレート仕様-データベース.md: MySQL は DATETIME(6)。"""
        assert "DATETIME(6)" in self.content

    def test_on_update_current_timestamp(self) -> None:
        assert "ON UPDATE CURRENT_TIMESTAMP" in self.content

    def test_innodb(self) -> None:
        assert "InnoDB" in self.content

    def test_indexes(self) -> None:
        assert "idx_examples_status" in self.content
        assert "idx_examples_created_at" in self.content


class TestMysqlDownContent:
    """テンプレート仕様-データベース.md: MySQL 001_init.down.sql の内容検証。"""

    def setup_method(self) -> None:
        self.content = (DB / "mysql" / "001_init.down.sql.tera").read_text(encoding="utf-8")

    def test_drop_table(self) -> None:
        assert "DROP TABLE IF EXISTS" in self.content

    def test_drop_database(self) -> None:
        assert "DROP DATABASE IF EXISTS" in self.content


class TestSqliteUpContent:
    """テンプレート仕様-データベース.md: SQLite 001_init.up.sql の内容検証。"""

    def setup_method(self) -> None:
        self.content = (DB / "sqlite" / "001_init.up.sql.tera").read_text(encoding="utf-8")

    def test_create_table(self) -> None:
        assert "CREATE TABLE IF NOT EXISTS examples" in self.content

    def test_randomblob_id(self) -> None:
        """テンプレート仕様-データベース.md: SQLite は randomblob で UUID 代替。"""
        assert "randomblob" in self.content

    def test_text_type(self) -> None:
        """テンプレート仕様-データベース.md: SQLite は TEXT 型。"""
        assert "TEXT PRIMARY KEY" in self.content

    def test_strftime_timestamp(self) -> None:
        assert "strftime" in self.content

    def test_update_trigger(self) -> None:
        assert "trigger_update_updated_at" in self.content
        assert "CREATE TRIGGER IF NOT EXISTS" in self.content

    def test_indexes(self) -> None:
        assert "idx_examples_status" in self.content
        assert "idx_examples_created_at" in self.content


class TestSqliteDownContent:
    """テンプレート仕様-データベース.md: SQLite 001_init.down.sql の内容検証。"""

    def setup_method(self) -> None:
        self.content = (DB / "sqlite" / "001_init.down.sql.tera").read_text(encoding="utf-8")

    def test_drop_trigger(self) -> None:
        assert "DROP TRIGGER IF EXISTS" in self.content

    def test_drop_indexes(self) -> None:
        assert "DROP INDEX IF EXISTS idx_examples_created_at" in self.content
        assert "DROP INDEX IF EXISTS idx_examples_status" in self.content

    def test_drop_table(self) -> None:
        assert "DROP TABLE IF EXISTS examples" in self.content


class TestMigrationSymmetry:
    """テンプレート仕様-データベース.md: up/down の対称性検証。"""

    @pytest.mark.parametrize("db_type", ["postgresql", "mysql", "sqlite"])
    def test_both_files_exist(self, db_type: str) -> None:
        assert (DB / db_type / "001_init.up.sql.tera").exists()
        assert (DB / db_type / "001_init.down.sql.tera").exists()

    @pytest.mark.parametrize("db_type", ["postgresql", "mysql", "sqlite"])
    def test_down_has_drop(self, db_type: str) -> None:
        """テンプレート仕様-データベース.md: down は up の逆操作。"""
        content = (DB / db_type / "001_init.down.sql.tera").read_text(encoding="utf-8")
        assert "DROP" in content

    @pytest.mark.parametrize("db_type", ["postgresql", "mysql", "sqlite"])
    def test_up_has_create(self, db_type: str) -> None:
        content = (DB / db_type / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "CREATE" in content


# ============================================================================
# テンプレート仕様-データベース.md ギャップ補完テスト (8件)
# ============================================================================

DOCS = ROOT / "docs"


class TestMigrationNamingConvention:
    """テンプレート仕様-データベース.md: マイグレーション命名規則テスト。"""

    def test_naming_format(self) -> None:
        """テンプレート仕様-データベース.md: {番号}_{説明}.{方向}.sql の形式。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "{番号}_{説明}.{方向}.sql" in docs_content

    def test_number_zero_padded(self) -> None:
        """テンプレート仕様-データベース.md: 番号は3桁ゼロ埋め。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "3桁ゼロ埋め" in docs_content

    @pytest.mark.parametrize("db_type", ["postgresql", "mysql", "sqlite"])
    def test_filename_follows_convention(self, db_type: str) -> None:
        """テンプレート仕様-データベース.md: ファイル名が命名規則に従う。"""
        assert (DB / db_type / "001_init.up.sql.tera").exists()
        assert (DB / db_type / "001_init.down.sql.tera").exists()


class TestMigrationPlacementPath:
    """テンプレート仕様-データベース.md: 配置パステスト。"""

    def test_system_path(self) -> None:
        """テンプレート仕様-データベース.md: system 階層の配置パス。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "regions/system/server/{lang}/{service_name}/migrations/" in docs_content

    def test_business_path(self) -> None:
        """テンプレート仕様-データベース.md: business 階層の配置パス。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "regions/business/{domain}/server/{lang}/{service_name}/migrations/" in docs_content

    def test_service_path(self) -> None:
        """テンプレート仕様-データベース.md: service 階層の配置パス。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "regions/service/{service_name}/server/{lang}/migrations/" in docs_content


class TestMigrationTemplateVariables:
    """テンプレート仕様-データベース.md: テンプレート変数検証。"""

    def test_service_name_variable(self) -> None:
        """テンプレート仕様-データベース.md: {{ service_name }} 変数。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "{{ service_name }}" in docs_content

    def test_lang_variable(self) -> None:
        """テンプレート仕様-データベース.md: {{ lang }} 変数。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "{{ lang }}" in docs_content

    def test_domain_variable(self) -> None:
        """テンプレート仕様-データベース.md: {{ domain }} 変数。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "{{ domain }}" in docs_content

    def test_database_type_variable(self) -> None:
        """テンプレート仕様-データベース.md: {{ database_type }} 変数。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "{{ database_type }}" in docs_content


class TestRDBMSSyntaxDifferences:
    """テンプレート仕様-データベース.md: RDBMS 間の構文差分表テスト。"""

    def test_docs_have_syntax_diff_table(self) -> None:
        """テンプレート仕様-データベース.md: 構文差分表がある。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "RDBMS 間の構文差分" in docs_content

    def test_postgresql_uuid(self) -> None:
        """テンプレート仕様-データベース.md: PostgreSQL の UUID 生成方法。"""
        content = (DB / "postgresql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "uuid_generate_v4()" in content

    def test_mysql_uuid(self) -> None:
        """テンプレート仕様-データベース.md: MySQL の UUID 生成方法。"""
        content = (DB / "mysql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "UUID()" in content

    def test_sqlite_uuid_alternative(self) -> None:
        """テンプレート仕様-データベース.md: SQLite の UUID 代替 (randomblob)。"""
        content = (DB / "sqlite" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "randomblob" in content

    def test_postgresql_timestamp(self) -> None:
        """テンプレート仕様-データベース.md: PostgreSQL は TIMESTAMPTZ。"""
        content = (DB / "postgresql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "TIMESTAMPTZ" in content

    def test_mysql_timestamp(self) -> None:
        """テンプレート仕様-データベース.md: MySQL は DATETIME(6)。"""
        content = (DB / "mysql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "DATETIME(6)" in content

    def test_sqlite_timestamp(self) -> None:
        """テンプレート仕様-データベース.md: SQLite は TEXT + strftime。"""
        content = (DB / "sqlite" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "strftime" in content


class TestMigrationOperationRules:
    """テンプレート仕様-データベース.md: マイグレーション運用ルールテスト。"""

    def test_idempotency_rule(self) -> None:
        """テンプレート仕様-データベース.md: 冪等性ルール。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "冪等性" in docs_content

    def test_symmetry_rule(self) -> None:
        """テンプレート仕様-データベース.md: 対称性ルール。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "対称性" in docs_content

    def test_verification_rule(self) -> None:
        """テンプレート仕様-データベース.md: 検証ルール。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "ステージング環境" in docs_content

    def test_separation_rule(self) -> None:
        """テンプレート仕様-データベース.md: データとスキーマの分離ルール。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "分離" in docs_content


class TestMigrationToolConfig:
    """テンプレート仕様-データベース.md: マイグレーションツール設定テスト。"""

    def test_go_golang_migrate(self) -> None:
        """テンプレート仕様-データベース.md: Go は golang-migrate を使用。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "migrate" in docs_content
        assert "DATABASE_URL" in docs_content

    def test_rust_sqlx_cli(self) -> None:
        """テンプレート仕様-データベース.md: Rust は sqlx-cli を使用。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "sqlx migrate" in docs_content

    def test_go_migrate_commands(self) -> None:
        """テンプレート仕様-データベース.md: Go migrate コマンド例。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "migrate -path ./migrations" in docs_content

    def test_rust_sqlx_commands(self) -> None:
        """テンプレート仕様-データベース.md: Rust sqlx コマンド例。"""
        docs_content = (DOCS / "テンプレート仕様-データベース.md").read_text(encoding="utf-8")
        assert "sqlx migrate run" in docs_content
        assert "sqlx migrate revert" in docs_content


class TestMysqlServiceNameSnakeVariable:
    """テンプレート仕様-データベース.md: MySQL の service_name_snake 変数テスト。"""

    def test_up_uses_service_name_snake(self) -> None:
        """テンプレート仕様-データベース.md: MySQL up に {{ service_name_snake }} 変数。"""
        content = (DB / "mysql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "{{ service_name_snake }}" in content

    def test_down_uses_service_name_snake(self) -> None:
        """テンプレート仕様-データベース.md: MySQL down に {{ service_name_snake }} 変数。"""
        content = (DB / "mysql" / "001_init.down.sql.tera").read_text(encoding="utf-8")
        assert "{{ service_name_snake }}" in content


class TestMigrationIdempotency:
    """テンプレート仕様-データベース.md: 冪等性テスト。"""

    def test_postgresql_if_not_exists(self) -> None:
        """テンプレート仕様-データベース.md: PostgreSQL up に IF NOT EXISTS。"""
        content = (DB / "postgresql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "IF NOT EXISTS" in content

    def test_sqlite_if_not_exists(self) -> None:
        """テンプレート仕様-データベース.md: SQLite up に IF NOT EXISTS。"""
        content = (DB / "sqlite" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "IF NOT EXISTS" in content

    def test_mysql_if_not_exists(self) -> None:
        """テンプレート仕様-データベース.md: MySQL up に IF NOT EXISTS。"""
        content = (DB / "mysql" / "001_init.up.sql.tera").read_text(encoding="utf-8")
        assert "IF NOT EXISTS" in content

    @pytest.mark.parametrize("db_type", ["postgresql", "mysql", "sqlite"])
    def test_down_if_exists(self, db_type: str) -> None:
        """テンプレート仕様-データベース.md: down に IF EXISTS。"""
        content = (DB / db_type / "001_init.down.sql.tera").read_text(encoding="utf-8")
        assert "IF EXISTS" in content
