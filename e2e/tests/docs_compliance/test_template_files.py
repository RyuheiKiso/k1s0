"""テンプレートエンジン仕様.md の仕様準拠テスト。

CLI/templates/ 配下のテンプレートファイルが
テンプレートエンジン仕様.md で定義された構成と一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
DOCS = ROOT / "docs"
ENGINE_SPEC = DOCS / "テンプレートエンジン仕様.md"


class TestServerGoTemplates:
    """server/go テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "cmd/main.go.tera",
            "internal/domain/model/entity.go.tera",
            "internal/domain/repository/repository.go.tera",
            "internal/usecase/usecase.go.tera",
            "internal/usecase/usecase_test.go.tera",
            "internal/adapter/handler/rest_handler.go.tera",
            "internal/adapter/handler/grpc_handler.go.tera",
            "internal/adapter/handler/graphql_resolver.go.tera",
            "internal/adapter/handler/handler_test.go.tera",
            "internal/infra/persistence/db.go.tera",
            "internal/infra/persistence/repository.go.tera",
            "internal/infra/persistence/repository_test.go.tera",
            "internal/infra/messaging/kafka.go.tera",
            "internal/infra/config/config.go.tera",
            "config/config.yaml.tera",
            "api/openapi/openapi.yaml.tera",
            "api/proto/service.proto.tera",
            "api/graphql/schema.graphql.tera",
            "gqlgen.yml.tera",
            "buf.yaml.tera",
            "buf.gen.yaml.tera",
            "go.mod.tera",
            "oapi-codegen.yaml.tera",
            "Dockerfile.tera",
            "README.md.tera",
        ],
    )
    def test_server_go_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: server/go テンプレートが存在する。"""
        path = TEMPLATES / "server" / "go" / template
        assert path.exists(), f"server/go/{template} が存在しません"


class TestServerRustTemplates:
    """server/rust テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "src/main.rs.tera",
            "src/domain/mod.rs.tera",
            "src/domain/model.rs.tera",
            "src/domain/repository.rs.tera",
            "src/usecase/mod.rs.tera",
            "src/usecase/service.rs.tera",
            "src/adapter/mod.rs.tera",
            "src/adapter/handler/mod.rs.tera",
            "src/adapter/handler/rest.rs.tera",
            "src/adapter/handler/grpc.rs.tera",
            "src/adapter/handler/graphql.rs.tera",
            "src/infra/mod.rs.tera",
            "src/infra/persistence.rs.tera",
            "src/infra/messaging.rs.tera",
            "src/infra/config.rs.tera",
            "config/config.yaml.tera",
            "tests/integration_test.rs.tera",
            "build.rs.tera",
            "buf.yaml.tera",
            "Cargo.toml.tera",
            "Dockerfile.tera",
            "README.md.tera",
        ],
    )
    def test_server_rust_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: server/rust テンプレートが存在する。"""
        path = TEMPLATES / "server" / "rust" / template
        assert path.exists(), f"server/rust/{template} が存在しません"


class TestClientReactTemplates:
    """client/react テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "package.json.tera",
            "tsconfig.json.tera",
            "vite.config.ts.tera",
            "eslint.config.mjs.tera",
            ".prettierrc.tera",
            "vitest.config.ts.tera",
            "src/app/App.tsx.tera",
            "src/lib/api-client.ts.tera",
            "src/lib/query-client.ts.tera",
            "tests/testutil/setup.ts.tera",
            "tests/testutil/msw-setup.ts.tera",
            "tests/App.test.tsx.tera",
            "Dockerfile.tera",
            "nginx.conf.tera",
            "README.md.tera",
        ],
    )
    def test_client_react_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: client/react テンプレートが存在する。"""
        path = TEMPLATES / "client" / "react" / template
        assert path.exists(), f"client/react/{template} が存在しません"


class TestClientFlutterTemplates:
    """client/flutter テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "pubspec.yaml.tera",
            "analysis_options.yaml.tera",
            "lib/main.dart.tera",
            "lib/app/router.dart.tera",
            "lib/utils/dio_client.dart.tera",
            "test/widget_test.dart.tera",
            "Dockerfile.tera",
            "nginx.conf.tera",
            "README.md.tera",
        ],
    )
    def test_client_flutter_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: client/flutter テンプレートが存在する。"""
        path = TEMPLATES / "client" / "flutter" / template
        assert path.exists(), f"client/flutter/{template} が存在しません"


class TestLibraryGoTemplates:
    """library/go テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "go.mod.tera",
            "{name}.go.tera",
            "internal/internal.go.tera",
        ],
    )
    def test_library_go_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: library/go テンプレートが存在する。"""
        path = TEMPLATES / "library" / "go" / template
        assert path.exists(), f"library/go/{template} が存在しません"


class TestLibraryRustTemplates:
    """library/rust テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "Cargo.toml.tera",
            "src/lib.rs.tera",
            "src/{module}.rs.tera",
        ],
    )
    def test_library_rust_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: library/rust テンプレートが存在する。"""
        path = TEMPLATES / "library" / "rust" / template
        assert path.exists(), f"library/rust/{template} が存在しません"


class TestLibraryTypescriptTemplates:
    """library/typescript テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "package.json.tera",
            "tsconfig.json.tera",
            "src/index.ts.tera",
        ],
    )
    def test_library_typescript_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: library/typescript テンプレートが存在する。"""
        path = TEMPLATES / "library" / "typescript" / template
        assert path.exists(), f"library/typescript/{template} が存在しません"


class TestLibraryDartTemplates:
    """library/dart テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "pubspec.yaml.tera",
            "lib/{name}.dart.tera",
            "lib/src/{module}.dart.tera",
        ],
    )
    def test_library_dart_template_exists(self, template: str) -> None:
        """テンプレートエンジン仕様.md: library/dart テンプレートが存在する。"""
        path = TEMPLATES / "library" / "dart" / template
        assert path.exists(), f"library/dart/{template} が存在しません"


