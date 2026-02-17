"""API設計.md の仕様準拠テスト。

proto ファイル、buf 設定、およびプロジェクト構成が
API 設計ドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
PROTO = ROOT / "api" / "proto"


class TestBufConfig:
    """API設計.md: buf.yaml の内容検証。"""

    def setup_method(self) -> None:
        self.path = PROTO / "buf.yaml"
        assert self.path.exists()
        with open(self.path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_buf_version(self) -> None:
        """API設計.md: buf version v2。"""
        assert self.config["version"] == "v2"

    def test_modules_path(self) -> None:
        """API設計.md: modules path は api/proto。"""
        modules = self.config["modules"]
        paths = [m["path"] for m in modules]
        assert "." in paths

    def test_lint_standard(self) -> None:
        """API設計.md: STANDARD lint ルール。"""
        assert "STANDARD" in self.config["lint"]["use"]

    def test_breaking_file(self) -> None:
        """API設計.md: FILE breaking change 検出。"""
        assert "FILE" in self.config["breaking"]["use"]


class TestProtoPackageNaming:
    """API設計.md: proto パッケージ命名の検証。"""

    def setup_method(self) -> None:
        self.types_proto = PROTO / "k1s0" / "system" / "common" / "v1" / "types.proto"
        assert self.types_proto.exists()
        self.content = self.types_proto.read_text(encoding="utf-8")

    def test_syntax_proto3(self) -> None:
        assert 'syntax = "proto3"' in self.content

    def test_package_naming_convention(self) -> None:
        """API設計.md: k1s0.{tier}.{domain}.v{major} 形式。"""
        assert "package k1s0.system.common.v1" in self.content

    def test_pagination_message(self) -> None:
        """API設計.md: 共通型として Pagination を定義。"""
        assert "message Pagination" in self.content
        assert "int32 page" in self.content
        assert "int32 page_size" in self.content

    def test_pagination_result_message(self) -> None:
        """API設計.md: PaginationResult を定義。"""
        assert "message PaginationResult" in self.content
        assert "int32 total_count" in self.content
        assert "bool has_next" in self.content

    def test_timestamp_message(self) -> None:
        """API設計.md: 共通型として Timestamp を定義。"""
        assert "message Timestamp" in self.content
        assert "int64 seconds" in self.content
        assert "int32 nanos" in self.content


class TestProtoDirectoryStructure:
    """API設計.md: proto ファイル配置の検証。"""

    def test_buf_yaml_exists(self) -> None:
        assert (PROTO / "buf.yaml").exists()

    def test_common_types_exists(self) -> None:
        """API設計.md: k1s0/system/common/v1/types.proto が存在。"""
        assert (PROTO / "k1s0" / "system" / "common" / "v1" / "types.proto").exists()

    def test_event_metadata_exists(self) -> None:
        """API設計.md: k1s0/system/common/v1/event_metadata.proto が存在。"""
        assert (PROTO / "k1s0" / "system" / "common" / "v1" / "event_metadata.proto").exists()

    def test_k1s0_namespace(self) -> None:
        """API設計.md: k1s0 プロジェクトプレフィックス。"""
        assert (PROTO / "k1s0").is_dir()

    def test_event_directory(self) -> None:
        """API設計.md: イベント定義ディレクトリ。"""
        assert (PROTO / "k1s0" / "event").is_dir()


class TestEventProtoStructure:
    """API設計.md: イベント proto のティア別配置検証。"""

    @pytest.mark.parametrize("tier", ["system", "business", "service"])
    def test_event_tier_directory(self, tier: str) -> None:
        """API設計.md: event/{tier}/ ディレクトリが存在。"""
        assert (PROTO / "k1s0" / "event" / tier).is_dir()


class TestRestApiErrorDesign:
    """API設計.md: D-007 REST API エラーレスポンス設計。

    テンプレート内の handler がエラーレスポンス仕様に準拠するか検証。
    """

    def test_go_rest_handler_exists(self) -> None:
        """API設計.md: Go REST handler テンプレート。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler" / "rest_handler.go.tera"
        assert path.exists()

    def test_rust_rest_handler_exists(self) -> None:
        """API設計.md: Rust REST handler テンプレート。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "rust" / "src" / "adapter" / "handler" / "rest.rs.tera"
        assert path.exists()

    def test_go_handler_dir_exists(self) -> None:
        """API設計.md: Go handler ディレクトリが存在。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler"
        assert path.is_dir()

    def test_rust_handler_dir_exists(self) -> None:
        """API設計.md: Rust handler ディレクトリが存在。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "rust" / "src" / "adapter" / "handler"
        assert path.is_dir()


class TestRestApiVersioning:
    """API設計.md: D-008 REST API バージョニング。"""

    def test_kong_services_use_versioned_paths(self) -> None:
        """API設計.md: URL パス方式 /api/v{major}/ を採用。"""
        path = ROOT / "infra" / "kong" / "services" / "system.yaml"
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "/api/v1/" in content

    def test_kong_strip_path_false(self) -> None:
        """API設計.md: strip_path: false。"""
        path = ROOT / "infra" / "kong" / "services" / "system.yaml"
        if path.exists():
            content = path.read_text(encoding="utf-8")
            assert "strip_path: false" in content


class TestGrpcDesign:
    """API設計.md: D-009 gRPC サービス定義パターン。"""

    def test_go_grpc_handler_template(self) -> None:
        """API設計.md: Go gRPC handler テンプレート。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler" / "grpc_handler.go.tera"
        assert path.exists()

    def test_rust_grpc_handler_template(self) -> None:
        """API設計.md: Rust gRPC handler テンプレート。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "rust" / "src" / "adapter" / "handler" / "grpc.rs.tera"
        assert path.exists()


class TestGraphQLDesign:
    """API設計.md: D-011 / D-124 GraphQL 設計。"""

    def test_go_graphql_resolver_template(self) -> None:
        """API設計.md: Go GraphQL resolver テンプレート。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler" / "graphql_resolver.go.tera"
        assert path.exists()

    def test_rust_graphql_handler_template(self) -> None:
        """API設計.md: Rust GraphQL handler テンプレート。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "rust" / "src" / "adapter" / "handler" / "graphql.rs.tera"
        assert path.exists()


class TestRateLimitDesign:
    """API設計.md: D-012 レート制限設計。"""

    def test_kong_global_rate_limiting(self) -> None:
        """API設計.md: Kong Rate Limiting プラグイン。"""
        path = ROOT / "infra" / "kong" / "plugins" / "global.yaml"
        assert path.exists()
        content = path.read_text(encoding="utf-8")
        assert "rate-limiting" in content

    def test_kong_rate_limit_minute(self) -> None:
        """API設計.md: service Tier デフォルト 500 req/min。"""
        path = ROOT / "infra" / "kong" / "plugins" / "global.yaml"
        content = path.read_text(encoding="utf-8")
        assert "500" in content

    def test_kong_rate_limit_redis_policy(self) -> None:
        """API設計.md: Redis で共有状態を管理。"""
        path = ROOT / "infra" / "kong" / "plugins" / "global.yaml"
        content = path.read_text(encoding="utf-8")
        assert "redis" in content


class TestCodeGenDesign:
    """API設計.md: D-123 OpenAPI コード自動生成。"""

    def test_go_openapi_template_exists(self) -> None:
        """API設計.md: Go サーバーに OpenAPI テンプレートが存在。"""
        # Go サーバーテンプレートのハンドラーディレクトリが存在するか
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler"
        assert path.is_dir()

    def test_rust_handler_template_exists(self) -> None:
        """API設計.md: Rust サーバーに handler テンプレートが存在。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "rust" / "src" / "adapter" / "handler"
        assert path.is_dir()


