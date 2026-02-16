"""認証認可設計.md の仕様準拠テスト。

Keycloak Realm 設定と Vault ポリシーが
設計ドキュメントと一致するかを検証する。
"""
import json
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]


class TestKeycloakRealm:
    """認証認可設計.md: Keycloak Realm 設定の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        assert path.exists()
        self.realm = json.loads(path.read_text(encoding="utf-8"))

    def test_realm_json_exists(self) -> None:
        assert (ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json").exists()

    def test_realm_name(self) -> None:
        assert self.realm["realm"] == "k1s0"

    def test_realm_enabled(self) -> None:
        assert self.realm["enabled"] is True

    def test_ssl_required(self) -> None:
        assert self.realm["sslRequired"] == "external"

    def test_brute_force_protection(self) -> None:
        """認証認可設計.md: ブルートフォース保護が有効。"""
        assert self.realm["bruteForceProtected"] is True
        assert self.realm["failureFactor"] == 5

    def test_access_token_lifespan(self) -> None:
        """認証認可設計.md: Access Token 有効期限 900秒 (15分)。"""
        assert self.realm["accessTokenLifespan"] == 900

    def test_sso_session_idle_timeout(self) -> None:
        assert self.realm["ssoSessionIdleTimeout"] == 1800

    def test_refresh_token_revocation(self) -> None:
        """認証認可設計.md: Refresh Token は再利用不可。"""
        assert self.realm["revokeRefreshToken"] is True
        assert self.realm["refreshTokenMaxReuse"] == 0

    def test_signature_algorithm(self) -> None:
        assert self.realm["defaultSignatureAlgorithm"] == "RS256"


class TestKeycloakClients:
    """認証認可設計.md: Keycloak Client 設定の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        self.clients = {c["clientId"]: c for c in self.realm["clients"]}

    @pytest.mark.parametrize(
        "client_id",
        ["react-spa", "flutter-mobile", "k1s0-cli", "k1s0-bff", "k1s0-service"],
    )
    def test_client_defined(self, client_id: str) -> None:
        """認証認可設計.md: 5つのクライアントが定義されている。"""
        assert client_id in self.clients, f"Client '{client_id}' が定義されていません"

    def test_react_spa_public_client(self) -> None:
        """認証認可設計.md: React SPA は publicClient。"""
        c = self.clients["react-spa"]
        assert c["publicClient"] is True
        assert c["standardFlowEnabled"] is True

    def test_react_spa_pkce(self) -> None:
        """認証認可設計.md: PKCE (S256) を使用。"""
        c = self.clients["react-spa"]
        assert c["attributes"]["pkce.code.challenge.method"] == "S256"

    def test_flutter_mobile_public_client(self) -> None:
        c = self.clients["flutter-mobile"]
        assert c["publicClient"] is True
        assert c["standardFlowEnabled"] is True

    def test_cli_device_flow(self) -> None:
        """認証認可設計.md: CLI は Device Authorization Grant。"""
        c = self.clients["k1s0-cli"]
        assert c["attributes"]["oauth2.device.authorization.grant.enabled"] == "true"

    def test_bff_confidential_client(self) -> None:
        """認証認可設計.md: BFF は Confidential Client。"""
        c = self.clients["k1s0-bff"]
        assert c["publicClient"] is False
        assert c["standardFlowEnabled"] is True

    def test_service_client_credentials(self) -> None:
        """認証認可設計.md: Service は Client Credentials Grant。"""
        c = self.clients["k1s0-service"]
        assert c["publicClient"] is False
        assert c["serviceAccountsEnabled"] is True
        assert c["standardFlowEnabled"] is False

    def test_tier_access_mapper(self) -> None:
        """認証認可設計.md: tier_access カスタムクレームが設定されている。"""
        for client_id in ["react-spa", "flutter-mobile", "k1s0-cli", "k1s0-bff", "k1s0-service"]:
            c = self.clients[client_id]
            mappers = [m["name"] for m in c.get("protocolMappers", [])]
            assert "tier-access-mapper" in mappers, (
                f"Client '{client_id}' に tier-access-mapper がありません"
            )