class TestDatabaseTemplates:
    """database テンプレートの検証。"""

    @pytest.mark.parametrize(
        "db_type,template",
        [
            ("postgresql", "001_init.up.sql.tera"),
            ("postgresql", "001_init.down.sql.tera"),
            ("mysql", "001_init.up.sql.tera"),
            ("mysql", "001_init.down.sql.tera"),
            ("sqlite", "001_init.up.sql.tera"),
            ("sqlite", "001_init.down.sql.tera"),
        ],
    )
    def test_database_template_exists(self, db_type: str, template: str) -> None:
        """テンプレートエンジン仕様.md: database テンプレートが存在する。"""
        path = TEMPLATES / "database" / db_type / template
        assert path.exists(), f"database/{db_type}/{template} が存在しません"


# --- ギャップ 1: テンプレート変数一覧(全20変数)の存在検証 ---


class TestTemplateVariables:
    """テンプレートエンジン仕様.md: テンプレート変数一覧の検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "variable",
        [
            "service_name",
            "service_name_snake",
            "service_name_pascal",
            "service_name_camel",
            "tier",
            "domain",
            "module_path",
            "language",
            "kind",
            "api_style",
            "has_database",
            "database_type",
            "has_kafka",
            "has_redis",
            "go_module",
            "rust_crate",
            "docker_registry",
            "docker_project",
            "api_styles",
            "framework",
        ],
    )
    def test_variable_documented(self, variable: str) -> None:
        """テンプレートエンジン仕様.md: 全20変数が仕様書に記載されている。"""
        assert f"| `{variable}`" in self.content, (
            f"変数 '{variable}' がテンプレート変数一覧に記載されていません"
        )


# --- ギャップ 2: ケース変換(snake/pascal/camel)の導出ルール ---


class TestCaseConversionRules:
    """テンプレートエンジン仕様.md: ケース変換の導出ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_snake_case_rule(self) -> None:
        """テンプレートエンジン仕様.md: snake_case 変換ルールが記載されている。"""
        assert "snake_case" in self.content
        assert "ハイフンをアンダースコアに置換" in self.content

    def test_pascal_case_rule(self) -> None:
        """テンプレートエンジン仕様.md: PascalCase 変換ルールが記載されている。"""
        assert "PascalCase" in self.content
        assert "各セグメントの先頭を大文字化して結合" in self.content

    def test_camel_case_rule(self) -> None:
        """テンプレートエンジン仕様.md: camelCase 変換ルールが記載されている。"""
        assert "camelCase" in self.content
        assert "先頭文字を小文字化" in self.content

    @pytest.mark.parametrize(
        "kebab,snake,pascal,camel",
        [
            ("order-api", "order_api", "OrderApi", "orderApi"),
            ("user-auth-service", "user_auth_service", "UserAuthService", "userAuthService"),
            ("inventory", "inventory", "Inventory", "inventory"),
        ],
    )
    def test_conversion_examples(self, kebab: str, snake: str, pascal: str, camel: str) -> None:
        """テンプレートエンジン仕様.md: 変換例がドキュメントに記載されている。"""
        assert kebab in self.content
        assert snake in self.content
        assert pascal in self.content
        assert camel in self.content


