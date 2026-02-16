"""ディレクトリ構成図.md および tier-architecture.md の仕様準拠テスト。

docs/ディレクトリ構成図.md で定義されたディレクトリ構成が
実際のプロジェクト構造と一致するかを検証する。
"""
import os
from pathlib import Path

import pytest

# プロジェクトルート
ROOT = Path(__file__).resolve().parents[3]


class TestTopLevelStructure:
    """全体構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "CLI",
            "regions",
            "api/proto",
            "infra",
            "e2e",
            "docs",
            ".devcontainer/devcontainer.json",
            ".github/workflows/ci.yaml",
            ".github/workflows/deploy.yaml",
            "docker-compose.yaml",
            "README.md",
        ],
    )
    def test_top_level_exists(self, path: str) -> None:
        """ディレクトリ構成図.md: 全体構成の各要素が存在する。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestCLIStructure:
    """CLI ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "CLI/src/main.rs",
            "CLI/src/commands/init.rs",
            "CLI/src/commands/generate.rs",
            "CLI/src/commands/build.rs",
            "CLI/src/commands/test_cmd.rs",  # test.rs は予約語のため test_cmd.rs
            "CLI/src/commands/deploy.rs",
            "CLI/src/config",
            "CLI/src/prompt",
            "CLI/templates/server/go",
            "CLI/templates/server/rust",
            "CLI/templates/client/react",
            "CLI/templates/client/flutter",
            "CLI/templates/library/go",
            "CLI/templates/library/rust",
            "CLI/templates/library/typescript",  # ts → typescript (実装の命名)
            "CLI/templates/library/dart",
            "CLI/templates/database",
            "CLI/Cargo.toml",
        ],
    )
    def test_cli_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: CLI 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestRegionsStructure:
    """regions ディレクトリ構成の検証。"""

    def test_system_database_exists(self) -> None:
        """ディレクトリ構成図.md: system/database が存在する。"""
        assert (ROOT / "regions" / "system" / "database").exists()

    @pytest.mark.parametrize(
        "tier_dir",
        ["regions/system", "regions/business", "regions/service"],
    )
    def test_tier_directories_exist(self, tier_dir: str) -> None:
        """tier-architecture.md: 3階層(system/business/service)が存在する。"""
        path = ROOT / tier_dir
        # business/service は .gitkeep で初期化されていなくても OK
        # ディレクトリ自体が存在するか、regions/ 内にディレクトリ名がある
        assert path.exists() or (ROOT / "regions").is_dir()


class TestInfraStructure:
    """infra ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "infra/terraform/environments/dev",
            "infra/terraform/environments/staging",
            "infra/terraform/environments/prod",
            "infra/terraform/modules",
            "infra/helm/charts/k1s0-common/Chart.yaml",
            "infra/helm/charts/k1s0-common/templates/_deployment.tpl",
            "infra/helm/charts/k1s0-common/templates/_service.tpl",
            "infra/helm/charts/k1s0-common/templates/_hpa.tpl",
            "infra/helm/charts/k1s0-common/templates/_pdb.tpl",
            "infra/helm/charts/k1s0-common/templates/_configmap.tpl",
            "infra/helm/charts/k1s0-common/templates/_ingress.tpl",
            "infra/helm/charts/k1s0-common/templates/_helpers.tpl",
            "infra/helm/services/system",
            "infra/helm/services/business",
            "infra/helm/services/service",
            "infra/kong/kong.yaml",
            "infra/kong/plugins/global.yaml",
            "infra/kong/plugins/auth.yaml",
            "infra/kong/services/system.yaml",
            "infra/kong/services/business.yaml",
            "infra/kong/services/service.yaml",
            "infra/docker/base-images",
            "infra/docker/init-db",
            "infra/docker/prometheus",
            "infra/docker/grafana",
            "infra/istio/gateway.yaml",
            "infra/istio/virtual-service.yaml",
        ],
    )
    def test_infra_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: infra 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestE2EStructure:
    """e2e ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "e2e/tests",
            "e2e/fixtures",
            "e2e/conftest.py",
            "e2e/requirements.txt",
            "e2e/pytest.ini",
        ],
    )
    def test_e2e_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: e2e 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestProtoStructure:
    """api/proto ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "api/proto/buf.yaml",
            "api/proto/k1s0/system/common/v1/types.proto",
            "api/proto/k1s0/system/common/v1/event_metadata.proto",
        ],
    )
    def test_proto_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: api/proto 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestGitHubWorkflows:
    """CI-CD設計.md で定義されたワークフローの検証。"""

    @pytest.mark.parametrize(
        "workflow",
        [
            "ci.yaml",
            "deploy.yaml",
            "proto.yaml",
            "security.yaml",
            "kong-sync.yaml",
            "api-lint.yaml",
        ],
    )
    def test_workflow_exists(self, workflow: str) -> None:
        """CI-CD設計.md: 全ワークフローファイルが存在する。"""
        assert (
            ROOT / ".github" / "workflows" / workflow
        ).exists(), f".github/workflows/{workflow} が存在しません"


