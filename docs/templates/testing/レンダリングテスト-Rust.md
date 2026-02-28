# テンプレート仕様 — レンダリングテスト — Rust 統合テスト

k1s0 CLI ひな形生成の Rust 統合テスト仕様。[レンダリングテスト](レンダリングテスト.md) から分割。

---

## Rust 統合テスト仕様

テスト配置: `CLI/tests/` 配下。

### ヘルパー関数

`server_template_rendering.rs` で確立されたパターンに準拠する。

#### `template_dir()` — テンプレートディレクトリ取得

```rust
fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}
```

`CARGO_MANIFEST_DIR` を用いて `CLI/templates/` への絶対パスを取得する。テスト実行時のカレントディレクトリに依存しない。

#### `render_server()` — サーバーテンプレートのレンダリング

```rust
fn render_server(
    lang: &str,
    api_style: &str,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>)
```

- `TemplateContextBuilder::new(service_name, tier, lang, kind)` でコンテキストを構築
- `TempDir` に出力し、生成されたファイルの相対パス一覧を返す
- パス区切りを `/` に正規化（Windows 互換）

テスト用デフォルト値:
- `service_name`: `"order-api"`
- `tier`: `"service"`

#### `read_output()` — 生成ファイルの読み取り

```rust
fn read_output(tmp: &TempDir, path: &str) -> String {
    fs::read_to_string(tmp.path().join("output").join(path)).unwrap()
}
```

#### 新規 kind 用ヘルパー

client / library / database 用のヘルパーも同様のパターンで作成する。

```rust
// client 用
fn render_client(lang: &str) -> (TempDir, Vec<String>)

// library 用
fn render_library(lang: &str) -> (TempDir, Vec<String>)

// database 用
fn render_database(db_type: &str) -> (TempDir, Vec<String>)
```

#### 追加 kind 用ヘルパー（terraform / docker-compose / devcontainer / service-mesh）

```rust
// Terraform テンプレートのレンダリング
fn render_terraform(environment: &str) -> (TempDir, Vec<String>)

// Docker Compose テンプレートのレンダリング
fn render_docker_compose(
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>)

// devcontainer テンプレートのレンダリング
fn render_devcontainer(
    lang: &str,
    has_database: bool,
    has_kafka: bool,
) -> (TempDir, Vec<String>)

// Service Mesh テンプレートのレンダリング
fn render_service_mesh(
    tier: &str,
    api_styles: Vec<&str>,
) -> (TempDir, Vec<String>)
```

### テストパターン

#### 1. ファイルリスト検証

生成されるファイルの一覧が仕様書と一致することを検証する。

```rust
#[test]
fn test_go_server_rest_full_stack_file_list() {
    let (tmp, names) = render_server("go", "rest", true, "postgresql", true, false);

    // 必須ファイルの存在確認
    assert!(names.iter().any(|n| n == "go.mod"), "go.mod missing");
    assert!(names.iter().any(|n| n == "cmd/main.go"), "cmd/main.go missing");
    // ...

    // 排他的ファイルの不在確認（REST の場合 gRPC/GraphQL ファイルは除外）
    assert!(!names.iter().any(|n| n.contains("grpc_handler")));
    assert!(!names.iter().any(|n| n.contains("graphql_resolver")));
}
```

検証項目:
- 必須ファイルの存在（`assert!` + `names.iter().any()`）
- 排他的ファイルの不在（`assert!` + `!names.iter().any()`）
- テストファイルの存在確認

#### 2. 条件付きファイル検証

API style / DB / Kafka / Redis の組み合わせによるファイル生成分岐を検証する。

| 条件 | 生成されるファイル | 生成されないファイル |
|------|-------------------|-------------------|
| `api_styles is containing("rest")` | `rest_handler.go` / `openapi.yaml` | `grpc_handler.go` / `service.proto` / `graphql_resolver.go` / `schema.graphql` |
| `api_styles is containing("grpc")` | `grpc_handler.go` / `service.proto` / `buf.yaml` | `rest_handler.go` / `openapi.yaml` / `graphql_resolver.go` |
| `api_styles is containing("graphql")` | `graphql_resolver.go` / `schema.graphql` / `gqlgen.yml` | `rest_handler.go` / `grpc_handler.go` / `service.proto` |
| `has_database == false` | --- | `persistence/` 配下全ファイル |
| `has_kafka == false` | --- | `messaging/` 配下全ファイル |