# --- ギャップ 3: tier → docker_project の導出ルール ---


class TestDockerProjectDerivation:
    """テンプレートエンジン仕様.md: docker_project の導出ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_derivation_formula(self) -> None:
        """テンプレートエンジン仕様.md: docker_project = 'k1s0-{tier}' が記載されている。"""
        assert 'docker_project = "k1s0-{tier}"' in self.content

    @pytest.mark.parametrize(
        "tier,expected",
        [
            ("system", "k1s0-system"),
            ("business", "k1s0-business"),
            ("service", "k1s0-service"),
        ],
    )
    def test_tier_to_docker_project(self, tier: str, expected: str) -> None:
        """テンプレートエンジン仕様.md: 各tierのdocker_project値が記載されている。"""
        assert expected in self.content


# --- ギャップ 4: domain 設定ルール(business Tierのみ) ---


class TestDomainSettingRule:
    """テンプレートエンジン仕様.md: domain 設定ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_business_tier_only(self) -> None:
        """テンプレートエンジン仕様.md: domain は business Tier のみで使用される。"""
        assert "business Tier のみ" in self.content

    def test_system_tier_empty(self) -> None:
        """テンプレートエンジン仕様.md: system Tier では空文字列。"""
        assert "system 層にドメイン概念はない" in self.content

    def test_service_tier_empty(self) -> None:
        """テンプレートエンジン仕様.md: service Tier では空文字列。"""
        assert "service 層はサービス名で直接分離" in self.content


# --- ギャップ 5: module_path 構成ルール(Tier別) ---


class TestModulePathRules:
    """テンプレートエンジン仕様.md: module_path 構成ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_service_tier_pattern(self) -> None:
        """テンプレートエンジン仕様.md: service Tier の module_path パターン。"""
        assert "regions/service/{service_name}/{kind}/{language}" in self.content

    def test_system_tier_pattern(self) -> None:
        """テンプレートエンジン仕様.md: system Tier の module_path パターン。"""
        assert "regions/system/{kind}/{language}/{service_name}" in self.content

    def test_business_tier_pattern(self) -> None:
        """テンプレートエンジン仕様.md: business Tier の module_path パターン。"""
        assert "regions/business/{domain}/{kind}/{language}/{service_name}" in self.content

    @pytest.mark.parametrize(
        "module_path",
        [
            "regions/service/order/server/go",
            "regions/system/library/rust/auth",
            "regions/business/accounting/server/go/ledger-api",
            "regions/business/fa/client/react/asset-app",
        ],
    )
    def test_module_path_examples(self, module_path: str) -> None:
        """テンプレートエンジン仕様.md: module_path の具体例が記載されている。"""
        assert module_path in self.content


# --- ギャップ 6: api_styles 複数選択対応 ---


class TestApiStylesMultiSelect:
    """テンプレートエンジン仕様.md: api_styles 複数選択対応の検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_api_styles_is_list(self) -> None:
        """テンプレートエンジン仕様.md: api_styles が Vec<String> 型として記載されている。"""
        assert "api_styles" in self.content
        assert "Vec" in self.content

    def test_backward_compatibility(self) -> None:
        """テンプレートエンジン仕様.md: api_style は api_styles の先頭要素。"""
        assert "api_styles" in self.content and "先頭要素" in self.content


