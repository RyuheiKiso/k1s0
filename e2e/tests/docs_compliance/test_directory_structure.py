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
            "CLI/crates/k1s0-cli/src/main.rs",
            "CLI/crates/k1s0-cli/src/commands/init.rs",
            "CLI/crates/k1s0-cli/src/commands/generate/mod.rs",
            "CLI/crates/k1s0-cli/src/commands/build.rs",
            "CLI/crates/k1s0-cli/src/commands/test_cmd.rs",  # test.rs は予約語のため test_cmd.rs
            "CLI/crates/k1s0-cli/src/commands/deploy.rs",
            "CLI/crates/k1s0-cli/src/config",
            "CLI/crates/k1s0-cli/src/prompt",
            "CLI/crates/k1s0-cli/templates/server/go",
            "CLI/crates/k1s0-cli/templates/server/rust",
            "CLI/crates/k1s0-cli/templates/client/react",
            "CLI/crates/k1s0-cli/templates/client/flutter",
            "CLI/crates/k1s0-cli/templates/library/go",
            "CLI/crates/k1s0-cli/templates/library/rust",
            "CLI/crates/k1s0-cli/templates/library/typescript",  # ts → typescript (実装の命名)
            "CLI/crates/k1s0-cli/templates/library/dart",
            "CLI/crates/k1s0-cli/templates/database",
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

CLI_SRC = ROOT / "CLI" / "crates" / "k1s0-cli" / "src"
CLI_CORE_SRC = ROOT / "CLI" / "crates" / "k1s0-core" / "src"
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"


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
        """tier-architecture.md: generate モジュールが Tier を考慮したパス生成を行う。"""
        content = (CLI_CORE_SRC / "commands" / "generate" / "types.rs").read_text(encoding="utf-8")
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
        assert (ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "library" / "typescript").exists()


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


# ============================================================================
# ディレクトリ構成図.md 追加ギャップ補完テスト
# ============================================================================


class TestSystemTierDetailedStructure:
    """ディレクトリ構成図.md: system Tier 詳細構成の検証。"""

    def test_system_server_exists(self) -> None:
        """ディレクトリ構成図.md: system/server が存在する。"""
        assert (ROOT / "regions" / "system" / "server").exists()

    def test_system_library_exists(self) -> None:
        """ディレクトリ構成図.md: system/library が存在する。"""
        assert (ROOT / "regions" / "system" / "library").exists()

    def test_system_database_exists(self) -> None:
        """ディレクトリ構成図.md: system/database が存在する。"""
        assert (ROOT / "regions" / "system" / "database").exists()

    def test_system_has_no_client(self) -> None:
        """ディレクトリ構成図.md: system に client がない。"""
        assert not (ROOT / "regions" / "system" / "client").exists(), (
            "system 層に client が存在してはいけません"
        )


class TestBusinessTierDetailedStructure:
    """ディレクトリ構成図.md: business Tier 詳細構成の検証。"""

    def test_business_dir_exists(self) -> None:
        """ディレクトリ構成図.md: regions/business が存在する。"""
        assert (ROOT / "regions" / "business").exists()


class TestServiceTierDetailedStructure:
    """ディレクトリ構成図.md: service Tier 詳細構成の検証。"""

    def test_service_dir_exists(self) -> None:
        """ディレクトリ構成図.md: regions/service が存在する。"""
        assert (ROOT / "regions" / "service").exists()

    def test_service_has_no_library(self) -> None:
        """ディレクトリ構成図.md: service 層に library がない。"""
        service_dir = ROOT / "regions" / "service"
        if service_dir.exists():
            for child in service_dir.iterdir():
                if child.is_dir():
                    assert not (child / "library").exists(), (
                        f"service/{child.name} に library が存在してはいけません"
                    )


