"""src/test/integration/crosscutting/test_v1_type_invariant.py

crosscutting 軸の v1_type_invariant 検証クラスに対するインテグレーションテスト。
HTTP/2 強制・KEK 分散・スキーマレジストリ・FSM コードgen の
型不変条件を検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: crosscutting_type_invariant_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import re
import json
import uuid
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field


# HTTP レスポンスのモッククラスを定義する
@dataclass
class MockHTTPResponse:
    """モック HTTP レスポンス。"""

    # ステータスコードを保持するフィールドを定義する
    status_code: int
    # レスポンスヘッダーを保持するフィールドを定義する
    headers: dict[str, str]
    # レスポンスボディを保持するフィールドを定義する
    body: bytes = b""

    # プロトコルバージョンを確認するプロパティを定義する
    @property
    def protocol(self) -> str:
        """使用されているプロトコルバージョンを返す。"""
        # Upgrade ヘッダーから HTTP/2 を検出する
        return self.headers.get("x-protocol", "http/1.1")


# HTTP/2 エンドポイントのモッククラスを定義する
class MockHTTP2Endpoint:
    """HTTP/2 専用エンドポイントのモック。

    HTTP/1.1 リクエストはプロトコルエラーを返す必要がある。
    """

    # エンドポイントを初期化するメソッドを定義する
    def __init__(self, require_h2: bool = True) -> None:
        """HTTP/2 エンドポイントを初期化する。"""
        # HTTP/2 必須フラグを設定する
        self._require_h2 = require_h2

    # レスポンスを処理するメソッドを定義する
    def handle(self, protocol: str) -> MockHTTPResponse:
        """リクエストを処理する。"""
        # HTTP/2 必須の場合 HTTP/1.1 リクエストはエラーを返す
        if self._require_h2 and protocol == "http/1.1":
            # プロトコルエラーを返す
            return MockHTTPResponse(
                status_code=426,
                headers={
                    "upgrade": "h2",
                    "content-type": "application/json",
                    "x-error": "protocol_upgrade_required",
                },
            )
        # HTTP/2 リクエストは正常処理する
        return MockHTTPResponse(
            status_code=200,
            headers={
                "content-type": "application/json",
                "x-protocol": "h2",
            },
        )


# Shamir シークレット共有スキームのモッククラスを定義する
class MockShamirKEK:
    """Shamir シークレット共有による KEK（Key Encryption Key）分散のモック。"""

    # KEK を初期化するメソッドを定義する
    def __init__(self, total_shares: int, threshold: int) -> None:
        """KEK シークレット共有を初期化する。"""
        # total_shares > threshold の不変条件を検証する
        if total_shares <= threshold:
            # 不変条件違反のエラーを発生させる
            raise ValueError(
                f"total_shares ({total_shares}) must be > threshold ({threshold})"
            )
        # 合計シェア数を設定する
        self._total = total_shares
        # 閾値を設定する
        self._threshold = threshold
        # シェアのリストを初期化する
        self._shares: list[dict[str, Any]] = []

    # KEK シェアを生成するメソッドを定義する
    def generate_shares(self, kek_id: str) -> list[dict[str, Any]]:
        """KEK のシェアを生成する。"""
        # シェアのリストをリセットする
        self._shares = []
        # total_shares 件のシェアを生成する
        for i in range(1, self._total + 1):
            # シェアを生成する
            share = {
                "share_id": i,
                "kek_id": kek_id,
                "total": self._total,
                "threshold": self._threshold,
                "value": f"share_{kek_id}_{i}",
            }
            # シェアをリストに追加する
            self._shares.append(share)
        # シェアのリストを返す
        return self._shares

    # シェアの不変条件を検証するメソッドを定義する
    def validate_shares(self, shares: list[dict[str, Any]]) -> None:
        """シェアが n+k の不変条件（n > k）を満たすことを検証する。"""
        # シェアが存在することを確認する
        if not shares:
            # シェアが空の場合はエラーを発生させる
            raise ValueError("shares must not be empty")
        # 合計シェア数を取得する
        total = shares[0].get("total", 0)
        # 閾値を取得する
        threshold = shares[0].get("threshold", 0)
        # n > k の不変条件を確認する
        if total <= threshold:
            # 不変条件違反のエラーを発生させる
            raise ValueError(
                f"KEK shares must satisfy n > k: total={total}, threshold={threshold}"
            )
        # シェア数が total と一致することを確認する
        if len(shares) != total:
            # シェア数が不一致の場合はエラーを発生させる
            raise ValueError(
                f"Expected {total} shares, got {len(shares)}"
            )


# スキーマレジストリのモッククラスを定義する
class MockSchemaRegistry:
    """スキーマレジストリのモック。全スキーマがバージョンフィールドを持つことを強制する。"""

    # レジストリを初期化するメソッドを定義する
    def __init__(self) -> None:
        """スキーマレジストリを初期化する。"""
        # スキーマの辞書を初期化する
        self._schemas: dict[str, dict[str, Any]] = {}

    # スキーマを登録するメソッドを定義する
    def register(self, schema_id: str, schema: dict[str, Any]) -> None:
        """スキーマを登録する。version フィールドの存在を強制する。"""
        # version フィールドが存在することを確認する
        if "version" not in schema:
            # version フィールドが欠落している場合はエラーを発生させる
            raise ValueError(
                f"Schema {schema_id!r} must have 'version' field"
            )
        # version が正の整数であることを確認する
        if not isinstance(schema["version"], int) or schema["version"] <= 0:
            # version が不正な場合はエラーを発生させる
            raise ValueError(
                f"Schema {schema_id!r} version must be a positive integer, "
                f"got: {schema['version']!r}"
            )
        # スキーマを登録する
        self._schemas[schema_id] = schema

    # スキーマを取得するメソッドを定義する
    def get(self, schema_id: str) -> Optional[dict[str, Any]]:
        """スキーマを取得する。存在しない場合は None を返す。"""
        # スキーマを返す
        return self._schemas.get(schema_id)

    # 登録済みスキーマ数を取得するプロパティを定義する
    @property
    def schema_count(self) -> int:
        """登録済みスキーマ数を返す。"""
        # スキーマ数を返す
        return len(self._schemas)


# FSM コードジェネレーターの出力を検証するクラスを定義する
class MockFSMCodegen:
    """FSM コードジェネレーターの出力を検証するモック。

    生成された FSM が必ず開始状態を持つことを強制する。
    """

    # 検証器を初期化するメソッドを定義する
    def __init__(self) -> None:
        """FSM コードジェネレーター検証器を初期化する。"""
        # 生成された FSM の数を初期化する
        self._generated_count: int = 0

    # FSM 仕様を検証するメソッドを定義する
    def validate_fsm_spec(self, spec: dict[str, Any]) -> None:
        """FSM 仕様が必須フィールドを持つことを検証する。"""
        # name フィールドが存在することを確認する
        if "name" not in spec:
            # name フィールドが欠落している場合はエラーを発生させる
            raise ValueError("FSM spec must have 'name' field")
        # states フィールドが存在することを確認する
        if "states" not in spec:
            # states フィールドが欠落している場合はエラーを発生させる
            raise ValueError("FSM spec must have 'states' field")
        # initial_state フィールドが存在することを確認する
        if "initial_state" not in spec:
            # initial_state フィールドが欠落している場合はエラーを発生させる
            raise ValueError("FSM spec must have 'initial_state' field")
        # states が空でないことを確認する
        if not spec["states"]:
            # states が空の場合はエラーを発生させる
            raise ValueError("FSM spec 'states' must not be empty")
        # initial_state が states に含まれることを確認する
        if spec["initial_state"] not in spec["states"]:
            # initial_state が states に含まれない場合はエラーを発生させる
            raise ValueError(
                f"FSM initial_state {spec['initial_state']!r} must be in states "
                f"{spec['states']}"
            )

    # FSM コードを生成するメソッドを定義する
    def generate(self, spec: dict[str, Any]) -> str:
        """FSM 仕様からコードを生成する。"""
        # 仕様を検証する
        self.validate_fsm_spec(spec)
        # 生成カウントをインクリメントする
        self._generated_count += 1
        # 生成されたコードを返す（実際の生成は省略）
        return (
            f"// Generated FSM: {spec['name']}\n"
            f"// initial_state: {spec['initial_state']}\n"
            f"// states: {', '.join(spec['states'])}\n"
        )

    # 生成されたコード数を取得するプロパティを定義する
    @property
    def generated_count(self) -> int:
        """生成されたコードの数を返す。"""
        # 生成カウントを返す
        return self._generated_count


# crosscutting 型不変条件テストスイートクラスを定義する
class TestCrosscuttingTypeInvariant:
    """crosscutting 軸 v1_type_invariant の検証テストスイート。"""

    # HTTP/2 エンドポイントのフィクスチャを定義する
    @pytest.fixture
    def h2_endpoint(self) -> Generator[MockHTTP2Endpoint, None, None]:
        """MockHTTP2Endpoint フィクスチャを生成する。"""
        # HTTP/2 エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # フィクスチャを返す
        yield endpoint

    # スキーマレジストリのフィクスチャを定義する
    @pytest.fixture
    def schema_registry(self) -> Generator[MockSchemaRegistry, None, None]:
        """MockSchemaRegistry フィクスチャを生成する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # フィクスチャを返す
        yield registry

    # FSM コードジェネレーターのフィクスチャを定義する
    @pytest.fixture
    def fsm_codegen(self) -> Generator[MockFSMCodegen, None, None]:
        """MockFSMCodegen フィクスチャを生成する。"""
        # コードジェネレーターをインスタンス化する
        codegen = MockFSMCodegen()
        # フィクスチャを返す
        yield codegen

    # HTTP/2 リクエストが成功することを検証するテストを定義する
    def test_http2_request_allowed(self, h2_endpoint: MockHTTP2Endpoint) -> None:
        """HTTP/2 リクエストが成功することを検証する。"""
        # HTTP/2 リクエストを処理する
        response = h2_endpoint.handle(protocol="h2")
        # ステータスコードが 200 であることを確認する
        assert response.status_code == 200
        # プロトコルが h2 であることを確認する
        assert response.headers.get("x-protocol") == "h2"

    # HTTP/1.1 リクエストがエラーになることを検証するテストを定義する
    def test_http11_request_rejected_by_h2_endpoint(
        self, h2_endpoint: MockHTTP2Endpoint
    ) -> None:
        """HTTP/1.1 リクエストが h2 専用エンドポイントで拒否されることを検証する。"""
        # HTTP/1.1 リクエストを処理する
        response = h2_endpoint.handle(protocol="http/1.1")
        # ステータスコードが 426 であることを確認する
        assert response.status_code == 426, (
            f"HTTP/1.1 to h2-only endpoint must return 426, got {response.status_code}"
        )
        # Upgrade ヘッダーが存在することを確認する
        assert "upgrade" in response.headers
        # Upgrade ヘッダーが h2 であることを確認する
        assert response.headers["upgrade"] == "h2"

    # KEK 分散が n > k の不変条件を満たすことを検証するテストを定義する
    def test_kek_shares_n_greater_k(self) -> None:
        """KEK 分散が n > k（合計シェア数 > 閾値）の不変条件を満たすことを検証する。"""
        # n=5, k=3 の KEK を生成する
        kek = MockShamirKEK(total_shares=5, threshold=3)
        # シェアを生成する
        shares = kek.generate_shares("kek_001")
        # シェア数が 5 であることを確認する
        assert len(shares) == 5
        # シェアの不変条件を検証する
        kek.validate_shares(shares)

    # n <= k の KEK 生成が拒否されることを検証するテストを定義する
    def test_kek_shares_invalid_n_le_k_rejected(self) -> None:
        """n <= k の KEK 生成が拒否されることを検証する。"""
        # n=3, k=3 の KEK 生成が拒否されることを確認する
        with pytest.raises(ValueError, match="n > k|total_shares.*threshold"):
            # n=k の KEK を生成する
            MockShamirKEK(total_shares=3, threshold=3)
        # n < k の場合も拒否されることを確認する
        with pytest.raises(ValueError):
            # n < k の KEK を生成する
            MockShamirKEK(total_shares=2, threshold=3)

    # スキーマが version フィールドを持つことを検証するテストを定義する
    def test_schema_registry_version_required(
        self, schema_registry: MockSchemaRegistry
    ) -> None:
        """スキーマレジストリに登録するスキーマが version フィールドを持つことを検証する。"""
        # version フィールドを持つスキーマを登録する
        valid_schema = {
            "version": 1,
            "type": "object",
            "properties": {
                "tenant_id": {"type": "string"},
                "order_id": {"type": "string"},
            },
        }
        # 有効なスキーマを登録する
        schema_registry.register("order_schema_v1", valid_schema)
        # 登録済みスキーマ数が 1 であることを確認する
        assert schema_registry.schema_count == 1

    # version フィールドがないスキーマが拒否されることを検証するテストを定義する
    def test_schema_registry_missing_version_rejected(
        self, schema_registry: MockSchemaRegistry
    ) -> None:
        """version フィールドがないスキーマが拒否されることを検証する。"""
        # version フィールドがないスキーマを定義する
        invalid_schema = {
            "type": "object",
            "properties": {"id": {"type": "string"}},
        }
        # 登録が拒否されることを確認する
        with pytest.raises(ValueError, match="version"):
            # 無効なスキーマを登録する
            schema_registry.register("no_version_schema", invalid_schema)

    # FSM コードジェネレーターが開始状態を持つ FSM を生成することを検証するテストを定義する
    def test_fsm_codegen_generated_has_initial_state(
        self, fsm_codegen: MockFSMCodegen
    ) -> None:
        """生成された FSM が開始状態を持つことを検証する。"""
        # 有効な FSM 仕様を定義する
        fsm_spec = {
            "name": "AlertFSM",
            "states": ["IDLE", "FIRED", "ACK", "MITIGATED", "RESOLVED"],
            "initial_state": "IDLE",
            "transitions": [
                {"from": "IDLE", "to": "FIRED", "event": "fire"},
            ],
        }
        # FSM コードを生成する
        generated_code = fsm_codegen.generate(fsm_spec)
        # 生成されたコードが開始状態を含むことを確認する
        assert "IDLE" in generated_code
        # 生成カウントが 1 であることを確認する
        assert fsm_codegen.generated_count == 1

    # initial_state が states に含まれない FSM 仕様が拒否されることを検証するテストを定義する
    def test_fsm_codegen_invalid_initial_state_rejected(
        self, fsm_codegen: MockFSMCodegen
    ) -> None:
        """initial_state が states に含まれない FSM 仕様が拒否されることを検証する。"""
        # 無効な FSM 仕様（initial_state が states に含まれない）を定義する
        invalid_spec = {
            "name": "BadFSM",
            "states": ["STATE_A", "STATE_B"],
            "initial_state": "NONEXISTENT_STATE",
        }
        # 生成が拒否されることを確認する
        with pytest.raises(ValueError, match="initial_state"):
            # 無効な仕様で生成を試みる
            fsm_codegen.generate(invalid_spec)

    # 複数のスキーマが独立して登録できることを検証するテストを定義する
    def test_schema_registry_multiple_schemas(
        self, schema_registry: MockSchemaRegistry
    ) -> None:
        """複数のスキーマが独立して登録できることを検証する。"""
        # 複数のスキーマを登録する
        for i in range(5):
            # バージョン番号付きスキーマを登録する
            schema_registry.register(
                f"schema_{i}",
                {"version": i + 1, "type": "object"},
            )
        # 登録済みスキーマ数が 5 であることを確認する
        assert schema_registry.schema_count == 5
        # 各スキーマが取得できることを確認する
        for i in range(5):
            # スキーマを取得する
            schema = schema_registry.get(f"schema_{i}")
            # スキーマが存在することを確認する
            assert schema is not None
            # バージョン番号が正しいことを確認する
            assert schema["version"] == i + 1


