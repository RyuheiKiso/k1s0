"""config設計.md の仕様準拠テスト。

config.yaml テンプレートの内容が config 設計ドキュメントの
スキーマ定義と一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
GO_CONFIG = ROOT / "CLI" / "templates" / "server" / "go" / "config" / "config.yaml.tera"
RUST_CONFIG = ROOT / "CLI" / "templates" / "server" / "rust" / "config" / "config.yaml.tera"


class TestGoConfigYamlSchema:
    """config設計.md: Go config.yaml.tera のスキーマ検証。"""

    def setup_method(self) -> None:
        assert GO_CONFIG.exists()
        self.content = GO_CONFIG.read_text(encoding="utf-8")

    def test_app_section(self) -> None:
        """config設計.md: app セクション。"""
        assert "app:" in self.content
        assert "name:" in self.content
        assert "version:" in self.content
        assert "tier:" in self.content
        assert "environment:" in self.content

    def test_app_name_variable(self) -> None:
        """config設計.md: app.name はサービス名。"""
        assert "{{ service_name }}" in self.content

    def test_app_tier_variable(self) -> None:
        """config設計.md: app.tier は Tera 変数。"""
        assert "{{ tier }}" in self.content

    def test_server_section(self) -> None:
        """config設計.md: server セクション。"""
        assert "server:" in self.content
        assert "host:" in self.content
        assert "port:" in self.content
        assert "read_timeout:" in self.content
        assert "write_timeout:" in self.content
        assert "shutdown_timeout:" in self.content

    def test_server_host(self) -> None:
        """config設計.md: server.host は 0.0.0.0。"""
        assert '"0.0.0.0"' in self.content

    def test_server_port(self) -> None:
        """config設計.md: server.port は 8080。"""
        assert "8080" in self.content

    def test_grpc_conditional(self) -> None:
        """config設計.md: gRPC 有効時のみ grpc セクション。"""
        assert "grpc:" in self.content
        assert "50051" in self.content
        assert "max_recv_msg_size:" in self.content

    def test_database_conditional(self) -> None:
        """config設計.md: DB 有効時のみ database セクション。"""
        assert "database:" in self.content
        assert "has_database" in self.content

    def test_database_fields(self) -> None:
        """config設計.md: database セクションの必須フィールド。"""
        assert "host:" in self.content
        assert "name:" in self.content
        assert "user:" in self.content
        assert "password:" in self.content
        assert "max_open_conns:" in self.content
        assert "max_idle_conns:" in self.content
        assert "conn_max_lifetime:" in self.content

    def test_database_password_empty(self) -> None:
        """config設計.md: DB パスワードは空文字で定義。"""
        assert 'password: ""' in self.content

    def test_database_vault_path_comment(self) -> None:
        """config設計.md: Vault パスがコメントに記載。"""
        assert "Vault" in self.content
        assert "secret/data/k1s0" in self.content

    def test_kafka_conditional(self) -> None:
        """config設計.md: Kafka 有効時のみ kafka セクション。"""
        assert "kafka:" in self.content
        assert "has_kafka" in self.content

    def test_kafka_fields(self) -> None:
        """config設計.md: kafka セクションの必須フィールド。"""
        assert "brokers:" in self.content
        assert "consumer_group:" in self.content
        assert "security_protocol:" in self.content

    def test_kafka_sasl_section(self) -> None:
        """config設計.md: SASL 設定。"""
        assert "sasl:" in self.content
        assert "mechanism:" in self.content
        assert "SCRAM-SHA-512" in self.content

    def test_kafka_tls_section(self) -> None:
        """config設計.md: TLS 設定。"""
        assert "tls:" in self.content
        assert "ca_cert_path:" in self.content

    def test_kafka_topics_section(self) -> None:
        """config設計.md: topics セクション。"""
        assert "topics:" in self.content
        assert "publish:" in self.content
        assert "subscribe:" in self.content

    def test_redis_conditional(self) -> None:
        """config設計.md: Redis 有効時のみ redis セクション。"""
        assert "redis:" in self.content
        assert "has_redis" in self.content

    def test_redis_fields(self) -> None:
        """config設計.md: redis セクションの必須フィールド。"""
        assert "pool_size:" in self.content

    def test_redis_session_section(self) -> None:
        """config設計.md: BFF Proxy 用セッションストア。"""
        assert "redis_session:" in self.content

    def test_observability_section(self) -> None:
        """config設計.md: observability セクション。"""
        assert "observability:" in self.content

    def test_observability_log(self) -> None:
        """config設計.md: ログ設定。"""
        assert "log:" in self.content
        assert "level:" in self.content
        assert "format:" in self.content

    def test_observability_trace(self) -> None:
        """config設計.md: トレース設定。"""
        assert "trace:" in self.content
        assert "enabled:" in self.content
        assert "endpoint:" in self.content
        assert "sample_rate:" in self.content

    def test_observability_metrics(self) -> None:
        """config設計.md: メトリクス設定。"""
        assert "metrics:" in self.content
        assert "path:" in self.content
        assert '"/metrics"' in self.content

    def test_auth_section(self) -> None:
        """config設計.md: auth セクション。"""
        assert "auth:" in self.content

    def test_auth_jwt(self) -> None:
        """config設計.md: JWT 設定。"""
        assert "jwt:" in self.content
        assert "issuer:" in self.content
        assert "audience:" in self.content
        assert "public_key_path:" in self.content

    def test_auth_oidc(self) -> None:
        """config設計.md: OIDC 設定。"""
        assert "oidc:" in self.content
        assert "discovery_url:" in self.content
        assert "client_id:" in self.content
        assert "client_secret:" in self.content
        assert "redirect_uri:" in self.content
        assert "scopes:" in self.content
        assert "jwks_uri:" in self.content
        assert "jwks_cache_ttl:" in self.content

    def test_oidc_scopes(self) -> None:
        """config設計.md: OIDC scopes は openid, profile, email。"""
        assert "openid" in self.content
        assert "profile" in self.content
        assert "email" in self.content


class TestRustConfigYamlSchema:
    """config設計.md: Rust config.yaml.tera の検証。"""

    def test_rust_config_exists(self) -> None:
        assert RUST_CONFIG.exists()

    def test_rust_config_app_section(self) -> None:
        content = RUST_CONFIG.read_text(encoding="utf-8")
        assert "app:" in content

    def test_rust_config_server_section(self) -> None:
        content = RUST_CONFIG.read_text(encoding="utf-8")
        assert "server:" in content

    def test_rust_config_observability_section(self) -> None:
        content = RUST_CONFIG.read_text(encoding="utf-8")
        assert "observability:" in content

    def test_rust_config_auth_section(self) -> None:
        content = RUST_CONFIG.read_text(encoding="utf-8")
        assert "auth:" in content


class TestConfigDesignConstraints:
    """config設計.md: 設計上の制約の検証。"""

    def setup_method(self) -> None:
        self.content = GO_CONFIG.read_text(encoding="utf-8")

    def test_no_real_secrets(self) -> None:
        """config設計.md: config.yaml にシークレットの実値を記載してはならない。"""
        lines = self.content.split("\n")
        for line in lines:
            stripped = line.strip()
            if stripped.startswith("password:"):
                value = stripped.split(":", 1)[1].strip()
                if "#" in value:
                    value = value[:value.index("#")].strip()
                assert value == '""', f"password フィールドに実値が含まれています: {line}"

    def test_vault_annotations(self) -> None:
        """config設計.md: シークレットフィールドに Vault パスのコメントがある。"""
        assert "Vault パス:" in self.content

    def test_environment_options_in_comment(self) -> None:
        """config設計.md: environment は dev | staging | prod。"""
        assert "dev" in self.content
        assert "staging" in self.content or "prod" in self.content


class TestConfigGoImplementation:
    """config設計.md: Go での読み込み実装テンプレートの検証。"""

    def test_go_config_loader_template(self) -> None:
        """config設計.md: Go config ローダーテンプレートが存在。"""
        path = ROOT / "CLI" / "templates" / "server" / "go" / "internal" / "infra" / "config" / "config.go.tera"
        assert path.exists()

    def test_go_config_struct(self) -> None:
        """config設計.md: Config 構造体が定義されている。"""
        path = ROOT / "CLI" / "templates" / "server" / "go" / "internal" / "infra" / "config" / "config.go.tera"
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "Config" in content

    def test_go_config_yaml_tags(self) -> None:
        """config設計.md: yaml タグが使用されている。"""
        path = ROOT / "CLI" / "templates" / "server" / "go" / "internal" / "infra" / "config" / "config.go.tera"
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "yaml:" in content


class TestConfigRustImplementation:
    """config設計.md: Rust での読み込み実装テンプレートの検証。"""

    def test_rust_config_loader_template(self) -> None:
        """config設計.md: Rust config ローダーテンプレートが存在。"""
        path = ROOT / "CLI" / "templates" / "server" / "rust" / "src" / "infra" / "config.rs.tera"
        assert path.exists()

    def test_rust_config_struct(self) -> None:
        """config設計.md: Rust Config 構造体が定義されている。"""
        path = ROOT / "CLI" / "templates" / "server" / "rust" / "src" / "infra" / "config.rs.tera"
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "Config" in content


class TestConfigMountPath:
    """config設計.md: config.yaml のマウントパス仕様。"""

    @pytest.mark.parametrize("lang", ["go", "rust"])
    def test_config_dir_in_template(self, lang: str) -> None:
        """config設計.md: ローカル開発は config/config.yaml。"""
        assert (ROOT / "CLI" / "templates" / "server" / lang / "config").is_dir()

    @pytest.mark.parametrize("lang", ["go", "rust"])
    def test_dockerfile_template_exists(self, lang: str) -> None:
        """config設計.md: Dockerfile テンプレートが存在。"""
        assert (ROOT / "CLI" / "templates" / "server" / lang / "Dockerfile.tera").exists()