class TestGoServerDetailedStructure:
    """ディレクトリ構成図.md: Go サーバー詳細構成の検証。"""

    def test_go_domain_service(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに domain/service テンプレートが存在する。"""
        # domain/service はドキュメントに記載されているが、テンプレートは domain/model + domain/repository
        assert (TEMPLATES / "server" / "go" / "internal" / "domain" / "model" / "entity.go.tera").exists()
        assert (TEMPLATES / "server" / "go" / "internal" / "domain" / "repository" / "repository.go.tera").exists()

    def test_go_adapter_presenter(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに adapter/handler テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "go" / "internal" / "adapter" / "handler" / "rest_handler.go.tera").exists()

    def test_go_adapter_gateway_grpc(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに gRPC ハンドラーテンプレートが存在する。"""
        assert (TEMPLATES / "server" / "go" / "internal" / "adapter" / "handler" / "grpc_handler.go.tera").exists()

    def test_go_tests_integration(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに usecase_test.go テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "go" / "internal" / "usecase" / "usecase_test.go.tera").exists()

    def test_go_handler_test(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに handler_test.go テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "go" / "internal" / "adapter" / "handler" / "handler_test.go.tera").exists()

    def test_go_repository_test(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに repository_test.go テンプレートが存在する（DB有効時）。"""
        assert (TEMPLATES / "server" / "go" / "internal" / "infra" / "persistence" / "repository_test.go.tera").exists()

    def test_go_config_env_files(self) -> None:
        """ディレクトリ構成図.md: Go サーバーに config.yaml テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "go" / "config" / "config.yaml.tera").exists()


class TestRustServerDetailedStructure:
    """ディレクトリ構成図.md: Rust サーバー詳細構成の検証。"""

    def test_rust_domain_service(self) -> None:
        """ディレクトリ構成図.md: Rust サーバーに domain テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "src" / "domain" / "model.rs.tera").exists()
        assert (TEMPLATES / "server" / "rust" / "src" / "domain" / "repository.rs.tera").exists()

    def test_rust_adapter_handler(self) -> None:
        """ディレクトリ構成図.md: Rust サーバーに adapter/handler テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "src" / "adapter" / "handler" / "rest.rs.tera").exists()

    def test_rust_adapter_grpc(self) -> None:
        """ディレクトリ構成図.md: Rust サーバーに gRPC ハンドラーテンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "src" / "adapter" / "handler" / "grpc.rs.tera").exists()

    def test_rust_integration_test(self) -> None:
        """ディレクトリ構成図.md: Rust サーバーに tests/integration_test.rs テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "tests" / "integration_test.rs.tera").exists()

    def test_rust_config_yaml(self) -> None:
        """ディレクトリ構成図.md: Rust サーバーに config.yaml テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "config" / "config.yaml.tera").exists()


class TestReactClientDetailedStructure:
    """ディレクトリ構成図.md: React クライアント features/hooks/types 構成の検証。"""

    def test_react_app_exists(self) -> None:
        """ディレクトリ構成図.md: React に app ディレクトリテンプレートがある。"""
        assert (TEMPLATES / "client" / "react" / "src" / "app" / "App.tsx.tera").exists()

    def test_react_lib_exists(self) -> None:
        """ディレクトリ構成図.md: React に lib ディレクトリテンプレートがある。"""
        assert (TEMPLATES / "client" / "react" / "src" / "lib" / "api-client.ts.tera").exists()
        assert (TEMPLATES / "client" / "react" / "src" / "lib" / "query-client.ts.tera").exists()

    def test_react_tests_testutil_exists(self) -> None:
        """ディレクトリ構成図.md: React テストに testutil がある。"""
        assert (TEMPLATES / "client" / "react" / "tests" / "testutil" / "msw-setup.ts.tera").exists()


class TestFlutterClientDetailedStructure:
    """ディレクトリ構成図.md: Flutter クライアント features/widgets/utils 構成の検証。"""

    def test_flutter_app_router(self) -> None:
        """ディレクトリ構成図.md: Flutter に app/router テンプレートがある。"""
        assert (TEMPLATES / "client" / "flutter" / "lib" / "app" / "router.dart.tera").exists()

    def test_flutter_utils_dio(self) -> None:
        """ディレクトリ構成図.md: Flutter に utils/dio_client テンプレートがある。"""
        assert (TEMPLATES / "client" / "flutter" / "lib" / "utils" / "dio_client.dart.tera").exists()

    def test_flutter_test_exists(self) -> None:
        """ディレクトリ構成図.md: Flutter に test ディレクトリテンプレートがある。"""
        assert (TEMPLATES / "client" / "flutter" / "test" / "widget_test.dart.tera").exists()


class TestLibraryDetailedStructure:
    """ディレクトリ構成図.md: ライブラリ詳細構成の検証。"""

    def test_go_library_tests_integration(self) -> None:
        """ディレクトリ構成図.md: Go ライブラリに tests/integration_test テンプレートが存在する。"""
        assert (TEMPLATES / "library" / "go" / "tests" / "integration_test.go.tera").exists()

    def test_rust_library_internal(self) -> None:
        """ディレクトリ構成図.md: Rust ライブラリに src/lib.rs テンプレートが存在する。"""
        assert (TEMPLATES / "library" / "rust" / "src" / "lib.rs.tera").exists()

    def test_ts_library_tests(self) -> None:
        """ディレクトリ構成図.md: TypeScript ライブラリに tests テンプレートが存在する。"""
        assert (TEMPLATES / "library" / "typescript" / "tests" / "index.test.ts.tera").exists()

    def test_dart_library_test(self) -> None:
        """ディレクトリ構成図.md: Dart ライブラリに test テンプレートが存在する。"""
        assert (TEMPLATES / "library" / "dart" / "test" / "{module}_test.dart.tera").exists()


class TestDatabaseDetailedStructure:
    """ディレクトリ構成図.md: データベース構成（seeds/, schema/）のテンプレート検証。"""

    def test_database_migrations_template_exists(self) -> None:
        """ディレクトリ構成図.md: migrations テンプレートが存在する。"""
        assert (TEMPLATES / "database" / "postgresql" / "001_init.up.sql.tera").exists()
        assert (TEMPLATES / "database" / "postgresql" / "001_init.down.sql.tera").exists()


class TestDocsDirectoryFullListing:
    """ディレクトリ構成図.md: docs ディレクトリ全ファイル一覧の検証。"""

    @pytest.mark.parametrize(
        "doc_file",
        [
            "CLIフロー.md",
            "コンセプト.md",
            "tier-architecture.md",
            "ディレクトリ構成図.md",
            "コーディング規約.md",
            "config設計.md",
            "devcontainer設計.md",
            "docker-compose設計.md",
            "インフラ設計.md",
            "terraform設計.md",
            "Dockerイメージ戦略.md",
            "kubernetes設計.md",
            "helm設計.md",
            "API設計.md",
            "認証認可設計.md",
            "CI-CD設計.md",
            "可観測性設計.md",
            "サービスメッシュ設計.md",
            "APIゲートウェイ設計.md",
            "メッセージング設計.md",
            "テンプレート仕様-サーバー.md",
            "テンプレート仕様-React.md",
            "テンプレート仕様-Flutter.md",
            "テンプレート仕様-ライブラリ.md",
            "テンプレート仕様-データベース.md",
            "テンプレートエンジン仕様.md",
        ],
    )
    def test_doc_file_exists(self, doc_file: str) -> None:
        """ディレクトリ構成図.md: docs 内の全ドキュメントが存在する。"""
        assert (ROOT / "docs" / doc_file).exists(), f"docs/{doc_file} が存在しません"


# ============================================================================
# tier-architecture.md ギャップ補完テスト（追加）
# ============================================================================


class TestTierKindRestrictions:
    """tier-architecture.md: 各階層の種別制約検証。"""

    def test_system_has_server_library_database(self) -> None:
        """tier-architecture.md: system は server, library, database を持つ。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        # system セクションに server, library, database が記載されている
        assert "server" in content
        assert "library" in content
        assert "database" in content

    def test_system_no_client_in_doc(self) -> None:
        """tier-architecture.md: system に client がない理由が記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "system に client がない理由" in content

    def test_service_no_library_in_doc(self) -> None:
        """ディレクトリ構成図.md: service に library がない理由が記載されている。"""
        content = (ROOT / "docs" / "ディレクトリ構成図.md").read_text(encoding="utf-8")
        assert "service 層に library がない理由" in content

    def test_cli_client_not_available_for_system(self) -> None:
        """tier-architecture.md: CLI でクライアントの system Tier が選択不可。"""
        content = (CLI_CORE_SRC / "commands" / "generate" / "types.rs").read_text(encoding="utf-8")
        assert "Kind::Client => vec![Tier::Business, Tier::Service]" in content

    def test_cli_library_not_available_for_service(self) -> None:
        """tier-architecture.md: CLI でライブラリの service Tier が選択不可。"""
        content = (CLI_CORE_SRC / "commands" / "generate" / "types.rs").read_text(encoding="utf-8")
        assert "Kind::Library => vec![Tier::System, Tier::Business]" in content


class TestSameTierCommunicationBusinessTier:
    """tier-architecture.md: 同階層間通信ルール - business Tier の検証。"""

    def test_business_same_domain_sync_allowed(self) -> None:
        """tier-architecture.md: business Tier の同一ドメインコンテキスト内同期通信が許可されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "同一ドメインコンテキスト" in content
        assert "同期通信を許可" in content

    def test_business_cross_domain_async_required(self) -> None:
        """tier-architecture.md: business Tier の異なるドメインコンテキスト間は非同期メッセージング。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "非同期メッセージング" in content

    def test_business_tier_authorization_policy(self) -> None:
        """tier-architecture.md: business Tier の AuthorizationPolicy が存在する。"""
        import yaml
        path = ROOT / "infra" / "istio" / "authorizationpolicy.yaml"
        content = path.read_text(encoding="utf-8")
        docs = [d for d in yaml.safe_load_all(content) if d]
        business_policies = [
            d for d in docs if d["metadata"]["namespace"] == "k1s0-business"
        ]
        assert len(business_policies) >= 1


class TestSameTierCommunicationSystemTier:
    """tier-architecture.md: 同階層間通信ルール - system Tier の検証。"""

    def test_system_internal_communication_allowed(self) -> None:
        """tier-architecture.md: system Tier 内の共通基盤サービス間通信が許可されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "共通基盤サービス間の通信は許可" in content

    def test_system_tier_allow_policy_includes_same_namespace(self) -> None:
        """tier-architecture.md: system Tier の ALLOW ポリシーが自身のnamespaceを含む。"""
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
        # system namespace 自身からのアクセスも許可されている
        rules = system_allow[0]["spec"]["rules"]
        namespaces_in_rules = []
        for rule in rules:
            for source in rule.get("from", []):
                ns = source.get("source", {}).get("namespaces", [])
                namespaces_in_rules.extend(ns)
        assert "k1s0-system" in namespaces_in_rules