#### 3. テンプレート変数置換検証

`service_name` のケース変換が正しく適用されることを検証する。

```rust
#[test]
fn test_tera_variable_substitution_consistency() {
    // service_name = "user-auth" で検証
    let ctx = TemplateContextBuilder::new("user-auth", "service", "go", "server")
        .api_style("rest")
        .with_database("postgresql")
        .build();

    // PascalCase
    assert!(entity.contains("type UserAuthEntity struct {"));
    // camelCase
    assert!(persistence.contains("type userAuthRepository struct {"));
    // kebab-case（ルーティング）
    assert!(handler.contains("v1.GET(\"/user-auth\""));
    // snake_case（DB名）
    assert!(config_yaml.contains("name: \"user_auth_db\""));
}
```

検証対象のケース変換:

| 変数名 | ケース | 使用箇所 |
|--------|--------|---------|
| `service_name` | kebab-case | ルーティングパス、モジュールパス、config |
| `service_name_snake` | snake_case | DB名、パッケージ名 |
| `service_name_pascal` | PascalCase | 構造体名、インターフェース名 |
| `service_name_camel` | camelCase | プライベート構造体名 |

#### 4. Tera 構文残留チェック

レンダリング結果に Tera の制御構文が残っていないことを検証する。

検証対象の構文パターン:
- `{{` — 変数展開の未解決
- `{%` — 制御構文の未解決
- `{#` — コメントの未解決

このチェックはすべてのレンダリングテストで暗黙的に行われる（正しく置換されていればこれらの構文は残らない）。専用テストとして明示的に追加する場合:

```rust
#[test]
fn test_no_tera_syntax_remaining() {
    let (tmp, names) = render_server("go", "rest", true, "postgresql", true, true);
    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera variable syntax remaining in {name}");
        assert!(!content.contains("{%"), "Tera control syntax remaining in {name}");
        assert!(!content.contains("{#"), "Tera comment syntax remaining in {name}");
    }
}
```

#### 5. 設定ファイル内容検証

`config/config.yaml` 等の設定ファイルが正しい内容で生成されることを検証する。

```rust
#[test]
fn test_go_server_rest_config_yaml() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", true, true);
    let content = read_output(&tmp, "config/config.yaml");

    assert!(content.contains("name: \"order-api\""));
    assert!(content.contains("port: 8080"));
    assert!(content.contains("database:"));
    assert!(content.contains("port: 5432"));          // postgresql
    assert!(content.contains("kafka:"));
    assert!(content.contains("redis:"));
    assert!(content.contains("observability:"));
}
```

DB 種別ごとの設定差分:

| DB 種別 | ポート | ドライバ固有設定 |
|---------|-------|----------------|
| postgresql | 5432 | `ssl_mode: "disable"` |
| mysql | 3306 | `parseTime=true&charset=utf8mb4` |
| sqlite | --- | ファイルパス指定 |

---

## スナップショットテスト

`insta` crate を使用したスナップショットテストにより、テンプレートレンダリング結果のファイル一覧を固定化・回帰検証する。

テスト配置: `CLI/tests/snapshot_tests.rs`