class TestKeycloakRoles:
    """認証認可設計.md: Keycloak Realm ロールの検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        self.role_names = [r["name"] for r in self.realm["roles"]["realm"]]

    @pytest.mark.parametrize(
        "role",
        ["user", "sys_admin", "sys_operator", "sys_auditor"],
    )
    def test_system_role_defined(self, role: str) -> None:
        assert role in self.role_names, f"Role '{role}' が定義されていません"

    @pytest.mark.parametrize(
        "role",
        ["biz_accounting_admin", "biz_accounting_manager", "biz_accounting_viewer"],
    )
    def test_business_role_defined(self, role: str) -> None:
        assert role in self.role_names, f"Role '{role}' が定義されていません"

    @pytest.mark.parametrize(
        "role",
        ["svc_order_admin", "svc_order_user", "svc_order_viewer"],
    )
    def test_service_role_defined(self, role: str) -> None:
        assert role in self.role_names, f"Role '{role}' が定義されていません"


class TestKeycloakLDAP:
    """認証認可設計.md: LDAP 連携設定の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        providers = self.realm["components"]["org.keycloak.storage.UserStorageProvider"]
        self.ldap = providers[0]

    def test_ldap_provider_exists(self) -> None:
        assert self.ldap["providerId"] == "ldap"

    def test_ldap_vendor_ad(self) -> None:
        """認証認可設計.md: Active Directory 連携。"""
        assert self.ldap["config"]["vendor"] == ["ad"]

    def test_ldap_read_only(self) -> None:
        assert self.ldap["config"]["editMode"] == ["READ_ONLY"]

    def test_ldap_uses_ldaps(self) -> None:
        """認証認可設計.md: LDAPS (TLS) を使用。"""
        assert self.ldap["config"]["connectionUrl"][0].startswith("ldaps://")


class TestKeycloakEvents:
    """認証認可設計.md: イベント設定の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        self.events = self.realm["eventsConfig"]

    def test_events_enabled(self) -> None:
        assert self.events["eventsEnabled"] is True

    def test_admin_events_enabled(self) -> None:
        assert self.events["adminEventsEnabled"] is True

    @pytest.mark.parametrize(
        "event_type",
        ["LOGIN", "LOGIN_ERROR", "LOGOUT", "CODE_TO_TOKEN"],
    )
    def test_key_event_types_enabled(self, event_type: str) -> None:
        assert event_type in self.events["enabledEventTypes"]


class TestVaultPolicies:
    """認証認可設計.md: Vault ポリシーの検証。"""

    @pytest.mark.parametrize("policy", ["system.hcl", "business.hcl", "service.hcl"])
    def test_policy_file_exists(self, policy: str) -> None:
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / policy
        assert path.exists(), f"policies/{policy} が存在しません"

    def test_system_policy_kv_path(self) -> None:
        """認証認可設計.md: system Tier は secret/data/k1s0/system/* にアクセス可。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "system.hcl"
        content = path.read_text(encoding="utf-8")
        assert "secret/data/k1s0/system/*" in content

    def test_system_policy_db_path(self) -> None:
        """認証認可設計.md: system Tier は database/creds/system-* にアクセス可。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "system.hcl"
        content = path.read_text(encoding="utf-8")
        assert "database/creds/system-*" in content

    def test_system_policy_pki(self) -> None:
        """認証認可設計.md: system Tier は PKI 証明書を発行可。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "system.hcl"
        content = path.read_text(encoding="utf-8")
        assert "pki/issue/system" in content

    def test_business_policy_kv_path(self) -> None:
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "business.hcl"
        content = path.read_text(encoding="utf-8")
        assert "secret/data/k1s0/business/*" in content

    def test_business_policy_kafka_sasl(self) -> None:
        """認証認可設計.md: business Tier は Kafka SASL 認証情報にアクセス可。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "business.hcl"
        content = path.read_text(encoding="utf-8")
        assert "kafka/sasl" in content

    def test_service_policy_kv_path(self) -> None:
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "service.hcl"
        content = path.read_text(encoding="utf-8")
        assert "secret/data/k1s0/service/*" in content

    def test_service_policy_kafka_sasl(self) -> None:
        """認証認可設計.md: service Tier は Kafka SASL 認証情報にアクセス可。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "policies" / "service.hcl"
        content = path.read_text(encoding="utf-8")
        assert "kafka/sasl" in content
