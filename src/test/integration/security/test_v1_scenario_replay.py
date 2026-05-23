"""src/test/integration/security/test_v1_scenario_replay.py

security 軸の v1_scenario_replay 検証クラスに対するインテグレーションテスト。
未署名イメージの拒否・署名済みイメージの許可・STRIDE HIGH 脅威ゲート・
cosign バンドル検証のシナリオを検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: security_scenario_replay_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import base64
import hashlib
import json
import uuid
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto


# アドミッション結果を表す列挙型を定義する
class AdmissionResult(Enum):
    """Kubernetes アドミッションコントローラーの結果。"""

    # 許可を表す結果を定義する
    ALLOWED = "allowed"
    # 拒否を表す結果を定義する
    DENIED = "denied"


# コンテナイメージを表すデータクラスを定義する
@dataclass
class ContainerImage:
    """コンテナイメージのメタデータ。"""

    # イメージ参照を保持するフィールドを定義する
    reference: str
    # イメージのダイジェストを保持するフィールドを定義する
    digest: str
    # 署名バンドルを保持するフィールドを定義する（未署名は None）
    signature_bundle: Optional[dict[str, Any]] = None

    # 署名済みかどうかを確認するプロパティを定義する
    @property
    def is_signed(self) -> bool:
        """イメージが署名されているかどうかを返す。"""
        # 署名バンドルが存在するかどうかを返す
        return self.signature_bundle is not None

    # 有効な署名バンドルを持つかどうかを確認するプロパティを定義する
    @property
    def has_valid_bundle(self) -> bool:
        """有効な cosign バンドルを持つかどうかを返す。"""
        # 署名バンドルが存在しない場合は False を返す
        if self.signature_bundle is None:
            # バンドルなしの場合は False を返す
            return False
        # 必須フィールドを確認する
        required_fields = ["mediaType", "verificationMaterial", "messageSignature"]
        # 全必須フィールドが存在するかどうかを確認する
        return all(f in self.signature_bundle for f in required_fields)


# Kyverno アドミッションコントローラーのモッククラスを定義する
class MockKyvernoAdmissionController:
    """cosign 署名ポリシーを強制する Kyverno アドミッションコントローラーのモック。"""

    # コントローラーを初期化するメソッドを定義する
    def __init__(self, require_signature: bool = True) -> None:
        """アドミッションコントローラーを初期化する。"""
        # 署名必須フラグを設定する
        self._require_signature = require_signature
        # 許可されたイメージのリストを初期化する
        self._allowed_images: list[str] = []
        # 拒否されたイメージのリストを初期化する
        self._denied_images: list[str] = []

    # アドミッション審査を実行するメソッドを定義する
    def review(self, image: ContainerImage) -> dict[str, Any]:
        """コンテナイメージのアドミッション審査を行う。"""
        # 署名が必須の場合は署名を確認する
        if self._require_signature:
            # 署名済みでない場合は拒否する
            if not image.is_signed:
                # 拒否リストに追加する
                self._denied_images.append(image.reference)
                # 拒否レスポンスを返す
                return {
                    "allowed": False,
                    "status": {
                        "code": 403,
                        "message": (
                            f"Image {image.reference!r} does not have a "
                            "required cosign signature"
                        ),
                    },
                }
            # 署名バンドルが無効な場合は拒否する
            if not image.has_valid_bundle:
                # 拒否リストに追加する
                self._denied_images.append(image.reference)
                # 拒否レスポンスを返す
                return {
                    "allowed": False,
                    "status": {
                        "code": 403,
                        "message": (
                            f"Image {image.reference!r} has an invalid "
                            "cosign bundle structure"
                        ),
                    },
                }
        # 許可リストに追加する
        self._allowed_images.append(image.reference)
        # 許可レスポンスを返す
        return {
            "allowed": True,
            "status": {"code": 200, "message": "Admission allowed"},
        }

    # 審査統計を取得するプロパティを定義する
    @property
    def stats(self) -> dict[str, int]:
        """審査統計を返す。"""
        # 許可・拒否数を返す
        return {
            "allowed": len(self._allowed_images),
            "denied": len(self._denied_images),
        }


# STRIDE 脅威ゲートのモッククラスを定義する
class MockSTRIDEThreatGate:
    """STRIDE HIGH 脅威の未緩和をブロックするゲートモック。"""

    # ゲートを初期化するメソッドを定義する
    def __init__(self) -> None:
        """脅威ゲートを初期化する。"""
        # 登録された脅威を保持する辞書を初期化する
        self._threats: dict[str, dict[str, Any]] = {}
        # 登録された緩和策を保持する辞書を初期化する
        self._mitigations: dict[str, dict[str, Any]] = {}

    # 脅威を登録するメソッドを定義する
    def register_threat(
        self,
        threat_id: str,
        severity: str,
        description: str,
    ) -> None:
        """脅威をゲートに登録する。"""
        # 脅威を登録する
        self._threats[threat_id] = {
            "threat_id": threat_id,
            "severity": severity,
            "description": description,
            "mitigated": False,
        }

    # 緩和策を登録するメソッドを定義する
    def register_mitigation(
        self,
        mitigation_id: str,
        threat_id: str,
        description: str,
    ) -> None:
        """緩和策を登録して対象脅威を緩和済みにする。"""
        # 緩和策を登録する
        self._mitigations[mitigation_id] = {
            "mitigation_id": mitigation_id,
            "threat_id": threat_id,
            "description": description,
        }
        # 対象脅威が存在する場合は緩和済みにする
        if threat_id in self._threats:
            # 緩和済みフラグを設定する
            self._threats[threat_id]["mitigated"] = True

    # ゲートチェックを実行するメソッドを定義する
    def check_gate(self) -> dict[str, Any]:
        """HIGH 以上の未緩和脅威があればゲートをブロックする。"""
        # HIGH 以上の未緩和脅威を検索する
        blocking_threats = [
            t for t in self._threats.values()
            if t["severity"] in ("HIGH", "CRITICAL") and not t["mitigated"]
        ]
        # ブロックする脅威がある場合はゲートをブロックする
        if blocking_threats:
            # ブロックレスポンスを返す
            return {
                "gate_passed": False,
                "blocking_threats": [t["threat_id"] for t in blocking_threats],
                "message": (
                    f"{len(blocking_threats)} HIGH/CRITICAL threat(s) "
                    "without mitigation"
                ),
            }
        # ブロックする脅威がない場合はゲートを通過させる
        return {
            "gate_passed": True,
            "blocking_threats": [],
            "message": "All HIGH/CRITICAL threats have mitigations",
        }


# cosign バンドル検証のモッククラスを定義する
class MockCosignVerifier:
    """cosign verify-blob 相当の検証を行うモック。"""

    # 検証器を初期化するメソッドを定義する
    def __init__(self) -> None:
        """検証器を初期化する。"""
        # 有効な公開鍵のセットを初期化する
        self._valid_public_keys: set[str] = {
            "k1s0-cosign-public-key-v1",
            "k1s0-cosign-public-key-v2",
        }

    # バンドルを検証するメソッドを定義する
    def verify_bundle(
        self,
        bundle: dict[str, Any],
        expected_signer: str,
    ) -> dict[str, Any]:
        """cosign バンドルの署名を検証する。"""
        # バンドルが None の場合は検証失敗を返す
        if bundle is None:
            # 失敗レスポンスを返す
            return {
                "verified": False,
                "reason": "Bundle is None",
            }
        # 必須フィールドを確認する
        required = ["mediaType", "verificationMaterial", "messageSignature"]
        # 必須フィールドが欠落している場合は検証失敗を返す
        for field_name in required:
            # フィールドが存在しない場合は失敗を返す
            if field_name not in bundle:
                # 失敗レスポンスを返す
                return {
                    "verified": False,
                    "reason": f"Missing required field: {field_name}",
                }
        # 検証マテリアルから公開鍵を取得する
        verification_material = bundle.get("verificationMaterial", {})
        # 署名者の公開鍵を取得する
        signer_key = verification_material.get("signerPublicKey", "")
        # 公開鍵が有効なセットに含まれるかを確認する
        if signer_key not in self._valid_public_keys:
            # 無効な公開鍵の場合は失敗を返す
            return {
                "verified": False,
                "reason": f"Invalid signer public key: {signer_key!r}",
            }
        # 期待される署名者と一致するかを確認する
        if signer_key != expected_signer:
            # 署名者が一致しない場合は失敗を返す
            return {
                "verified": False,
                "reason": f"Signer mismatch: expected {expected_signer!r}, got {signer_key!r}",
            }
        # 検証成功を返す
        return {
            "verified": True,
            "reason": "Signature verification succeeded",
            "signer": signer_key,
        }


# security 軸シナリオリプレイテストスイートクラスを定義する
class TestSecurityScenarioReplay:
    """security 軸 v1_scenario_replay の検証テストスイート。"""

    # アドミッションコントローラーのフィクスチャを定義する
    @pytest.fixture
    def admission_controller(
        self,
    ) -> Generator[MockKyvernoAdmissionController, None, None]:
        """MockKyvernoAdmissionController フィクスチャを生成する。"""
        # アドミッションコントローラーをインスタンス化する
        controller = MockKyvernoAdmissionController(require_signature=True)
        # フィクスチャを返す
        yield controller

    # STRIDE 脅威ゲートのフィクスチャを定義する
    @pytest.fixture
    def threat_gate(self) -> Generator[MockSTRIDEThreatGate, None, None]:
        """MockSTRIDEThreatGate フィクスチャを生成する。"""
        # 脅威ゲートをインスタンス化する
        gate = MockSTRIDEThreatGate()
        # フィクスチャを返す
        yield gate

    # cosign 検証器のフィクスチャを定義する
    @pytest.fixture
    def cosign_verifier(self) -> Generator[MockCosignVerifier, None, None]:
        """MockCosignVerifier フィクスチャを生成する。"""
        # 検証器をインスタンス化する
        verifier = MockCosignVerifier()
        # フィクスチャを返す
        yield verifier

    # 未署名イメージがアドミッションで拒否されることをテストする
    def test_scenario_unsigned_image_admission_denied(
        self, admission_controller: MockKyvernoAdmissionController
    ) -> None:
        """未署名のコンテナイメージがアドミッションで拒否されることを検証する。"""
        # 未署名のイメージを生成する
        unsigned_image = ContainerImage(
            reference="registry.k1s0.io/manufacturing-api:latest",
            digest="sha256:abc123def456",
            signature_bundle=None,
        )
        # アドミッション審査を実行する
        result = admission_controller.review(unsigned_image)
        # アドミッションが拒否されたことを確認する
        assert result["allowed"] is False, (
            "Unsigned image must be denied by admission controller"
        )
        # ステータスコードが 403 であることを確認する
        assert result["status"]["code"] == 403
        # エラーメッセージに署名情報が含まれることを確認する
        assert "signature" in result["status"]["message"].lower()

    # 署名済みイメージがアドミッションで許可されることをテストする
    def test_scenario_signed_image_admission_allowed(
        self, admission_controller: MockKyvernoAdmissionController
    ) -> None:
        """有効な cosign 署名を持つイメージがアドミッションで許可されることを検証する。"""
        # 有効な cosign バンドルを持つイメージを生成する
        signed_image = ContainerImage(
            reference="registry.k1s0.io/manufacturing-api@sha256:abc123",
            digest="sha256:abc123def456789",
            signature_bundle={
                "mediaType": "application/vnd.dev.sigstore.bundle+json;version=0.3",
                "verificationMaterial": {
                    "certificate": "LS0tLS1CRUdJTiBDRVJUSUZJQ0FU...",
                    "signerPublicKey": "k1s0-cosign-public-key-v1",
                },
                "messageSignature": {
                    "signature": "MEUCIQDexample_valid_signature...",
                    "messageDigest": {
                        "algorithm": "SHA2_256",
                        "digest": "abc123def456",
                    },
                },
            },
        )
        # アドミッション審査を実行する
        result = admission_controller.review(signed_image)
        # アドミッションが許可されたことを確認する
        assert result["allowed"] is True, (
            "Signed image with valid bundle must be allowed by admission controller"
        )
        # ステータスコードが 200 であることを確認する
        assert result["status"]["code"] == 200

    # STRIDE HIGH 脅威に緩和策がない場合にゲートがブロックされることをテストする
    def test_scenario_stride_high_threat_missing_mitigation_blocks_gate(
        self, threat_gate: MockSTRIDEThreatGate
    ) -> None:
        """緩和策のない STRIDE HIGH 脅威がリリースゲートをブロックすることを検証する。"""
        # HIGH 脅威を登録する
        threat_gate.register_threat(
            threat_id="STRIDE-S-001",
            severity="HIGH",
            description="Unauthorized API access via stolen JWT",
        )
        # ゲートチェックを実行する（緩和策なし）
        gate_result = threat_gate.check_gate()
        # ゲートがブロックされていることを確認する
        assert gate_result["gate_passed"] is False, (
            "Gate must block when HIGH threat has no mitigation"
        )
        # ブロックしている脅威リストに STRIDE-S-001 が含まれることを確認する
        assert "STRIDE-S-001" in gate_result["blocking_threats"]

    # 緩和策を追加するとゲートが通過することをテストする
    def test_scenario_stride_high_threat_with_mitigation_gate_passes(
        self, threat_gate: MockSTRIDEThreatGate
    ) -> None:
        """緩和策が追加されると STRIDE HIGH 脅威でもゲートが通過することを検証する。"""
        # HIGH 脅威を登録する
        threat_gate.register_threat(
            threat_id="STRIDE-T-042",
            severity="HIGH",
            description="Data tampering in transit",
        )
        # 緩和策を登録する
        threat_gate.register_mitigation(
            mitigation_id="MIT-T-042-01",
            threat_id="STRIDE-T-042",
            description="TLS 1.3 with mutual authentication",
        )
        # ゲートチェックを実行する（緩和策あり）
        gate_result = threat_gate.check_gate()
        # ゲートが通過していることを確認する
        assert gate_result["gate_passed"] is True, (
            "Gate must pass when all HIGH threats have mitigations"
        )
        # ブロックしている脅威リストが空であることを確認する
        assert gate_result["blocking_threats"] == []

    # 有効な cosign バンドルの検証が成功することをテストする
    def test_scenario_valid_cosign_bundle_verify_succeeds(
        self, cosign_verifier: MockCosignVerifier
    ) -> None:
        """有効な cosign バンドルの verify-blob が成功することを検証する。"""
        # 有効なバンドルを生成する
        valid_bundle = {
            "mediaType": "application/vnd.dev.sigstore.bundle+json;version=0.3",
            "verificationMaterial": {
                "certificate": "LS0tLS1CRUdJTiBDRVJUSUZJQ0FU...",
                "signerPublicKey": "k1s0-cosign-public-key-v1",
            },
            "messageSignature": {
                "signature": "MEUCIQDvalid_signature...",
                "messageDigest": {
                    "algorithm": "SHA2_256",
                    "digest": "deadbeef...",
                },
            },
        }
        # バンドルを検証する
        result = cosign_verifier.verify_bundle(
            bundle=valid_bundle,
            expected_signer="k1s0-cosign-public-key-v1",
        )
        # 検証が成功したことを確認する
        assert result["verified"] is True, (
            f"Valid cosign bundle verification should succeed: {result}"
        )
        # 署名者が正しく返されることを確認する
        assert result["signer"] == "k1s0-cosign-public-key-v1"

    # None バンドルの検証が失敗することをテストする
    def test_scenario_none_bundle_verify_fails(
        self, cosign_verifier: MockCosignVerifier
    ) -> None:
        """None バンドルの verify-blob が失敗することを検証する。"""
        # None バンドルを検証する
        result = cosign_verifier.verify_bundle(
            bundle=None,
            expected_signer="k1s0-cosign-public-key-v1",
        )
        # 検証が失敗したことを確認する
        assert result["verified"] is False, "None bundle should fail verification"

    # CRITICAL 脅威も緩和策なしでゲートをブロックすることをテストする
    def test_scenario_critical_threat_also_blocks_gate(
        self, threat_gate: MockSTRIDEThreatGate
    ) -> None:
        """緩和策のない CRITICAL 脅威もゲートをブロックすることを検証する。"""
        # CRITICAL 脅威を登録する
        threat_gate.register_threat(
            threat_id="STRIDE-E-001",
            severity="CRITICAL",
            description="Privilege escalation via container escape",
        )
        # ゲートチェックを実行する
        gate_result = threat_gate.check_gate()
        # ゲートがブロックされていることを確認する
        assert gate_result["gate_passed"] is False
        # CRITICAL 脅威がブロックリストに含まれることを確認する
        assert "STRIDE-E-001" in gate_result["blocking_threats"]

    # MEDIUM 脅威は緩和策なしでもゲートを通過することをテストする
    def test_scenario_medium_threat_does_not_block_gate(
        self, threat_gate: MockSTRIDEThreatGate
    ) -> None:
        """MEDIUM 脅威は緩和策なしでもゲートをブロックしないことを検証する。"""
        # MEDIUM 脅威を登録する
        threat_gate.register_threat(
            threat_id="STRIDE-I-005",
            severity="MEDIUM",
            description="Verbose error messages expose internal paths",
        )
        # ゲートチェックを実行する（MEDIUM 脅威は緩和策なしでも通過）
        gate_result = threat_gate.check_gate()
        # ゲートが通過することを確認する
        assert gate_result["gate_passed"] is True, (
            "MEDIUM severity threats should not block the gate"
        )

    # アドミッション統計が正確であることをテストする
    def test_scenario_admission_statistics_accurate(
        self, admission_controller: MockKyvernoAdmissionController
    ) -> None:
        """アドミッション審査の統計が正確であることを検証する。"""
        # 3 件の未署名イメージを審査する
        for i in range(3):
            # 未署名のイメージを審査する
            admission_controller.review(
                ContainerImage(
                    reference=f"registry.k1s0.io/unsigned_{i}:latest",
                    digest=f"sha256:unsigned_{i}",
                    signature_bundle=None,
                )
            )
        # 2 件の署名済みイメージを審査する
        for i in range(2):
            # 署名済みのイメージを審査する
            admission_controller.review(
                ContainerImage(
                    reference=f"registry.k1s0.io/signed_{i}:v1.0",
                    digest=f"sha256:signed_{i}",
                    signature_bundle={
                        "mediaType": "application/vnd.dev.sigstore.bundle+json;version=0.3",
                        "verificationMaterial": {"certificate": "cert_data"},
                        "messageSignature": {"signature": "sig_data", "messageDigest": {}},
                    },
                )
            )
        # 統計を取得する
        stats = admission_controller.stats
        # 拒否された件数が 3 であることを確認する
        assert stats["denied"] == 3, f"Expected 3 denied, got {stats['denied']}"
        # 許可された件数が 2 であることを確認する
        assert stats["allowed"] == 2, f"Expected 2 allowed, got {stats['allowed']}"


# security シナリオリプレイ拡張テストスイートクラスを定義する
class TestSecurityScenarioReplayExtended:
    """security 軸 v1_scenario_replay の拡張テストスイート。追加のシナリオを網羅的に検証する。"""

    # Kyverno アドミッションコントローラーのフィクスチャを定義する
    @pytest.fixture
    def admission_controller(
        self,
    ) -> Generator[MockKyvernoAdmissionController, None, None]:
        """MockKyvernoAdmissionController フィクスチャを生成する。"""
        # アドミッションコントローラーをインスタンス化する（署名必須）
        controller = MockKyvernoAdmissionController(require_signature=True)
        # フィクスチャを返す
        yield controller

    # STRIDE 脅威ゲートのフィクスチャを定義する
    @pytest.fixture
    def threat_gate(self) -> Generator[MockSTRIDEThreatGate, None, None]:
        """MockSTRIDEThreatGate フィクスチャを生成する。"""
        # 脅威ゲートをインスタンス化する
        gate = MockSTRIDEThreatGate()
        # フィクスチャを返す
        yield gate

    # 未署名イメージが拒否されることをテストする
    def test_unsigned_image_denied(
        self, admission_controller: MockKyvernoAdmissionController
    ) -> None:
        """未署名イメージが Kyverno アドミッションコントローラーに拒否されることを検証する。"""
        # 未署名イメージを作成する
        image = ContainerImage(
            reference="registry.example.com/app:latest",
            digest="sha256:abc123",
            signature_bundle=None,
        )
        # アドミッション審査を実行する
        result = admission_controller.review(image)
        # 拒否されたことを確認する
        assert result["allowed"] is False
        # ステータスコードが 403 であることを確認する
        assert result["status"]["code"] == 403

    # 署名済みイメージが許可されることをテストする
    def test_signed_image_allowed(
        self, admission_controller: MockKyvernoAdmissionController
    ) -> None:
        """署名済みイメージが Kyverno アドミッションコントローラーに許可されることを検証する。"""
        # 署名済みイメージを作成する
        image = ContainerImage(
            reference="registry.example.com/trusted:v1.0",
            digest="sha256:def456",
            signature_bundle={
                "mediaType": "application/vnd.dev.sigstore.bundle+json;version=0.3",
                "verificationMaterial": {"certificate": "cert_data"},
                "messageSignature": {"signature": "sig", "messageDigest": {}},
            },
        )
        # アドミッション審査を実行する
        result = admission_controller.review(image)
        # 許可されたことを確認する
        assert result["allowed"] is True
        # ステータスコードが 200 であることを確認する
        assert result["status"]["code"] == 200

    # 複数の未署名・署名済みイメージの統計が正確であることをテストする
    def test_admission_stats_accurate(
        self, admission_controller: MockKyvernoAdmissionController
    ) -> None:
        """混在した審査の統計が正確であることを検証する。"""
        # 4 件の未署名イメージを審査する
        for i in range(4):
            # 未署名イメージを審査する
            admission_controller.review(ContainerImage(
                reference=f"unsigned_{i}:latest",
                digest=f"sha256:u{i}",
                signature_bundle=None,
            ))
        # 3 件の署名済みイメージを審査する
        for i in range(3):
            # 署名済みイメージを審査する
            admission_controller.review(ContainerImage(
                reference=f"signed_{i}:v1.0",
                digest=f"sha256:s{i}",
                signature_bundle={
                    "mediaType": "application/vnd.dev.sigstore.bundle+json;version=0.3",
                    "verificationMaterial": {"certificate": "c"},
                    "messageSignature": {"signature": "s", "messageDigest": {}},
                },
            ))
        # 統計を確認する
        stats = admission_controller.stats
        # 拒否が 4 件であることを確認する
        assert stats["denied"] == 4
        # 許可が 3 件であることを確認する
        assert stats["allowed"] == 3

    # STRIDE HIGH 脅威がゲートをブロックすることをテストする
    def test_stride_high_threat_blocks_gate(
        self, threat_gate: MockSTRIDEThreatGate
    ) -> None:
        """HIGH 重大度の STRIDE 脅威が未緩和でゲートをブロックすることを検証する。"""
        # HIGH 重大度の脅威を登録する（緩和策なし）
        threat_gate.register_threat(
            threat_id="STRIDE-S-001",
            severity="HIGH",
            description="Spoofing test threat",
        )
        # ゲートをチェックする
        gate_result = threat_gate.check_gate()
        # ゲートがブロックされていることを確認する
        assert gate_result["gate_passed"] is False
        # ブロックされた脅威 ID が含まれていることを確認する
        assert "STRIDE-S-001" in gate_result["blocking_threats"]

    # 全脅威が緩和済みならゲートが通過することをテストする
    def test_stride_all_mitigated_gate_passes(
        self, threat_gate: MockSTRIDEThreatGate
    ) -> None:
        """全 HIGH/CRITICAL 脅威が緩和済みならゲートが通過することを検証する。"""
        # HIGH 重大度の脅威を登録する
        threat_gate.register_threat(
            threat_id="STRIDE-T-001",
            severity="HIGH",
            description="Tampering test threat",
        )
        # CRITICAL 重大度の脅威を登録する
        threat_gate.register_threat(
            threat_id="STRIDE-E-001",
            severity="CRITICAL",
            description="Elevation test threat",
        )
        # 脅威 T-001 に緩和策を登録する
        threat_gate.register_mitigation(
            mitigation_id="MIT-001",
            threat_id="STRIDE-T-001",
            description="Implement input validation",
        )
        # 脅威 E-001 に緩和策を登録する
        threat_gate.register_mitigation(
            mitigation_id="MIT-002",
            threat_id="STRIDE-E-001",
            description="Enforce RBAC policies",
        )
        # ゲートをチェックする
        gate_result = threat_gate.check_gate()
        # ゲートが通過することを確認する
        assert gate_result["gate_passed"] is True


# security シナリオリプレイの第三拡張テストクラスを定義する
class TestSecurityScenarioReplayExtended3:
    """コンテナ署名・Kyverno 管理・STRIDE 脅威ゲートのシナリオ追加テスト群（第三拡張）。"""

    # STRIDE 脅威ゲートがすべての脅威が緩和済みの場合に pass することをテストする
    def test_scenario_stride_all_mitigated_gate_passes(self) -> None:
        """全 STRIDE 脅威が緩和済みの場合にゲートが通過することを検証する。"""
        # 脅威ゲートをインスタンス化する
        gate = MockSTRIDEThreatGate()
        # 脅威を登録する
        gate.register_threat("STRIDE-S-001", "HIGH", "Spoofing attack vector")
        # 緩和策を登録する
        gate.register_mitigation("MIT-001", "STRIDE-S-001", "Implement auth")
        # ゲートを確認する
        result = gate.check_gate()
        # ゲートが通過することを確認する
        assert result.get("gate_passed") is True

    # Kyverno 管理者が署名なしイメージを拒否することをテストする
    def test_scenario_admission_unsigned_denied(self) -> None:
        """Kyverno 管理者が署名なしイメージを拒否することを検証する。"""
        # Kyverno 管理者をインスタンス化する
        controller = MockKyvernoAdmissionController(require_signature=True)
        # 署名なしイメージを定義する
        unsigned_img = ContainerImage(
            reference="registry.example.com/app:latest",
            digest="sha256:aaaa0000",
            signature_bundle=None,
        )
        # レビューを実行する
        result = controller.review(unsigned_img)
        # 拒否されることを確認する
        assert result.get("allowed") is False

    # Kyverno 管理者が署名済みイメージを許可することをテストする
    def test_scenario_admission_signed_allowed(self) -> None:
        """Kyverno 管理者が署名済みイメージを許可することを検証する。"""
        # Kyverno 管理者をインスタンス化する
        controller = MockKyvernoAdmissionController(require_signature=True)
        # 署名済みイメージを定義する
        signed_img = ContainerImage(
            reference="registry.example.com/app:v1.2.3",
            digest="sha256:bbbb1111",
            signature_bundle={
                "media_type": "application/vnd.dev.cosign.simplesigning.v1+json",
                "signature": "valid_signature_here",
                "certificate": "-----BEGIN CERT-----",
            },
        )
        # レビューを実行する
        result = controller.review(signed_img)
        # 許可されることを確認する
        assert result.get("allowed") is True

    # STRIDE 脅威ゲートが緩和なしの高リスク脅威をブロックすることをテストする
    def test_scenario_stride_unmitigated_blocks_gate(self) -> None:
        """緩和されていない高リスク STRIDE 脅威がゲートをブロックすることを検証する。"""
        # 脅威ゲートをインスタンス化する
        gate = MockSTRIDEThreatGate()
        # 高リスク脅威を複数登録する
        gate.register_threat("STRIDE-D-001", "CRITICAL", "Denial of service")
        # 緩和策を登録しない
        result = gate.check_gate()
        # ゲートがブロックされることを確認する
        assert result.get("gate_passed") is False
        # blocking_threats に脅威が含まれることを確認する
        assert len(result.get("blocking_threats", [])) >= 1

    # Kyverno 管理者の統計が正確であることをテストする
    def test_scenario_admission_stats_correct(self) -> None:
        """Kyverno 管理者の統計が複数のレビュー後も正確であることを検証する。"""
        # Kyverno 管理者をインスタンス化する
        controller = MockKyvernoAdmissionController(require_signature=True)
        # 2 つの署名済みイメージを許可する
        for i in range(2):
            # 署名済みイメージを定義する
            img = ContainerImage(
                reference=f"registry.example.com/app:v{i}",
                digest=f"sha256:signed{i:04d}",
                signature_bundle={"media_type": "cosign", "signature": f"sig_{i}"},
            )
            # レビューを実行する
            controller.review(img)
        # 2 つの署名なしイメージを拒否する
        for i in range(2):
            # 署名なしイメージを定義する
            img = ContainerImage(
                reference=f"registry.example.com/unverified:v{i}",
                digest=f"sha256:unsigned{i:04d}",
                signature_bundle=None,
            )
            # レビューを実行する
            controller.review(img)
        # 統計を確認する
        stats = controller.stats
        # 許可が 2 件であることを確認する
        assert stats.get("allowed") == 2
        # 拒否が 2 件であることを確認する
        assert stats.get("denied") == 2
