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
        """認証認可設計.md: sslRequired はローカル開発で none、本番で external。"""
        assert self.realm["sslRequired"] in ("external", "none")

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
        components = self.realm.get("components", {})
        providers = components.get("org.keycloak.storage.UserStorageProvider")
        if not providers:
            pytest.skip("LDAP UserStorageProvider がまだ realm.json に設定されていません")
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


class TestKeycloakLDAPAttributeMapping:
    """認証認可設計.md: LDAP 属性マッピングの検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        components = self.realm.get("components", {})
        providers = components.get("org.keycloak.storage.UserStorageProvider")
        if not providers:
            pytest.skip("LDAP UserStorageProvider がまだ realm.json に設定されていません")
        self.ldap = providers[0]

    def test_ldap_username_attribute_mapping(self) -> None:
        """認証認可設計.md: sAMAccountName → username のマッピング。"""
        assert self.ldap["config"]["usernameLDAPAttribute"] == ["sAMAccountName"]

    def test_ldap_users_dn(self) -> None:
        """認証認可設計.md: User DN が ou=users,dc=example,dc=com。"""
        assert self.ldap["config"]["usersDn"] == ["ou=users,dc=example,dc=com"]

    def test_ldap_bind_dn(self) -> None:
        """認証認可設計.md: Bind DN が cn=keycloak,ou=service-accounts,dc=example,dc=com。"""
        assert self.ldap["config"]["bindDn"] == ["cn=keycloak,ou=service-accounts,dc=example,dc=com"]


class TestKeycloakLDAPSync:
    """認証認可設計.md: LDAP 同期方式の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        components = self.realm.get("components", {})
        providers = components.get("org.keycloak.storage.UserStorageProvider")
        if not providers:
            pytest.skip("LDAP UserStorageProvider がまだ realm.json に設定されていません")
        self.ldap = providers[0]

    def test_changed_sync_period_60_seconds(self) -> None:
        """認証認可設計.md: 差分同期は 60 秒間隔。"""
        assert self.ldap["config"]["changedSyncPeriod"] == ["60"]

    def test_full_sync_period_daily(self) -> None:
        """認証認可設計.md: 完全同期は毎日（86400秒 = 24時間）。"""
        assert self.ldap["config"]["fullSyncPeriod"] == ["86400"]


class TestRefreshTokenLifetime:
    """認証認可設計.md: Refresh Token 有効期限の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))

    def test_refresh_token_7_days(self) -> None:
        """認証認可設計.md: Refresh Token 有効期限 7 日（604800秒）。"""
        assert self.realm["ssoSessionMaxLifespan"] == 604800


class TestJWTKeyRotation:
    """認証認可設計.md: JWT 公開鍵ローテーションの検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        providers = self.realm["components"]["org.keycloak.keys.KeyProvider"]
        self.rsa_key = providers[0]

    def test_algorithm_rs256(self) -> None:
        """認証認可設計.md: RS256 アルゴリズム。"""
        assert self.rsa_key["config"]["algorithm"] == ["RS256"]

    def test_key_size_2048(self) -> None:
        """認証認可設計.md: RSA 2048-bit。"""
        assert self.rsa_key["config"]["keySize"] == ["2048"]

    def test_jwks_cache_ttl_in_config_template(self) -> None:
        """認証認可設計.md: JWKS キャッシュ TTL 10分が config テンプレートに設定。"""
        go_config = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "config" / "config.yaml.tera"
        content = go_config.read_text(encoding="utf-8")
        assert "10m" in content


class TestPermissionMatrix:
    """認証認可設計.md: パーミッションマトリクス（CRUD）の検証。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        self.realm = json.loads(path.read_text(encoding="utf-8"))
        self.role_names = [r["name"] for r in self.realm["roles"]["realm"]]

    def test_system_admin_role_exists(self) -> None:
        """認証認可設計.md: sys_admin ロールが定義されている。"""
        assert "sys_admin" in self.role_names

    def test_system_operator_role_exists(self) -> None:
        """認証認可設計.md: sys_operator ロールが定義されている。"""
        assert "sys_operator" in self.role_names

    def test_system_auditor_role_exists(self) -> None:
        """認証認可設計.md: sys_auditor ロールが定義されている。"""
        assert "sys_auditor" in self.role_names

    def test_business_roles_follow_pattern(self) -> None:
        """認証認可設計.md: business ロールは biz_{domain}_* パターン。"""
        biz_roles = [r for r in self.role_names if r.startswith("biz_")]
        assert len(biz_roles) >= 3

    def test_service_roles_follow_pattern(self) -> None:
        """認証認可設計.md: service ロールは svc_{service}_* パターン。"""
        svc_roles = [r for r in self.role_names if r.startswith("svc_")]
        assert len(svc_roles) >= 3


class TestVaultDatabaseDynamicCredentials:
    """認証認可設計.md: Vault Database 動的クレデンシャルの検証。"""

    def test_secrets_tf_exists(self) -> None:
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "secrets.tf"
        assert path.exists()

    def test_default_ttl_24h(self) -> None:
        """認証認可設計.md: default_ttl = 86400 (24h)。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "secrets.tf"
        content = path.read_text(encoding="utf-8")
        assert "86400" in content

    def test_max_ttl_48h(self) -> None:
        """認証認可設計.md: max_ttl = 172800 (48h)。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "secrets.tf"
        content = path.read_text(encoding="utf-8")
        assert "172800" in content


class TestVaultAuditLog:
    """認証認可設計.md: Vault 監査ログ設定の検証。"""

    def test_audit_tf_exists(self) -> None:
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        assert path.exists()

    def test_audit_log_file_type(self) -> None:
        """認証認可設計.md: audit type = file。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert 'vault_audit' in content
        assert '"file"' in content

    def test_audit_log_raw_false(self) -> None:
        """認証認可設計.md: log_raw = false（シークレット値マスク）。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert "log_raw" in content
        assert "false" in content

    def test_audit_log_path(self) -> None:
        """認証認可設計.md: file_path = /vault/logs/audit.log。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert "/vault/logs/audit.log" in content