依存追加: `CLI/Cargo.toml`
```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

### 対象パターン（サーバー代表6パターン）

| # | パターン | kind | language | api_style | DB | Kafka | Redis |
|---|---------|------|----------|-----------|-----|-------|-------|
| 1 | Go REST フルスタック | server | go | rest | postgresql | true | true |
| 2 | Go gRPC 最小 | server | go | grpc | --- | false | false |
| 3 | Go GraphQL 最小 | server | go | graphql | --- | false | false |
| 4 | Rust REST + DB | server | rust | rest | postgresql | false | false |
| 5 | Rust gRPC 最小 | server | rust | grpc | --- | false | false |
| 6 | Rust GraphQL 最小 | server | rust | graphql | --- | false | false |

### 対象パターン（クライアント・ライブラリ・DB・Helm・CICD 追加12パターン）

| # | パターン | kind | language/framework | オプション |
|---|---------|------|-------------------|-----------|
| 7 | React クライアント | client | react | --- |
| 8 | Flutter クライアント | client | flutter | --- |
| 9 | Go ライブラリ | library | go | --- |
| 10 | Rust ライブラリ | library | rust | --- |
| 11 | TypeScript ライブラリ | library | typescript | --- |
| 12 | Dart ライブラリ | library | dart | --- |
| 13 | PostgreSQL DB | database | postgresql | --- |
| 14 | MySQL DB | database | mysql | --- |
| 15 | SQLite DB | database | sqlite | --- |
| 16 | Helm Chart (REST+DB) | helm | --- | rest, postgresql |
| 17 | CICD Go (REST+DB) | cicd | go | rest, postgresql, server |
| 18 | CICD Rust (gRPC) | cicd | rust | grpc, server |

### 対象パターン（複数APIスタイル・BFF 追加4パターン）

| # | パターン | kind | language | api_styles | DB | Kafka | Redis |
|---|---------|------|----------|-----------|-----|-------|-------|
| 19 | Go REST+gRPC + PostgreSQL | server | go | [rest, grpc] | postgresql | - | - |
| 20 | Rust REST+gRPC (最小) | server | rust | [rest, grpc] | - | - | - |
| 21 | Go BFF | bff | go | [graphql] | - | - | - |
| 22 | Rust BFF | bff | rust | [graphql] | - | - | - |

パターン 19-20 は、複数の API スタイルを同時に選択した場合のレンダリング結果を検証する。従来の `render_server()` は単一の `api_style` のみを受け付けるため、複数 API スタイル対応の `render_server_multi()` ヘルパーを使用する。

パターン 21-22 は、BFF テンプレートのレンダリング結果を検証する。BFF は service Tier + GraphQL 選択時に生成される GraphQL BFF ゲートウェイのひな形であり、`kind = "bff"` として `TemplateContextBuilder` に渡す。

### 対象パターン（Helm スナップショット追加3パターン）

| # | パターン | kind | language | api_styles | DB | Kafka | Redis |
|---|---------|------|----------|-----------|-----|-------|-------|
| 23 | Helm Chart (gRPC) | helm | --- | grpc | false | false | false |
| 24 | Helm Chart (GraphQL) | helm | --- | graphql | false | false | false |
| 25 | Helm Chart (REST+gRPC) | helm | --- | [rest, grpc] | true | false | false |

パターン 23-24 は、単一 API スタイルの Helm Chart レンダリングを検証する。パターン 25 は、複数 API スタイル対応の `render_helm_multi()` ヘルパーを使用する。

### 対象パターン（Terraform・Docker Compose・devcontainer・Service Mesh 追加8パターン）

| # | パターン | kind | オプション |
|---|---------|------|-----------|
| 26 | Terraform (dev) | terraform | dev |
| 27 | Terraform (prod) | terraform | prod |
| 28 | Docker Compose (full) | docker-compose | postgresql, kafka, redis |
| 29 | Docker Compose (minimal) | docker-compose | postgresql のみ |
| 30 | devcontainer (Go+DB) | devcontainer | go, postgresql |
| 31 | devcontainer (Rust) | devcontainer | rust |
| 32 | Service Mesh (service tier) | service-mesh | service, [rest] |
| 33 | Service Mesh (system tier) | service-mesh | system, [grpc] |

### `render_server_multi()` — 複数APIスタイル対応ヘルパー

パターン 19-20 で使用する。従来の `render_server()` が単一の `api_style: &str` を受け取るのに対し、`render_server_multi()` は `api_styles: Vec<&str>` を受け取り、複数の API スタイルを同時に設定する。

```rust
fn render_server_multi(
    lang: &str,
    api_styles: Vec<&str>,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>)
