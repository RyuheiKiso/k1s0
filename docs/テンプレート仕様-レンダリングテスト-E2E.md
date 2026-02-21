# テンプレート仕様 — レンダリングテスト — E2E テスト

本ドキュメントは、[テンプレート仕様-レンダリングテスト](テンプレート仕様-レンダリングテスト.md) から分割された E2E テスト仕様である。

---

## E2E テスト仕様

テスト配置: `e2e/tests/docs_compliance/` 配下。

### テスト構造パターン

E2E テストは pytest のクラスベーススタイルを採用する。

```python
ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"

class TestGoModContent:
    """テンプレート仕様-サーバー.md: go.mod.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "go.mod.tera").read_text(encoding="utf-8")

    def test_module_variable(self) -> None:
        assert "{{ go_module }}" in self.content
```

#### パターン要素

| 要素 | 説明 |
|------|------|
| `ROOT` | プロジェクトルート（`Path(__file__).resolve().parents[3]`） |
| `TEMPLATES` | テンプレートディレクトリ（`ROOT / "CLI" / "templates"`） |
| `setup_method` | テスト対象ファイルの読み込み |
| `pytest.mark.parametrize` | 複数値のパラメトリックテスト |
| クラス名 | `Test{対象ファイル}Content` |
| テストメソッド名 | `test_{検証項目}` |

### ファイル存在テスト

テンプレートエンジン仕様.md で定義されたディレクトリ構成に対応するテンプレートファイルの存在を検証する。

テストファイル: `test_template_files.py`

テスト対象クラス:

| クラス | 対象ディレクトリ |
|--------|----------------|
| `TestServerGoTemplates` | `server/go/` |
| `TestServerRustTemplates` | `server/rust/` |
| `TestClientReactTemplates` | `client/react/` |
| `TestClientFlutterTemplates` | `client/flutter/` |
| `TestLibraryGoTemplates` | `library/go/` |
| `TestLibraryRustTemplates` | `library/rust/` |
| `TestLibraryTypescriptTemplates` | `library/typescript/` |
| `TestLibraryDartTemplates` | `library/dart/` |
| `TestDatabasePostgresqlTemplates` | `database/postgresql/` |
| `TestDatabaseMysqlTemplates` | `database/mysql/` |
| `TestDatabaseSqliteTemplates` | `database/sqlite/` |

#### 新規カテゴリのテストメソッド例

| テストメソッド | 対象 |
|--------------|------|
| `test_client_react_file_list` | React クライアントのファイル一覧 |
| `test_client_react_package_json` | React の package.json 検証 |
| `test_client_flutter_file_list` | Flutter クライアントのファイル一覧 |
| `test_client_flutter_pubspec` | Flutter の pubspec.yaml 検証 |
| `test_library_go_file_list` | Go ライブラリのファイル一覧 |
| `test_library_go_module_path` | Go ライブラリのモジュールパス検証 |
| `test_library_rust_file_list` | Rust ライブラリのファイル一覧 |
| `test_library_typescript_file_list` | TypeScript ライブラリのファイル一覧 |
| `test_library_dart_file_list` | Dart ライブラリのファイル一覧 |
| `test_database_postgresql_file_list` | PostgreSQL のファイル一覧 |
| `test_database_mysql_file_list` | MySQL のファイル一覧 |
| `test_database_sqlite_file_list` | SQLite のファイル一覧 |

### 内容準拠テスト

テンプレートファイルの内容が仕様書のコードブロックと一致することを検証する。

テストファイル:
- `test_template_content_server.py` — サーバーテンプレートの内容検証
- `test_template_content_client.py` — クライアントテンプレートの内容検証
- `test_template_content_library.py` — ライブラリテンプレートの内容検証
- `test_template_content_database.py` — データベーステンプレートの内容検証
- `test_template_content_helm.py` — Helm テンプレートの内容検証
- `test_template_content_cicd.py` — CI/CD テンプレートの内容検証
- `test_template_content_terraform.py` — Terraform テンプレートの内容検証
- `test_template_content_docker_compose.py` — Docker Compose テンプレートの内容検証
- `test_template_content_devcontainer.py` — devcontainer テンプレートの内容検証
- `test_template_content_service_mesh.py` — Service Mesh テンプレートの内容検証

#### 検証項目

1. **テンプレート変数の存在**: `{{ service_name }}` 等の Tera 変数が含まれること
2. **条件分岐の存在**: `{% if has_database %}` 等の制御構文が含まれること
3. **依存ライブラリの宣言**: 仕様書に定義された依存が含まれること
4. **構造体・関数の定義**: 仕様書に定義されたシグネチャが含まれること

#### パラメトリックテストの活用