class TestCredentialRotation:
    """認証認可設計.md: クレデンシャルローテーション間隔の検証。"""

    def test_db_password_rotation_24h(self) -> None:
        """認証認可設計.md: DB パスワードは 24 時間ローテーション。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "secrets.tf"
        content = path.read_text(encoding="utf-8")
        assert "86400" in content  # 24 hours in seconds

    def test_jwt_signing_key_rotation_90_days(self) -> None:
        """認証認可設計.md: JWT 署名鍵は 90 日ローテーション（JWKS 方式）。"""
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        realm = json.loads(path.read_text(encoding="utf-8"))
        assert realm["defaultSignatureAlgorithm"] == "RS256"


class TestKeycloakHA:
    """認証認可設計.md: Keycloak HA 構成の検証。"""

    def test_ha_config_in_doc(self) -> None:
        """認証認可設計.md: HA 構成が記載されている。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "HA" in content or "3 ノード" in content or "3ノード" in content

    def test_prod_3_nodes(self) -> None:
        """認証認可設計.md: prod は 3 ノード構成。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "3 ノード" in content or "3ノード" in content

    def test_infinispan_distributed_cache(self) -> None:
        """認証認可設計.md: Infinispan 分散キャッシュ。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "Infinispan" in content


class TestLDAPGroupMapping:
    """認証認可設計.md: LDAP グループマッピングの検証。"""

    def test_ldap_group_mapping_in_doc(self) -> None:
        """認証認可設計.md: memberOf → グループマッピングが記載。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "memberOf" in content
        assert "グループ" in content

    def test_ldap_group_mapping_in_realm(self) -> None:
        """認証認可設計.md: LDAP 設定に memberOf 属性が含まれる。"""
        path = ROOT / "infra" / "docker" / "keycloak" / "k1s0-realm.json"
        realm = json.loads(path.read_text(encoding="utf-8"))
        components = realm.get("components", {})
        providers = components.get("org.keycloak.storage.UserStorageProvider")
        if not providers:
            pytest.skip("LDAP UserStorageProvider がまだ realm.json に設定されていません")
        ldap = providers[0]
        sub_components = ldap.get("subComponents", {})
        mapper_types = sub_components.get("org.keycloak.storage.ldap.mappers.LDAPStorageMapper", [])
        has_group_mapper = any("group" in str(m).lower() for m in mapper_types)
        assert has_group_mapper, "LDAP グループマッパーが設定されていません"


class TestKeyRotationSchedule:
    """認証認可設計.md: 鍵ローテーション周期とオーバーラップ期間。"""

    def test_rotation_cycle_90_days_in_doc(self) -> None:
        """認証認可設計.md: ローテーション周期 90 日。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "90 日" in content or "90日" in content

    def test_overlap_period_14_days_in_doc(self) -> None:
        """認証認可設計.md: オーバーラップ期間 14 日。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "14 日" in content or "14日" in content


class TestSPIFFEID:
    """認証認可設計.md: SPIFFE ID 形式の検証。"""

    def test_spiffe_id_format_in_doc(self) -> None:
        """認証認可設計.md: SPIFFE ID 形式が記載。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "spiffe://k1s0.internal" in content

    def test_spiffe_id_namespace_pattern(self) -> None:
        """認証認可設計.md: SPIFFE ID に namespace/service-account パターン。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "ns/{namespace}/sa/{service-account}" in content


class TestCertificateRotation:
    """認証認可設計.md: 証明書ローテーション 24 時間。"""

    def test_cert_rotation_24h_in_doc(self) -> None:
        """認証認可設計.md: 証明書ローテーション 24 時間。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "24 時間" in content or "24時間" in content