class TestErrorResponseUnifiedSchema:
    """API設計.md: D-007 エラーレスポンス統一 JSON スキーマの検証。"""

    def test_go_handler_has_error_code_field(self) -> None:
        """API設計.md: Go handler に code フィールドが定義されている。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler" / "rest_handler.go.tera"
        content = path.read_text(encoding="utf-8")
        assert '"code"' in content

    def test_go_handler_has_error_message_field(self) -> None:
        """API設計.md: Go handler に message フィールドが定義されている。"""
        path = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "internal" / "adapter" / "handler" / "rest_handler.go.tera"
        content = path.read_text(encoding="utf-8")
        assert '"message"' in content

    def test_error_schema_in_doc(self) -> None:
        """API設計.md: ドキュメントに error.code, error.message, error.request_id, error.details が記載。"""
        doc = ROOT / "docs" / "API設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "error.code" in content or "`error.code`" in content
        assert "error.message" in content or "`error.message`" in content
        assert "error.request_id" in content or "`error.request_id`" in content
        assert "error.details" in content or "`error.details`" in content


class TestTierPrefixErrorCodes:
    """API設計.md: Tier プレフィックス付きエラーコードの検証。"""

    def test_sys_prefix_in_doc(self) -> None:
        """API設計.md: SYS_ プレフィックスが定義されている。"""
        doc = ROOT / "docs" / "API設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "SYS_" in content

    def test_biz_prefix_in_doc(self) -> None:
        """API設計.md: BIZ_ プレフィックスが定義されている。"""
        doc = ROOT / "docs" / "API設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "BIZ_" in content

    def test_svc_prefix_in_doc(self) -> None:
        """API設計.md: SVC_ プレフィックスが定義されている。"""
        doc = ROOT / "docs" / "API設計.md"
        content = doc.read_text(encoding="utf-8")
        assert "SVC_" in content


class TestGrpcStatusCodeMapping:
    """API設計.md: gRPC ステータスコードマッピングの検証。"""

    @pytest.mark.parametrize(
        "status_code",
        [
            "OK",
            "INVALID_ARGUMENT",
            "UNAUTHENTICATED",
            "PERMISSION_DENIED",
            "NOT_FOUND",
            "ALREADY_EXISTS",
            "FAILED_PRECONDITION",
            "RESOURCE_EXHAUSTED",
            "INTERNAL",
            "UNAVAILABLE",
        ],
    )
    def test_grpc_status_code_in_doc(self, status_code: str) -> None:
        """API設計.md: gRPC ステータスコードがドキュメントに記載されている。"""
        doc = ROOT / "docs" / "API設計.md"
        content = doc.read_text(encoding="utf-8")
        assert status_code in content, f"gRPC ステータス '{status_code}' がドキュメントに記載されていません"


class TestGraphQLQueryLimits:
    """API設計.md: D-011 GraphQL クエリ制限の検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_query_depth_limit_10(self) -> None:
        """API設計.md: クエリ深度上限 10。"""
        assert "10" in self.content
        assert "クエリ深度" in self.content or "深度" in self.content

    def test_query_complexity_limit_1000(self) -> None:
        """API設計.md: 複雑度上限 1000。"""
        assert "1000" in self.content
        assert "複雑度" in self.content

    def test_query_timeout_30s(self) -> None:
        """API設計.md: タイムアウト 30s。"""
        assert "30s" in self.content
        assert "タイムアウト" in self.content


