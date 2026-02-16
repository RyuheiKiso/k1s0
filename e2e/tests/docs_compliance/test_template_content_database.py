"""テンプレート仕様-データベース.md の内容準拠テスト。

CLI/templates/database/ のテンプレートファイルの内容が
仕様ドキュメントのコードブロックと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
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
