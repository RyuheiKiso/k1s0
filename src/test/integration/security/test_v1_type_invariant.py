"""src/test/integration/security/test_v1_type_invariant.py

security 軸の v1_type_invariant 検証クラスに対するインテグレーションテスト。
STRIDE 脅威 ID 命名規則・cosign バンドルフィールド・Kyverno ポリシー構造・
OpenBao トランジットキーのローテーションメタデータを検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: security_type_invariant_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import re
import json
import uuid
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum


# STRIDE 脅威カテゴリの列挙型を定義する
class STRIDECategory(str, Enum):
    """STRIDE 脅威カテゴリ。"""

    # なりすましを表す脅威カテゴリを定義する
    SPOOFING = "S"
    # 改ざんを表す脅威カテゴリを定義する
    TAMPERING = "T"
    # 否認を表す脅威カテゴリを定義する
    REPUDIATION = "R"
    # 情報漏洩を表す脅威カテゴリを定義する
    INFO_DISCLOSURE = "I"
    # サービス拒否を表す脅威カテゴリを定義する
    DENIAL_OF_SERVICE = "D"
    # 権限昇格を表す脅威カテゴリを定義する
    ELEVATION_OF_PRIVILEGE = "E"


# STRIDE 脅威 ID の命名規則パターンを定義する
# 例: STRIDE-S-001, STRIDE-T-042, STRIDE-E-100
STRIDE_ID_PATTERN = re.compile(
    r"^STRIDE-[STRIDE]-\d{3}$"
)


# STRIDE 脅威の重大度を表す列挙型を定義する
class ThreatSeverity(str, Enum):
    """STRIDE 脅威の重大度。"""

    # 重大な脅威を表す重大度を定義する
    CRITICAL = "CRITICAL"
    # 高い脅威を表す重大度を定義する
    HIGH = "HIGH"
    # 中程度の脅威を表す重大度を定義する
    MEDIUM = "MEDIUM"
    # 低い脅威を表す重大度を定義する
    LOW = "LOW"


# STRIDE 脅威エントリを表すデータクラスを定義する
@dataclass
class STRIDEThreat:
    """STRIDE 脅威エントリのデータ型。"""

    # 脅威 ID を保持するフィールドを定義する
    threat_id: str
    # 脅威カテゴリを保持するフィールドを定義する
    category: STRIDECategory
    # 脅威の説明を保持するフィールドを定義する
    description: str
    # 脅威の重大度を保持するフィールドを定義する
    severity: ThreatSeverity
    # 緩和策 ID のリストを保持するフィールドを定義する
    mitigations: list[str] = field(default_factory=list)

    # 脅威 ID の命名規則を検証するメソッドを定義する
    def validate_id_format(self) -> None:
        """脅威 ID が STRIDE-{C}-{NNN} 命名規則に従うことを検証する。"""
        # 脅威 ID が命名規則に一致しない場合はエラーを発生させる
        if not STRIDE_ID_PATTERN.match(self.threat_id):
            # バリデーションエラーを発生させる
            raise ValueError(
                f"STRIDE threat ID must match STRIDE-{{C}}-{{NNN}} pattern, "
                f"got: {self.threat_id!r}"
            )
        # 脅威 ID のカテゴリ文字がカテゴリと一致することを確認する
        id_category = self.threat_id.split("-")[1]
        # カテゴリが一致しない場合はエラーを発生させる
        if id_category != self.category.value:
            # カテゴリ不一致のエラーを発生させる
            raise ValueError(
                f"Threat ID category {id_category!r} does not match "
                f"declared category {self.category.value!r}"
            )


# cosign バンドルを表すデータクラスを定義する
@dataclass
class CosignBundle:
    """cosign 署名バンドルのデータ型。"""

    # バンドルのメディアタイプを保持するフィールドを定義する
    media_type: str
    # 検証証明書を保持するフィールドを定義する
    verification_material: dict[str, Any]
    # メッセージ署名を保持するフィールドを定義する
    message_signature: dict[str, Any]

    # バンドルの必須フィールドを検証するメソッドを定義する
    def validate(self) -> None:
        """cosign バンドルの必須フィールドを検証する。"""
        # media_type が空でないことを確認する
        if not self.media_type.strip():
            # メディアタイプが空の場合はエラーを発生させる
            raise ValueError("media_type must not be empty")
        # verification_material に certificate フィールドが含まれることを確認する
        if "certificate" not in self.verification_material:
            # certificate フィールドが欠落している場合はエラーを発生させる
            raise ValueError(
                "verification_material must contain 'certificate' field"
            )
        # message_signature に signature フィールドが含まれることを確認する
        if "signature" not in self.message_signature:
            # signature フィールドが欠落している場合はエラーを発生させる
            raise ValueError(
                "message_signature must contain 'signature' field"
            )
        # message_signature に messageDigest フィールドが含まれることを確認する
        if "messageDigest" not in self.message_signature:
            # messageDigest フィールドが欠落している場合はエラーを発生させる
            raise ValueError(
                "message_signature must contain 'messageDigest' field"
            )


# Kyverno ポリシーを表すデータクラスを定義する
@dataclass
class KyvernoPolicy:
    """Kyverno ポリシーのデータ型。"""

    # ポリシーの API バージョンを保持するフィールドを定義する
    api_version: str
    # ポリシーの種類を保持するフィールドを定義する
    kind: str
    # ポリシーのメタデータを保持するフィールドを定義する
    metadata: dict[str, Any]
    # ポリシースペックを保持するフィールドを定義する
    spec: dict[str, Any]

    # ポリシー構造の不変条件を検証するメソッドを定義する
    def validate(self) -> None:
        """Kyverno ポリシーの必須構造を検証する。"""
        # apiVersion が kyverno.io で始まることを確認する
        if not self.api_version.startswith("kyverno.io/"):
            # API バージョンが不正な場合はエラーを発生させる
            raise ValueError(
                f"apiVersion must start with 'kyverno.io/', got: {self.api_version!r}"
            )
        # kind が ClusterPolicy または Policy であることを確認する
        if self.kind not in ("ClusterPolicy", "Policy"):
            # kind が不正な場合はエラーを発生させる
            raise ValueError(
                f"kind must be 'ClusterPolicy' or 'Policy', got: {self.kind!r}"
            )
        # spec に rules フィールドが含まれることを確認する
        if "rules" not in self.spec:
            # rules フィールドが欠落している場合はエラーを発生させる
            raise ValueError("spec must contain 'rules' field")
        # rules が空でないことを確認する
        if not self.spec["rules"]:
            # rules が空の場合はエラーを発生させる
            raise ValueError("spec.rules must not be empty")
        # 各ルールに name フィールドがあることを確認する
        for i, rule in enumerate(self.spec["rules"]):
            # name フィールドが欠落している場合はエラーを発生させる
            if "name" not in rule:
                # name フィールドが欠落している場合はエラーを発生させる
                raise ValueError(
                    f"spec.rules[{i}] must have 'name' field"
                )


# OpenBao トランジットキーのメタデータを表すデータクラスを定義する
@dataclass
class OpenBaoTransitKey:
    """OpenBao Transit キーのメタデータ型。"""

    # キー名を保持するフィールドを定義する
    key_name: str
    # キータイプを保持するフィールドを定義する
    key_type: str
    # 最小復号バージョンを保持するフィールドを定義する
    min_decryption_version: int
    # 最小暗号化バージョンを保持するフィールドを定義する
    min_encryption_version: int
    # 最新バージョン番号を保持するフィールドを定義する
    latest_version: int
    # ローテーション間隔（秒）を保持するフィールドを定義する
    rotation_period_seconds: int
    # 削除不可フラグを保持するフィールドを定義する
    deletion_allowed: bool = False

    # トランジットキーのメタデータを検証するメソッドを定義する
    def validate(self) -> None:
        """Transit キーの必須メタデータを検証する。"""
        # min_decryption_version が正の整数であることを確認する
        if self.min_decryption_version < 1:
            # バリデーションエラーを発生させる
            raise ValueError(
                f"min_decryption_version must be >= 1, got {self.min_decryption_version}"
            )
        # latest_version が min_decryption_version 以上であることを確認する
        if self.latest_version < self.min_decryption_version:
            # バリデーションエラーを発生させる
            raise ValueError(
                f"latest_version ({self.latest_version}) must be >= "
                f"min_decryption_version ({self.min_decryption_version})"
            )
        # rotation_period_seconds が正の整数であることを確認する
        if self.rotation_period_seconds <= 0:
            # バリデーションエラーを発生させる
            raise ValueError(
                f"rotation_period_seconds must be positive, got {self.rotation_period_seconds}"
            )
        # key_type が許可されたリストに含まれることを確認する
        allowed_types = {"aes256-gcm96", "chacha20-poly1305", "ed25519", "ecdsa-p256"}
        # key_type が許可リストに含まれない場合はエラーを発生させる
        if self.key_type not in allowed_types:
            # バリデーションエラーを発生させる
            raise ValueError(
                f"key_type must be one of {allowed_types}, got: {self.key_type!r}"
            )


# security 軸型不変条件テストスイートクラスを定義する
class TestSecurityTypeInvariant:
    """security 軸 v1_type_invariant の検証テストスイート。"""

    # STRIDE 脅威の命名規則が正しいことを検証するテストを定義する
    def test_stride_threat_id_valid_format(self) -> None:
        """有効な STRIDE 脅威 ID が命名規則を満たすことを検証する。"""
        # 有効な STRIDE 脅威を生成する
        threat = STRIDEThreat(
            threat_id="STRIDE-S-001",
            category=STRIDECategory.SPOOFING,
            description="Unauthorized access via stolen credentials",
            severity=ThreatSeverity.HIGH,
        )
        # 命名規則の検証がパスすることを確認する
        threat.validate_id_format()

    # 各 STRIDE カテゴリの脅威 ID が正しいことを検証するテストを定義する
    @pytest.mark.parametrize(
        "threat_id,category",
        [
            # なりすましカテゴリの脅威 ID を定義する
            ("STRIDE-S-001", STRIDECategory.SPOOFING),
            # 改ざんカテゴリの脅威 ID を定義する
            ("STRIDE-T-042", STRIDECategory.TAMPERING),
            # 否認カテゴリの脅威 ID を定義する
            ("STRIDE-R-010", STRIDECategory.REPUDIATION),
            # 情報漏洩カテゴリの脅威 ID を定義する
            ("STRIDE-I-099", STRIDECategory.INFO_DISCLOSURE),
            # サービス拒否カテゴリの脅威 ID を定義する
            ("STRIDE-D-003", STRIDECategory.DENIAL_OF_SERVICE),
            # 権限昇格カテゴリの脅威 ID を定義する
            ("STRIDE-E-007", STRIDECategory.ELEVATION_OF_PRIVILEGE),
        ],
    )
    def test_stride_all_categories_valid(
        self, threat_id: str, category: STRIDECategory
    ) -> None:
        """全 STRIDE カテゴリの脅威 ID が命名規則を満たすことを検証する。"""
        # 各カテゴリの脅威を生成する
        threat = STRIDEThreat(
            threat_id=threat_id,
            category=category,
            description=f"Test threat for {category.value}",
            severity=ThreatSeverity.MEDIUM,
        )
        # 命名規則の検証がパスすることを確認する
        threat.validate_id_format()

    # 無効な STRIDE 脅威 ID が拒否されることを検証するテストを定義する
    def test_stride_invalid_id_format_rejected(self) -> None:
        """無効な STRIDE 脅威 ID が命名規則違反として拒否されることを検証する。"""
        # 無効な脅威 ID のリストを定義する
        invalid_ids = [
            # 数字部分が短すぎる場合は拒否する
            "STRIDE-S-01",
            # カテゴリなしの場合は拒否する
            "STRIDE-001",
            # プレフィックスが異なる場合は拒否する
            "THREAT-S-001",
            # 空文字列は拒否する
            "",
            # 小文字のカテゴリは拒否する
            "STRIDE-s-001",
        ]
        # 各無効な ID でエラーが発生することを確認する
        for invalid_id in invalid_ids:
            # 無効な脅威を生成する
            threat = STRIDEThreat(
                threat_id=invalid_id,
                category=STRIDECategory.SPOOFING,
                description="Invalid threat",
                severity=ThreatSeverity.LOW,
            )
            # 命名規則の検証が失敗することを確認する
            with pytest.raises(ValueError):
                # バリデーションを実行する
                threat.validate_id_format()

    # 有効な cosign バンドルが検証を通過することを確認するテストを定義する
    def test_cosign_bundle_valid_passes(self) -> None:
        """必須フィールドを持つ cosign バンドルが検証を通過することを検証する。"""
        # 有効な cosign バンドルを生成する
        bundle = CosignBundle(
            media_type="application/vnd.dev.sigstore.bundle+json;version=0.3",
            verification_material={
                "certificate": "LS0tLS1CRUdJTiBDRVJUSUZJQ0FU...",
                "tlogEntries": [],
            },
            message_signature={
                "signature": "MEUCIQDexample...",
                "messageDigest": {
                    "algorithm": "SHA2_256",
                    "digest": "abc123...",
                },
            },
        )
        # バリデーションがパスすることを確認する
        bundle.validate()

    # certificate フィールドが欠落した cosign バンドルが拒否されることを検証するテストを定義する
    def test_cosign_bundle_missing_certificate_rejected(self) -> None:
        """certificate フィールドが欠落した cosign バンドルが拒否されることを検証する。"""
        # certificate フィールドが欠落したバンドルを生成する
        bundle = CosignBundle(
            media_type="application/vnd.dev.sigstore.bundle+json;version=0.3",
            verification_material={},
            message_signature={
                "signature": "MEUCIQDexample...",
                "messageDigest": {"algorithm": "SHA2_256", "digest": "abc123"},
            },
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="certificate"):
            # バリデーションを実行する
            bundle.validate()

    # 有効な Kyverno ポリシーが検証を通過することを確認するテストを定義する
    def test_kyverno_policy_valid_passes(self) -> None:
        """必須構造を持つ Kyverno ポリシーが検証を通過することを検証する。"""
        # 有効な Kyverno ポリシーを生成する
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="ClusterPolicy",
            metadata={"name": "require-cosign-signature"},
            spec={
                "rules": [
                    {
                        "name": "check-image-signature",
                        "match": {"any": [{"resources": {"kinds": ["Pod"]}}]},
                        "verifyImages": [],
                    }
                ]
            },
        )
        # バリデーションがパスすることを確認する
        policy.validate()

    # spec.rules が空の Kyverno ポリシーが拒否されることを検証するテストを定義する
    def test_kyverno_policy_empty_rules_rejected(self) -> None:
        """spec.rules が空の Kyverno ポリシーが拒否されることを検証する。"""
        # rules が空のポリシーを生成する
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="ClusterPolicy",
            metadata={"name": "empty_rules_policy"},
            spec={"rules": []},
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="empty"):
            # バリデーションを実行する
            policy.validate()

    # 有効な OpenBao トランジットキーが検証を通過することを確認するテストを定義する
    def test_openbao_transit_key_valid_passes(self) -> None:
        """有効なメタデータを持つ OpenBao Transit キーが検証を通過することを検証する。"""
        # 有効なトランジットキーを生成する
        key = OpenBaoTransitKey(
            key_name="k1s0-kek-v1",
            key_type="aes256-gcm96",
            min_decryption_version=1,
            min_encryption_version=2,
            latest_version=3,
            rotation_period_seconds=86400,
        )
        # バリデーションがパスすることを確認する
        key.validate()

    # ローテーション期間が 0 の Transit キーが拒否されることを検証するテストを定義する
    def test_openbao_transit_key_zero_rotation_period_rejected(self) -> None:
        """rotation_period_seconds が 0 の Transit キーが拒否されることを検証する。"""
        # ローテーション期間が 0 のキーを生成する
        key = OpenBaoTransitKey(
            key_name="bad-key",
            key_type="aes256-gcm96",
            min_decryption_version=1,
            min_encryption_version=1,
            latest_version=1,
            rotation_period_seconds=0,
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="rotation_period_seconds"):
            # バリデーションを実行する
            key.validate()

    # 不正な key_type を持つ Transit キーが拒否されることを検証するテストを定義する
    def test_openbao_transit_key_invalid_type_rejected(self) -> None:
        """許可されていない key_type を持つ Transit キーが拒否されることを検証する。"""
        # 不正な key_type を持つキーを生成する
        key = OpenBaoTransitKey(
            key_name="unknown-key",
            key_type="rsa-2048",
            min_decryption_version=1,
            min_encryption_version=1,
            latest_version=1,
            rotation_period_seconds=3600,
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="key_type"):
            # バリデーションを実行する
            key.validate()


# security 軸型不変条件追加テストスイートクラスを定義する
class TestSecurityTypeInvariantExtended:
    """security 軸 v1_type_invariant の追加検証テストスイート。

    STRIDE 脅威の緩和策フィールド・Kyverno ポリシーの rule name・OpenBao の version 整合性を検証する。
    """

    # STRIDE 脅威が緩和策リストを保持できることを検証するテストを定義する
    def test_stride_threat_holds_mitigations(self) -> None:
        """STRIDE 脅威が複数の緩和策を保持できることを検証する。"""
        # 複数の緩和策を持つ脅威を生成する
        threat = STRIDEThreat(
            threat_id="STRIDE-T-010",
            category=STRIDECategory.TAMPERING,
            description="Man-in-the-middle attack on API",
            severity=ThreatSeverity.HIGH,
            mitigations=["MIT-001", "MIT-002", "MIT-003"],
        )
        # 命名規則の検証がパスすることを確認する
        threat.validate_id_format()
        # 緩和策の数が 3 であることを確認する
        assert len(threat.mitigations) == 3

    # Kyverno ポリシーの rules 内の各ルールが name を持つことを検証するテストを定義する
    def test_kyverno_policy_rules_all_have_names(self) -> None:
        """Kyverno ポリシーの全ルールが name フィールドを持つことを検証する。"""
        # 複数のルールを持つポリシーを生成する
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="ClusterPolicy",
            metadata={"name": "multi_rule_policy"},
            spec={
                "rules": [
                    {"name": "rule_check_image", "match": {}},
                    {"name": "rule_check_labels", "match": {}},
                    {"name": "rule_check_resources", "match": {}},
                ]
            },
        )
        # バリデーションがパスすることを確認する
        policy.validate()
        # ルール数が 3 であることを確認する
        assert len(policy.spec["rules"]) == 3

    # name がないルールを含む Kyverno ポリシーが拒否されることを検証するテストを定義する
    def test_kyverno_policy_rule_without_name_rejected(self) -> None:
        """name フィールドがないルールを含む Kyverno ポリシーが拒否されることを検証する。"""
        # name なしのルールを含むポリシーを生成する
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="ClusterPolicy",
            metadata={"name": "nameless_rule_policy"},
            spec={
                "rules": [
                    {"match": {}},
                ]
            },
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="name"):
            # バリデーションを実行する
            policy.validate()

    # OpenBao の latest_version と min_decryption_version の整合性を検証するテストを定義する
    def test_openbao_transit_key_version_consistency(self) -> None:
        """latest_version が min_decryption_version 以上であることを検証する。"""
        # latest_version < min_decryption_version のキーを生成する
        invalid_key = OpenBaoTransitKey(
            key_name="inconsistent_key",
            key_type="aes256-gcm96",
            min_decryption_version=5,
            min_encryption_version=5,
            latest_version=3,
            rotation_period_seconds=3600,
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="latest_version"):
            # バリデーションを実行する
            invalid_key.validate()

    # 複数の有効な STRIDE 脅威が登録できることを検証するテストを定義する
    def test_stride_multiple_threats_all_valid(self) -> None:
        """複数の STRIDE 脅威が全て命名規則を満たすことを検証する。"""
        # 各カテゴリの脅威リストを生成する
        threats = [
            STRIDEThreat("STRIDE-S-001", STRIDECategory.SPOOFING, "Spoofing", ThreatSeverity.HIGH),
            STRIDEThreat("STRIDE-T-001", STRIDECategory.TAMPERING, "Tampering", ThreatSeverity.MEDIUM),
            STRIDEThreat("STRIDE-R-001", STRIDECategory.REPUDIATION, "Repudiation", ThreatSeverity.LOW),
            STRIDEThreat("STRIDE-I-001", STRIDECategory.INFO_DISCLOSURE, "Info", ThreatSeverity.MEDIUM),
            STRIDEThreat("STRIDE-D-001", STRIDECategory.DENIAL_OF_SERVICE, "DoS", ThreatSeverity.HIGH),
            STRIDEThreat("STRIDE-E-001", STRIDECategory.ELEVATION_OF_PRIVILEGE, "EoP", ThreatSeverity.CRITICAL),
        ]
        # 全脅威の命名規則検証がパスすることを確認する
        for threat in threats:
            # 命名規則の検証を実行する
            threat.validate_id_format()
        # 全 6 件の脅威が登録されたことを確認する
        assert len(threats) == 6


# security 軸型不変条件第 2 拡張テストスイートクラスを定義する
class TestSecurityTypeInvariantExtended2:
    """security 軸 v1_type_invariant の第 2 拡張テストスイート。追加の型不変条件を検証する。"""

    # 全 STRIDE カテゴリに対して有効な脅威 ID が生成できることをテストする
    @pytest.mark.parametrize(
        "category,threat_id",
        [
            # なりすまし (S) のケースを定義する
            (STRIDECategory.SPOOFING, "STRIDE-S-001"),
            # 改ざん (T) のケースを定義する
            (STRIDECategory.TAMPERING, "STRIDE-T-042"),
            # 否認 (R) のケースを定義する
            (STRIDECategory.REPUDIATION, "STRIDE-R-100"),
            # 情報漏洩 (I) のケースを定義する
            (STRIDECategory.INFO_DISCLOSURE, "STRIDE-I-007"),
            # サービス拒否 (D) のケースを定義する
            (STRIDECategory.DENIAL_OF_SERVICE, "STRIDE-D-050"),
            # 権限昇格 (E) のケースを定義する
            (STRIDECategory.ELEVATION_OF_PRIVILEGE, "STRIDE-E-999"),
        ],
    )
    def test_stride_all_categories_valid(
        self, category: STRIDECategory, threat_id: str
    ) -> None:
        """全 STRIDE カテゴリに対して有効な脅威 ID が生成できることを検証する。"""
        # 脅威エントリを生成する
        threat = STRIDEThreat(
            threat_id=threat_id,
            category=category,
            description=f"Test threat for category {category.value}",
            severity=ThreatSeverity.MEDIUM,
        )
        # バリデーションがパスすることを確認する
        threat.validate_id_format()

    # STRIDE カテゴリが脅威 ID と不一致の場合に拒否されることをテストする
    def test_stride_category_mismatch_rejected(self) -> None:
        """STRIDE カテゴリが脅威 ID のカテゴリ文字と不一致の場合に拒否されることを検証する。"""
        # カテゴリが S なのに ID が T の脅威を生成する
        threat = STRIDEThreat(
            threat_id="STRIDE-T-001",
            category=STRIDECategory.SPOOFING,
            description="Category mismatch test",
            severity=ThreatSeverity.LOW,
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="category"):
            # カテゴリ不一致のバリデーションを実行する
            threat.validate_id_format()

    # 無効な形式の STRIDE ID が拒否されることをテストする
    @pytest.mark.parametrize(
        "invalid_id",
        [
            # プレフィックスなしの ID を無効として定義する
            "S-001",
            # カテゴリなしの ID を無効として定義する
            "STRIDE-001",
            # 数字が 4 桁の ID を無効として定義する
            "STRIDE-S-1234",
            # 大文字以外のカテゴリを無効として定義する
            "STRIDE-s-001",
            # 空文字列を無効として定義する
            "",
            # 数字なしの ID を無効として定義する
            "STRIDE-S-",
        ],
    )
    def test_stride_invalid_id_formats_rejected(self, invalid_id: str) -> None:
        """無効な形式の STRIDE ID が拒否されることを検証する。"""
        # 無効な ID を持つ脅威を生成する
        threat = STRIDEThreat(
            threat_id=invalid_id,
            category=STRIDECategory.SPOOFING,
            description="Invalid format test",
            severity=ThreatSeverity.LOW,
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError):
            # バリデーションを実行する
            threat.validate_id_format()

    # cosign バンドルが全フィールドを持つ場合に有効であることをテストする
    def test_cosign_bundle_all_fields_valid(self) -> None:
        """cosign バンドルが全フィールドを持つ場合にバリデーションが通過することを検証する。"""
        # cosign バンドルを生成する
        bundle = CosignBundle(
            media_type="application/vnd.dev.cosign.bundle+json;version=0.3",
            verification_material={
                "certificate": "-----BEGIN CERTIFICATE-----\n...cert...\n-----END CERTIFICATE-----",
                "chain": [],
            },
            message_signature={
                "signature": "base64encodedSig==",
                "messageDigest": {
                    "algorithm": "SHA2_256",
                    "digest": "abc123",
                },
            },
        )
        # バリデーションがパスすることを確認する
        bundle.validate()

    # cosign バンドルが空のメディアタイプを拒否することをテストする
    def test_cosign_bundle_empty_media_type_rejected(self) -> None:
        """cosign バンドルが空のメディアタイプを拒否することを検証する。"""
        # 空のメディアタイプを持つバンドルを生成する
        bundle = CosignBundle(
            media_type="   ",
            verification_material={"certificate": "cert"},
            message_signature={"signature": "sig", "messageDigest": "digest"},
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="media_type"):
            # バリデーションを実行する
            bundle.validate()

    # Kyverno ポリシーが空のルールリストを拒否することをテストする
    def test_kyverno_policy_empty_rules_rejected(self) -> None:
        """Kyverno ポリシーが空のルールリストを拒否することを検証する。"""
        # 空のルールリストを持つポリシーを生成する
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="ClusterPolicy",
            metadata={"name": "empty-rules-policy"},
            spec={"rules": []},
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="rules"):
            # バリデーションを実行する
            policy.validate()

    # Kyverno ポリシーが無効な API バージョンを拒否することをテストする
    def test_kyverno_policy_invalid_api_version_rejected(self) -> None:
        """Kyverno ポリシーが無効な API バージョンを拒否することを検証する。"""
        # 無効な API バージョンを持つポリシーを生成する
        policy = KyvernoPolicy(
            api_version="apps/v1",
            kind="ClusterPolicy",
            metadata={"name": "invalid-api-policy"},
            spec={"rules": [{"name": "rule1"}]},
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="apiVersion"):
            # バリデーションを実行する
            policy.validate()

    # Kyverno ポリシーが有効な構造を持つ場合にバリデーションが通過することをテストする
    def test_kyverno_policy_valid_structure(self) -> None:
        """Kyverno ポリシーが有効な構造を持つ場合にバリデーションが通過することを検証する。"""
        # 有効なポリシーを生成する
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="ClusterPolicy",
            metadata={"name": "require-signed-images", "namespace": ""},
            spec={
                "rules": [
                    {
                        "name": "check-image-signature",
                        "match": {"resources": {"kinds": ["Pod"]}},
                        "verifyImages": [{"image": "*"}],
                    }
                ]
            },
        )
        # バリデーションがパスすることを確認する
        policy.validate()


# security 型不変条件の第三拡張テストクラスを定義する
class TestSecurityTypeInvariantExtended3:
    """STRIDE 脅威・CosignBundle・Kyverno ポリシーの型不変条件の追加テスト群（第三拡張）。"""

    # STRIDE 脅威の severity が全種別で有効であることをテストする
    @pytest.mark.parametrize("severity", list(ThreatSeverity))
    def test_stride_threat_all_severities_valid(self, severity: ThreatSeverity) -> None:
        """STRIDE 脅威が全ての severity 種別で有効であることを検証する。"""
        # STRIDE 脅威を生成する
        threat = STRIDEThreat(
            threat_id=f"STRIDE-S-001",
            category=STRIDECategory.S,
            description="Spoofing threat",
            severity=severity,
            mitigations=["mitigation_1"],
        )
        # バリデーションが成功することを確認する
        threat.validate_id_format()

    # Kyverno ポリシーが Policy kind でも有効であることをテストする
    def test_kyverno_policy_kind_valid(self) -> None:
        """Kyverno ポリシーが 'Policy' kind でも有効であることを検証する。"""
        # Kyverno ポリシーを生成する（Policy kind）
        policy = KyvernoPolicy(
            api_version="kyverno.io/v1",
            kind="Policy",
            metadata={"name": "namespace-policy", "namespace": "default"},
            spec={"rules": [{"name": "block-unsigned"}]},
        )
        # バリデーションが成功することを確認する（例外なし）
        policy.validate()

    # CosignBundle が署名検証に必要なフィールドを持つことをテストする
    def test_cosign_bundle_has_required_fields(self) -> None:
        """CosignBundle が全ての必須フィールドを持つことを検証する。"""
        # CosignBundle を生成する
        bundle = CosignBundle(
            media_type="application/vnd.dev.cosign.simplesigning.v1+json",
            verification_material={"certificate": "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----"},
            message_signature={"signature": "base64encoded==", "messageDigest": {"algorithm": "sha256", "digest": "abc123"}},
        )
        # バリデーションが成功することを確認する（例外なし）
        bundle.validate()
        # media_type が設定されていることを確認する
        assert bundle.media_type != ""

    # STRIDE 脅威の category が ID の 2 番目のセクションと一致することをテストする
    @pytest.mark.parametrize("category,expected_char", [
        (STRIDECategory.S, "S"),
        (STRIDECategory.T, "T"),
        (STRIDECategory.R, "R"),
        (STRIDECategory.I, "I"),
        (STRIDECategory.D, "D"),
        (STRIDECategory.E, "E"),
    ])
    def test_stride_id_matches_category(self, category: STRIDECategory, expected_char: str) -> None:
        """STRIDE 脅威 ID の 2 番目のセクションがカテゴリと一致することを検証する。"""
        # 対応する ID を生成する
        threat_id = f"STRIDE-{expected_char}-001"
        # STRIDE 脅威を生成する
        threat = STRIDEThreat(
            threat_id=threat_id,
            category=category,
            description="Test threat",
            severity=ThreatSeverity.LOW,
            mitigations=[],
        )
        # ID フォーマットが有効であることを確認する
        threat.validate_id_format()

    # Kyverno ポリシーの rules が名前なしの場合に拒否されることをテストする
    def test_kyverno_policy_rule_without_name_rejected(self) -> None:
        """Kyverno ポリシーの rules エントリに name がない場合に拒否されることを検証する。"""
        # name のない rule を含むポリシーを生成する
        with pytest.raises((ValueError, AssertionError)):
            # name なし rule のポリシーを生成する
            policy = KyvernoPolicy(
                api_version="kyverno.io/v1",
                kind="ClusterPolicy",
                metadata={"name": "bad-policy"},
                spec={"rules": [{"description": "rule without name"}]},
            )
            # バリデーションを実行する
            policy.validate()