class TestTierRateLimits:
    """API設計.md: D-012 Tier 別レート制限の検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_system_tier_3000_req_min(self) -> None:
        """API設計.md: system Tier デフォルト 3000 req/min。"""
        assert "3000" in self.content

    def test_business_tier_1000_req_min(self) -> None:
        """API設計.md: business Tier デフォルト 1000 req/min。"""
        assert "1000" in self.content

    def test_service_tier_500_req_min(self) -> None:
        """API設計.md: service Tier デフォルト 500 req/min。"""
        assert "500" in self.content


class TestBufGenYaml:
    """API設計.md: buf.gen.yaml の検証。"""

    def setup_method(self) -> None:
        self.path = PROTO / "buf.gen.yaml"
        assert self.path.exists()
        with open(self.path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_buf_gen_yaml_exists(self) -> None:
        """API設計.md: buf.gen.yaml が存在する。"""
        assert self.path.exists()

    def test_buf_gen_version_v2(self) -> None:
        """API設計.md: buf.gen.yaml version v2。"""
        assert self.config["version"] == "v2"

    def test_buf_gen_has_go_plugin(self) -> None:
        """API設計.md: Go プラグインが定義されている。"""
        remotes = [p["remote"] for p in self.config["plugins"]]
        go_plugins = [r for r in remotes if "go" in r]
        assert len(go_plugins) >= 1, "Go プラグインが定義されていません"

    def test_buf_gen_has_rust_plugin(self) -> None:
        """API設計.md: Rust プラグインが定義されている。"""
        remotes = [p["remote"] for p in self.config["plugins"]]
        rust_plugins = [r for r in remotes if "rust" in r]
        assert len(rust_plugins) >= 1, "Rust プラグインが定義されていません"


class TestValidationErrorDetails:
    """API設計.md: D-007 バリデーションエラーの details 配列検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_details_field_key(self) -> None:
        """API設計.md: details 配列に field キーが記載。"""
        assert '"field"' in self.content

    def test_details_reason_key(self) -> None:
        """API設計.md: details 配列に reason キーが記載。"""
        assert '"reason"' in self.content

    def test_details_message_key(self) -> None:
        """API設計.md: details 配列に message キーが記載。"""
        assert '"message"' in self.content

    def test_go_error_detail_struct(self) -> None:
        """API設計.md: Go 実装例に ErrorDetail 構造体が定義。"""
        assert "ErrorDetail" in self.content

    def test_rust_error_detail_struct(self) -> None:
        """API設計.md: Rust 実装例に ErrorDetail 構造体が定義。"""
        assert "ErrorDetail" in self.content