```

引数:
- `lang`: `"go"` | `"rust"` — サーバー言語
- `api_styles`: `Vec<&str>` — API スタイルのリスト（例: `vec!["rest", "grpc"]`）
- `has_database`: DB を含むか
- `database_type`: `"postgresql"` | `"mysql"` | `"sqlite"` | `""`
- `has_kafka`: Kafka を含むか
- `has_redis`: Redis を含むか

戻り値:
- `(TempDir, Vec<String>)` — 一時ディレクトリと生成されたファイルの相対パス一覧

実装イメージ:

```rust
fn render_server_multi(
    lang: &str,
    api_styles: Vec<&str>,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, "server")
        .api_styles(api_styles.clone());

    if has_database {
        builder = builder.with_database(database_type);
    }
    if has_kafka {
        builder = builder.with_kafka();
    }
    if has_redis {
        builder = builder.with_redis();
    }

    let ctx = builder.build();
    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    (tmp, names)
}
```

`render_server()` との主な違い:
- `api_style(&str)` の代わりに `api_styles(Vec<&str>)` を使用
- 複数 API スタイル分のファイルが同時に生成される（REST + gRPC の場合、`rest_handler` と `grpc_handler` の両方が存在）

### `render_helm_multi()` — 複数APIスタイル対応 Helm ヘルパー

パターン 25 で使用する。従来の `render_helm()` が単一の `api_style: &str` を受け取るのに対し、`render_helm_multi()` は `api_styles: Vec<&str>` を受け取り、複数の API スタイルを同時に設定する。

```rust
fn render_helm_multi(
    api_styles: Vec<&str>,
    has_database: bool,
    database_type: &str,
) -> (TempDir, Vec<String>)
```

引数:
- `api_styles`: `Vec<&str>` --- API スタイルのリスト（例: `vec!["rest", "grpc"]`）
- `has_database`: DB を含むか
- `database_type`: `"postgresql"` | `"mysql"` | `"sqlite"` | `""`

戻り値:
- `(TempDir, Vec<String>)` --- 一時ディレクトリと生成されたファイルの相対パス一覧

実装イメージ:

```rust
fn render_helm_multi(
    api_styles: Vec<&str>,
    has_database: bool,
    database_type: &str,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", "", "helm")
        .api_styles(api_styles.clone());

    if has_database {
        builder = builder.with_database(database_type);
    }

    let ctx = builder.build();
    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    (tmp, names)
}
```

### `render_bff()` — BFF テンプレートヘルパー

パターン 21-22 で使用する。BFF テンプレートを `kind = "bff"` としてレンダリングする。

```rust
fn render_bff(lang: &str) -> (TempDir, Vec<String>)
```

引数:
- `lang`: `"go"` | `"rust"` — BFF 言語

戻り値:
- `(TempDir, Vec<String>)` — 一時ディレクトリと生成されたファイルの相対パス一覧

実装イメージ:

```rust
fn render_bff(lang: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("order-api", "service", lang, "bff")
        .api_style("graphql")
        .build();
    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    (tmp, names)
}
```

BFF は常に GraphQL ベースのため、`api_style("graphql")` を固定で設定する。

### 追加パターン用ヘルパー関数（パターン 7-18）

パターン 7〜18 のレンダリングには、以下のヘルパー関数を使用する。`render_server()` と同様に `TemplateContextBuilder` でコンテキストを構築し、`TempDir` に出力する。

```rust
/// クライアントテンプレートのレンダリング
/// framework: "react" | "flutter"
fn render_client(framework: &str) -> (TempDir, Vec<String>)

/// ライブラリテンプレートのレンダリング
/// lang: "go" | "rust" | "typescript" | "dart"
fn render_library(lang: &str) -> (TempDir, Vec<String>)

/// データベーステンプレートのレンダリング
/// db_type: "postgresql" | "mysql" | "sqlite"
fn render_database(db_type: &str) -> (TempDir, Vec<String>)