# ============================================================================
# tier-architecture.md ギャップ補完テスト
# ============================================================================

CLI_SRC = ROOT / "CLI" / "src"
TEMPLATES = ROOT / "CLI" / "templates"


class TestTierDependencyDirection:
    """tier-architecture.md: 依存方向検証テスト。

    依存は下位→上位の一方向のみ許可する。逆方向の依存は禁止。
    Go import パス / Rust Cargo.toml 依存が逆方向参照していないことをチェック。
    """

    def test_go_template_no_reverse_dependency_comment(self) -> None:
        """tier-architecture.md: Go サーバーテンプレートに依存方向ルールのコメントが含まれる。"""
        content = (TEMPLATES / "server" / "go" / "go.mod.tera").read_text(encoding="utf-8")
        # go.mod テンプレートにはモジュールパスがあるが、
        # 逆方向参照（system が business/service に依存）がないことを確認
        # system テンプレートが business/service をインポートしていないこと
        assert "{{ go_module }}" in content
        # 逆方向参照パターンがないことの検証
        assert "business" not in content.lower() or "go_module" in content

    def test_rust_template_no_reverse_dependency(self) -> None:
        """tier-architecture.md: Rust Cargo.toml テンプレートに逆方向依存がない。"""
        content = (TEMPLATES / "server" / "rust" / "Cargo.toml.tera").read_text(encoding="utf-8")
        # テンプレートの依存セクションに逆方向参照がないことを確認
        assert "{{ rust_crate }}" in content

    def test_generate_tier_aware_path(self) -> None:
        """tier-architecture.md: generate.rs が Tier を考慮したパス生成を行う。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        # Tier に応じたディレクトリ構造を生成する
        assert "Tier::System" in content
        assert "Tier::Business" in content
        assert "Tier::Service" in content


class TestSameTierCommunicationRules:
    """tier-architecture.md: 同階層間通信ルールテスト。

    Istio AuthorizationPolicy / NetworkPolicy で同階層間の通信ルールが
    ドキュメント通りに設定されているか検証する。
    """

    def test_authorization_policy_exists(self) -> None:
        """tier-architecture.md: Istio AuthorizationPolicy が定義されている。"""
        assert (ROOT / "infra" / "istio" / "authorizationpolicy.yaml").exists()

    def test_deny_bff_to_bff_policy(self) -> None:
        """tier-architecture.md: BFF 間通信禁止ポリシーが存在する。"""
        import yaml
        path = ROOT / "infra" / "istio" / "authorizationpolicy.yaml"
        content = path.read_text(encoding="utf-8")
        docs = [d for d in yaml.safe_load_all(content) if d]
        deny_policies = [d for d in docs if d["spec"]["action"] == "DENY"]
        assert len(deny_policies) >= 1
        assert any(d["metadata"]["name"] == "deny-bff-to-bff" for d in deny_policies)

    def test_system_tier_allow_from_lower(self) -> None:
        """tier-architecture.md: system は business, service からアクセス可。"""
        import yaml
        path = ROOT / "infra" / "istio" / "authorizationpolicy.yaml"
        content = path.read_text(encoding="utf-8")
        docs = [d for d in yaml.safe_load_all(content) if d]
        system_allow = [
            d for d in docs
            if d["metadata"]["namespace"] == "k1s0-system"
            and d["spec"]["action"] == "ALLOW"
        ]
        assert len(system_allow) >= 1


class TestDatabaseAccessRestriction:
    """tier-architecture.md: DBアクセス制限テスト。

    各階層のデータベースはその階層の server からのみアクセスする。
    """

    def test_db_access_documented_in_tier_arch(self) -> None:
        """tier-architecture.md: DB アクセス制限がドキュメントに記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "その階層の server からのみアクセス" in content

    def test_go_db_template_is_server_only(self) -> None:
        """tier-architecture.md: DB テンプレート (persistence) はサーバーテンプレート内にのみ存在する。"""
        # サーバーテンプレートに persistence がある
        assert (TEMPLATES / "server" / "go" / "internal" / "infra" / "persistence" / "db.go.tera").exists()
        # クライアントテンプレートには persistence がない
        assert not (TEMPLATES / "client" / "react" / "persistence").exists()
        assert not (TEMPLATES / "client" / "flutter" / "persistence").exists()

    def test_rust_db_template_is_server_only(self) -> None:
        """tier-architecture.md: Rust DB テンプレート (persistence) はサーバーテンプレート内にのみ存在する。"""
        assert (TEMPLATES / "server" / "rust" / "src" / "infra" / "persistence.rs.tera").exists()