# --- ギャップ 7: framework 設定ルール(client kindのみ) ---


class TestFrameworkSettingRule:
    """テンプレートエンジン仕様.md: framework 設定ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_client_kind_only(self) -> None:
        """テンプレートエンジン仕様.md: framework は client kind のみ。"""
        assert "**client** kind のみ" in self.content

    @pytest.mark.parametrize(
        "framework,language",
        [
            ("react", "typescript"),
            ("flutter", "dart"),
        ],
    )
    def test_framework_language_mapping(self, framework: str, language: str) -> None:
        """テンプレートエンジン仕様.md: framework と language の対応。"""
        assert framework in self.content
        assert language in self.content


# --- ギャップ 8: helm_path 構成ルール ---


class TestHelmPathRules:
    """テンプレートエンジン仕様.md: helm_path 構成ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_system_service_pattern(self) -> None:
        """テンプレートエンジン仕様.md: system/service Tier の helm_path パターン。"""
        assert 'helm_path = "{tier}/{service_name}"' in self.content

    def test_business_pattern(self) -> None:
        """テンプレートエンジン仕様.md: business Tier の helm_path パターン。"""
        assert 'helm_path = "business/{domain}/{service_name}"' in self.content

    @pytest.mark.parametrize(
        "helm_path",
        [
            "service/order",
            "system/auth",
            "business/accounting/ledger",
        ],
    )
    def test_helm_path_examples(self, helm_path: str) -> None:
        """テンプレートエンジン仕様.md: helm_path の具体例が記載されている。"""
        assert helm_path in self.content


# --- ギャップ 9: go_module 構成ルール ---


class TestGoModuleRules:
    """テンプレートエンジン仕様.md: go_module 構成ルール検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_derivation_formula(self) -> None:
        """テンプレートエンジン仕様.md: go_module の導出式。"""
        assert 'go_module = "github.com/org/k1s0/{module_path}"' in self.content

    @pytest.mark.parametrize(
        "go_module",
        [
            "github.com/org/k1s0/regions/service/order/server/go",
            "github.com/org/k1s0/regions/system/server/go/auth",
            "github.com/org/k1s0/regions/business/accounting/server/go/ledger-api",
        ],
    )
    def test_go_module_examples(self, go_module: str) -> None:
        """テンプレートエンジン仕様.md: go_module の具体例が記載されている。"""
        assert go_module in self.content


# --- ギャップ 10: 条件分岐構文パターン ---


class TestConditionalSyntaxPatterns:
    """テンプレートエンジン仕様.md: 条件分岐構文パターンの検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "pattern",
        [
            "{% if has_database %}",
            '{% if api_style == "rest" %}',
            '{% elif api_style == "grpc" %}',
            '{% elif api_style == "graphql" %}',
            '{% if language == "go" %}',
            '{% elif language == "rust" %}',
            "{% if has_kafka %}",
            "{% if has_redis %}",
            "{% endif %}",
        ],
    )
    def test_conditional_pattern(self, pattern: str) -> None:
        """テンプレートエンジン仕様.md: 条件分岐パターンが記載されている。"""
        assert pattern in self.content

    def test_compound_condition(self) -> None:
        """テンプレートエンジン仕様.md: 複合条件の例が記載されている。"""
        assert 'has_database and database_type == "postgresql"' in self.content


# --- ギャップ 11: ループ構文 ---