/// Helm Chart テンプレートのレンダリング
/// api_style: "rest" | "grpc" | "graphql"
/// has_database: DB を含むか
/// database_type: "postgresql" | "mysql" | "sqlite"
fn render_helm(api_style: &str, has_database: bool, database_type: &str) -> (TempDir, Vec<String>)

/// CI/CD テンプレートのレンダリング
/// lang: "go" | "rust"
/// kind: "server" | "client" | "library"
/// api_style: "rest" | "grpc" | "graphql"
/// has_database: DB を含むか
/// database_type: "postgresql" | "mysql" | "sqlite"
fn render_cicd(lang: &str, kind: &str, api_style: &str, has_database: bool, database_type: &str) -> (TempDir, Vec<String>)
```

### テストパターン（スナップショット）

```rust
// サーバーパターン（既存）
#[test]
fn test_snapshot_go_rest_full_stack() {
    let (_, names) = render_server("go", "rest", true, "postgresql", true, true);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("go_rest_full_stack", sorted);
}

// クライアントパターン（#7, #8）
#[test]
fn test_snapshot_client_react() {
    let (_, names) = render_client("react");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("client_react", sorted);
}

#[test]
fn test_snapshot_client_flutter() {
    let (_, names) = render_client("flutter");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("client_flutter", sorted);
}

// ライブラリパターン（#9〜#12）
#[test]
fn test_snapshot_library_go() {
    let (_, names) = render_library("go");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_go", sorted);
}

#[test]
fn test_snapshot_library_rust() {
    let (_, names) = render_library("rust");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_rust", sorted);
}

#[test]
fn test_snapshot_library_typescript() {
    let (_, names) = render_library("typescript");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_typescript", sorted);
}

#[test]
fn test_snapshot_library_dart() {
    let (_, names) = render_library("dart");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_dart", sorted);
}

