"""APIゲートウェイ設計.md の仕様準拠テスト。

Kong decK 設定、プラグイン、サービス定義が
設計ドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
KONG = ROOT / "infra" / "kong"


class TestKongDeckConfig:
    """APIゲートウェイ設計.md: Kong decK 設定の検証。"""

    def setup_method(self) -> None:
        path = KONG / "kong.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.config = yaml.safe_load(self.content)

    def test_kong_yaml_exists(self) -> None:
        assert (KONG / "kong.yaml").exists()

    def test_format_version(self) -> None:
        assert self.config["_format_version"] == "3.0"

    def test_services_defined(self) -> None:
        assert "services" in self.config
        assert len(self.config["services"]) >= 3

    def test_plugins_defined(self) -> None:
        assert "plugins" in self.config

    @pytest.mark.parametrize(
        "service_name",
        ["auth-v1", "accounting-ledger-v1", "order-v1"],
    )
    def test_service_defined(self, service_name: str) -> None:
        names = [s["name"] for s in self.config["services"]]
        assert service_name in names, f"Service '{service_name}' が定義されていません"


class TestKongGlobalPlugins:
    """APIゲートウェイ設計.md: グローバルプラグインの検証。"""

    def setup_method(self) -> None:
        path = KONG / "plugins" / "global.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.config = yaml.safe_load(self.content)

    def test_global_plugins_file_exists(self) -> None:
        assert (KONG / "plugins" / "global.yaml").exists()

    @pytest.mark.parametrize(
        "plugin_name",
        ["rate-limiting", "cors", "prometheus", "file-log", "post-function"],
    )
    def test_plugin_defined(self, plugin_name: str) -> None:
        names = [p["name"] for p in self.config["plugins"]]
        assert plugin_name in names, f"Plugin '{plugin_name}' が定義されていません"

    def test_rate_limiting_config(self) -> None:
        """APIゲートウェイ設計.md: デフォルト 500 req/min。"""
        rl = [p for p in self.config["plugins"] if p["name"] == "rate-limiting"][0]
        assert rl["config"]["minute"] == 500
        assert rl["config"]["policy"] == "redis"

    def test_cors_origins(self) -> None:
        """APIゲートウェイ設計.md: k1s0 ドメインの CORS 設定。"""
        cors = [p for p in self.config["plugins"] if p["name"] == "cors"][0]
        assert "https://*.k1s0.internal.example.com" in cors["config"]["origins"]

    def test_prometheus_metrics(self) -> None:
        """APIゲートウェイ設計.md: Prometheus メトリクス収集。"""
        prom = [p for p in self.config["plugins"] if p["name"] == "prometheus"][0]
        assert prom["config"]["per_consumer"] is True
        assert prom["config"]["status_code_metrics"] is True

    def test_post_function_jwt_headers(self) -> None:
        """APIゲートウェイ設計.md: JWT Claims をヘッダーに転送。"""
        pf = [p for p in self.config["plugins"] if p["name"] == "post-function"][0]
        code = str(pf["config"]["header_filter"])
        assert "X-User-Id" in code
        assert "X-User-Roles" in code


class TestKongAuthPlugin:
    """APIゲートウェイ設計.md: 認証プラグインの検証。"""

    def setup_method(self) -> None:
        path = KONG / "plugins" / "auth.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.config = yaml.safe_load(self.content)

    def test_auth_plugins_file_exists(self) -> None:
        assert (KONG / "plugins" / "auth.yaml").exists()

    def test_jwt_plugin_defined(self) -> None:
        names = [p["name"] for p in self.config["plugins"]]
        assert "jwt" in names

    def test_jwt_max_expiration(self) -> None:
        """APIゲートウェイ設計.md: Access Token 最大有効期限 900秒(15分)。"""
        jwt = [p for p in self.config["plugins"] if p["name"] == "jwt"][0]
        assert jwt["config"]["maximum_expiration"] == 900

    def test_jwt_claims_verify_exp(self) -> None:
        jwt = [p for p in self.config["plugins"] if p["name"] == "jwt"][0]
        assert "exp" in jwt["config"]["claims_to_verify"]

    def test_keycloak_consumer(self) -> None:
        """APIゲートウェイ設計.md: Keycloak JWKS 連携。"""
        assert "consumers" in self.config
        usernames = [c["username"] for c in self.config["consumers"]]
        assert "keycloak" in usernames

    def test_keycloak_rs256(self) -> None:
        consumer = [c for c in self.config["consumers"] if c["username"] == "keycloak"][0]
        assert consumer["jwt_secrets"][0]["algorithm"] == "RS256"


class TestKongServices:
    """APIゲートウェイ設計.md: Tier 別サービス定義の検証。"""

    def test_system_services_file_exists(self) -> None:
        assert (KONG / "services" / "system.yaml").exists()

    def test_business_services_file_exists(self) -> None:
        assert (KONG / "services" / "business.yaml").exists()

    def test_service_services_file_exists(self) -> None:
        assert (KONG / "services" / "service.yaml").exists()

    def test_system_auth_service(self) -> None:
        """APIゲートウェイ設計.md: auth-v1 サービスが system Tier に定義。"""
        path = KONG / "services" / "system.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        names = [s["name"] for s in config["services"]]
        assert "auth-v1" in names

    def test_system_auth_login_rate_limit(self) -> None:
        """APIゲートウェイ設計.md: /auth/login に 30 req/min の制限。"""
        path = KONG / "services" / "system.yaml"
        content = path.read_text(encoding="utf-8")
        assert "30" in content
        assert "rate-limiting" in content

    def test_business_ledger_service(self) -> None:
        """APIゲートウェイ設計.md: accounting-ledger-v1 が business Tier に定義。"""
        path = KONG / "services" / "business.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        names = [s["name"] for s in config["services"]]
        assert "accounting-ledger-v1" in names

    def test_service_order_service(self) -> None:
        """APIゲートウェイ設計.md: order-v1 が service Tier に定義。"""
        path = KONG / "services" / "service.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        names = [s["name"] for s in config["services"]]
        assert "order-v1" in names

    def test_service_urls_use_cluster_local(self) -> None:
        """APIゲートウェイ設計.md: サービス URL が cluster.local を使用。"""
        for tier_file in ["system.yaml", "business.yaml", "service.yaml"]:
            path = KONG / "services" / tier_file
            content = path.read_text(encoding="utf-8")
            assert "svc.cluster.local" in content

    def test_strip_path_false(self) -> None:
        """APIゲートウェイ設計.md: strip_path=false でパスを維持。"""
        for tier_file in ["system.yaml", "business.yaml", "service.yaml"]:
            path = KONG / "services" / tier_file
            content = path.read_text(encoding="utf-8")
            assert "strip_path: false" in content