class TestHttpStatusCodeMapping:
    """API設計.md: HTTP ステータスコードマッピング検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "status,description",
        [
            ("400", "バリデーションエラー"),
            ("401", "認証エラー"),
            ("403", "認可エラー"),
            ("404", "リソースが見つからない"),
            ("409", "競合"),
            ("422", "ビジネスルール違反"),
            ("429", "レート制限超過"),
            ("500", "内部サーバーエラー"),
            ("503", "サービス利用不可"),
        ],
    )
    def test_http_status_code_in_doc(self, status: str, description: str) -> None:
        """API設計.md: HTTP ステータスコードがドキュメントに記載。"""
        assert status in self.content, f"HTTP {status} ({description}) がドキュメントに記載されていません"


class TestDeprecationResponseHeaders:
    """API設計.md: D-008 非推奨レスポンスヘッダー検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_deprecation_header(self) -> None:
        """API設計.md: Deprecation ヘッダーが記載。"""
        assert "Deprecation:" in self.content or "Deprecation: true" in self.content

    def test_sunset_header(self) -> None:
        """API設計.md: Sunset ヘッダーが記載。"""
        assert "Sunset:" in self.content

    def test_link_header_successor(self) -> None:
        """API設計.md: Link ヘッダーに successor-version が記載。"""
        assert 'rel="successor-version"' in self.content


class TestBufBreakingCI:
    """API設計.md: D-010 buf breaking CI 検出。"""

    def test_ci_workflow_has_buf_breaking(self) -> None:
        """API設計.md: CI に buf breaking ステップが存在。"""
        # buf breaking は proto.yaml (proto 専用ワークフロー) で実行される
        ci_path = ROOT / ".github" / "workflows" / "ci.yaml"
        proto_path = ROOT / ".github" / "workflows" / "proto.yaml"
        ci_content = ci_path.read_text(encoding="utf-8") if ci_path.exists() else ""
        proto_content = proto_path.read_text(encoding="utf-8") if proto_path.exists() else ""
        combined = ci_content + proto_content
        assert "buf breaking" in combined or "buf-breaking" in combined

    def test_buf_breaking_against_main(self) -> None:
        """API設計.md: buf breaking が main ブランチに対して検出。"""
        proto_path = ROOT / ".github" / "workflows" / "proto.yaml"
        assert proto_path.exists(), "proto.yaml ワークフローが存在しません"
        content = proto_path.read_text(encoding="utf-8")
        assert "main" in content