class TestLoopSyntax:
    """テンプレートエンジン仕様.md: ループ構文の検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_for_endfor(self) -> None:
        """テンプレートエンジン仕様.md: for/endfor 構文が記載されている。"""
        assert "{% for" in self.content
        assert "{% endfor %}" in self.content

    @pytest.mark.parametrize(
        "prop",
        [
            "loop.index",
            "loop.index0",
            "loop.first",
            "loop.last",
        ],
    )
    def test_loop_properties(self, prop: str) -> None:
        """テンプレートエンジン仕様.md: loop オブジェクトプロパティが記載されている。"""
        assert prop in self.content


# --- ギャップ 12: カスタムフィルタの登録・動作 ---


class TestCustomFilters:
    """テンプレートエンジン仕様.md: カスタムフィルタの検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "filter_name",
        [
            "snake_case",
            "pascal_case",
            "camel_case",
            "kebab_case",
            "upper_case",
            "lower_case",
        ],
    )
    def test_custom_filter_documented(self, filter_name: str) -> None:
        """テンプレートエンジン仕様.md: カスタムフィルタが記載されている。"""
        assert f"| `{filter_name}`" in self.content

    @pytest.mark.parametrize(
        "builtin_filter",
        [
            "default(value=",
            "trim",
            "replace(from=",
            "upper",
            "lower",
            "capitalize",
            "title",
            "length",
            "join(sep=",
        ],
    )
    def test_builtin_filter_documented(self, builtin_filter: str) -> None:
        """テンプレートエンジン仕様.md: 組み込みフィルタが記載されている。"""
        assert builtin_filter in self.content


# --- ギャップ 13: 生成後の後処理 ---


class TestPostProcessing:
    """テンプレートエンジン仕様.md: 生成後の後処理の検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "language,command",
        [
            ("Go", "go mod tidy"),
            ("Rust", "cargo check"),
            ("TypeScript/React", "npm install"),
            ("Dart/Flutter", "flutter pub get"),
            ("protobuf", "buf generate"),
            ("OpenAPI", "oapi-codegen"),
        ],
    )
    def test_post_processing_command(self, language: str, command: str) -> None:
        """テンプレートエンジン仕様.md: 後処理コマンドが記載されている。"""
        assert command in self.content

    def test_execution_order(self) -> None:
        """テンプレートエンジン仕様.md: 後処理の実行順序が記載されている。"""
        content = self.content
        step1 = content.find("テンプレートからファイルを生成")
        step2 = content.find("言語固有の依存解決")
        step3 = content.find("コード生成（`buf generate`")
        step4 = content.find("SQL マイグレーションの初期化")
        assert step1 < step2 < step3 < step4, "後処理の実行順序が仕様と一致しません"

    def test_failure_behavior(self) -> None:
        """テンプレートエンジン仕様.md: 後処理失敗時の動作が記載されている。"""
        assert "エラー内容を標準エラー出力に表示" in self.content
        assert "ロールバックしない" in self.content
        assert "手動での修正を促すメッセージを表示" in self.content


# --- ギャップ 14: テンプレート作成ガイドライン ---


class TestTemplateGuidelines:
    """テンプレートエンジン仕様.md: テンプレート作成ガイドライン検証。"""

    def setup_method(self) -> None:
        self.content = ENGINE_SPEC.read_text(encoding="utf-8")

    def test_tera_extension_rule(self) -> None:
        """テンプレートエンジン仕様.md: .tera 拡張子ルールが記載されている。"""
        assert ".tera" in self.content

    def test_no_blank_lines_in_conditional(self) -> None:
        """テンプレートエンジン仕様.md: 条件ブロック前後の空行禁止ルール。"""
        assert "条件ブロックの前後に空行を入れない" in self.content

    def test_max_nesting_depth(self) -> None:
        """テンプレートエンジン仕様.md: ネスト最大2段ルール。"""
        assert "ネストは最大2段まで" in self.content

    def test_common_code_outside_condition(self) -> None:
        """テンプレートエンジン仕様.md: 共通部分を条件外に出すルール。"""
        assert "共通部分は条件の外に出す" in self.content

    def test_test_method_documented(self) -> None:
        """テンプレートエンジン仕様.md: テンプレートのテスト方法が記載されている。"""
        assert "テストコンテキスト（JSON）を作成する" in self.content
        assert "Tera CLI でレンダリングを確認する" in self.content
