"""テンプレートエンジン仕様.md の仕様準拠テスト。

CLI/templates/ 配下のテンプレートファイルが
テンプレートエンジン仕様.md で定義された構成と一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"


class TestServerGoTemplates:
    """server/go テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "cmd/main.go.tera",
            "internal/domain/model/entity.go.tera",
            "internal/domain/repository/repository.go.tera",
            "internal/usecase/usecase.go.tera",
            "internal/adapter/handler/rest_handler.go.tera",
            "internal/adapter/handler/grpc_handler.go.tera",
            "internal/adapter/handler/graphql_resolver.go.tera",
            "internal/infra/persistence/db.go.tera",
            "internal/infra/persistence/repository.go.tera",
            "internal/infra/messaging/kafka.go.tera",
            "internal/infra/config/config.go.tera",
            "config/config.yaml.tera",
            "api/openapi/openapi.yaml.tera",
            "api/proto/service.proto.tera",
            "go.mod.tera",
            "Dockerfile.tera",
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
            "Cargo.toml.tera",
            "Dockerfile.tera",
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
            "tests/testutil/msw-setup.ts.tera",
            "Dockerfile.tera",
            "nginx.conf.tera",
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
            "Dockerfile.tera",
            "nginx.conf.tera",
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