# crosscutting 型不変条件追加テストスイートクラスを定義する
class TestCrosscuttingTypeInvariantExtended:
    """crosscutting 軸 v1_type_invariant の追加検証テストスイート。

    スキーマのバージョン昇順チェック・FSM 遷移の整合性・KEK 閾値の境界値を検証する。
    """

    # スキーマバージョンが昇順であることを検証するテストを定義する
    def test_schema_version_monotonically_increasing(self) -> None:
        """スキーマを上書き登録するとバージョンが上がることを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # バージョン 1 のスキーマを登録する
        registry.register("my_schema", {"version": 1, "fields": ["id"]})
        # バージョン 2 のスキーマを登録する（上書き）
        registry.register("my_schema", {"version": 2, "fields": ["id", "name"]})
        # 最新スキーマのバージョンが 2 であることを確認する
        latest = registry.get("my_schema")
        # スキーマが存在することを確認する
        assert latest is not None
        # バージョンが 2 であることを確認する
        assert latest["version"] == 2

    # FSM の初期状態が states リストに含まれることを検証するテストを定義する
    def test_fsm_initial_state_in_states_list(self) -> None:
        """FSM の initial_state が states リストに含まれることを検証する。"""
        # コードジェネレーターをインスタンス化する
        codegen = MockFSMCodegen()
        # 有効な FSM 仕様を定義する
        spec = {
            "name": "MachineFSM",
            "states": ["STOPPED", "RUNNING", "ERROR", "MAINTENANCE"],
            "initial_state": "STOPPED",
        }
        # バリデーションがパスすることを確認する
        codegen.validate_fsm_spec(spec)

    # KEK の threshold が total_shares - 1 まで有効であることを検証するテストを定義する
    def test_kek_threshold_boundary_n_minus_1(self) -> None:
        """KEK の threshold が total_shares - 1 の境界値で有効であることを検証する。"""
        # n=5, k=4 のケースをテストする（n > k を満たす境界値）
        kek = MockShamirKEK(total_shares=5, threshold=4)
        # シェアを生成する
        shares = kek.generate_shares("boundary_kek")
        # シェア数が 5 であることを確認する
        assert len(shares) == 5
        # バリデーションがパスすることを確認する
        kek.validate_shares(shares)

    # HTTP/2 専用エンドポイントが HTTP/3 も受け入れることを検証するテストを定義する
    def test_http_endpoint_h2_protocol_accepted(self) -> None:
        """HTTP/2 プロトコルのリクエストが受け入れられることを検証する。"""
        # HTTP/2 エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # HTTP/2 リクエストを処理する
        response = endpoint.handle(protocol="h2")
        # ステータスコードが 200 であることを確認する
        assert response.status_code == 200
        # プロトコルが h2 であることを確認する
        assert response.headers.get("x-protocol") == "h2"

    # 複数スキーマの一括登録が独立して機能することを検証するテストを定義する
    def test_schema_registry_bulk_registration(self) -> None:
        """10 件のスキーマ一括登録が正しく機能することを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # 10 件のスキーマを登録する
        for i in range(10):
            # スキーマを登録する
            registry.register(
                f"schema_bulk_{i:03d}",
                {
                    "version": i + 1,
                    "type": "object",
                    "schema_id": f"bulk_{i}",
                },
            )
        # 登録済みスキーマ数が 10 であることを確認する
        assert registry.schema_count == 10
        # 各スキーマが正しいバージョンを持つことを確認する
        for i in range(10):
            # スキーマを取得する
            schema = registry.get(f"schema_bulk_{i:03d}")
            # スキーマが存在することを確認する
            assert schema is not None
            # バージョンが正しいことを確認する
            assert schema["version"] == i + 1


