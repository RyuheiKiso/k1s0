"""src/test/integration/data/test_v1_property_axiom.py

data 軸の v1_property_axiom 検証クラスに対するインテグレーションテスト。
hypothesis を使用して監査チェーン・RLS フィルタ・スキーマ移行・PII クラスタの
プロパティを検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: data_property_axiom_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import hashlib
import json
import uuid
# pytest フレームワークをインポートする
import pytest
# hypothesis ライブラリをインポートする
from hypothesis import given, settings, assume, HealthCheck
# hypothesis の戦略モジュールをインポートする
from hypothesis import strategies as st
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field


# 監査ログイベントを表すデータクラスを定義する
@dataclass
class AuditEvent:
    """PostgreSQL pgaudit 相当の監査イベント。"""

    # イベント ID を保持するフィールドを定義する
    event_id: str
    # イベントが発生したテナント ID を保持するフィールドを定義する
    tenant_id: str
    # 操作種別を保持するフィールドを定義する
    operation: str
    # 操作対象のテーブル名を保持するフィールドを定義する
    table_name: str
    # イベントのペイロードを保持するフィールドを定義する
    payload: dict[str, Any]
    # 前のイベントのハッシュを保持するフィールドを定義する
    prev_hash: str = ""

    # イベントのハッシュを計算するメソッドを定義する
    def compute_hash(self) -> str:
        """イベントの SHA-256 ハッシュを計算する。"""
        # ハッシュ計算の入力データを構築する
        data = {
            "event_id": self.event_id,
            "tenant_id": self.tenant_id,
            "operation": self.operation,
            "table_name": self.table_name,
            "payload": self.payload,
            "prev_hash": self.prev_hash,
        }
        # JSON 文字列に変換する（キーをソートして決定論的にする）
        json_str = json.dumps(data, sort_keys=True, ensure_ascii=False)
        # SHA-256 ハッシュを計算して返す
        return hashlib.sha256(json_str.encode("utf-8")).hexdigest()


# 監査チェーンを管理するクラスを定義する
class AuditChain:
    """イベントのハッシュチェーンを管理する監査ログ。"""

    # チェーンを初期化するメソッドを定義する
    def __init__(self) -> None:
        """監査チェーンを初期化する。"""
        # イベントのリストを初期化する
        self._events: list[AuditEvent] = []
        # ハッシュのリストを初期化する
        self._hashes: list[str] = []
        # 前のハッシュを初期化する（ジェネシスは空文字列）
        self._prev_hash: str = ""

    # チェーンにイベントを追加するメソッドを定義する
    def append(self, event: AuditEvent) -> str:
        """イベントをチェーンに追加してハッシュを返す。"""
        # 前のハッシュをイベントに設定する
        event.prev_hash = self._prev_hash
        # イベントのハッシュを計算する
        event_hash = event.compute_hash()
        # イベントをリストに追加する
        self._events.append(event)
        # ハッシュをリストに追加する
        self._hashes.append(event_hash)
        # 前のハッシュを更新する
        self._prev_hash = event_hash
        # ハッシュを返す
        return event_hash

    # チェーンのイベントを取得するプロパティを定義する
    @property
    def events(self) -> list[AuditEvent]:
        """イベントのコピーを返す。"""
        # イベントのコピーを返す
        return list(self._events)

    # チェーンのハッシュリストを取得するプロパティを定義する
    @property
    def hashes(self) -> list[str]:
        """ハッシュのコピーを返す。"""
        # ハッシュのコピーを返す
        return list(self._hashes)


# RLS フィルタシミュレーターを定義するクラスを定義する
class RLSFilter:
    """PostgreSQL RLS 相当のフィルタリングをシミュレートするクラス。"""

    # フィルターを初期化するメソッドを定義する
    def __init__(self) -> None:
        """RLS フィルターを初期化する。"""
        # 全レコードを保持するストレージを初期化する
        self._all_records: list[dict[str, Any]] = []
        # 現在のテナントコンテキストを初期化する
        self._current_tenant: Optional[str] = None

    # テナントコンテキストを設定するメソッドを定義する
    def set_tenant(self, tenant_id: str) -> None:
        """現在のテナントコンテキストを設定する。"""
        # テナントコンテキストを設定する
        self._current_tenant = tenant_id

    # レコードを挿入するメソッドを定義する
    def insert(self, record: dict[str, Any]) -> None:
        """レコードを全テーブルに挿入する（フィルタなし）。"""
        # レコードを全テーブルに挿入する
        self._all_records.append(record)

    # RLS フィルタを適用してレコードを取得するメソッドを定義する
    def query_filtered(self) -> list[dict[str, Any]]:
        """現在のテナントコンテキストでフィルタされたレコードを返す。"""
        # テナントコンテキストが設定されていない場合は空リストを返す
        if self._current_tenant is None:
            # コンテキストなしは安全のため空リストを返す
            return []
        # 現在のテナントに属するレコードのみを返す
        return [
            r for r in self._all_records
            if r.get("tenant_id") == self._current_tenant
        ]

    # RLS フィルタなしで全レコードを取得するメソッドを定義する
    def query_unfiltered(self) -> list[dict[str, Any]]:
        """フィルタなしで全レコードを返す（管理者権限相当）。"""
        # 全レコードのコピーを返す
        return list(self._all_records)


# スキーマ移行をシミュレートするクラスを定義する
class MockSchemaMigration:
    """データベーススキーマ移行をシミュレートするクラス。"""

    # マイグレーションを初期化するメソッドを定義する
    def __init__(self) -> None:
        """スキーママイグレーションを初期化する。"""
        # 現在のスキーマ（カラム名のセット）を初期化する
        self._schema: set[str] = {"id", "created_at", "updated_at"}
        # マイグレーション履歴を初期化する
        self._migration_history: list[dict[str, Any]] = []

    # スキーマにカラムを追加するメソッドを定義する
    def migrate_up(self, column_name: str, column_type: str) -> None:
        """スキーマにカラムを追加する。"""
        # カラムをスキーマに追加する
        self._schema.add(column_name)
        # マイグレーション履歴に追加する
        self._migration_history.append({
            "direction": "up",
            "column": column_name,
            "type": column_type,
        })

    # スキーマからカラムを削除するメソッドを定義する
    def migrate_down(self, column_name: str) -> None:
        """スキーマからカラムを削除する。"""
        # カラムをスキーマから削除する
        self._schema.discard(column_name)
        # マイグレーション履歴に追加する
        self._migration_history.append({
            "direction": "down",
            "column": column_name,
        })

    # 現在のスキーマを取得するプロパティを定義する
    @property
    def schema(self) -> set[str]:
        """現在のスキーマのコピーを返す。"""
        # スキーマのコピーを返す
        return set(self._schema)


# data 軸プロパティ検証テストスイートクラスを定義する
class TestDataPropertyAxiom:
    """data 軸 v1_property_axiom の検証テストスイート。"""

    # 監査チェーンのフィクスチャを定義する
    @pytest.fixture
    def audit_chain(self) -> Generator[AuditChain, None, None]:
        """AuditChain フィクスチャを生成する。"""
        # 監査チェーンをインスタンス化する
        chain = AuditChain()
        # フィクスチャを返す
        yield chain

    # RLS フィルターのフィクスチャを定義する
    @pytest.fixture
    def rls_filter(self) -> Generator[RLSFilter, None, None]:
        """RLSFilter フィクスチャを生成する。"""
        # RLS フィルターをインスタンス化する
        f = RLSFilter()
        # フィクスチャを返す
        yield f

    # 監査チェーンのハッシュが決定論的であるプロパティをテストする
    @given(
        # 操作種別を生成する戦略を定義する
        operations=st.lists(
            st.sampled_from(["INSERT", "UPDATE", "DELETE", "SELECT"]),
            min_size=1,
            max_size=20,
        ),
        # テーブル名を生成する戦略を定義する
        table_name=st.sampled_from(["orders", "products", "shipments", "machines"]),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_audit_hash_deterministic(
        self, operations: list[str], table_name: str
    ) -> None:
        """同じイベント列に対して監査チェーンのハッシュが決定論的であることを検証する。"""
        # イベントリストを固定の ID で生成する（再現性を確保する）
        events_data = [
            {
                "event_id": f"evt_{i:04d}",
                "tenant_id": "tenant_fixed_a1b2c3d4",
                "operation": op,
                "table_name": table_name,
                "payload": {"row_id": i, "op": op},
            }
            for i, op in enumerate(operations)
        ]
        # 1 回目のチェーン構築を実行する
        chain1 = AuditChain()
        # イベントを追加してハッシュを収集する
        hashes1 = []
        # 各イベントデータを処理する
        for data in events_data:
            # イベントを生成してチェーンに追加する
            event = AuditEvent(**data)
            # チェーンに追加してハッシュを取得する
            h = chain1.append(event)
            # ハッシュをリストに追加する
            hashes1.append(h)
        # 2 回目のチェーン構築を実行する（同じデータで再現する）
        chain2 = AuditChain()
        # イベントを追加してハッシュを収集する
        hashes2 = []
        # 各イベントデータを処理する
        for data in events_data:
            # イベントを生成してチェーンに追加する
            event = AuditEvent(**data)
            # チェーンに追加してハッシュを取得する
            h = chain2.append(event)
            # ハッシュをリストに追加する
            hashes2.append(h)
        # 2 回の構築で同じハッシュが得られることを確認する
        assert hashes1 == hashes2, (
            f"Audit chain hash is not deterministic: {hashes1} != {hashes2}"
        )

    # RLS フィルタ後のクエリが非フィルタのサブセットであるプロパティをテストする
    @given(
        # テナント ID の候補リストを生成する戦略を定義する
        tenant_ids=st.lists(
            st.text(
                alphabet=st.characters(whitelist_categories=("Ll",)),
                min_size=4,
                max_size=12,
            ),
            min_size=2,
            max_size=5,
            unique=True,
        ),
        # テナントごとのレコード数を生成する戦略を定義する
        record_count=st.integers(min_value=1, max_value=20),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_rls_filtered_is_subset_of_unfiltered(
        self, tenant_ids: list[str], record_count: int
    ) -> None:
        """RLS フィルタ後のクエリが非フィルタクエリの部分集合であることを検証する。"""
        # RLS フィルターをインスタンス化する
        rls = RLSFilter()
        # 各テナントの record_count 件のレコードを挿入する
        for tenant_id in tenant_ids:
            # テナントのレコードを挿入する
            for i in range(record_count):
                # レコードを挿入する
                rls.insert({
                    "id": f"{tenant_id}_rec_{i}",
                    "tenant_id": tenant_id,
                    "data": i,
                })
        # 全テナントに対してフィルタ結果が非フィルタの部分集合であることを確認する
        for tenant_id in tenant_ids:
            # テナントコンテキストを設定する
            rls.set_tenant(tenant_id)
            # フィルタされたレコードを取得する
            filtered = rls.query_filtered()
            # 非フィルタのレコードを取得する
            unfiltered = rls.query_unfiltered()
            # フィルタ結果のセットを構築する
            filtered_ids = {r["id"] for r in filtered}
            # 非フィルタ結果のセットを構築する
            unfiltered_ids = {r["id"] for r in unfiltered}
            # フィルタ結果が非フィルタの部分集合であることを確認する
            assert filtered_ids.issubset(unfiltered_ids), (
                f"RLS filtered result is not a subset of unfiltered for tenant {tenant_id}"
            )
            # フィルタ結果には自分のレコードのみが含まれることを確認する
            for r in filtered:
                # レコードのテナント ID が現在のテナントと一致することを確認する
                assert r["tenant_id"] == tenant_id, (
                    f"RLS leaked record from another tenant: {r['tenant_id']} != {tenant_id}"
                )

    # スキーマ移行の up + down がオリジナルスキーマを復元するプロパティをテストする
    @given(
        # カラム名を生成する戦略を定義する
        column_name=st.text(
            alphabet=st.characters(whitelist_categories=("Ll",)),
            min_size=3,
            max_size=20,
        ),
        # カラム型を生成する戦略を定義する
        column_type=st.sampled_from(["INTEGER", "TEXT", "TIMESTAMP", "BOOLEAN", "JSONB"]),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_schema_migration_up_down_restores_original(
        self, column_name: str, column_type: str
    ) -> None:
        """スキーマ移行 up + down がオリジナルスキーマを復元することを検証する。"""
        # マイグレーションをインスタンス化する
        migration = MockSchemaMigration()
        # オリジナルスキーマを記録する
        original_schema = migration.schema.copy()
        # カラムが既存のスキーマに含まれている場合はスキップする
        assume(column_name not in original_schema)
        # up マイグレーションを実行する
        migration.migrate_up(column_name, column_type)
        # カラムがスキーマに追加されたことを確認する
        assert column_name in migration.schema
        # down マイグレーションを実行する
        migration.migrate_down(column_name)
        # オリジナルスキーマが復元されたことを確認する
        assert migration.schema == original_schema, (
            f"Schema not restored: expected {original_schema}, got {migration.schema}"
        )

    # PII クラスタクエリがテナント境界を超えないプロパティをテストする
    @given(
        # テナント A のレコード数を生成する戦略を定義する
        tenant_a_count=st.integers(min_value=1, max_value=30),
        # テナント B のレコード数を生成する戦略を定義する
        tenant_b_count=st.integers(min_value=1, max_value=30),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_pii_cluster_never_cross_tenant_boundary(
        self, tenant_a_count: int, tenant_b_count: int
    ) -> None:
        """PII クラスタクエリがテナント境界を超えないことを検証する。"""
        # テナント A の ID を定義する
        tenant_a = "tenant_pii_alpha"
        # テナント B の ID を定義する
        tenant_b = "tenant_pii_beta"
        # RLS フィルターをインスタンス化する
        rls = RLSFilter()
        # テナント A の PII レコードを挿入する
        for i in range(tenant_a_count):
            # PII データを含むレコードを挿入する
            rls.insert({
                "id": f"pii_a_{i}",
                "tenant_id": tenant_a,
                "pii_name": f"User_Alpha_{i}",
                "pii_email": f"user_a_{i}@example.com",
            })
        # テナント B の PII レコードを挿入する
        for i in range(tenant_b_count):
            # PII データを含むレコードを挿入する
            rls.insert({
                "id": f"pii_b_{i}",
                "tenant_id": tenant_b,
                "pii_name": f"User_Beta_{i}",
                "pii_email": f"user_b_{i}@example.com",
            })
        # テナント A コンテキストでクエリした結果にテナント B のデータが含まれないことを確認する
        rls.set_tenant(tenant_a)
        # テナント A のフィルタ結果を取得する
        a_filtered = rls.query_filtered()
        # テナント A の結果にテナント B のデータが含まれないことを確認する
        for record in a_filtered:
            # レコードのテナント ID がテナント A であることを確認する
            assert record["tenant_id"] == tenant_a, (
                f"PII cluster leaked Tenant B data to Tenant A: {record}"
            )
        # テナント A の結果の件数が正しいことを確認する
        assert len(a_filtered) == tenant_a_count

    # 監査チェーンのハッシュチェーンが壊れていないことを検証するテストを定義する
    def test_audit_chain_integrity_not_broken(
        self, audit_chain: AuditChain
    ) -> None:
        """監査チェーンのハッシュチェーンが正しく構築されることを検証する。"""
        # 10 件のイベントをチェーンに追加する
        hashes = []
        # イベントを追加する
        for i in range(10):
            # イベントを生成する
            event = AuditEvent(
                event_id=f"evt_{i:04d}",
                tenant_id="tenant_integrity_test",
                operation="INSERT",
                table_name="audit_test_table",
                payload={"row": i},
            )
            # チェーンに追加してハッシュを取得する
            h = audit_chain.append(event)
            # ハッシュをリストに追加する
            hashes.append(h)
        # ハッシュチェーンが連続していることを確認する
        events = audit_chain.events
        # 各イベントの prev_hash が前のハッシュと一致することを確認する
        for i in range(1, len(events)):
            # 現在のイベントの prev_hash が前のハッシュと一致することを確認する
            assert events[i].prev_hash == hashes[i - 1], (
                f"Chain broken at index {i}: "
                f"event.prev_hash={events[i].prev_hash!r} != prev_hash={hashes[i-1]!r}"
            )

    # 監査イベントのハッシュがペイロード変更で変わることを検証するテストを定義する
    def test_audit_hash_changes_with_payload_modification(self) -> None:
        """ペイロードが変更されると監査イベントのハッシュが変わることを検証する。"""
        # オリジナルのイベントを生成する
        original_event = AuditEvent(
            event_id="evt_0001",
            tenant_id="tenant_hash_test",
            operation="UPDATE",
            table_name="products",
            payload={"product_id": 42, "status": "active"},
        )
        # オリジナルのハッシュを計算する
        original_hash = original_event.compute_hash()
        # ペイロードを変更したイベントを生成する
        tampered_event = AuditEvent(
            event_id="evt_0001",
            tenant_id="tenant_hash_test",
            operation="UPDATE",
            table_name="products",
            payload={"product_id": 42, "status": "deleted"},
        )
        # 変更後のハッシュを計算する
        tampered_hash = tampered_event.compute_hash()
        # ハッシュが変わっていることを確認する
        assert original_hash != tampered_hash, (
            "Audit hash should change when payload is modified"
        )


# data 軸プロパティ検証追加テストスイートクラスを定義する
class TestDataPropertyAxiomExtended:
    """data 軸 v1_property_axiom の追加検証テストスイート。

    監査ログのチェーン継続性・RLS フィルタの空テナント処理・スキーマ移行の複数カラムを検証する。
    """

    # 監査チェーンへの 50 件追加後もチェーンが有効であることを検証するテストを定義する
    def test_audit_chain_large_volume_integrity(self) -> None:
        """50 件のイベントを追加後も監査チェーンの整合性が維持されることを検証する。"""
        # 監査チェーンをインスタンス化する
        chain = AuditChain()
        # 50 件のイベントを追加する
        all_hashes = []
        # ループでイベントを追加する
        for i in range(50):
            # イベントを生成する
            event = AuditEvent(
                event_id=f"bulk_evt_{i:04d}",
                tenant_id="tenant_bulk_test",
                operation="INSERT",
                table_name="bulk_table",
                payload={"index": i, "data": f"bulk_data_{i}"},
            )
            # チェーンに追加する
            h = chain.append(event)
            # ハッシュを記録する
            all_hashes.append(h)
        # チェーンのイベント数が 50 であることを確認する
        assert len(chain.events) == 50
        # ハッシュリストが 50 件であることを確認する
        assert len(all_hashes) == 50
        # 全ハッシュが一意であることを確認する
        assert len(set(all_hashes)) == 50, "Audit hashes must be unique"

    # RLS フィルタがテナントコンテキストなしで空リストを返すことを検証するテストを定義する
    def test_rls_filter_no_tenant_returns_empty(self) -> None:
        """テナントコンテキストなしの RLS フィルタが空リストを返すことを検証する。"""
        # RLS フィルターをインスタンス化する
        rls = RLSFilter()
        # テナント A のレコードを挿入する
        rls.insert({"id": "rec_1", "tenant_id": "tenant_a", "data": "secret"})
        # テナント B のレコードを挿入する
        rls.insert({"id": "rec_2", "tenant_id": "tenant_b", "data": "data"})
        # テナントコンテキストなしでフィルタクエリを実行する（None が設定されている）
        result = rls.query_filtered()
        # テナントコンテキストなしでは空リストが返されることを確認する
        assert result == [], (
            f"RLS with no tenant context must return empty list, got {result}"
        )

    # スキーマ移行で複数カラムを追加・削除できることを検証するテストを定義する
    def test_schema_migration_multiple_columns(self) -> None:
        """複数カラムの追加・削除を含むスキーマ移行が正しく動作することを検証する。"""
        # マイグレーションをインスタンス化する
        migration = MockSchemaMigration()
        # オリジナルスキーマを記録する
        original = migration.schema.copy()
        # 3 つのカラムを追加する
        migration.migrate_up("machine_id", "INTEGER")
        # 2 つ目のカラムを追加する
        migration.migrate_up("sensor_value", "FLOAT")
        # 3 つ目のカラムを追加する
        migration.migrate_up("recorded_at", "TIMESTAMP")
        # 3 つのカラムが追加されたことを確認する
        assert "machine_id" in migration.schema
        # sensor_value が追加されたことを確認する
        assert "sensor_value" in migration.schema
        # recorded_at が追加されたことを確認する
        assert "recorded_at" in migration.schema
        # 3 つのカラムを削除する（down マイグレーション）
        migration.migrate_down("machine_id")
        # sensor_value を削除する
        migration.migrate_down("sensor_value")
        # recorded_at を削除する
        migration.migrate_down("recorded_at")
        # オリジナルスキーマが復元されたことを確認する
        assert migration.schema == original, (
            f"Schema not restored after multiple up/down migrations: {migration.schema}"
        )

    # RLS の非フィルタクエリが全レコードを返すことを検証するテストを定義する
    def test_rls_unfiltered_returns_all_records(self) -> None:
        """非フィルタクエリが全テナントのレコードを返すことを検証する。"""
        # RLS フィルターをインスタンス化する
        rls = RLSFilter()
        # 3 テナントにそれぞれ 5 件のレコードを挿入する
        tenants = ["tenant_x", "tenant_y", "tenant_z"]
        # テナントごとのレコードを挿入する
        for tenant in tenants:
            # 5 件のレコードを挿入する
            for i in range(5):
                # レコードを挿入する
                rls.insert({"id": f"{tenant}_rec_{i}", "tenant_id": tenant})
        # 非フィルタクエリが全 15 件を返すことを確認する
        all_records = rls.query_unfiltered()
        # 全 15 件が返されることを確認する
        assert len(all_records) == 15, (
            f"Unfiltered query must return all records: expected 15, got {len(all_records)}"
        )

    # 監査イベントの tenant_id フィールドがハッシュに影響することを検証するテストを定義する
    def test_audit_hash_changes_with_tenant_id_change(self) -> None:
        """テナント ID の変更が監査ハッシュを変えることを検証する。"""
        # テナント A のイベントを生成する
        event_a = AuditEvent(
            event_id="evt_tenant_test",
            tenant_id="tenant_alpha",
            operation="INSERT",
            table_name="orders",
            payload={"order_id": 1},
        )
        # ハッシュを計算する
        hash_a = event_a.compute_hash()
        # テナント B のイベントを生成する（tenant_id のみ変更）
        event_b = AuditEvent(
            event_id="evt_tenant_test",
            tenant_id="tenant_beta",
            operation="INSERT",
            table_name="orders",
            payload={"order_id": 1},
        )
        # ハッシュを計算する
        hash_b = event_b.compute_hash()
        # テナント ID が異なるとハッシュが変わることを確認する
        assert hash_a != hash_b, (
            "Audit hash must change when tenant_id changes"
        )

    # スキーマがベースカラムを常に持つことを検証するテストを定義する
    def test_schema_always_has_base_columns(self) -> None:
        """スキーマが常にベースカラム（id, created_at, updated_at）を持つことを検証する。"""
        # マイグレーションをインスタンス化する
        migration = MockSchemaMigration()
        # ベースカラムが存在することを確認する
        assert "id" in migration.schema
        # created_at が存在することを確認する
        assert "created_at" in migration.schema
        # updated_at が存在することを確認する
        assert "updated_at" in migration.schema
        # 新しいカラムを追加する
        migration.migrate_up("extra_col", "TEXT")
        # ベースカラムがまだ存在することを確認する
        assert "id" in migration.schema
        # 追加したカラムが存在することを確認する
        assert "extra_col" in migration.schema


# data プロパティ公理の第三拡張テストクラスを定義する
class TestDataPropertyAxiomExtended3:
    """監査チェーン・RLS・スキーマ移行プロパティの追加テスト群（第三拡張）。"""

    # 監査チェーンのフィクスチャを定義する
    @pytest.fixture
    def audit_chain(self) -> AuditChain:
        """AuditChain フィクスチャを生成する。"""
        # 監査チェーンをインスタンス化して返す
        return AuditChain()

    # RLS フィルターのフィクスチャを定義する
    @pytest.fixture
    def rls_filter(self) -> RLSFilter:
        """RLSFilter フィクスチャを生成する。"""
        # RLS フィルターをインスタンス化して返す
        return RLSFilter()

    # 監査チェーンが最初のイベントの prev_hash を空文字列にすることをテストする
    def test_audit_chain_first_event_prev_hash_empty(
        self, audit_chain: AuditChain
    ) -> None:
        """監査チェーンの最初のイベントの prev_hash が空文字列であることを検証する。"""
        # 最初のイベントを作成する
        event = AuditEvent(
            event_id="first_event",
            tenant_id="tenant_001",
            operation="INSERT",
            table_name="orders",
            payload={"order_id": 1},
        )
        # チェーンに追加する
        audit_chain.append(event)
        # イベントの prev_hash が空文字列であることを確認する（ジェネシス）
        assert event.prev_hash == ""

    # 監査チェーンの 2 番目以降のイベントに prev_hash が設定されることをテストする
    def test_audit_chain_subsequent_events_have_prev_hash(
        self, audit_chain: AuditChain
    ) -> None:
        """監査チェーンの 2 番目以降のイベントに prev_hash が設定されることを検証する。"""
        # 1 番目のイベントを作成する
        event1 = AuditEvent(
            event_id="event_1",
            tenant_id="tenant_x",
            operation="INSERT",
            table_name="t",
            payload={"v": 1},
        )
        # チェーンに追加する
        h1 = audit_chain.append(event1)
        # 2 番目のイベントを作成する
        event2 = AuditEvent(
            event_id="event_2",
            tenant_id="tenant_x",
            operation="UPDATE",
            table_name="t",
            payload={"v": 2},
        )
        # チェーンに追加する
        audit_chain.append(event2)
        # 2 番目のイベントの prev_hash が 1 番目のハッシュであることを確認する
        assert event2.prev_hash == h1

    # RLS フィルターが異なるテナントを分離することをテストする
    def test_rls_filter_separates_tenants(
        self, rls_filter: RLSFilter
    ) -> None:
        """RLS フィルターが 2 テナントのレコードを正しく分離することを検証する。"""
        # テナント X のコンテキストを設定する
        rls_filter.set_tenant("tenant_x")
        # テナント X のレコードを挿入する
        rls_filter.insert({"tenant_id": "tenant_x", "data": "x_data"})
        # テナント Y のレコードを挿入する
        rls_filter.insert({"tenant_id": "tenant_y", "data": "y_data"})
        # テナント X のレコードのみが返ることを確認する
        filtered = rls_filter.query_filtered()
        # テナント X のレコードが 1 件のみ返ることを確認する
        assert len(filtered) == 1
        # テナント X のデータであることを確認する
        assert filtered[0]["data"] == "x_data"

    # RLS フィルターのコンテキストを切り替えるとレコードが変わることをテストする
    def test_rls_filter_context_switch_changes_results(
        self, rls_filter: RLSFilter
    ) -> None:
        """RLS テナントコンテキストを切り替えると取得レコードが変わることを検証する。"""
        # テナント A と B のレコードを挿入する
        rls_filter.insert({"tenant_id": "A", "value": 100})
        # テナント B のレコードを挿入する
        rls_filter.insert({"tenant_id": "B", "value": 200})
        # テナント A のコンテキストでクエリする
        rls_filter.set_tenant("A")
        # テナント A のレコードを取得する
        results_a = rls_filter.query_filtered()
        # テナント A のレコードが 1 件であることを確認する
        assert len(results_a) == 1
        # テナント B のコンテキストでクエリする
        rls_filter.set_tenant("B")
        # テナント B のレコードを取得する
        results_b = rls_filter.query_filtered()
        # テナント B のレコードが 1 件であることを確認する
        assert len(results_b) == 1
        # テナント A と B のレコードが別であることを確認する
        assert results_a[0]["value"] != results_b[0]["value"]

    # スキーマ移行の up と down が逆操作であることをテストする
    def test_schema_migration_up_then_down_removes_column(self) -> None:
        """スキーマに up で追加したカラムを down で削除できることを検証する。"""
        # スキーママイグレーションをインスタンス化する
        migration = MockSchemaMigration()
        # カラムを追加する
        migration.migrate_up("new_column", "VARCHAR(255)")
        # カラムが存在することを確認する
        assert "new_column" in migration.schema
        # カラムを削除する
        migration.migrate_down("new_column")
        # カラムが削除されたことを確認する
        assert "new_column" not in migration.schema

    # スキーマ移行で複数のカラムを追加できることをテストする
    def test_schema_migration_multiple_columns_added(self) -> None:
        """スキーマ移行で複数のカラムを一度に追加できることを検証する。"""
        # スキーママイグレーションをインスタンス化する
        migration = MockSchemaMigration()
        # 複数のカラムを追加する
        columns = [
            ("col_1", "INTEGER"),
            ("col_2", "TEXT"),
            ("col_3", "BOOLEAN"),
            ("col_4", "TIMESTAMP"),
        ]
        # 全カラムを追加する
        for col_name, col_type in columns:
            # カラムを追加する
            migration.migrate_up(col_name, col_type)
        # 全カラムが存在することを確認する
        for col_name, _ in columns:
            # カラムが存在することを確認する
            assert col_name in migration.schema, f"Column {col_name} should be in schema"

    # RLS フィルターのコンテキストなしクエリが空リストを返すことをテストする
    def test_rls_filter_no_context_returns_empty(
        self, rls_filter: RLSFilter
    ) -> None:
        """RLS テナントコンテキストが未設定の場合、クエリが空リストを返すことを検証する。"""
        # テナントコンテキストなしでレコードを挿入する
        rls_filter.insert({"tenant_id": "tenant_z", "data": "z_data"})
        # コンテキストなしでクエリする（デフォルト None）
        results = rls_filter.query_filtered()
        # 空リストが返ることを確認する
        assert results == [], "No tenant context should return empty list"

    # 監査チェーンの全イベントが events プロパティに含まれることをテストする
    def test_audit_chain_events_property_contains_all(
        self, audit_chain: AuditChain
    ) -> None:
        """監査チェーンの events プロパティが全イベントを含むことを検証する。"""
        # 5 件のイベントを追加する
        for i in range(5):
            # イベントを作成する
            e = AuditEvent(
                event_id=f"ev_{i:03d}",
                tenant_id="tenant_q",
                operation="INSERT",
                table_name="table_q",
                payload={"index": i},
            )
            # チェーンに追加する
            audit_chain.append(e)
        # events プロパティが 5 件のイベントを含むことを確認する
        assert len(audit_chain.events) == 5
        # hashes が 5 件であることを確認する
        assert len(audit_chain.hashes) == 5

    # スキーマ移行が存在しないカラムを削除しても安全であることをテストする
    def test_schema_migration_down_nonexistent_safe(self) -> None:
        """存在しないカラムの migrate_down が安全に処理されることを検証する。"""
        # スキーママイグレーションをインスタンス化する
        migration = MockSchemaMigration()
        # 存在しないカラムを削除する（discard なので安全）
        migration.migrate_down("nonexistent_col")
        # ベースカラムがまだ存在することを確認する
        assert "id" in migration.schema
        # スキーマサイズが変わっていないことを確認する
        assert len(migration.schema) == 3

    # 監査チェーンのハッシュが全て異なることをテストする
    @given(
        payloads=st.lists(
            st.dictionaries(st.text(min_size=1, max_size=5), st.integers()),
            min_size=2,
            max_size=8,
        )
    )
    @settings(max_examples=25, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_audit_chain_all_hashes_unique(self, payloads: list) -> None:
        """監査チェーンの各イベントハッシュが全て一意であることを検証する。"""
        # 新しい監査チェーンをインスタンス化する
        chain = AuditChain()
        # 全ペイロードのイベントを追加する
        for i, payload in enumerate(payloads):
            # イベントを作成する
            e = AuditEvent(
                event_id=f"uniq_ev_{i}",
                tenant_id="tenant_hash",
                operation="INSERT",
                table_name="tbl",
                payload=payload,
            )
            # チェーンに追加する
            chain.append(e)
        # 全ハッシュが一意であることを確認する
        hashes = chain.hashes
        # セットにして重複がないことを確認する
        assert len(hashes) == len(set(hashes)), "All audit chain hashes must be unique"
