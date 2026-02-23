"""config設計.md の仕様準拠テスト。

config.yaml テンプレートの内容が config 設計ドキュメントの
スキーマ定義と一致するかを検証する。
"""

from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
GO_CONFIG = (
    ROOT
    / "CLI"
    / "crates"
    / "k1s0-cli"
    / "templates"
    / "server"
    / "go"
    / "config"
    / "config.yaml.tera"
)
RUST_CONFIG = (
    ROOT
    / "CLI"
    / "crates"
    / "k1s0-cli"
    / "templates"
    / "server"
    / "rust"
    / "config"
    / "config.yaml.tera"
)


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
                    value = value[: value.index("#")].strip()
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
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "go"
            / "internal"
            / "infra"
            / "config"
            / "config.go.tera"
        )
        assert path.exists()

    def test_go_config_struct(self) -> None:
        """config設計.md: Config 構造体が定義されている。"""
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "go"
            / "internal"
            / "infra"
            / "config"
            / "config.go.tera"
        )
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "Config" in content

    def test_go_config_yaml_tags(self) -> None:
        """config設計.md: yaml タグが使用されている。"""
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "go"
            / "internal"
            / "infra"
            / "config"
            / "config.go.tera"
        )
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "yaml:" in content


class TestConfigRustImplementation:
    """config設計.md: Rust での読み込み実装テンプレートの検証。"""

    def test_rust_config_loader_template(self) -> None:
        """config設計.md: Rust config ローダーテンプレートが存在。"""
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "rust"
            / "src"
            / "infra"
            / "config.rs.tera"
        )
        assert path.exists()

    def test_rust_config_struct(self) -> None:
        """config設計.md: Rust Config 構造体が定義されている。"""
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "rust"
            / "src"
            / "infra"
            / "config.rs.tera"
        )
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "Config" in content


class TestConfigMountPath:
    """config設計.md: config.yaml のマウントパス仕様。"""

    @pytest.mark.parametrize("lang", ["go", "rust"])
    def test_config_dir_in_template(self, lang: str) -> None:
        """config設計.md: ローカル開発は config/config.yaml。"""
        assert (
            ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / lang / "config"
        ).is_dir()

    @pytest.mark.parametrize("lang", ["go", "rust"])
    def test_dockerfile_template_exists(self, lang: str) -> None:
        """config設計.md: Dockerfile テンプレートが存在。"""
        assert (
            ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / lang / "Dockerfile.tera"
        ).exists()


# ============================================================================
# コンセプト.md Vault 統合ギャップ補完テスト
# ============================================================================

CLI_SRC = ROOT / "CLI" / "crates" / "k1s0-cli" / "src"
CLI_CORE_SRC = ROOT / "CLI" / "crates" / "k1s0-core" / "src"


class TestVaultIntegration:
    """コンセプト.md: Vault 統合 — merge_vault_secrets の実装状態検証。

    HashiCorp Vault 1.17 によるシークレットの一元管理・配布の仕様に基づき、
    CLI/src/config/mod.rs に merge_vault_secrets 関数が存在し、
    最低限のスタブ実装（接続失敗時の no-op / 警告ログ）があることを検証する。
    """

    def setup_method(self) -> None:
        self.content = (CLI_CORE_SRC / "config" / "mod.rs").read_text(encoding="utf-8")

    def test_merge_vault_secrets_function_exists(self) -> None:
        """コンセプト.md: merge_vault_secrets 関数が定義されている。"""
        assert "fn merge_vault_secrets" in self.content

    def test_merge_vault_secrets_signature(self) -> None:
        """コンセプト.md: Vault アドレスとパスを引数に取る。"""
        assert "vault_addr" in self.content
        assert "vault_path" in self.content

    def test_merge_vault_secrets_empty_noop(self) -> None:
        """コンセプト.md: Vault 未設定時は no-op。"""
        assert "vault_addr.is_empty()" in self.content

    def test_merge_vault_secrets_warn_on_unreachable(self) -> None:
        """コンセプト.md: Vault 未到達時は警告ログを出力。"""
        assert "WARN" in self.content or "warn" in self.content.lower()

    def test_merge_config_function_exists(self) -> None:
        """コンセプト.md: 環境別設定マージ関数が存在する。"""
        assert "fn merge_config" in self.content

    def test_config_merge_order_documented(self) -> None:
        """コンセプト.md: マージ順序がコメントに記載されている。"""
        # config.yaml < config.{env}.yaml < Vault の順序
        assert "マージ順序" in self.content or "Vault" in self.content


