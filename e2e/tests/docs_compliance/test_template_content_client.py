"""テンプレート仕様-クライアント.md の内容準拠テスト。

CLI/templates/client/ のテンプレートファイルの内容が
仕様ドキュメントのコードブロックと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
REACT = TEMPLATES / "client" / "react"
FLUTTER = TEMPLATES / "client" / "flutter"


class TestReactPackageJsonContent:
    """テンプレート仕様-クライアント.md: package.json.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "package.json.tera").read_text(encoding="utf-8")

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_version(self) -> None:
        assert '"0.1.0"' in self.content

    def test_type_module(self) -> None:
        assert '"type": "module"' in self.content

    @pytest.mark.parametrize(
        "script",
        ["dev", "build", "test", "test:watch", "test:coverage", "lint", "lint:fix", "format", "format:check"],
    )
    def test_script_defined(self, script: str) -> None:
        assert f'"{script}"' in self.content

    @pytest.mark.parametrize(
        "dep",
        [
            "react",
            "react-dom",
            "@tanstack/react-query",
            "@tanstack/react-router",
            "zustand",
            "react-hook-form",
            "zod",
            "axios",
        ],
    )
    def test_dependency_defined(self, dep: str) -> None:
        assert dep in self.content

    @pytest.mark.parametrize(
        "dev_dep",
        [
            "typescript",
            "vite",
            "vitest",
            "@testing-library/react",
            "msw",
            "eslint",
            "prettier",
            "tailwindcss",
        ],
    )
    def test_dev_dependency_defined(self, dev_dep: str) -> None:
        assert dev_dep in self.content


class TestReactTsconfigContent:
    """テンプレート仕様-クライアント.md: tsconfig.json.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "tsconfig.json.tera").read_text(encoding="utf-8")

    def test_strict(self) -> None:
        assert '"strict": true' in self.content

    def test_jsx(self) -> None:
        assert '"react-jsx"' in self.content

    def test_module(self) -> None:
        assert '"ESNext"' in self.content

    def test_path_alias(self) -> None:
        assert '"@/*"' in self.content


class TestReactViteConfigContent:
    """テンプレート仕様-クライアント.md: vite.config.ts.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "vite.config.ts.tera").read_text(encoding="utf-8")

    def test_react_plugin(self) -> None:
        assert "react()" in self.content

    def test_tailwind_plugin(self) -> None:
        assert "tailwindcss()" in self.content

    def test_proxy(self) -> None:
        assert "'/api'" in self.content
        assert "'http://localhost:8080'" in self.content

    def test_port(self) -> None:
        assert "3000" in self.content


class TestReactEslintConfigContent:
    """テンプレート仕様-クライアント.md: eslint.config.mjs.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "eslint.config.mjs.tera").read_text(encoding="utf-8")

    def test_eslint_import(self) -> None:
        assert "from '@eslint/js'" in self.content

    def test_typescript_eslint(self) -> None:
        assert "typescript-eslint" in self.content

    def test_react_hooks_rules(self) -> None:
        assert "'react-hooks/rules-of-hooks': 'error'" in self.content

    def test_import_order(self) -> None:
        assert "'import/order'" in self.content


class TestReactPrettierrcContent:
    """テンプレート仕様-クライアント.md: .prettierrc.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / ".prettierrc.tera").read_text(encoding="utf-8")

    def test_semi(self) -> None:
        assert '"semi": true' in self.content

    def test_single_quote(self) -> None:
        assert '"singleQuote": true' in self.content

    def test_print_width(self) -> None:
        assert '"printWidth": 100' in self.content


class TestReactVitestConfigContent:
    """テンプレート仕様-クライアント.md: vitest.config.ts.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "vitest.config.ts.tera").read_text(encoding="utf-8")

    def test_jsdom_environment(self) -> None:
        assert "'jsdom'" in self.content

    def test_v8_coverage(self) -> None:
        assert "'v8'" in self.content

    def test_setup_files(self) -> None:
        assert "setup" in self.content


class TestReactAppTsxContent:
    """テンプレート仕様-クライアント.md: App.tsx.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "src" / "app" / "App.tsx.tera").read_text(encoding="utf-8")

    def test_query_client_provider(self) -> None:
        assert "QueryClientProvider" in self.content

    def test_router_provider(self) -> None:
        assert "RouterProvider" in self.content

    def test_tanstack_router(self) -> None:
        assert "@tanstack/react-router" in self.content


class TestReactApiClientContent:
    """テンプレート仕様-クライアント.md: api-client.ts.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "src" / "lib" / "api-client.ts.tera").read_text(encoding="utf-8")

    def test_axios_create(self) -> None:
        assert "axios.create" in self.content

    def test_with_credentials(self) -> None:
        """テンプレート仕様-クライアント.md: BFF + HttpOnly Cookie 方式。"""
        assert "withCredentials: true" in self.content

    def test_csrf_token(self) -> None:
        assert "X-CSRF-Token" in self.content

    def test_error_interceptor(self) -> None:
        assert "401" in self.content
        assert "403" in self.content
        assert "500" in self.content


class TestReactQueryClientContent:
    """テンプレート仕様-クライアント.md: query-client.ts.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "src" / "lib" / "query-client.ts.tera").read_text(encoding="utf-8")

    def test_query_client(self) -> None:
        assert "QueryClient" in self.content

    def test_stale_time(self) -> None:
        assert "staleTime" in self.content

    def test_gc_time(self) -> None:
        assert "gcTime" in self.content