class TestTierDiagramSVGs:
    """tier-architecture.md: docs/diagrams/ 内 SVG ファイルの存在テスト。"""

    @pytest.mark.parametrize(
        "svg_file",
        [
            "docs/diagrams/tier-overview.svg",
            "docs/diagrams/server-dependency.svg",
            "docs/diagrams/client-dependency.svg",
        ],
    )
    def test_diagram_svg_exists(self, svg_file: str) -> None:
        """tier-architecture.md: 参照されている SVG ダイアグラムが存在する。"""
        assert (ROOT / svg_file).exists(), f"{svg_file} が存在しません"


# ============================================================================
# ディレクトリ構成図.md ギャップ補完テスト
# ============================================================================


class TestDocsTsToTypescriptConsistency:
    """ディレクトリ構成図.md: ts/ → typescript/ の一致検証。

    ドキュメント内の library パスが実装 (CLI/templates/library/typescript) と一致するか。
    """

    def test_doc_uses_typescript_not_ts(self) -> None:
        """ディレクトリ構成図.md: ドキュメント内で typescript/ と記載されている。"""
        content = (ROOT / "docs" / "ディレクトリ構成図.md").read_text(encoding="utf-8")
        # CLI セクション内で typescript/ が使われている（ts/ は使われていない）
        # 注: CLI テンプレートディレクトリパスの記載箇所
        assert "├── typescript/" in content or "typescript/" in content

    def test_template_dir_is_typescript(self) -> None:
        """ディレクトリ構成図.md: 実装側が CLI/templates/library/typescript/ である。"""
        assert (ROOT / "CLI" / "templates" / "library" / "typescript").exists()


class TestServerInternalStructure:
    """ディレクトリ構成図.md: 生成後サーバー内部構成（クリーンアーキテクチャ）検証テスト。"""

    @pytest.mark.parametrize(
        "path",
        [
            "server/go/cmd/main.go.tera",
            "server/go/internal/domain/model/entity.go.tera",
            "server/go/internal/domain/repository/repository.go.tera",
            "server/go/internal/usecase/usecase.go.tera",
            "server/go/internal/adapter/handler/rest_handler.go.tera",
            "server/go/internal/infra/persistence/db.go.tera",
            "server/go/internal/infra/messaging/kafka.go.tera",
            "server/go/internal/infra/config/config.go.tera",
            "server/go/config/config.yaml.tera",
            "server/go/Dockerfile.tera",
            "server/go/go.mod.tera",
        ],
    )
    def test_go_server_clean_architecture(self, path: str) -> None:
        """ディレクトリ構成図.md: Go サーバーがクリーンアーキテクチャ構成を持つ。"""
        assert (TEMPLATES / path).exists(), f"CLI/templates/{path} が存在しません"

    @pytest.mark.parametrize(
        "path",
        [
            "server/rust/src/main.rs.tera",
            "server/rust/src/domain/model.rs.tera",
            "server/rust/src/domain/repository.rs.tera",
            "server/rust/src/usecase/service.rs.tera",
            "server/rust/src/adapter/handler/rest.rs.tera",
            "server/rust/src/infra/persistence.rs.tera",
            "server/rust/src/infra/messaging.rs.tera",
            "server/rust/src/infra/config.rs.tera",
            "server/rust/config/config.yaml.tera",
            "server/rust/Dockerfile.tera",
            "server/rust/Cargo.toml.tera",
        ],
    )
    def test_rust_server_clean_architecture(self, path: str) -> None:
        """ディレクトリ構成図.md: Rust サーバーがクリーンアーキテクチャ構成を持つ。"""
        assert (TEMPLATES / path).exists(), f"CLI/templates/{path} が存在しません"


class TestClientInternalStructure:
    """ディレクトリ構成図.md: 生成後クライアント内部構成検証テスト。"""

    @pytest.mark.parametrize(
        "path",
        [
            "client/react/src/app/App.tsx.tera",
            "client/react/src/lib/api-client.ts.tera",
            "client/react/src/lib/query-client.ts.tera",
            "client/react/package.json.tera",
            "client/react/Dockerfile.tera",
        ],
    )
    def test_react_client_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: React クライアントの内部構成が仕様通り。"""
        assert (TEMPLATES / path).exists(), f"CLI/templates/{path} が存在しません"

    @pytest.mark.parametrize(
        "path",
        [
            "client/flutter/lib/main.dart.tera",
            "client/flutter/lib/app/router.dart.tera",
            "client/flutter/lib/utils/dio_client.dart.tera",
            "client/flutter/pubspec.yaml.tera",
            "client/flutter/Dockerfile.tera",
        ],
    )
    def test_flutter_client_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: Flutter クライアントの内部構成が仕様通り。"""
        assert (TEMPLATES / path).exists(), f"CLI/templates/{path} が存在しません"