```python
@pytest.mark.parametrize(
    "condition,dep",
    [
        ('api_styles is containing("rest")', "oapi-codegen"),
        ('api_styles is containing("grpc")', "google.golang.org/grpc"),
        ('api_styles is containing("graphql")', "gqlgen"),
        ("has_database", "github.com/jmoiron/sqlx"),
        ('database_type == "postgresql"', "github.com/lib/pq"),
        ('database_type == "mysql"', "go-sql-driver/mysql"),
        ('database_type == "sqlite"', "go-sqlite3"),
        ("has_kafka", "kafka-go"),
        ("has_redis", "go-redis"),
    ],
)
def test_conditional_dependency(self, condition: str, dep: str) -> None:
    assert condition in self.content
    assert dep in self.content
```

### レンダリング結果テスト（将来拡張）

E2E テストで CLI を実際に実行し、レンダリング結果の構造・内容を検証する。現在は Rust 統合テストがこの役割を担っている。

将来的には以下を追加する:
- CLI コマンドの E2E 実行（`subprocess` で CLI を呼び出し）
- 生成されたプロジェクトのビルド検証（`go build` / `cargo check`）
- 生成されたプロジェクトのテスト実行（`go test` / `cargo test`）

---

## BFF テンプレート E2E テスト

### E2E テスト — BFF テンプレート存在・内容検証

テストファイル: `e2e/tests/docs_compliance/test_template_content_bff.py`

```python
ROOT = Path(__file__).resolve().parents[3]
BFF_GO = ROOT / "CLI" / "templates" / "bff" / "go"
BFF_RUST = ROOT / "CLI" / "templates" / "bff" / "rust"


class TestBffGoTemplates:
    """BFF Go テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "cmd/main.go.tera",
            "go.mod.tera",
            "internal/handler/graphql_resolver.go.tera",
            "api/graphql/schema.graphql.tera",
            "api/graphql/gqlgen.yml.tera",
            "config/config.yaml.tera",
            "Dockerfile.tera",
            "README.md.tera",
        ],
    )
    def test_bff_go_template_exists(self, template: str) -> None:
        path = BFF_GO / template
        assert path.exists(), f"bff/go/{template} が存在しません"


class TestBffRustTemplates:
    """BFF Rust テンプレートの検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "src/main.rs.tera",
            "src/handler/graphql.rs.tera",
            "Cargo.toml.tera",
            "config/config.yaml.tera",
            "Dockerfile.tera",
            "README.md.tera",
        ],
    )
    def test_bff_rust_template_exists(self, template: str) -> None:
        path = BFF_RUST / template
        assert path.exists(), f"bff/rust/{template} が存在しません"


class TestBffGoConfigContent:
    """BFF Go config.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_GO / "config" / "config.yaml.tera").read_text(encoding="utf-8")

    def test_upstream_section(self) -> None:
        assert "upstream:" in self.content

    def test_no_database(self) -> None:
        assert "database:" not in self.content

    def test_no_kafka(self) -> None:
        assert "kafka:" not in self.content
```

---

## 新規カテゴリ E2E テスト仕様

各カテゴリの E2E テストクラスを以下に定義する。

### test_template_content_helm.py

```python
class TestHelmValuesContent:
    """Helm values.yaml.tera の内容検証。"""
    def test_service_name_variable(self): ...
    def test_grpc_port_condition(self): ...
    def test_database_vault_condition(self): ...

class TestHelmDeploymentContent:
    """Helm deployment.yaml.tera の内容検証。"""
    def test_library_chart_include(self): ...
    def test_healthcheck_paths(self): ...
```

### test_template_content_terraform.py

```python
class TestTerraformMainContent:
    """Terraform main.tf.tera の内容検証。"""
    def test_module_references(self): ...
    def test_environment_variable(self): ...

class TestTerraformBackendContent:
    """Terraform backend.tf.tera の内容検証。"""
    def test_consul_backend(self): ...
```

### test_template_content_docker_compose.py

```python
class TestDockerComposeContent:
    """docker-compose.yaml.tera の内容検証。"""
    def test_database_condition(self): ...
    def test_kafka_condition(self): ...
    def test_healthcheck_definitions(self): ...
```

### test_template_content_devcontainer.py

```python
class TestDevcontainerJsonContent:
    """devcontainer.json.tera の内容検証。"""
    def test_language_features(self): ...
    def test_forward_ports(self): ...
```

### test_template_content_service_mesh.py

```python
class TestVirtualServiceContent:
    """virtual-service.yaml.tera の内容検証。"""
    def test_tier_timeout_defaults(self): ...
    def test_retry_configuration(self): ...

class TestDestinationRuleContent:
    """destination-rule.yaml.tera の内容検証。"""
    def test_circuit_breaker_defaults(self): ...
    def test_mtls_mode(self): ...
```

---

## Library Chart E2E テスト

### E2E テスト — Library Chart ヘルパーテンプレート存在検証

テスト配置: `e2e/tests/docs_compliance/test_helm_library_chart.py`

`infra/helm/charts/k1s0-common/templates/` 配下に、Library Chart のヘルパーテンプレートが全て存在することを検証する。

#### 検証対象ファイル