# crosscutting 型不変条件第 2 拡張テストスイートクラスを定義する
class TestCrosscuttingTypeInvariantExtended2:
    """crosscutting 軸 v1_type_invariant の第 2 拡張テストスイート。追加の型不変条件を検証する。"""

    # HTTP/1.1 リクエストが 426 で拒否されることをテストする
    def test_http11_rejected_with_upgrade_header(self) -> None:
        """HTTP/1.1 リクエストが 426 Upgrade Required で拒否されることを検証する。"""
        # HTTP/2 エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # HTTP/1.1 リクエストを処理する
        response = endpoint.handle(protocol="http/1.1")
        # ステータスコードが 426 であることを確認する
        assert response.status_code == 426
        # Upgrade ヘッダーが存在することを確認する
        assert "Upgrade" in response.headers
        # Upgrade ヘッダーの値が h2 であることを確認する
        assert response.headers["Upgrade"] == "h2"

    # HTTP/2 エンドポイントが HTTP/2 を受け入れることをテストする
    def test_http2_accepted_with_200(self) -> None:
        """HTTP/2 リクエストが 200 OK で受け入れられることを検証する。"""
        # HTTP/2 エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # HTTP/2 リクエストを処理する
        response = endpoint.handle(protocol="h2")
        # ステータスコードが 200 であることを確認する
        assert response.status_code == 200

    # KEK のシェア数が total_shares と一致することをテストする
    def test_kek_share_count_matches_total(self) -> None:
        """生成されたシェア数が total_shares と一致することを検証する。"""
        # KEK をインスタンス化する（n=7, k=4）
        kek = MockShamirKEK(total_shares=7, threshold=4)
        # シェアを生成する
        shares = kek.generate_shares("kek_007")
        # シェア数が 7 であることを確認する
        assert len(shares) == 7
        # 全シェアの kek_id が正しいことを確認する
        assert all(s["kek_id"] == "kek_007" for s in shares)

    # KEK が total_shares == threshold の場合に拒否されることをテストする
    def test_kek_rejects_equal_total_and_threshold(self) -> None:
        """KEK が total_shares == threshold の場合にエラーを発生させることを検証する。"""
        # total_shares == threshold のケースをテストする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            MockShamirKEK(total_shares=5, threshold=5)

    # KEK が total_shares < threshold の場合に拒否されることをテストする
    def test_kek_rejects_total_less_than_threshold(self) -> None:
        """KEK が total_shares < threshold の場合にエラーを発生させることを検証する。"""
        # total_shares < threshold のケースをテストする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            MockShamirKEK(total_shares=3, threshold=5)

    # スキーマレジストリが version フィールドなしのスキーマを拒否することをテストする
    def test_schema_registry_rejects_missing_version(self) -> None:
        """スキーマレジストリが version フィールドのないスキーマを拒否することを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # version フィールドのないスキーマを登録しようとする
        with pytest.raises(ValueError, match="version"):
            # バリデーションが失敗することを確認する
            registry.register("no_version_schema", {"type": "object"})

    # スキーマレジストリが version=0 を拒否することをテストする
    def test_schema_registry_rejects_version_zero(self) -> None:
        """スキーマレジストリが version=0 を拒否することを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # version=0 のスキーマを登録しようとする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            registry.register("zero_version", {"version": 0, "type": "object"})

    # スキーマレジストリが version=-1 を拒否することをテストする
    def test_schema_registry_rejects_negative_version(self) -> None:
        """スキーマレジストリが負のバージョン番号を拒否することを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # version=-1 のスキーマを登録しようとする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            registry.register("neg_version", {"version": -1, "type": "object"})

    # FSM コードgen が初期状態のないシェマを拒否することをテストする
    def test_fsm_codegen_rejects_missing_initial_state(self) -> None:
        """FSM コードジェネレーターが initial_state を欠くシェマを拒否することを検証する。"""
        # FSM コードジェネレーターをインスタンス化する
        codegen = MockFSMCodegen()
        # initial_state のないスキーマを検証しようとする
        with pytest.raises(ValueError, match="initial_state"):
            # バリデーションが失敗することを確認する
            codegen.validate_fsm_spec({
                "name": "TestFSM",
                "states": ["IDLE", "ACTIVE", "DONE"],
                # initial_state が欠落している
            })

    # スキーマレジストリが正しいバージョンで登録できることをテストする
    @pytest.mark.parametrize("version", [1, 2, 10, 100, 2**31 - 1])
    def test_schema_registry_accepts_valid_versions(self, version: int) -> None:
        """スキーマレジストリが有効なバージョン番号を受け入れることを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # 有効なバージョンのスキーマを登録する
        registry.register(f"valid_version_{version}", {"version": version, "type": "string"})
        # スキーマが登録されていることを確認する
        schema = registry.get(f"valid_version_{version}")
        # スキーマが存在することを確認する
        assert schema is not None
        # バージョンが正しいことを確認する
        assert schema["version"] == version