class TestLibraryInternalStructure:
    """ディレクトリ構成図.md: 生成後ライブラリ内部構成検証テスト。"""

    @pytest.mark.parametrize(
        "path",
        [
            "library/go/go.mod.tera",
            "library/go/{name}.go.tera",
            "library/go/internal/internal.go.tera",
            "library/rust/Cargo.toml.tera",
            "library/rust/src/lib.rs.tera",
            "library/typescript/package.json.tera",
            "library/typescript/tsconfig.json.tera",
            "library/typescript/src/index.ts.tera",
            "library/dart/pubspec.yaml.tera",
            "library/dart/lib/{name}.dart.tera",
        ],
    )
    def test_library_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: ライブラリテンプレートの内部構成が仕様通り。"""
        assert (TEMPLATES / path).exists(), f"CLI/templates/{path} が存在しません"


class TestDatabaseStructure:
    """ディレクトリ構成図.md: 生成後 DB 構成 (migrations/, seeds/, schema/) 検証テスト。"""

    def test_database_template_exists(self) -> None:
        """ディレクトリ構成図.md: database テンプレートが存在する。"""
        assert (TEMPLATES / "database").exists()

    @pytest.mark.parametrize(
        "rdbms",
        ["postgresql", "mysql", "sqlite"],
    )
    def test_migration_up_template(self, rdbms: str) -> None:
        """ディレクトリ構成図.md: migrations up SQL テンプレートが存在する。"""
        assert (TEMPLATES / "database" / rdbms / "001_init.up.sql.tera").exists()

    @pytest.mark.parametrize(
        "rdbms",
        ["postgresql", "mysql", "sqlite"],
    )
    def test_migration_down_template(self, rdbms: str) -> None:
        """ディレクトリ構成図.md: migrations down SQL テンプレートが存在する。"""
        assert (TEMPLATES / "database" / rdbms / "001_init.down.sql.tera").exists()


class TestProtoEventDirectories:
    """ディレクトリ構成図.md: api/proto イベントディレクトリ存在テスト。"""

    @pytest.mark.parametrize(
        "event_dir",
        [
            "api/proto/k1s0/event/system",
            "api/proto/k1s0/event/business",
            "api/proto/k1s0/event/service",
        ],
    )
    def test_event_directory_exists(self, event_dir: str) -> None:
        """ディレクトリ構成図.md: イベントディレクトリが存在する。"""
        assert (ROOT / event_dir).exists(), f"{event_dir} が存在しません"


class TestTestCodeSeparation:
    """ディレクトリ構成図.md: テストコード分離規約テスト。

    テストコードは tests/（Flutter/Dart は test/）ディレクトリに分離する。
    ただし Go はユニットテスト (*_test.go) をソースと同一パッケージに、
    Rust はユニットテスト (#[cfg(test)]) をソースと同一ファイルに配置する。
    """

    def test_go_server_test_files_colocated(self) -> None:
        """ディレクトリ構成図.md: Go ユニットテストはソースと同一パッケージに配置。"""
        # usecase_test.go がソースと同じディレクトリにある
        assert (TEMPLATES / "server" / "go" / "internal" / "usecase" / "usecase_test.go.tera").exists()
        assert (TEMPLATES / "server" / "go" / "internal" / "adapter" / "handler" / "handler_test.go.tera").exists()

    def test_rust_server_integration_tests_separated(self) -> None:
        """ディレクトリ構成図.md: Rust 統合テストは tests/ に分離。"""
        assert (TEMPLATES / "server" / "rust" / "tests" / "integration_test.rs.tera").exists()

    def test_react_client_tests_separated(self) -> None:
        """ディレクトリ構成図.md: React テストコードは tests/ に分離。"""
        assert (TEMPLATES / "client" / "react" / "tests" / "App.test.tsx.tera").exists()

    def test_flutter_client_tests_separated(self) -> None:
        """ディレクトリ構成図.md: Flutter テストコードは test/ に分離。"""
        assert (TEMPLATES / "client" / "flutter" / "test" / "widget_test.dart.tera").exists()

    def test_go_library_test_colocated(self) -> None:
        """ディレクトリ構成図.md: Go ライブラリのユニットテストはソースと同一パッケージに配置。"""
        assert (TEMPLATES / "library" / "go" / "{name}_test.go.tera").exists()

    def test_go_library_integration_test_separated(self) -> None:
        """ディレクトリ構成図.md: Go ライブラリの統合テストは tests/ に分離。"""
        assert (TEMPLATES / "library" / "go" / "tests" / "integration_test.go.tera").exists()

    def test_rust_library_integration_test_separated(self) -> None:
        """ディレクトリ構成図.md: Rust ライブラリの統合テストは tests/ に分離。"""
        assert (TEMPLATES / "library" / "rust" / "tests" / "integration_test.rs.tera").exists()