| ヘルパーテンプレート | 役割 |
|--------------------|----|
| `_deployment.tpl` | Deployment マニフェスト生成ヘルパー |
| `_service.tpl` | Service マニフェスト生成ヘルパー |
| `_hpa.tpl` | HorizontalPodAutoscaler マニフェスト生成ヘルパー |
| `_pdb.tpl` | PodDisruptionBudget マニフェスト生成ヘルパー |
| `_configmap.tpl` | ConfigMap マニフェスト生成ヘルパー |
| `_helpers.tpl` | 共通ラベル・セレクタ等のユーティリティヘルパー |
| `_ingress.tpl` | Ingress マニフェスト生成ヘルパー |

#### テストパターン

```python
ROOT = Path(__file__).resolve().parents[3]
LIBRARY_CHART_TEMPLATES = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates"


class TestLibraryChartHelperTemplates:
    """Library Chart のヘルパーテンプレートが全て存在することを検証する。"""

    @pytest.mark.parametrize(
        "template",
        [
            "_deployment.tpl",
            "_service.tpl",
            "_hpa.tpl",
            "_pdb.tpl",
            "_configmap.tpl",
            "_helpers.tpl",
            "_ingress.tpl",
        ],
    )
    def test_helper_template_exists(self, template: str) -> None:
        path = LIBRARY_CHART_TEMPLATES / template
        assert path.exists(), (
            f"k1s0-common/templates/{template} が存在しません"
        )
```

#### Chart.yaml 依存関係検証

生成される Helm Chart の `Chart.yaml` が Library Chart への依存を正しく宣言していることも検証する。

```python
class TestHelmChartDependency:
    """生成される Chart.yaml が k1s0-common への依存を宣言していることを検証する。"""

    def setup_method(self) -> None:
        self.chart_template = (
            ROOT / "CLI" / "templates" / "helm" / "Chart.yaml.tera"
        )

    def test_library_chart_dependency_declared(self) -> None:
        content = self.chart_template.read_text(encoding="utf-8")
        assert "k1s0-common" in content, (
            "Chart.yaml.tera に k1s0-common への依存宣言がありません"
        )

    def test_library_chart_repository(self) -> None:
        content = self.chart_template.read_text(encoding="utf-8")
        assert "file://" in content or "repository:" in content, (
            "Chart.yaml.tera に Library Chart のリポジトリ参照がありません"
        )
```

---

## 関連ドキュメント

- [テンプレート仕様-レンダリングテスト](テンプレート仕様-レンダリングテスト.md) --- 概要・テスト対象マトリクス・CI統合・テスト命名規則
- [テンプレート仕様-レンダリングテスト-Rust](テンプレート仕様-レンダリングテスト-Rust.md) --- Rust統合テスト仕様
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) --- テンプレート変数・フィルタ・構文の定義
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) --- サーバーテンプレートの詳細仕様
- [テンプレート仕様-React](テンプレート仕様-React.md) --- React テンプレートの詳細仕様
- [テンプレート仕様-Flutter](テンプレート仕様-Flutter.md) --- Flutter テンプレートの詳細仕様
- [テンプレート仕様-ライブラリ](テンプレート仕様-ライブラリ.md) --- ライブラリテンプレートの詳細仕様
- [テンプレート仕様-データベース](テンプレート仕様-データベース.md) --- データベーステンプレートの詳細仕様
- [テンプレート仕様-Terraform](テンプレート仕様-Terraform.md) --- Terraform テンプレートの詳細仕様
- [テンプレート仕様-DockerCompose](テンプレート仕様-DockerCompose.md) --- Docker Compose テンプレートの詳細仕様
- [テンプレート仕様-devcontainer](テンプレート仕様-devcontainer.md) --- devcontainer テンプレートの詳細仕様
- [テンプレート仕様-ServiceMesh](テンプレート仕様-ServiceMesh.md) --- Service Mesh テンプレートの詳細仕様
- [テンプレート仕様-Grafana](テンプレート仕様-Grafana.md) --- Grafana ダッシュボードテンプレートの詳細仕様
- [テンプレート仕様-OpenTelemetry](テンプレート仕様-OpenTelemetry.md) --- OpenTelemetry Collector テンプレートの詳細仕様
- [テンプレート仕様-Loki](テンプレート仕様-Loki.md) --- Loki ログ収集テンプレートの詳細仕様
- [テンプレート仕様-Alertmanager](テンプレート仕様-Alertmanager.md) --- Alertmanager 通知テンプレートの詳細仕様
- [テンプレート仕様-Kafka](テンプレート仕様-Kafka.md) --- Kafka トピックテンプレートの詳細仕様
- [テンプレート仕様-Vault](テンプレート仕様-Vault.md) --- Vault シークレット管理テンプレートの詳細仕様
- [テンプレート仕様-Flagger](テンプレート仕様-Flagger.md) --- Flagger カナリアリリーステンプレートの詳細仕様
- [テンプレート仕様-Consul](テンプレート仕様-Consul.md) --- Consul State Backend テンプレートの詳細仕様
- [テンプレート仕様-Storage](テンプレート仕様-Storage.md) --- Ceph ストレージテンプレートの詳細仕様