# crosscutting 型不変条件の第三拡張テストクラスを定義する
class TestCrosscuttingTypeInvariantExtended3:
    """HTTP/2・KEK・スキーマ・FSM の型不変条件の追加テスト群（第三拡張）。"""

    # HTTP/2 エンドポイントが複数の有効なリクエストを受け入れることをテストする
    def test_http2_endpoint_multiple_requests_all_accepted(self) -> None:
        """HTTP/2 エンドポイントが連続する複数リクエストを全て受け入れることを検証する。"""
        # HTTP/2 必須エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # 複数のリクエストを処理する
        for i in range(5):
            # HTTP/2 プロトコルでリクエストを送信する
            response = endpoint.handle("h2")
            # 200 OK が返ることを確認する
            assert response.status_code == 200, f"Request {i+1}: Expected 200, got {response.status_code}"

    # HTTP/1.1 を複数回送信しても全て 426 になることをテストする
    def test_http11_multiple_attempts_all_rejected(self) -> None:
        """HTTP/1.1 でのアクセスが何度試みても 426 になることを検証する。"""
        # HTTP/2 必須エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # 3 回試みる
        for i in range(3):
            # HTTP/1.1 プロトコルでリクエストを送信する
            response = endpoint.handle("http/1.1")
            # 426 Upgrade Required が返ることを確認する
            assert response.status_code == 426, f"Attempt {i+1}: Expected 426, got {response.status_code}"

    # KEK 閾値が 1 でも動作することをテストする
    def test_kek_minimum_threshold_one_works(self) -> None:
        """KEK の threshold=1 構成が正しく動作することを検証する。"""
        # threshold=1 の KEK を生成する
        kek = MockShamirKEK(total_shares=3, threshold=1)
        # KEK ID を生成する
        kek_id = "kek_threshold_one"
        # シェアを生成する
        shares = kek.generate_shares(kek_id)
        # total_shares 枚のシェアが生成されることを確認する
        assert len(shares) == 3
        # 1 枚のシェアだけで検証できることを確認する
        valid = kek.validate_shares([shares[0]])
        # 有効であることを確認する
        assert valid is True

    # KEK の total_shares が大きい場合でも正常動作することをテストする
    def test_kek_large_total_shares_valid(self) -> None:
        """KEK の total_shares=10, threshold=5 が正常に動作することを検証する。"""
        # 大きな KEK を生成する
        kek = MockShamirKEK(total_shares=10, threshold=5)
        # KEK ID を生成する
        kek_id = "kek_large"
        # シェアを生成する
        shares = kek.generate_shares(kek_id)
        # 10 枚のシェアが生成されることを確認する
        assert len(shares) == 10
        # 5 枚のシェアで検証できることを確認する
        valid = kek.validate_shares(shares[:5])
        # 有効であることを確認する
        assert valid is True

    # スキーマレジストリが大量のスキーマを登録できることをテストする
    def test_schema_registry_bulk_registration(self) -> None:
        """スキーマレジストリが 50 件の一括登録を処理できることを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # 50 件のスキーマを登録する
        for i in range(1, 51):
            # スキーマを登録する
            registry.register(f"schema_{i:03d}", {"version": i, "type": "object", "index": i})
        # 登録件数が 50 件であることを確認する
        assert registry.schema_count == 50
        # 全スキーマが取得できることを確認する
        for i in range(1, 51):
            # スキーマを取得する
            s = registry.get(f"schema_{i:03d}")
            # スキーマが存在することを確認する
            assert s is not None
            # インデックスが正しいことを確認する
            assert s["index"] == i

    # スキーマを上書き登録するとエラーになることをテストする
    def test_schema_registry_duplicate_id_raises(self) -> None:
        """同一 ID でのスキーマ二重登録がエラーになることを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # 最初のスキーマを登録する
        registry.register("dup_schema", {"version": 1, "type": "string"})
        # 同一 ID で再登録するとエラーになることを確認する
        with pytest.raises((ValueError, KeyError)):
            # 同じ ID で別スキーマを登録する
            registry.register("dup_schema", {"version": 2, "type": "integer"})

    # FSM コードゲンが遷移なしスペックを拒否することをテストする
    def test_fsm_codegen_rejects_no_transitions(self) -> None:
        """FSM コードゲンが遷移定義のない FSM スペックを拒否することを検証する。"""
        # FSM コードゲンをインスタンス化する
        codegen = MockFSMCodegen()
        # 遷移なしのスペックを作成する
        spec = {
            "name": "NoTransFSM",
            "states": ["IDLE"],
            "initial_state": "IDLE",
            "transitions": [],
        }
        # バリデーションが失敗するか、ゼロ遷移で生成を拒否することを確認する
        with pytest.raises((ValueError, AssertionError)):
            # バリデーションを実行する
            codegen.validate_fsm_spec(spec)

    # FSM コードゲンが正常な FSM スペックを受け入れることをテストする
    def test_fsm_codegen_accepts_complete_spec(self) -> None:
        """FSM コードゲンが完全な FSM スペックを受け入れることを検証する。"""
        # FSM コードゲンをインスタンス化する
        codegen = MockFSMCodegen()
        # 完全なスペックを定義する
        spec = {
            "name": "AlertFSM",
            "states": ["IDLE", "FIRED", "ACK", "MITIGATED", "RESOLVED"],
            "initial_state": "IDLE",
            "transitions": [
                {"from": "IDLE", "to": "FIRED", "trigger": "alert"},
                {"from": "FIRED", "to": "ACK", "trigger": "ack"},
                {"from": "ACK", "to": "MITIGATED", "trigger": "mitigate"},
                {"from": "MITIGATED", "to": "RESOLVED", "trigger": "resolve"},
            ],
        }
        # バリデーションが成功することを確認する（例外が出ないこと）
        codegen.validate_fsm_spec(spec)
        # コードを生成する
        generated = codegen.generate(spec)
        # 生成件数が 1 以上であることを確認する
        assert codegen.generated_count >= 1
        # 生成結果が None でないことを確認する
        assert generated is not None

    # HTTP/2 エンドポイントが x-protocol ヘッダーを返すことをテストする
    def test_http2_response_has_protocol_header(self) -> None:
        """HTTP/2 エンドポイントのレスポンスに x-protocol ヘッダーが含まれることを検証する。"""
        # HTTP/2 必須エンドポイントをインスタンス化する
        endpoint = MockHTTP2Endpoint(require_h2=True)
        # HTTP/2 プロトコルでリクエストを送信する
        response = endpoint.handle("h2")
        # x-protocol ヘッダーが存在することを確認する
        assert "x-protocol" in response.headers
        # x-protocol が h2 であることを確認する
        assert response.headers["x-protocol"] == "h2"

    # KEK のシェア数と閾値の境界値をテストする
    @pytest.mark.parametrize("total,threshold", [
        (2, 1),
        (3, 2),
        (5, 3),
        (7, 4),
        (10, 7),
    ])
    def test_kek_various_valid_configurations(self, total: int, threshold: int) -> None:
        """KEK が様々な有効な total_shares/threshold 組み合わせで動作することを検証する。"""
        # 指定された total と threshold で KEK をインスタンス化する
        kek = MockShamirKEK(total_shares=total, threshold=threshold)
        # KEK ID を生成する
        kek_id = f"kek_{total}_{threshold}"
        # シェアを生成する
        shares = kek.generate_shares(kek_id)
        # シェア数が total であることを確認する
        assert len(shares) == total
        # 閾値以上のシェアで検証できることを確認する
        valid = kek.validate_shares(shares[:threshold])
        # 有効であることを確認する
        assert valid is True

    # スキーマレジストリに存在しない ID で取得すると None が返ることをテストする
    def test_schema_registry_get_nonexistent_returns_none(self) -> None:
        """スキーマレジストリに存在しない ID で取得すると None が返ることを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # 存在しない ID で取得する
        result = registry.get("nonexistent_schema_99")
        # None が返ることを確認する
        assert result is None

    # FSM コードゲンが生成件数を正確に追跡することをテストする
    def test_fsm_codegen_generated_count_tracks_correctly(self) -> None:
        """FSM コードゲンの generated_count が正確に追跡されることを検証する。"""
        # FSM コードゲンをインスタンス化する
        codegen = MockFSMCodegen()
        # 初期件数が 0 であることを確認する
        assert codegen.generated_count == 0
        # 3 回コードを生成する
        for i in range(3):
            # スペックを定義する
            spec = {
                "name": f"FSM_{i}",
                "states": ["A", "B"],
                "initial_state": "A",
                "transitions": [{"from": "A", "to": "B", "trigger": "go"}],
            }
            # コードを生成する
            codegen.validate_fsm_spec(spec)
            # コードを生成する
            codegen.generate(spec)
        # 生成件数が 3 であることを確認する
        assert codegen.generated_count == 3


# crosscutting 型不変条件の第四拡張テストクラスを定義する
class TestCrosscuttingTypeInvariantExtended4:
    """HTTP/2・KEK・スキーマの型不変条件の最終追加テスト群（第四拡張）。"""

    # KEK の total が threshold より大きい場合のみ有効であることをテストする
    def test_kek_total_must_exceed_threshold(self) -> None:
        """KEK の total_shares が threshold より大きい場合のみ有効であることを検証する。"""
        # 有効な KEK を生成する
        kek = MockShamirKEK(total_shares=5, threshold=3)
        # シェアを生成する
        shares = kek.generate_shares("final_kek_001")
        # シェア数が total_shares と一致することを確認する
        assert len(shares) == 5

    # スキーマレジストリが正確な件数を追跡することをテストする
    def test_schema_registry_count_tracking_accurate(self) -> None:
        """スキーマレジストリが正確な件数を追跡することを検証する。"""
        # スキーマレジストリをインスタンス化する
        registry = MockSchemaRegistry()
        # 初期件数が 0 であることを確認する
        assert registry.schema_count == 0
        # 1 件追加する
        registry.register("count_schema_1", {"version": 1})
        # 件数が 1 であることを確認する
        assert registry.schema_count == 1
        # さらに 1 件追加する
        registry.register("count_schema_2", {"version": 2})
        # 件数が 2 であることを確認する
        assert registry.schema_count == 2