// データベースパターン（#13〜#15）
#[test]
fn test_snapshot_database_postgresql() {
    let (_, names) = render_database("postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("database_postgresql", sorted);
}

#[test]
fn test_snapshot_database_mysql() {
    let (_, names) = render_database("mysql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("database_mysql", sorted);
}

#[test]
fn test_snapshot_database_sqlite() {
    let (_, names) = render_database("sqlite");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("database_sqlite", sorted);
}

// Helm Chart パターン（#16）
#[test]
fn test_snapshot_helm_rest_postgresql() {
    let (_, names) = render_helm("rest", true, "postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_rest_postgresql", sorted);
}

// CI/CD パターン（#17, #18）
#[test]
fn test_snapshot_cicd_go_rest_postgresql() {
    let (_, names) = render_cicd("go", "server", "rest", true, "postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("cicd_go_rest_postgresql", sorted);
}

#[test]
fn test_snapshot_cicd_rust_grpc() {
    let (_, names) = render_cicd("rust", "server", "grpc", false, "");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("cicd_rust_grpc", sorted);
}
```

// 複数APIスタイルパターン（#19, #20）
#[test]
fn test_snapshot_go_rest_grpc_postgresql() {
    let (_, names) = render_server_multi("go", vec!["rest", "grpc"], true, "postgresql", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("go_rest_grpc_postgresql", sorted);
}

#[test]
fn test_snapshot_rust_rest_grpc_minimal() {
    let (_, names) = render_server_multi("rust", vec!["rest", "grpc"], false, "", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("rust_rest_grpc_minimal", sorted);
}

// BFF パターン（#21, #22）
#[test]
fn test_snapshot_bff_go() {
    let (_, names) = render_bff("go");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("bff_go", sorted);
}

#[test]
fn test_snapshot_bff_rust() {
    let (_, names) = render_bff("rust");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("bff_rust", sorted);
}

// Helm スナップショット追加パターン（#23, #24, #25）
#[test]
fn test_snapshot_helm_grpc() {
    let (_, names) = render_helm("grpc", false, "");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_grpc", sorted);
}

#[test]
fn test_snapshot_helm_graphql() {
    let (_, names) = render_helm("graphql", false, "");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_graphql", sorted);
}

#[test]
fn test_snapshot_helm_rest_grpc_postgresql() {
    let (_, names) = render_helm_multi(vec!["rest", "grpc"], true, "postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_rest_grpc_postgresql", sorted);
}

// Terraform パターン（#26, #27）
#[test]
fn test_snapshot_terraform_dev() {
    let (_, names) = render_terraform("dev");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("terraform_dev", sorted);
}

#[test]
fn test_snapshot_terraform_prod() {
    let (_, names) = render_terraform("prod");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("terraform_prod", sorted);
}

// Docker Compose パターン（#28, #29）
#[test]
fn test_snapshot_docker_compose_full() {
    let (_, names) = render_docker_compose(true, "postgresql", true, true);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("docker_compose_full", sorted);
}

#[test]
fn test_snapshot_docker_compose_minimal() {
    let (_, names) = render_docker_compose(true, "postgresql", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("docker_compose_minimal", sorted);
}

// devcontainer パターン（#30, #31）
#[test]
fn test_snapshot_devcontainer_go_db() {
    let (_, names) = render_devcontainer("go", true, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("devcontainer_go_db", sorted);
}

#[test]
fn test_snapshot_devcontainer_rust() {
    let (_, names) = render_devcontainer("rust", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("devcontainer_rust", sorted);
}

// Service Mesh パターン（#32, #33）
#[test]
fn test_snapshot_service_mesh_service_rest() {
    let (_, names) = render_service_mesh("service", vec!["rest"]);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("service_mesh_service_rest", sorted);
}

#[test]
fn test_snapshot_service_mesh_system_grpc() {
    let (_, names) = render_service_mesh("system", vec!["grpc"]);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("service_mesh_system_grpc", sorted);
}
```

各テストは生成ファイル一覧をソートし、YAML スナップショットとして保存する。初回実行時は `cargo insta test --accept` でスナップショットを生成し、以降はスナップショットとの差分を検出する。

### BFF スナップショットの説明

BFF（Backend for Frontend）スナップショットテスト（パターン 21-22）は、service Tier で GraphQL を選択した際に生成される GraphQL BFF ゲートウェイのファイル一覧を検証する。

BFF テンプレートは通常のサーバーテンプレートと同じ構造を持つが、以下の点が異なる:

- `kind` が `"bff"` として `TemplateContextBuilder` に渡される
- API スタイルは常に `"graphql"` 固定
- 生成先は `regions/service/{service_name}/server/{lang}/bff/` 配下
- GraphQL スキーマ定義（`schema.graphql`）とリゾルバが必ず含まれる

BFF スナップショットで検証されるファイル構成の例（Go BFF の場合）:

| ファイル | 説明 |
|---------|------|
| `cmd/main.go` | BFF エントリポイント |
| `internal/handler/graphql_resolver.go` | GraphQL リゾルバ |
| `schema.graphql` | GraphQL スキーマ定義 |
| `gqlgen.yml` | gqlgen 設定ファイル |
| `go.mod` | Go モジュール定義 |
| `Dockerfile` | コンテナイメージ定義 |
| `config/config.yaml` | アプリケーション設定 |

Rust BFF の場合も同様の構造で、Rust 固有のファイル（`Cargo.toml`、`src/main.rs` 等）が生成される。

---

## BFF テンプレートレンダリング内容検証テスト

BFF テンプレートのレンダリング結果が仕様書（テンプレート仕様-BFF.md）と一致することを検証する。

### Rust 統合テスト

```rust
#[test]
fn test_bff_go_main_go_content() {
    let (tmp, _) = render_bff("go");
    let content = read_output(&tmp, "cmd/main.go");

    assert!(content.contains("func main()"));
    assert!(content.contains("handler.NewResolver()"));
    assert!(content.contains("handler.NewGraphQLServer"));
    assert!(content.contains("-bff starting on :8080"));
    assert!(content.contains("http.ListenAndServe"));
}

#[test]
fn test_bff_go_resolver_content() {
    let (tmp, _) = render_bff("go");
    let content = read_output(&tmp, "internal/handler/graphql_resolver.go");

    assert!(content.contains("type Resolver struct"));
    assert!(content.contains("func NewResolver()"));
    assert!(content.contains("func NewGraphQLServer"));
    assert!(content.contains("/query"));
    assert!(content.contains("/healthz"));
}

#[test]
fn test_bff_go_config_yaml_content() {
    let (tmp, _) = render_bff("go");
    let content = read_output(&tmp, "config/config.yaml");

    assert!(content.contains("upstream:"));
    assert!(content.contains("http_url:"));
    assert!(content.contains("-bff"));
    // DB / Kafka / Redis セクションがないことを検証
    assert!(!content.contains("database:"));
    assert!(!content.contains("kafka:"));
    assert!(!content.contains("redis:"));
}

#[test]
fn test_bff_go_no_database_dependency() {
    let (tmp, _) = render_bff("go");
    let content = read_output(&tmp, "go.mod");

    assert!(content.contains("gqlgen"));
    assert!(content.contains("gqlparser"));
    // DB / Kafka / Redis 依存がないことを検証
    assert!(!content.contains("sqlx"));
    assert!(!content.contains("kafka-go"));
    assert!(!content.contains("go-redis"));
}

#[test]
fn test_bff_rust_main_rs_content() {
    let (tmp, _) = render_bff("rust");
    let content = read_output(&tmp, "src/main.rs");

    assert!(content.contains("-bff starting on :8080"));
    assert!(content.contains("/healthz"));
    assert!(content.contains("handler::graphql::configure"));
}

#[test]
fn test_bff_rust_cargo_toml_content() {
    let (tmp, _) = render_bff("rust");
    let content = read_output(&tmp, "Cargo.toml");

    assert!(content.contains("async-graphql"));
    assert!(content.contains("-bff"));
    // DB / Kafka / Redis 依存がないことを検証
    assert!(!content.contains("sqlx"));
    assert!(!content.contains("rdkafka"));
    assert!(!content.contains("redis"));
}

#[test]
fn test_bff_rust_config_yaml_content() {
    let (tmp, _) = render_bff("rust");
    let content = read_output(&tmp, "config/config.yaml");

    assert!(content.contains("upstream:"));
    assert!(content.contains("-bff"));
    assert!(!content.contains("database:"));
    assert!(!content.contains("kafka:"));
}
```

---

## Library Chart 連携テスト

Helm テンプレートから生成されるマニフェストファイルが、k1s0-common Library Chart を正しく呼び出していることを検証する。

### 概要

k1s0 の Helm テンプレートは、共通ロジックを Library Chart（`k1s0-common`）に集約している。各サービスの Helm Chart はアプリケーション Chart として、Library Chart が提供するヘルパーテンプレートを `include` で呼び出す構造となっている。本テストでは、この連携が正しく機能することを検証する。

### Rust 統合テスト — Library Chart 呼び出し検証

テスト配置: `CLI/tests/snapshot_tests.rs`（スナップショットテストと同ファイル、または `CLI/tests/helm_library_chart_tests.rs` として分離）

Helm テンプレートのレンダリング結果（`deployment.yaml`, `service.yaml` 等）が、Library Chart のヘルパーテンプレートを正しく参照していることを検証する。

#### 検証対象の include 呼び出し

| マニフェストファイル | 期待される include 呼び出し |
|--------------------|-----------------------------|
| `deployment.yaml` | `{{- include "k1s0-common.deployment" . }}` |
| `service.yaml` | `{{- include "k1s0-common.service" . }}` |
| `hpa.yaml` | `{{- include "k1s0-common.hpa" . }}` |
| `pdb.yaml` | `{{- include "k1s0-common.pdb" . }}` |
| `configmap.yaml` | `{{- include "k1s0-common.configmap" . }}` |
| `ingress.yaml` | `{{- include "k1s0-common.ingress" . }}` |

#### テストパターン

```rust
#[test]
fn test_helm_deployment_includes_library_chart() {
    let (tmp, _) = render_helm("rest", true, "postgresql");
    let content = read_output(&tmp, "templates/deployment.yaml");
    assert!(
        content.contains("{{- include \"k1s0-common.deployment\" . }}"),
        "deployment.yaml が k1s0-common.deployment を include していません"
    );
}

#[test]
fn test_helm_service_includes_library_chart() {
    let (tmp, _) = render_helm("rest", true, "postgresql");
    let content = read_output(&tmp, "templates/service.yaml");
    assert!(
        content.contains("{{- include \"k1s0-common.service\" . }}"),
        "service.yaml が k1s0-common.service を include していません"
    );
}

#[test]
fn test_helm_hpa_includes_library_chart() {
    let (tmp, _) = render_helm("rest", true, "postgresql");
    let content = read_output(&tmp, "templates/hpa.yaml");
    assert!(
        content.contains("{{- include \"k1s0-common.hpa\" . }}"),
        "hpa.yaml が k1s0-common.hpa を include していません"
    );
}

#[test]
fn test_helm_pdb_includes_library_chart() {
    let (tmp, _) = render_helm("rest", true, "postgresql");
    let content = read_output(&tmp, "templates/pdb.yaml");
    assert!(
        content.contains("{{- include \"k1s0-common.pdb\" . }}"),
        "pdb.yaml が k1s0-common.pdb を include していません"
    );
}

#[test]
fn test_helm_configmap_includes_library_chart() {
    let (tmp, _) = render_helm("rest", true, "postgresql");
    let content = read_output(&tmp, "templates/configmap.yaml");
    assert!(
        content.contains("{{- include \"k1s0-common.configmap\" . }}"),
        "configmap.yaml が k1s0-common.configmap を include していません"
    );
}

#[test]
fn test_helm_ingress_includes_library_chart() {
    let (tmp, _) = render_helm("rest", true, "postgresql");
    let content = read_output(&tmp, "templates/ingress.yaml");
    assert!(
        content.contains("{{- include \"k1s0-common.ingress\" . }}"),
        "ingress.yaml が k1s0-common.ingress を include していません"
    );
}
```

---

## テストデータ定義

### 標準コンテキスト

Rust 統合テストで使用するデフォルトのテンプレートコンテキスト値。

| 変数 | テスト用デフォルト値 | 備考 |
|------|-------------------|------|
| `service_name` | `"order-api"` | kebab-case 正規形 |
| `service_name_snake` | `"order_api"` | 自動導出 |
| `service_name_pascal` | `"OrderApi"` | 自動導出 |
| `service_name_camel` | `"orderApi"` | 自動導出 |
| `tier` | `"service"` | 最も一般的な階層 |
| `domain` | `""` | service tier では空 |
| `language` | テストにより変動 | `"rust"` / `"typescript"` / `"dart"` |
| `kind` | テストにより変動 | `"server"` / `"client"` / `"library"` / `"database"` |
| `api_style` | テストにより変動 | `"rest"` / `"grpc"` / `"graphql"` |
| `has_database` | テストにより変動 | `true` / `false` |
| `database_type` | テストにより変動 | `"postgresql"` / `"mysql"` / `"sqlite"` |
| `has_kafka` | テストにより変動 | `true` / `false` |
| `has_redis` | テストにより変動 | `true` / `false` |
| `rust_crate` | 自動導出 | `"order-api"` |
| `docker_registry` | `"harbor.internal.example.com"` | デフォルト値 |
| `docker_project` | `"k1s0-service"` | 自動導出: `"k1s0-{tier}"` |

### 変数置換テスト用コンテキスト

ケース変換の正確性を検証するため、別名 `"user-auth"` を使用する。

| 変数 | 値 |
|------|-----|
| `service_name` | `"user-auth"` |
| `service_name_snake` | `"user_auth"` |
| `service_name_pascal` | `"UserAuth"` |
| `service_name_camel` | `"userAuth"` |

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [テンプレート仕様-レンダリングテスト](レンダリングテスト.md) --- 概要・テスト対象マトリクス・CI統合・テスト命名規則
- [テンプレート仕様-サーバー](../server/サーバー.md) --- サーバーテンプレートの詳細仕様