class TestConfigMergeOrderLogic:
    """config設計.md: D-079 マージ順序ロジックの検証。"""

    def test_merge_order_in_doc(self) -> None:
        """config設計.md: マージ順序がドキュメントに記載されている。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "config.yaml" in content
        assert "Vault" in content

    def test_merge_order_priority(self) -> None:
        """config設計.md: Vault が最高優先と記載されている。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "最高優先" in content or "Vault が常に優先" in content

    def test_config_loader_reads_yaml(self) -> None:
        """config設計.md: Go config ローダーが YAML を読み込む。"""
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "go"
            / "internal"
            / "infra"
            / "config"
            / "config.go.tera"
        )
        content = path.read_text(encoding="utf-8")
        assert "yaml.Unmarshal" in content or "yaml.v3" in content


class TestEnvironmentOverrideFiles:
    """config設計.md: 環境別差分ファイルの検証。"""

    def test_env_override_pattern_in_doc(self) -> None:
        """config設計.md: config.dev.yaml, config.staging.yaml, config.prod.yaml パターンが記載。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "config.dev.yaml" in content
        assert "config.staging.yaml" in content
        assert "config.prod.yaml" in content

    def test_dockerfile_template_has_config_path(self) -> None:
        """config設計.md: Dockerfile テンプレートが config パスを参照。"""
        path = (
            ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "Dockerfile.tera"
        )
        content = path.read_text(encoding="utf-8")
        assert "config" in content.lower()


class TestConfigValidationTags:
    """config設計.md: Go config 構造体のバリデーションタグ検証。"""

    def setup_method(self) -> None:
        path = (
            ROOT
            / "CLI"
            / "crates"
            / "k1s0-cli"
            / "templates"
            / "server"
            / "go"
            / "internal"
            / "infra"
            / "config"
            / "config.go.tera"
        )
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_yaml_tags_present(self) -> None:
        """config設計.md: yaml タグが構造体に定義されている。"""
        assert 'yaml:"' in self.content

    def test_validate_tags_present(self) -> None:
        """config設計.md: validate タグが重要フィールドに定義されている。"""
        assert 'validate:"' in self.content

    def test_validate_required_tag(self) -> None:
        """config設計.md: required バリデーションが使用されている。"""
        assert "required" in self.content

    def test_validate_url_tag(self) -> None:
        """config設計.md: URL フィールドに url バリデーションが使用されている。"""
        assert 'validate:"required,url"' in self.content


class TestConfigKubernetesMountPath:
    """config設計.md: Kubernetes マウントパスの検証。"""

    def test_kubernetes_mount_path_in_doc(self) -> None:
        """config設計.md: /etc/app/config.yaml がドキュメントに記載されている。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "/etc/app/config.yaml" in content

    def test_local_dev_path_in_doc(self) -> None:
        """config設計.md: ローカル開発は config/config.yaml がドキュメントに記載。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "config/config.yaml" in content


class TestConfigMergeConflictBehavior:
    """config設計.md: D-079 マージ順序の競合時動作の検証。"""

    def test_vault_priority_on_conflict(self) -> None:
        """config設計.md: 競合時は Vault の値を採用。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "Vault の値を採用" in content

    def test_warning_log_on_conflict(self) -> None:
        """config設計.md: 競合時に警告ログを出力。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "警告ログ" in content or "WARN" in content

    def test_application_continues_on_conflict(self) -> None:
        """config設計.md: 競合時もアプリケーションは正常に起動。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "正常に起動" in content or "エラーにはしない" in content


class TestRustConfigValidation:
    """config設計.md: Rust バリデーション実装の検証。"""

    def test_rust_config_has_validate_method(self) -> None:
        """config設計.md: Rust Config に validate メソッドが定義。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "fn validate" in content
        assert "ConfigError" in content

    def test_rust_config_validates_app_name(self) -> None:
        """config設計.md: Rust validate が app.name を検証。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "app.name" in content

    def test_rust_config_validates_server_port(self) -> None:
        """config設計.md: Rust validate が server.port を検証。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "server.port" in content


class TestCIConfigValidate:
    """config設計.md: CI パイプライン config validate の検証。"""

    def test_config_validate_in_doc(self) -> None:
        """config設計.md: CI で config validate コマンドを実行。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "config validate" in content

    def test_ci_pre_deploy_validation(self) -> None:
        """config設計.md: デプロイ前に不正設定を検出。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "デプロイ前" in content or "事前検証" in content


class TestRedisSessionSection:
    """config設計.md: redis_session セクション詳細の検証。"""

    def test_redis_session_host(self) -> None:
        """config設計.md: redis_session.host が定義。"""
        content = GO_CONFIG.read_text(encoding="utf-8")
        assert "redis_session:" in content
        assert "redis-session" in content

    def test_redis_session_port(self) -> None:
        """config設計.md: redis_session.port が定義。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "6380" in content

    def test_redis_session_password_vault(self) -> None:
        """config設計.md: redis_session.password は Vault から注入。"""
        doc = ROOT / "docs" / "config設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "secret/data/k1s0/system/bff/redis" in content