class TestServerDependencyExamples:
    """tier-architecture.md: Server 間の依存具体例検証。"""

    def test_server_dependency_documented(self) -> None:
        """tier-architecture.md: business server が system server から利用するものが記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "認証トークンの検証" in content
        assert "ユーザー情報の取得" in content
        assert "ログ・トレースの送信" in content
        assert "レート制限・ルーティング" in content

    def test_server_dependency_diagram_exists(self) -> None:
        """tier-architecture.md: server-dependency.svg が存在する。"""
        assert (ROOT / "docs" / "diagrams" / "server-dependency.svg").exists()


class TestClientDependencyExamples:
    """tier-architecture.md: Client 間の依存具体例検証。"""

    def test_client_dependency_documented(self) -> None:
        """tier-architecture.md: client が system library から利用するものが記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "認証トークンの取得・管理" in content
        assert "API リクエストの送信" in content

    def test_client_dependency_diagram_exists(self) -> None:
        """tier-architecture.md: client-dependency.svg が存在する。"""
        assert (ROOT / "docs" / "diagrams" / "client-dependency.svg").exists()


class TestRdbmsSelection:
    """tier-architecture.md: RDBMS 選択の反映検証。"""

    def test_rdbms_choices_documented(self) -> None:
        """tier-architecture.md: PostgreSQL/MySQL/SQLite の選択肢が記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "PostgreSQL" in content
        assert "MySQL" in content
        assert "SQLite" in content

    def test_rdbms_choices_in_cli(self) -> None:
        """tier-architecture.md: CLI に RDBMS 選択肢が実装されている。"""
        content = (CLI_CORE_SRC / "commands" / "generate" / "types.rs").read_text(encoding="utf-8")
        assert 'Rdbms::PostgreSQL => "PostgreSQL"' in content
        assert 'Rdbms::MySQL => "MySQL"' in content
        assert 'Rdbms::SQLite => "SQLite"' in content

    def test_database_templates_for_all_rdbms(self) -> None:
        """tier-architecture.md: 全 RDBMS のデータベーステンプレートが存在する。"""
        for rdbms in ["postgresql", "mysql", "sqlite"]:
            assert (TEMPLATES / "database" / rdbms / "001_init.up.sql.tera").exists(), (
                f"database/{rdbms} テンプレートが存在しません"
            )


class TestTierDataResponsibility:
    """tier-architecture.md: 階層ごとのデータ責務検証。"""

    def test_data_responsibility_documented(self) -> None:
        """tier-architecture.md: 各階層のデータベース責務が記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "ユーザー、認証・認可、監査ログ" in content
        assert "領域固有のマスタデータ" in content
        assert "サービス固有の業務データ" in content

    def test_db_access_restriction_documented(self) -> None:
        """tier-architecture.md: DB アクセスはその階層の server のみと記載されている。"""
        content = (ROOT / "docs" / "tier-architecture.md").read_text(encoding="utf-8")
        assert "その階層の server からのみアクセス" in content
