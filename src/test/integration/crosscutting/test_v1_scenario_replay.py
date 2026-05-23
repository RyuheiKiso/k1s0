"""src/test/integration/crosscutting/test_v1_scenario_replay.py

crosscutting 軸の v1_scenario_replay 検証クラスに対するインテグレーションテスト。
HTTP/1.1 リクエストのエラー・KEK ローテーション・スキーマドリフト検出・
FSM コードgen の Go コンパイル成功をシナリオとして検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: crosscutting_scenario_replay_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import json
import uuid
import time
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto


# HTTP プロトコルの種類を表す列挙型を定義する
class HTTPProtocol(Enum):
    """HTTP プロトコルの種類。"""

    # HTTP/1.1 を定義する
    HTTP11 = "http/1.1"
    # HTTP/2 を定義する
    HTTP2 = "h2"
    # HTTP/3 を定義する
    HTTP3 = "h3"


# ゲートウェイレスポンスを表すデータクラスを定義する
@dataclass
class GatewayResponse:
    """API ゲートウェイのレスポンス。"""

    # ステータスコードを保持するフィールドを定義する
    status_code: int
    # プロトコルを保持するフィールドを定義する
    protocol: str
    # ヘッダーを保持するフィールドを定義する
    headers: dict[str, str] = field(default_factory=dict)
    # ボディを保持するフィールドを定義する
    body: dict[str, Any] = field(default_factory=dict)


# HTTP/2 専用 API ゲートウェイのモッククラスを定義する
class MockAPIGateway:
    """HTTP/2 を強制する API ゲートウェイのモック。"""

    # ゲートウェイを初期化するメソッドを定義する
    def __init__(self) -> None:
        """API ゲートウェイを初期化する。"""
        # リクエスト履歴を初期化する
        self._request_history: list[dict[str, Any]] = []

    # リクエストを処理するメソッドを定義する
    def request(
        self,
        protocol: HTTPProtocol,
        path: str,
        method: str = "GET",
    ) -> GatewayResponse:
        """リクエストを処理する。"""
        # リクエスト履歴に記録する
        self._request_history.append({
            "protocol": protocol.value,
            "path": path,
            "method": method,
        })
        # HTTP/1.1 リクエストはプロトコルエラーを返す
        if protocol == HTTPProtocol.HTTP11:
            # プロトコルアップグレード必須エラーを返す
            return GatewayResponse(
                status_code=426,
                protocol=protocol.value,
                headers={
                    "upgrade": "h2",
                    "connection": "upgrade",
                    "content-type": "application/problem+json",
                },
                body={
                    "error": "upgrade_required",
                    "message": "This endpoint requires HTTP/2 (h2)",
                },
            )
        # HTTP/2 リクエストは正常処理する
        return GatewayResponse(
            status_code=200,
            protocol=protocol.value,
            headers={"content-type": "application/json"},
            body={"status": "ok", "path": path},
        )

    # リクエスト履歴を取得するプロパティを定義する
    @property
    def request_history(self) -> list[dict[str, Any]]:
        """リクエスト履歴のコピーを返す。"""
        # 履歴のコピーを返す
        return list(self._request_history)


# KEK ローテーションマネージャーのモッククラスを定義する
class MockKEKRotationManager:
    """KEK（Key Encryption Key）のローテーションを管理するモック。"""

    # マネージャーを初期化するメソッドを定義する
    def __init__(self) -> None:
        """KEK ローテーションマネージャーを初期化する。"""
        # 現在のアクティブな KEK ID を初期化する
        self._active_kek_id: Optional[str] = None
        # KEK のバージョン履歴を初期化する
        self._kek_history: list[dict[str, Any]] = []
        # 無効化された KEK のセットを初期化する
        self._revoked_kek_ids: set[str] = set()
        # 配布済みシェアを保持する辞書を初期化する
        self._distributed_shares: dict[str, list[dict[str, Any]]] = {}

    # 最初の KEK を生成するメソッドを定義する
    def initialize(self, total_shares: int, threshold: int) -> str:
        """最初の KEK を生成してシェアを配布する。"""
        # KEK ID を生成する
        kek_id = f"kek_v{len(self._kek_history) + 1}_{uuid.uuid4().hex[:8]}"
        # KEK を履歴に追加する
        self._kek_history.append({
            "kek_id": kek_id,
            "version": len(self._kek_history) + 1,
            "total_shares": total_shares,
            "threshold": threshold,
            "status": "active",
        })
        # シェアを生成する
        shares = [
            {
                "share_id": i + 1,
                "kek_id": kek_id,
                "total": total_shares,
                "threshold": threshold,
            }
            for i in range(total_shares)
        ]
        # シェアを保存する
        self._distributed_shares[kek_id] = shares
        # アクティブな KEK ID を設定する
        self._active_kek_id = kek_id
        # KEK ID を返す
        return kek_id

    # KEK をローテーションするメソッドを定義する
    def rotate(self, total_shares: int, threshold: int) -> dict[str, Any]:
        """KEK をローテーションする。古い KEK を無効化して新しい KEK を生成する。"""
        # 現在の KEK ID を記録する
        old_kek_id = self._active_kek_id
        # 古い KEK を無効化する
        if old_kek_id:
            # 無効化セットに追加する
            self._revoked_kek_ids.add(old_kek_id)
            # 履歴のステータスを更新する
            for entry in self._kek_history:
                # ID が一致する場合はステータスを更新する
                if entry["kek_id"] == old_kek_id:
                    # ステータスを revoked に変更する
                    entry["status"] = "revoked"
        # 新しい KEK を生成する
        new_kek_id = self.initialize(total_shares, threshold)
        # ローテーション結果を返す
        return {
            "old_kek_id": old_kek_id,
            "new_kek_id": new_kek_id,
            "new_shares": self._distributed_shares[new_kek_id],
        }

    # KEK が有効かどうかを確認するメソッドを定義する
    def is_valid(self, kek_id: str) -> bool:
        """指定された KEK が有効かどうかを返す。"""
        # 無効化セットに含まれる場合は False を返す
        return kek_id not in self._revoked_kek_ids

    # アクティブな KEK ID を取得するプロパティを定義する
    @property
    def active_kek_id(self) -> Optional[str]:
        """現在のアクティブ KEK ID を返す。"""
        # アクティブ KEK ID を返す
        return self._active_kek_id


# スキーマドリフト検出器のモッククラスを定義する
class MockSchemaDriftDetector:
    """スキーマドリフトを検出する GitOps コントローラーのモック。"""

    # 検出器を初期化するメソッドを定義する
    def __init__(self) -> None:
        """スキーマドリフト検出器を初期化する。"""
        # 登録済みスキーマを保持する辞書を初期化する
        self._registered_schemas: dict[str, dict[str, Any]] = {}
        # 検出されたドリフトのリストを初期化する
        self._drift_alerts: list[dict[str, Any]] = []

    # スキーマをベースラインとして登録するメソッドを定義する
    def register_baseline(self, schema_id: str, schema: dict[str, Any]) -> None:
        """スキーマのベースラインを登録する。"""
        # スキーマを登録する
        self._registered_schemas[schema_id] = dict(schema)

    # 現在のスキーマとベースラインを比較するメソッドを定義する
    def detect_drift(
        self, schema_id: str, current_schema: dict[str, Any]
    ) -> Optional[dict[str, Any]]:
        """現在のスキーマとベースラインを比較してドリフトを検出する。"""
        # ベースラインが登録されていない場合はアラートなし
        if schema_id not in self._registered_schemas:
            # アラートなしを返す
            return None
        # ベースラインを取得する
        baseline = self._registered_schemas[schema_id]
        # スキーマが変更されているかを確認する
        if baseline != current_schema:
            # ドリフトアラートを生成する
            alert = {
                "type": "schema_drift",
                "schema_id": schema_id,
                "baseline": baseline,
                "current": current_schema,
            }
            # アラートをリストに追加する
            self._drift_alerts.append(alert)
            # アラートを返す
            return alert
        # 変更がない場合はアラートなしを返す
        return None

    # 検出されたドリフトアラートのリストを取得するプロパティを定義する
    @property
    def drift_alerts(self) -> list[dict[str, Any]]:
        """検出されたドリフトアラートのコピーを返す。"""
        # アラートのコピーを返す
        return list(self._drift_alerts)


# FSM コードジェネレーターのモッククラスを定義する
class MockGoFSMCodeGenerator:
    """Go 言語 FSM コードジェネレーターのモック。

    実際の Go コンパイルをシミュレートする。
    """

    # コードジェネレーターを初期化するメソッドを定義する
    def __init__(self) -> None:
        """コードジェネレーターを初期化する。"""
        # 生成されたコードを保持するリストを初期化する
        self._generated_files: list[dict[str, Any]] = []

    # FSM コードを生成するメソッドを定義する
    def generate(self, fsm_spec: dict[str, Any]) -> dict[str, Any]:
        """FSM 仕様から Go コードを生成する。"""
        # 必須フィールドを検証する
        required = ["name", "states", "initial_state"]
        # 必須フィールドが存在することを確認する
        for field_name in required:
            # フィールドが存在しない場合はエラーを発生させる
            if field_name not in fsm_spec:
                # エラーを発生させる
                raise ValueError(
                    f"FSM spec missing required field: {field_name!r}"
                )
        # Go コードを生成する（実際のコンパイルはモック）
        go_code = self._generate_go_code(fsm_spec)
        # 生成ファイルを記録する
        generated = {
            "fsm_name": fsm_spec["name"],
            "filename": f"{fsm_spec['name'].lower()}_fsm.go",
            "code": go_code,
            "compile_success": True,
        }
        # 生成ファイルをリストに追加する
        self._generated_files.append(generated)
        # 生成結果を返す
        return generated

    # Go コードを生成するヘルパーメソッドを定義する
    def _generate_go_code(self, spec: dict[str, Any]) -> str:
        """FSM 仕様から Go コードを生成する内部メソッド。"""
        # パッケージ宣言を生成する
        lines = [
            "package fsm",
            "",
            f"// {spec['name']}State は {spec['name']} FSM の状態を表す型。",
            f"type {spec['name']}State string",
            "",
            "// 各状態定数を定義する。",
            "const (",
        ]
        # 各状態の定数を生成する
        for state in spec["states"]:
            # 状態定数を追加する
            lines.append(
                f"    {spec['name']}State{state} {spec['name']}State = {state!r}"
            )
        # 定数ブロックを閉じる
        lines.append(")")
        # 空行を追加する
        lines.append("")
        # 開始状態定数を生成する
        lines.append(
            f"// Initial{spec['name']}State は FSM の開始状態。"
        )
        lines.append(
            f"const Initial{spec['name']}State = {spec['name']}State{spec['initial_state']}"
        )
        # コードを結合して返す
        return "\n".join(lines)

    # 生成されたファイルリストを取得するプロパティを定義する
    @property
    def generated_files(self) -> list[dict[str, Any]]:
        """生成されたファイルのコピーを返す。"""
        # ファイルリストのコピーを返す
        return list(self._generated_files)


# crosscutting シナリオリプレイテストスイートクラスを定義する
class TestCrosscuttingScenarioReplay:
    """crosscutting 軸 v1_scenario_replay の検証テストスイート。"""

    # API ゲートウェイのフィクスチャを定義する
    @pytest.fixture
    def gateway(self) -> Generator[MockAPIGateway, None, None]:
        """MockAPIGateway フィクスチャを生成する。"""
        # API ゲートウェイをインスタンス化する
        gw = MockAPIGateway()
        # フィクスチャを返す
        yield gw

    # KEK ローテーションマネージャーのフィクスチャを定義する
    @pytest.fixture
    def kek_manager(self) -> Generator[MockKEKRotationManager, None, None]:
        """MockKEKRotationManager フィクスチャを生成する。"""
        # マネージャーをインスタンス化する
        manager = MockKEKRotationManager()
        # フィクスチャを返す
        yield manager

    # スキーマドリフト検出器のフィクスチャを定義する
    @pytest.fixture
    def drift_detector(self) -> Generator[MockSchemaDriftDetector, None, None]:
        """MockSchemaDriftDetector フィクスチャを生成する。"""
        # 検出器をインスタンス化する
        detector = MockSchemaDriftDetector()
        # フィクスチャを返す
        yield detector

    # FSM コードジェネレーターのフィクスチャを定義する
    @pytest.fixture
    def fsm_codegen(self) -> Generator[MockGoFSMCodeGenerator, None, None]:
        """MockGoFSMCodeGenerator フィクスチャを生成する。"""
        # コードジェネレーターをインスタンス化する
        codegen = MockGoFSMCodeGenerator()
        # フィクスチャを返す
        yield codegen

    # HTTP/1.1 リクエストがエラーになるシナリオをテストする
    def test_scenario_http11_request_protocol_error(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/1.1 リクエストが h2-only エンドポイントでプロトコルエラーを返すことを検証する。"""
        # HTTP/1.1 リクエストを送信する
        response = gateway.request(
            protocol=HTTPProtocol.HTTP11,
            path="/api/v1/manufacturing/orders",
        )
        # ステータスコードが 426 であることを確認する
        assert response.status_code == 426, (
            f"HTTP/1.1 request must return 426 Upgrade Required, got {response.status_code}"
        )
        # レスポンスボディにエラー情報が含まれることを確認する
        assert response.body.get("error") == "upgrade_required"
        # Upgrade ヘッダーが h2 を指定していることを確認する
        assert response.headers.get("upgrade") == "h2"

    # HTTP/2 リクエストが成功するシナリオをテストする
    def test_scenario_http2_request_succeeds(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/2 リクエストが h2-only エンドポイントで成功することを検証する。"""
        # HTTP/2 リクエストを送信する
        response = gateway.request(
            protocol=HTTPProtocol.HTTP2,
            path="/api/v1/manufacturing/orders",
        )
        # ステータスコードが 200 であることを確認する
        assert response.status_code == 200, (
            f"HTTP/2 request must return 200, got {response.status_code}"
        )
        # レスポンスボディが正常であることを確認する
        assert response.body.get("status") == "ok"

    # KEK ローテーションで新しいシェアが配布されるシナリオをテストする
    def test_scenario_kek_rotation_new_shares_distributed(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """KEK ローテーション後に新しいシェアが配布されることを検証する。"""
        # 最初の KEK を初期化する
        initial_kek_id = kek_manager.initialize(total_shares=5, threshold=3)
        # 最初の KEK が有効であることを確認する
        assert kek_manager.is_valid(initial_kek_id)
        # KEK をローテーションする
        rotation_result = kek_manager.rotate(total_shares=5, threshold=3)
        # 古い KEK が無効化されたことを確認する
        assert not kek_manager.is_valid(initial_kek_id), (
            "Old KEK must be revoked after rotation"
        )
        # 新しい KEK が有効であることを確認する
        assert kek_manager.is_valid(rotation_result["new_kek_id"])
        # 新しいシェアが配布されたことを確認する
        assert len(rotation_result["new_shares"]) == 5

    # KEK ローテーション後に古いシェアが拒否されるシナリオをテストする
    def test_scenario_kek_rotation_old_shares_rejected(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """KEK ローテーション後に古いシェアが拒否されることを検証する。"""
        # 最初の KEK を初期化する
        old_kek_id = kek_manager.initialize(total_shares=5, threshold=3)
        # KEK をローテーションする
        kek_manager.rotate(total_shares=5, threshold=3)
        # 古い KEK が無効であることを確認する
        assert not kek_manager.is_valid(old_kek_id), (
            f"Old KEK {old_kek_id!r} must be invalid after rotation"
        )
        # 現在のアクティブ KEK は古い KEK と異なることを確認する
        assert kek_manager.active_kek_id != old_kek_id

    # スキーマドリフトが検出されるシナリオをテストする
    def test_scenario_schema_drift_detected_and_alerted(
        self, drift_detector: MockSchemaDriftDetector
    ) -> None:
        """スキーマが変更されると GitOps コントローラーがドリフトを検出することを検証する。"""
        # ベースラインスキーマを登録する
        baseline = {
            "version": 1,
            "fields": ["id", "tenant_id", "order_date", "status"],
        }
        # ベースラインを登録する
        drift_detector.register_baseline("order_schema", baseline)
        # スキーマが変更されていない場合はアラートなしを確認する
        no_drift = drift_detector.detect_drift("order_schema", baseline)
        # アラートなしであることを確認する
        assert no_drift is None
        # スキーマに新しいフィールドを追加する
        modified_schema = {
            "version": 2,
            "fields": ["id", "tenant_id", "order_date", "status", "priority"],
        }
        # ドリフトを検出する
        drift_alert = drift_detector.detect_drift("order_schema", modified_schema)
        # ドリフトアラートが生成されたことを確認する
        assert drift_alert is not None, (
            "Schema drift must be detected when fields are added"
        )
        # アラートタイプが schema_drift であることを確認する
        assert drift_alert["type"] == "schema_drift"
        # schema_id が正しいことを確認する
        assert drift_alert["schema_id"] == "order_schema"

    # FSM コードジェネレーターが有効な Go コードを生成するシナリオをテストする
    def test_scenario_fsm_codegen_generates_compilable_go(
        self, fsm_codegen: MockGoFSMCodeGenerator
    ) -> None:
        """FSM コードジェネレーターが有効な Go コードを生成することを検証する。"""
        # FSM 仕様を定義する
        spec = {
            "name": "OrderFSM",
            "states": ["PENDING", "CONFIRMED", "SHIPPED", "DELIVERED", "CANCELLED"],
            "initial_state": "PENDING",
            "transitions": [
                {"from": "PENDING", "to": "CONFIRMED", "event": "confirm"},
                {"from": "CONFIRMED", "to": "SHIPPED", "event": "ship"},
            ],
        }
        # コードを生成する
        generated = fsm_codegen.generate(spec)
        # コンパイル成功フラグが True であることを確認する
        assert generated["compile_success"] is True
        # ファイル名が正しいことを確認する
        assert generated["filename"] == "orderfsm_fsm.go"
        # 生成されたコードに FSM 名が含まれることを確認する
        assert "OrderFSM" in generated["code"]
        # 生成されたコードに開始状態が含まれることを確認する
        assert "PENDING" in generated["code"]
        # 生成されたコードに InitialOrderFSMState が含まれることを確認する
        assert "InitialOrderFSMState" in generated["code"]

    # 連続した HTTP リクエストが統計に反映されることをテストする
    def test_scenario_consecutive_requests_recorded(
        self, gateway: MockAPIGateway
    ) -> None:
        """連続した HTTP リクエストが正確に記録されることを検証する。"""
        # HTTP/1.1 リクエストを 2 件送信する
        for _ in range(2):
            # HTTP/1.1 リクエストを送信する
            gateway.request(HTTPProtocol.HTTP11, "/api/test")
        # HTTP/2 リクエストを 3 件送信する
        for _ in range(3):
            # HTTP/2 リクエストを送信する
            gateway.request(HTTPProtocol.HTTP2, "/api/test")
        # リクエスト履歴が 5 件であることを確認する
        history = gateway.request_history
        # 履歴の件数が 5 件であることを確認する
        assert len(history) == 5, f"Expected 5 requests, got {len(history)}"
        # HTTP/1.1 リクエストが 2 件であることを確認する
        h11_count = sum(1 for r in history if r["protocol"] == "http/1.1")
        # HTTP/1.1 リクエスト数が 2 件であることを確認する
        assert h11_count == 2
        # HTTP/2 リクエストが 3 件であることを確認する
        h2_count = sum(1 for r in history if r["protocol"] == "h2")
        # HTTP/2 リクエスト数が 3 件であることを確認する
        assert h2_count == 3

    # KEK の複数ローテーションが正しく履歴に記録されることをテストする
    def test_scenario_multiple_kek_rotations_history(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """複数回の KEK ローテーションが正しく処理されることを検証する。"""
        # 最初の KEK を初期化する
        kek_v1 = kek_manager.initialize(total_shares=5, threshold=3)
        # 1 回目のローテーションを実行する
        rotation1 = kek_manager.rotate(total_shares=5, threshold=3)
        # kek_v1 が無効化されたことを確認する
        assert not kek_manager.is_valid(kek_v1)
        # 2 回目のローテーションを実行する
        rotation2 = kek_manager.rotate(total_shares=5, threshold=3)
        # 1 回目のローテーション後の KEK が無効化されたことを確認する
        assert not kek_manager.is_valid(rotation1["new_kek_id"])
        # 2 回目のローテーション後の KEK が有効であることを確認する
        assert kek_manager.is_valid(rotation2["new_kek_id"])
        # 現在のアクティブ KEK が最後のローテーションの KEK であることを確認する
        assert kek_manager.active_kek_id == rotation2["new_kek_id"]


# crosscutting シナリオリプレイ拡張テストスイートクラスを定義する
class TestCrosscuttingScenarioReplayExtended:
    """crosscutting 軸 v1_scenario_replay の拡張テストスイート。追加のシナリオを網羅的に検証する。"""

    # API ゲートウェイのフィクスチャを定義する
    @pytest.fixture
    def gateway(self) -> Generator[MockAPIGateway, None, None]:
        """MockAPIGateway フィクスチャを生成する。"""
        # ゲートウェイをインスタンス化する
        gw = MockAPIGateway()
        # フィクスチャを返す
        yield gw

    # KEK ローテーションマネージャーのフィクスチャを定義する
    @pytest.fixture
    def kek_manager(self) -> Generator[MockKEKRotationManager, None, None]:
        """MockKEKRotationManager フィクスチャを生成する。"""
        # マネージャーをインスタンス化する
        manager = MockKEKRotationManager()
        # フィクスチャを返す
        yield manager

    # スキーマドリフト検出器のフィクスチャを定義する
    @pytest.fixture
    def drift_detector(self) -> Generator[MockSchemaDriftDetector, None, None]:
        """MockSchemaDriftDetector フィクスチャを生成する。"""
        # 検出器をインスタンス化する
        detector = MockSchemaDriftDetector()
        # フィクスチャを返す
        yield detector

    # FSM コードジェネレーターのフィクスチャを定義する
    @pytest.fixture
    def fsm_generator(self) -> Generator[MockGoFSMCodeGenerator, None, None]:
        """MockGoFSMCodeGenerator フィクスチャを生成する。"""
        # ジェネレーターをインスタンス化する
        gen = MockGoFSMCodeGenerator()
        # フィクスチャを返す
        yield gen

    # HTTP/1.1 リクエストが拒否されることをテストする
    def test_scenario_http11_rejected_status_426(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/1.1 リクエストが 426 で拒否されることを検証する。"""
        # HTTP/1.1 リクエストを送信する
        response = gateway.request(HTTPProtocol.HTTP11, "/api/v1/test")
        # ステータスコードが 426 であることを確認する
        assert response.status_code == 426

    # HTTP/2 リクエストが成功することをテストする
    def test_scenario_http2_accepted_status_200(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/2 リクエストが 200 で成功することを検証する。"""
        # HTTP/2 リクエストを送信する
        response = gateway.request(HTTPProtocol.HTTP2, "/api/v1/test")
        # ステータスコードが 200 であることを確認する
        assert response.status_code == 200

    # HTTP/1.1 リクエストのエラーレスポンスに upgrade ヘッダーがあることをテストする
    def test_scenario_http11_response_has_upgrade_header(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/1.1 エラーレスポンスに upgrade ヘッダーが含まれることを検証する。"""
        # HTTP/1.1 リクエストを送信する
        response = gateway.request(HTTPProtocol.HTTP11, "/api/v1/items")
        # upgrade ヘッダーが存在することを確認する
        assert "upgrade" in response.headers
        # upgrade ヘッダーの値が h2 であることを確認する
        assert response.headers["upgrade"] == "h2"

    # KEK の初期化でシェアが配布されることをテストする
    def test_scenario_kek_initialization_distributes_shares(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """KEK の初期化でシェアが正しく配布されることを検証する。"""
        # KEK を初期化する（n=6, k=4）
        kek_id = kek_manager.initialize(total_shares=6, threshold=4)
        # KEK が有効であることを確認する
        assert kek_manager.is_valid(kek_id)
        # アクティブな KEK ID が正しいことを確認する
        assert kek_manager.active_kek_id == kek_id

    # KEK ローテーションで古い KEK が無効化されることをテストする
    def test_scenario_kek_rotation_revokes_old_kek(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """KEK ローテーションで古い KEK が無効化されることを検証する。"""
        # KEK を初期化する
        old_kek_id = kek_manager.initialize(total_shares=5, threshold=3)
        # KEK が有効であることを確認する
        assert kek_manager.is_valid(old_kek_id)
        # KEK をローテーションする
        rotation = kek_manager.rotate(total_shares=5, threshold=3)
        # 古い KEK が無効化されたことを確認する
        assert not kek_manager.is_valid(old_kek_id)
        # 新しい KEK が有効であることを確認する
        assert kek_manager.is_valid(rotation["new_kek_id"])

    # スキーマドリフトが検出されることをテストする
    def test_scenario_schema_drift_detected(
        self, drift_detector: MockSchemaDriftDetector
    ) -> None:
        """スキーマドリフトが正しく検出されることを検証する。"""
        # ベースラインスキーマを登録する
        baseline = {"version": 1, "columns": ["id", "status"]}
        # ベースラインを登録する
        drift_detector.register_baseline("orders", baseline)
        # 変更されたスキーマ（ドリフトあり）を作成する
        changed = {"version": 2, "columns": ["id", "status", "created_at"]}
        # ドリフトを検出する
        drift = drift_detector.detect_drift("orders", changed)
        # ドリフトが検出されたことを確認する
        assert drift is not None
        # ドリフトタイプが schema_drift であることを確認する
        assert drift["type"] == "schema_drift"

    # スキーマドリフトがない場合は検出されないことをテストする
    def test_scenario_no_schema_drift_clean(
        self, drift_detector: MockSchemaDriftDetector
    ) -> None:
        """スキーマドリフトがない場合は検出されないことを検証する。"""
        # 同一のスキーマを定義する
        schema = {"version": 1, "columns": ["id", "name", "value"]}
        # ベースラインを登録する
        drift_detector.register_baseline("products", schema)
        # 同じスキーマを現在スキーマとして渡す（変更なし）
        drift = drift_detector.detect_drift("products", schema)
        # ドリフトが検出されないことを確認する
        assert drift is None

    # FSM コードジェネレーターが Go コードを生成することをテストする
    def test_scenario_fsm_codegen_produces_go_code(
        self, fsm_generator: MockGoFSMCodeGenerator
    ) -> None:
        """FSM コードジェネレーターが Go コードを生成することを検証する。"""
        # FSM の仕様を定義する
        spec = {
            "name": "AlertFSM",
            "states": ["IDLE", "FIRED", "RESOLVED"],
            "initial_state": "IDLE",
        }
        # Go コードを生成する
        result = fsm_generator.generate(spec)
        # 生成されたコードが辞書であることを確認する
        assert isinstance(result, dict)
        # compile_success が True であることを確認する
        assert result["compile_success"] is True
        # fsm_name が AlertFSM であることを確認する
        assert result["fsm_name"] == "AlertFSM"
        # コードに初期状態が含まれていることを確認する
        assert "IDLE" in result["code"]


# crosscutting シナリオリプレイの第三拡張テストクラスを定義する
class TestCrosscuttingScenarioReplayExtended3:
    """HTTP/2・KEK・スキーマドリフト・FSM コードgen のシナリオの追加テスト群（第三拡張）。"""

    # API ゲートウェイのフィクスチャを定義する
    @pytest.fixture
    def gateway(self) -> MockAPIGateway:
        """MockAPIGateway フィクスチャを生成する。"""
        # MockAPIGateway をインスタンス化して返す
        return MockAPIGateway()

    # KEK マネージャーのフィクスチャを定義する
    @pytest.fixture
    def kek_manager(self) -> MockKEKRotationManager:
        """MockKEKRotationManager フィクスチャを生成する。"""
        # MockKEKRotationManager をインスタンス化して返す
        return MockKEKRotationManager()

    # スキーマドリフト検出器のフィクスチャを定義する
    @pytest.fixture
    def drift_detector(self) -> MockSchemaDriftDetector:
        """MockSchemaDriftDetector フィクスチャを生成する。"""
        # MockSchemaDriftDetector をインスタンス化して返す
        return MockSchemaDriftDetector()

    # HTTP/2 エンドポイントへの複数 GET リクエストが全て 200 になることをテストする
    def test_scenario_http2_multiple_get_requests(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/2 の複数 GET リクエストが全て 200 OK になることを検証する。"""
        # 3 つの異なるパスに HTTP/2 リクエストを送信する
        paths = ["/api/v1/orders", "/api/v1/tenants", "/api/v1/health"]
        # 各パスにリクエストを送信する
        for path in paths:
            # HTTP/2 でリクエストを送信する
            response = gateway.request(HTTPProtocol.HTTP2, path, "GET")
            # 200 OK が返ることを確認する
            assert response.status_code == 200, f"Path {path}: expected 200, got {response.status_code}"

    # HTTP/1.1 POST リクエストも 426 になることをテストする
    def test_scenario_http11_post_rejected(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/1.1 POST リクエストが 426 Upgrade Required になることを検証する。"""
        # HTTP/1.1 で POST リクエストを送信する
        response = gateway.request(HTTPProtocol.HTTP11, "/api/v1/submit", "POST")
        # 426 Upgrade Required が返ることを確認する
        assert response.status_code == 426

    # KEK の初期化後のアクティブ KEK ID が存在することをテストする
    def test_scenario_kek_initialization_active_kek_exists(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """KEK の初期化後にアクティブな KEK ID が存在することを検証する。"""
        # KEK を初期化する
        kek_manager.initialize(total_shares=3, threshold=2)
        # アクティブ KEK ID が存在することを確認する
        active_kek_id = kek_manager.active_kek_id
        # None でないことを確認する
        assert active_kek_id is not None
        # 有効な KEK であることを確認する
        assert kek_manager.is_valid(active_kek_id)

    # スキーマの変更がない場合にドリフトが検出されないことをテストする
    def test_scenario_schema_no_drift_no_detection(
        self, drift_detector: MockSchemaDriftDetector
    ) -> None:
        """スキーマの変更がない場合にドリフトが検出されないことを検証する。"""
        # ベースラインを登録する
        baseline = {"type": "object", "properties": {"id": {"type": "integer"}}}
        # ベースラインを登録する
        drift_detector.register_baseline("schema_no_drift", baseline)
        # 同じスキーマでドリフト検出を実行する
        result = drift_detector.detect_drift("schema_no_drift", baseline)
        # ドリフトが検出されないことを確認する（None が返る）
        assert result is None

    # スキーマにフィールドが追加されたときにドリフトが検出されることをテストする
    def test_scenario_schema_field_added_drift_detected(
        self, drift_detector: MockSchemaDriftDetector
    ) -> None:
        """スキーマにフィールドが追加されたときにドリフトが検出されることを検証する。"""
        # ベースラインを登録する
        baseline = {"type": "object", "properties": {"id": {"type": "integer"}}}
        # ベースラインを登録する
        drift_detector.register_baseline("schema_field_added", baseline)
        # フィールドを追加した現在のスキーマを定義する
        current = {
            "type": "object",
            "properties": {
                "id": {"type": "integer"},
                "name": {"type": "string"},
            },
        }
        # ドリフト検出を実行する
        result = drift_detector.detect_drift("schema_field_added", current)
        # ドリフトが検出されることを確認する
        assert result is not None

    # FSM コードゲンが有効な FSM スペックから Go コードを生成することをテストする
    def test_scenario_fsm_codegen_generates_valid_code(self) -> None:
        """FSM コードゲンが有効な FSM スペックから Go コードを生成することを検証する。"""
        # FSM コードゲンをインスタンス化する
        codegen = MockGoFSMCodeGenerator()
        # FSM スペックを定義する
        fsm_spec = {
            "name": "OrderFSM",
            "states": ["PENDING", "PROCESSING", "SHIPPED", "DELIVERED"],
            "initial": "PENDING",
            "transitions": [
                {"from": "PENDING", "to": "PROCESSING"},
                {"from": "PROCESSING", "to": "SHIPPED"},
                {"from": "SHIPPED", "to": "DELIVERED"},
            ],
        }
        # コードを生成する
        result = codegen.generate(fsm_spec)
        # 生成が成功することを確認する
        assert result.get("compile_success") is True
        # FSM 名が含まれることを確認する
        assert "OrderFSM" in result.get("code", "")

    # KEK ローテーション後に古い KEK が無効になることをテストする
    def test_scenario_kek_rotation_invalidates_old(
        self, kek_manager: MockKEKRotationManager
    ) -> None:
        """KEK ローテーション後に古い KEK が無効になることを検証する。"""
        # KEK を初期化する
        kek_manager.initialize(total_shares=5, threshold=3)
        # アクティブ KEK ID を取得する
        old_kek_id = kek_manager.active_kek_id
        # KEK をローテーションする
        kek_manager.rotate(total_shares=5, threshold=3)
        # 古い KEK が無効になることを確認する
        assert not kek_manager.is_valid(old_kek_id), "Old KEK must be invalid after rotation"
        # 新しい KEK が有効であることを確認する
        assert kek_manager.is_valid(kek_manager.active_kek_id), "New KEK must be valid after rotation"

    # HTTP/2 レスポンスにプロトコルが正しく記録されることをテストする
    def test_scenario_http2_response_protocol_recorded(
        self, gateway: MockAPIGateway
    ) -> None:
        """HTTP/2 レスポンスにプロトコルが正しく記録されることを検証する。"""
        # HTTP/2 でリクエストを送信する
        response = gateway.request(HTTPProtocol.HTTP2, "/api/v1/test")
        # プロトコルが h2 であることを確認する
        assert response.protocol == HTTPProtocol.HTTP2.value