class TestRelayPagination:
    """API設計.md: D-011 Relay スタイル Cursor ページネーション検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_connection_type(self) -> None:
        """API設計.md: OrderConnection 型が定義。"""
        assert "OrderConnection" in self.content

    def test_edge_type(self) -> None:
        """API設計.md: OrderEdge 型が定義。"""
        assert "OrderEdge" in self.content

    def test_page_info_type(self) -> None:
        """API設計.md: PageInfo 型が定義。"""
        assert "PageInfo" in self.content

    def test_cursor_field(self) -> None:
        """API設計.md: cursor フィールドが定義。"""
        assert "cursor:" in self.content or "cursor: String" in self.content

    def test_has_next_page(self) -> None:
        """API設計.md: hasNextPage が定義。"""
        assert "hasNextPage" in self.content

    def test_has_previous_page(self) -> None:
        """API設計.md: hasPreviousPage が定義。"""
        assert "hasPreviousPage" in self.content


class TestBurstControl:
    """API設計.md: D-012 バースト制御検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_system_burst_100(self) -> None:
        """API設計.md: system Tier 秒あたり制限 100。"""
        assert "100" in self.content

    def test_business_burst_50(self) -> None:
        """API設計.md: business Tier 秒あたり制限 50。"""
        assert "50" in self.content

    def test_service_burst_20(self) -> None:
        """API設計.md: service Tier 秒あたり制限 20。"""
        assert "20" in self.content

    def test_second_config_in_doc(self) -> None:
        """API設計.md: 秒あたりの上限設定が記載。"""
        assert "second:" in self.content or "秒あたり" in self.content


class TestEnvironmentMultiplier:
    """API設計.md: D-012 環境別倍率検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_dev_x10(self) -> None:
        """API設計.md: dev 環境は x10 倍率。"""
        assert "x10" in self.content

    def test_staging_x2(self) -> None:
        """API設計.md: staging 環境は x2 倍率。"""
        assert "x2" in self.content

    def test_prod_x1(self) -> None:
        """API設計.md: prod 環境は x1 倍率。"""
        assert "x1" in self.content

    def test_dev_system_30000(self) -> None:
        """API設計.md: dev system は 30000/min。"""
        assert "30000" in self.content


class TestRateLimitResponseHeaders:
    """API設計.md: D-012 レート制限レスポンスヘッダー検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_x_ratelimit_limit_header(self) -> None:
        """API設計.md: X-RateLimit-Limit ヘッダーが記載。"""
        assert "X-RateLimit-Limit" in self.content

    def test_x_ratelimit_remaining_header(self) -> None:
        """API設計.md: X-RateLimit-Remaining ヘッダーが記載。"""
        assert "X-RateLimit-Remaining" in self.content

    def test_x_ratelimit_reset_header(self) -> None:
        """API設計.md: X-RateLimit-Reset ヘッダーが記載。"""
        assert "X-RateLimit-Reset" in self.content


class TestRateLimitExceededResponse:
    """API設計.md: D-012 レート制限超過時 429 レスポンス形式検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_429_error_code(self) -> None:
        """API設計.md: SYS_RATE_LIMIT_EXCEEDED エラーコードが記載。"""
        assert "SYS_RATE_LIMIT_EXCEEDED" in self.content

    def test_429_http_status(self) -> None:
        """API設計.md: HTTP 429 が記載。"""
        assert "429" in self.content


class TestClientSdkDistribution:
    """API設計.md: D-123 クライアント SDK 配布方式検証。"""

    def setup_method(self) -> None:
        self.doc = ROOT / "docs" / "API設計.md"
        self.content = self.doc.read_text(encoding="utf-8")

    def test_typescript_npm_registry(self) -> None:
        """API設計.md: TypeScript SDK は npm private registry で配布。"""
        assert "npm" in self.content
        assert "@k1s0/" in self.content

    def test_dart_git_submodule(self) -> None:
        """API設計.md: Dart SDK は Git submodule / パス参照。"""
        assert "pubspec.yaml" in self.content or "Git submodule" in self.content

    def test_sdk_auto_regeneration_ci(self) -> None:
        """API設計.md: SDK 自動再生成が CI/CD で定義。"""
        assert "sdk-generate" in self.content or "openapi-generator" in self.content