class TestReactMswSetupContent:
    """テンプレート仕様-クライアント.md: msw-setup.ts.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "tests" / "testutil" / "msw-setup.ts.tera").read_text(encoding="utf-8")

    def test_msw_import(self) -> None:
        assert "setupServer" in self.content
        assert "msw/node" in self.content

    def test_lifecycle(self) -> None:
        assert "server.listen" in self.content
        assert "server.resetHandlers" in self.content
        assert "server.close" in self.content


class TestReactDockerfileContent:
    """テンプレート仕様-クライアント.md: Dockerfile.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "Dockerfile.tera").read_text(encoding="utf-8")

    def test_node_base_image(self) -> None:
        assert "node:22" in self.content

    def test_nginx_runtime(self) -> None:
        assert "nginx:" in self.content

    def test_npm_ci(self) -> None:
        assert "npm ci" in self.content

    def test_expose(self) -> None:
        assert "EXPOSE 8080" in self.content


class TestReactNginxConfContent:
    """テンプレート仕様-クライアント.md: nginx.conf.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (REACT / "nginx.conf.tera").read_text(encoding="utf-8")

    def test_listen_port(self) -> None:
        assert "listen 8080" in self.content

    def test_spa_fallback(self) -> None:
        assert "/index.html" in self.content
        assert "try_files" in self.content

    def test_gzip(self) -> None:
        assert "gzip on" in self.content

    def test_security_headers(self) -> None:
        assert "X-Frame-Options" in self.content
        assert "X-Content-Type-Options" in self.content


class TestFlutterPubspecContent:
    """テンプレート仕様-クライアント.md: pubspec.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (FLUTTER / "pubspec.yaml.tera").read_text(encoding="utf-8")

    def test_service_name_variable(self) -> None:
        assert "{{ service_name_snake }}" in self.content

    def test_publish_to_none(self) -> None:
        assert "publish_to: none" in self.content

    @pytest.mark.parametrize(
        "dep",
        ["flutter_riverpod", "go_router", "freezed_annotation", "json_annotation", "dio"],
    )
    def test_dependency_defined(self, dep: str) -> None:
        assert dep in self.content

    @pytest.mark.parametrize(
        "dev_dep",
        ["build_runner", "freezed", "json_serializable", "mocktail", "flutter_lints"],
    )
    def test_dev_dependency_defined(self, dev_dep: str) -> None:
        assert dev_dep in self.content


class TestFlutterAnalysisOptionsContent:
    """テンプレート仕様-クライアント.md: analysis_options.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (FLUTTER / "analysis_options.yaml.tera").read_text(encoding="utf-8")

    def test_flutter_lints(self) -> None:
        assert "flutter_lints/flutter.yaml" in self.content

    def test_prefer_const(self) -> None:
        assert "prefer_const_constructors" in self.content

    def test_avoid_print(self) -> None:
        assert "avoid_print" in self.content

    def test_exclude_generated(self) -> None:
        assert "*.g.dart" in self.content
        assert "*.freezed.dart" in self.content


class TestFlutterMainDartContent:
    """テンプレート仕様-クライアント.md: main.dart.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (FLUTTER / "lib" / "main.dart.tera").read_text(encoding="utf-8")

    def test_riverpod(self) -> None:
        assert "ProviderScope" in self.content

    def test_material_router(self) -> None:
        assert "MaterialApp.router" in self.content

    def test_material3(self) -> None:
        assert "useMaterial3: true" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name_pascal }}" in self.content


class TestFlutterRouterContent:
    """テンプレート仕様-クライアント.md: router.dart.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (FLUTTER / "lib" / "app" / "router.dart.tera").read_text(encoding="utf-8")

    def test_go_router(self) -> None:
        assert "GoRouter" in self.content

    def test_home_screen(self) -> None:
        assert "HomeScreen" in self.content

    def test_not_found_screen(self) -> None:
        assert "NotFoundScreen" in self.content

    def test_error_builder(self) -> None:
        assert "errorBuilder" in self.content


class TestFlutterDioClientContent:
    """テンプレート仕様-クライアント.md: dio_client.dart.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (FLUTTER / "lib" / "utils" / "dio_client.dart.tera").read_text(encoding="utf-8")

    def test_dio_import(self) -> None:
        assert "package:dio/dio.dart" in self.content

    def test_with_credentials(self) -> None:
        """テンプレート仕様-クライアント.md: BFF + Cookie 認証。"""
        assert "'withCredentials': true" in self.content

    def test_timeout(self) -> None:
        assert "connectTimeout" in self.content
        assert "receiveTimeout" in self.content

    def test_error_handling(self) -> None:
        assert "401" in self.content
        assert "403" in self.content
        assert "500" in self.content


class TestFlutterDockerfileContent:
    """テンプレート仕様-クライアント.md: Dockerfile.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (FLUTTER / "Dockerfile.tera").read_text(encoding="utf-8")

    def test_flutter_base_image(self) -> None:
        assert "flutter" in self.content

    def test_nginx_runtime(self) -> None:
        assert "nginx:" in self.content

    def test_flutter_build_web(self) -> None:
        assert "flutter build web" in self.content

    def test_expose(self) -> None:
        assert "EXPOSE 8080" in self.content


class TestFlutterNginxConfContent:
    """テンプレート仕様-クライアント.md: nginx.conf.tera が React と同じ。"""

    def test_nginx_conf_exists(self) -> None:
        assert (FLUTTER / "nginx.conf.tera").exists()

    def test_listen_port(self) -> None:
        content = (FLUTTER / "nginx.conf.tera").read_text(encoding="utf-8")
        assert "listen 8080" in content