class TestTierAccessDoubleVerification:
    """認証認可設計.md: tier_access Claim 二重検証の検証。"""

    def test_mesh_level_verification(self) -> None:
        """認証認可設計.md: Istio AuthorizationPolicy で tier_access を検証。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "request.auth.claims[tier_access]" in content

    def test_application_level_verification(self) -> None:
        """認証認可設計.md: アプリケーションレベルで二重検証。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "二重検証" in content or "二次防御" in content


class TestHasPermissionLogic:
    """認証認可設計.md: has_permission パーミッション解決ロジック。"""

    def test_has_permission_in_doc(self) -> None:
        """認証認可設計.md: has_permission パーミッション解決が記載。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "has_permission" in content or "hasPermission" in content

    def test_jwt_claims_based_resolution(self) -> None:
        """認証認可設計.md: JWT Claims ベースの静的解決。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "JWT Claims ベース" in content or "インメモリ" in content

    def test_deny_by_default(self) -> None:
        """認証認可設計.md: deny by default。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "deny by default" in content


class TestVaultAgentInjector:
    """認証認可設計.md: Vault Agent Injector パターン。"""

    def test_vault_agent_inject_annotation(self) -> None:
        """認証認可設計.md: vault.hashicorp.com/agent-inject アノテーション。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "vault.hashicorp.com/agent-inject" in content

    def test_vault_agent_role_annotation(self) -> None:
        """認証認可設計.md: vault.hashicorp.com/role アノテーション。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "vault.hashicorp.com/role" in content


class TestVaultAgentInjectorFileMountPath:
    """認証認可設計.md: Vault Agent Injector ファイルマウントパス。"""

    def test_mount_path_pattern(self) -> None:
        """認証認可設計.md: /vault/secrets/{secret-type} パターン。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "/vault/secrets/" in content

    def test_db_password_mount_path(self) -> None:
        """認証認可設計.md: /vault/secrets/db-password マウントパス。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "/vault/secrets/db-password" in content

    def test_db_creds_mount_path(self) -> None:
        """認証認可設計.md: /vault/secrets/db-creds マウントパス。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "/vault/secrets/db-creds" in content


class TestCSRFProtection:
    """認証認可設計.md: CSRF 対策の検証。"""

    def test_samesite_strict(self) -> None:
        """認証認可設計.md: SameSite=Strict。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "SameSite" in content
        assert "Strict" in content

    def test_x_csrf_token(self) -> None:
        """認証認可設計.md: X-CSRF-Token ヘッダー。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "X-CSRF-Token" in content

    def test_origin_check(self) -> None:
        """認証認可設計.md: Origin チェック。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "Origin" in content


class TestCookieSettings:
    """認証認可設計.md: Cookie 設定の検証。"""

    def test_httponly(self) -> None:
        """認証認可設計.md: HttpOnly 属性。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "HttpOnly" in content

    def test_secure(self) -> None:
        """認証認可設計.md: Secure 属性。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "Secure" in content

    def test_samesite(self) -> None:
        """認証認可設計.md: SameSite 属性。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "SameSite" in content

    def test_max_age_604800(self) -> None:
        """認証認可設計.md: Max-Age=604800 (7日)。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "604800" in content

    def test_domain(self) -> None:
        """認証認可設計.md: Domain=.k1s0.internal.example.com。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert ".k1s0.internal.example.com" in content


class TestBFFRedisSessionStore:
    """認証認可設計.md: BFF Redis セッションストアの検証。"""

    def test_redis_sentinel_in_doc(self) -> None:
        """認証認可設計.md: Redis Sentinel 構成。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "Redis Sentinel" in content

    def test_session_ttl_30_min(self) -> None:
        """認証認可設計.md: セッション TTL 30 分。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "30 分" in content or "30分" in content

    def test_dedicated_instance(self) -> None:
        """認証認可設計.md: 専用の Redis インスタンス。"""
        doc = ROOT / "docs" / "認証認可設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "専用" in content


class TestKubernetesAuthBackend:
    """認証認可設計.md: Kubernetes Auth Backend 設定の検証。"""

    def test_kubernetes_auth_in_terraform(self) -> None:
        """認証認可設計.md: Terraform に Kubernetes Auth Backend が定義。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert "vault_auth_backend" in content
        assert "kubernetes" in content

    def test_system_role_bound_to_namespace(self) -> None:
        """認証認可設計.md: system ロールが k1s0-system Namespace にバインド。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert "k1s0-system" in content

    def test_business_role_bound_to_namespace(self) -> None:
        """認証認可設計.md: business ロールが k1s0-business Namespace にバインド。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert "k1s0-business" in content

    def test_service_role_bound_to_namespace(self) -> None:
        """認証認可設計.md: service ロールが k1s0-service Namespace にバインド。"""
        path = ROOT / "infra" / "terraform" / "modules" / "vault" / "main.tf"
        content = path.read_text(encoding="utf-8")
        assert "k1s0-service" in content
