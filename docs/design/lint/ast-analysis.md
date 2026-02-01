# AST ベース解析の設計

## 概要

k1s0 v0.2.9 では、lint エンジンに tree-sitter ベースの AST（抽象構文木）解析を導入しました。これにより、コメント内や文字列リテラル内の誤検出を排除し、構文レベルで正確な違反検出を実現します。

## 動機

従来の grep/正規表現ベースの検出では、以下の問題が発生していました:

1. **偽陽性の多発**: コメント内の `unwrap()` やドキュメント内の SQL 例が違反として検出される
2. **コンテキスト無視**: テストコード内の `panic!` や、ビルドスクリプト内の環境変数アクセスが検出される
3. **構文的複雑さへの対応困難**: 複数行にまたがる文や、マクロ展開後の構造を扱えない

AST 解析により、これらの問題を構文レベルで解決します。

## アーキテクチャ

### モジュール構成

```
CLI/crates/k1s0-generator/src/lint/ast/
├── mod.rs              # 公開 API（AstContext, Language, ParserPool, QueryCache）
├── parser.rs           # ParserPool 実装（言語検出、パーサー管理）
├── query.rs            # QueryCache 実装（tree-sitter Query のコンパイル・キャッシュ）
├── context.rs          # AstContext 実装（is_non_code, is_in_test, query_matches）
└── languages/          # 言語固有のクエリ定義
    ├── rust.rs
    ├── go.rs
    ├── typescript.rs
    ├── python.rs
    ├── csharp.rs
    └── kotlin.rs
```

### コア API

#### `AstContext`

構文木全体へのアクセスとコンテキスト判定を提供します。

```rust
pub struct AstContext {
    tree: tree_sitter::Tree,
    source: Vec<u8>,
    language: Language,
}

impl AstContext {
    /// ファイルをパースして AstContext を作成
    pub fn parse(file_path: &Path, source: &str) -> Result<Self, String>;

    /// ノードがコメント・文字列リテラル・ドキュメント内かを判定
    pub fn is_non_code(&self, node: &Node) -> bool;

    /// ノードがテスト関数/モジュール内かを判定
    pub fn is_in_test(&self, node: &Node) -> bool;

    /// tree-sitter Query を実行
    pub fn query_matches(&self, query_id: QueryId) -> Result<Vec<QueryMatch>, String>;
}
```

**is_non_code() の判定ロジック:**

各言語で以下のノード種別をチェックします:

| 言語 | コメント | 文字列 | ドキュメント |
|------|---------|-------|------------|
| Rust | `line_comment`, `block_comment` | `string_literal`, `raw_string_literal` | `(attribute_item (identifier) @doc (#eq? @doc "doc"))` |
| Go | `comment` | `interpreted_string_literal`, `raw_string_literal` | — |
| TypeScript | `comment` | `string`, `template_string` | — |
| Python | `comment` | `string` | 関数/クラス直下の string を docstring として扱う |
| C# | `comment` | `string_literal`, `verbatim_string_literal` | `///` コメント |
| Kotlin | `comment`, `multiline_comment` | `string_literal` | `/**` コメント |

**is_in_test() の判定ロジック:**

| 言語 | 判定基準 |
|------|---------|
| Rust | `#[test]` 属性、`#[cfg(test)]` モジュール、ファイル名が `_test.rs` で終わる |
| Go | 関数名が `Test*` で始まる、ファイル名が `_test.go` で終わる |
| TypeScript | `describe(`, `it(`, `test(` ブロック内 |
| Python | `unittest.TestCase` クラス内、関数名が `test_` で始まる |
| C# | `[Test]`, `[TestMethod]`, `[Fact]` 属性を持つメソッド |
| Kotlin | `@Test` アノテーションを持つ関数、ファイル名が `Test.kt` で終わる |

#### `ParserPool`

言語ごとの tree-sitter パーサーを管理し、再利用します。

```rust
pub struct ParserPool {
    parsers: HashMap<Language, tree_sitter::Parser>,
}

impl ParserPool {
    /// ファイル拡張子から言語を検出
    pub fn detect_language(file_path: &Path) -> Option<Language>;

    /// パーサーを取得（なければ初期化）
    pub fn get_parser(&mut self, language: Language) -> &mut tree_sitter::Parser;
}
```

**対応言語と拡張子:**

| Language | 拡張子 |
|----------|-------|
| `Rust` | `.rs` |
| `Go` | `.go` |
| `TypeScript` | `.ts`, `.tsx` |
| `Python` | `.py` |
| `CSharp` | `.cs` |
| `Kotlin` | `.kt` |

#### `QueryCache`

tree-sitter Query のコンパイル結果をキャッシュします。

```rust
pub struct QueryCache {
    cache: HashMap<(Language, QueryId), tree_sitter::Query>,
}

impl QueryCache {
    /// Query を取得（なければコンパイルしてキャッシュ）
    pub fn get(&mut self, language: Language, query_id: QueryId) -> Result<&Query, String>;
}
```

**QueryId 定義:**

```rust
pub enum QueryId {
    // K029: Panic/unwrap/expect
    PanicCalls,

    // K050: SQL インジェクション
    SqlInterpolation,

    // K022: Clean Architecture
    ImportStatements,

    // K020: 環境変数
    EnvVarCalls,

    // K026: プロトコル型
    ProtocolImports,

    // K053: センシティブログ
    LogCalls,
}
```

### tree-sitter Query 例

#### Rust: unwrap/expect 検出

```scheme
;; メソッド呼び出し
(call_expression
  function: (field_expression
    field: (field_identifier) @method
    (#match? @method "^(unwrap|expect)$"))) @match

;; panic! 等のマクロ
(macro_invocation
  macro: (identifier) @macro
  (#match? @macro "^(panic|todo|unimplemented|unreachable)$")) @match
```

#### Go: SQL インジェクション検出

```scheme
;; fmt.Sprintf による SQL 文字列補間
(call_expression
  function: (selector_expression
    field: (field_identifier) @func
    (#eq? @func "Sprintf"))
  arguments: (argument_list
    (interpreted_string_literal) @sql
    (#match? @sql "(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER)"))) @match
```

#### Python: 環境変数検出

```scheme
;; os.getenv
(call
  function: (attribute
    object: (identifier) @mod
    attribute: (identifier) @func
    (#eq? @mod "os")
    (#eq? @func "getenv"))) @match

;; os.environ
(subscript
  value: (attribute
    object: (identifier) @mod
    attribute: (identifier) @attr
    (#eq? @mod "os")
    (#eq? @attr "environ"))) @match
```

## 対応ルールと検出方法

### K029: Panic/unwrap/expect

| 言語 | 検出対象 | Query |
|------|---------|-------|
| Rust | `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()` | `call_expression`, `macro_invocation` |
| Go | `panic(...)`, `log.Fatal(...)` | `call_expression` |
| Python | `raise ...`, `sys.exit(...)` | `raise_statement`, `call` |
| C# | `throw new ...`, `Environment.Exit(...)` | `throw_statement`, `invocation_expression` |
| Kotlin | `!!`, `error(...)`, `TODO(...)` | `postfix_expression`, `call_expression` |

**除外ロジック:**
- `is_non_code()` でコメント・文字列内を除外
- `is_in_test()` でテストコード内を除外
- Rust の場合、`src/main.rs` と `src/bin/` 内のエントリーポイントを除外

### K050: SQL インジェクション

| 言語 | 検出対象 | 安全な API（除外対象） |
|------|---------|---------------------|
| Rust | `format!("SELECT {} ...")` | `sqlx::query!`, `sqlx::query_as!` |
| Go | `fmt.Sprintf("SELECT %s ...")` | `db.Query($1, ...)` |
| TypeScript | `` `SELECT ${var} ...` `` | `sql.query($1, ...)` |
| Python | `f"SELECT {var} ..."` | `cursor.execute("SELECT %s", (var,))` |
| C# | `$"SELECT {var} ..."` | パラメータ化クエリ |
| Kotlin | `"SELECT ${var} ..."` | `PreparedStatement` |

**検出条件:**
1. 文字列補間構文を使用している
2. 文字列内に SQL キーワード（SELECT, INSERT, UPDATE, DELETE, DROP, ALTER）が含まれる
3. 安全な API（パラメータ化クエリ）を使用していない

### K022: Clean Architecture 依存違反

| 言語 | 検出対象 | 違反パターン例 |
|------|---------|--------------|
| Rust | `use crate::infrastructure` | `domain/` 内で `use crate::infrastructure::*` |
| Go | `import "myapp/infrastructure"` | `domain/` 内で `import "myapp/infrastructure/db"` |
| TypeScript | `import ... from '../infrastructure'` | `domain/` 内で `import { DB } from '../infrastructure'` |
| Python | `from infrastructure import ...` | `domain/` 内で `from infrastructure.db import ...` |
| C# | `using MyApp.Infrastructure` | `Domain/` 内で `using MyApp.Infrastructure.Persistence` |
| Kotlin | `import com.example.infrastructure` | `domain/` 内で `import com.example.infrastructure.db.*` |

**判定方法:**
1. ファイルのパスからレイヤーを判定（`domain/`, `application/`, `infrastructure/`, `presentation/`）
2. `import`/`use` 文のパスを解析してインポート先のレイヤーを判定
3. 禁止された依存方向（domain → infrastructure 等）を検出

### K020: 環境変数使用禁止

| 言語 | 検出対象 |
|------|---------|
| Rust | `std::env::var`, `env::var`, `dotenvy::var` |
| Go | `os.Getenv`, `os.LookupEnv`, `os.Environ` |
| TypeScript | `process.env.VAR`, `import.meta.env.VAR` |
| Python | `os.environ`, `os.getenv` |
| C# | `Environment.GetEnvironmentVariable` |
| Kotlin | `System.getenv` |

**除外対象:**
- ビルドスクリプト（`build.rs`, `Makefile` 等）
- テストファイル
- CI/CD スクリプト

### K026: Domain 層でのプロトコル型使用

| 言語 | 検出対象 | 違反例 |
|------|---------|-------|
| Rust | `axum::`, `tonic::`, `hyper::` | `use axum::http::StatusCode;` |
| Go | `net/http`, `google.golang.org/grpc` | `import "net/http"` |
| TypeScript | `express`, `@grpc/grpc-js` | `import { Request } from 'express'` |
| Python | `fastapi`, `grpcio` | `from fastapi import Request` |
| C# | `Microsoft.AspNetCore.Http`, `Grpc.Core` | `using Microsoft.AspNetCore.Http;` |
| Kotlin | `io.ktor.http`, `io.grpc` | `import io.ktor.http.HttpStatusCode` |

**判定方法:**
- ファイルが `domain/` ディレクトリ内にあることを確認
- インポート文がプロトコル関連のパッケージを参照していることを検出

### K053: センシティブデータのログ出力

| 言語 | 検出対象 | センシティブキーワード |
|------|---------|---------------------|
| Rust | `log::info!`, `tracing::info!`, `println!` | password, token, secret, api_key, credential, private_key |
| Go | `log.Println`, `log.Printf` | 同上 |
| TypeScript | `console.log`, `logger.info` | 同上 |
| Python | `logging.info`, `print` | 同上 |
| C# | `ILogger.LogInformation`, `Console.WriteLine` | 同上 |
| Kotlin | `logger.info`, `println` | 同上 |

**検出方法:**
1. ログ関数/マクロの呼び出しを検出
2. 引数内の変数名・フィールドアクセスを解析
3. センシティブキーワードが含まれていれば違反として報告

## フォールバック戦略

AST パースが失敗した場合、または `--fast` モードが指定された場合は、従来の grep ベース検出にフォールバックします。

```rust
#[cfg(feature = "ast")]
fn check_with_ast(source: &str, file_path: &Path) -> Vec<Violation> {
    match AstContext::parse(file_path, source) {
        Ok(ctx) => detect_via_ast(&ctx),
        Err(_) => detect_via_grep(source),  // フォールバック
    }
}

#[cfg(not(feature = "ast"))]
fn check_with_ast(source: &str, _file_path: &Path) -> Vec<Violation> {
    detect_via_grep(source)  // AST 機能が無効の場合
}
```

### `--fast` モード

```bash
# AST パースをスキップして grep ベースで実行（高速）
k1s0 lint --fast
```

**使用場面:**
- CI での高速チェック（精度より速度を優先）
- エディタ保存時の即時フィードバック
- 大規模リポジトリでの初回チェック

## パフォーマンス最適化

### パーサーキャッシュ

`ParserPool` は言語ごとに1つのパーサーインスタンスを再利用します。

```rust
// 初回のみパーサーを初期化
let mut pool = ParserPool::new();
for file in files {
    let lang = pool.detect_language(&file)?;
    let parser = pool.get_parser(lang);  // キャッシュから取得
    let tree = parser.parse(source, None)?;
}
```

### Query コンパイルキャッシュ

tree-sitter Query のコンパイルは比較的コストが高いため、`QueryCache` で結果をキャッシュします。

```rust
let mut cache = QueryCache::new();
for file in files {
    let ctx = AstContext::parse(&file, source)?;
    let query = cache.get(ctx.language(), QueryId::PanicCalls)?;  // キャッシュから取得
    let matches = ctx.query_matches(query)?;
}
```

### Capture 名フィルタリング

tree-sitter Query で `@match` capture を使い、不要なノードの処理をスキップします。

```scheme
;; 悪い例: 全ての call_expression がマッチ
(call_expression)

;; 良い例: unwrap/expect のみマッチ
(call_expression
  function: (field_expression
    field: (field_identifier) @method
    (#match? @method "^(unwrap|expect)$"))) @match
```

`@match` capture を持つノードのみを処理することで、重複検出を防ぎます。

## 制約事項

### 未対応言語

以下の言語は tree-sitter ABI version 15 に対応したクレートが存在しないため、AST 解析の対象外です:

- **Dart**: tree-sitter-dart に互換性のあるバージョンなし
- **YAML**: tree-sitter-yaml が tree-sitter 0.20 ABI を使用（0.25 と非互換）
- **Dockerfile**: tree-sitter-dockerfile が tree-sitter 0.20 ABI を使用（0.25 と非互換）

これらの言語では従来通り grep ベース検出が使用されます。

### AST 解析の限界

以下の高度な解析は現在未実装です:

1. **型推論**: 変数の型を追跡できない（例: `let x = create_db(); x.execute(sql)` の `x` が DB 接続であることを推論できない）
2. **クロスファイル解析**: 他のファイルで定義された関数やモジュールを解析できない
3. **re-export 追跡**: `pub use` で再エクスポートされた型を追跡できない
4. **型エイリアス解決**: 型エイリアス経由の依存を検出できない
5. **変数経由の文字列構築**: `let sql = format!("SELECT"); db.execute(sql)` のような間接的な SQL 構築を追跡できない

これらの機能は将来のバージョンで実装予定です。

## tree-sitter バージョンと ABI 互換性

### tree-sitter 0.25 へのアップグレード

k1s0 v0.2.9 では tree-sitter を 0.24 から 0.25 にアップグレードしました。

**理由:**
- Rust/Go/TypeScript/Python/C# の公式パーサーが ABI version 15 を要求
- ABI version 15 は tree-sitter 0.25 で導入された
- 0.24 では ABI version 14 までしか対応していない

### ABI バージョン対応表

| tree-sitter バージョン | ABI version | 対応パーサー例 |
|---------------------|------------|--------------|
| 0.25.x | 15 | tree-sitter-rust 0.24, tree-sitter-go 0.23 |
| 0.24.x | 14 | — |
| 0.20.x - 0.23.x | 13 | tree-sitter-yaml 0.6, tree-sitter-dockerfile 0.2 |

### `streaming-iterator` 依存の追加

tree-sitter 0.24 以降、`QueryMatches` の API が変更され、`StreamingIterator` を返すようになりました。

```rust
// tree-sitter 0.24+
use streaming_iterator::StreamingIterator;

let mut cursor = tree_sitter::QueryCursor::new();
let mut matches = cursor.matches(&query, root_node, source.as_bytes());
while let Some(m) = matches.next() {
    // ...
}
```

## LSP 統合の可能性（将来）

tree-sitter の増分パースを活用することで、以下の LSP 機能を実装可能です:

| 機能 | 説明 |
|------|------|
| リアルタイム診断 | ファイル編集中に lint 違反を即座に表示 |
| Quick Fix | AST ノード情報を利用した正確なコード修正提案 |
| ホバー情報 | 違反箇所にマウスオーバーで詳細説明を表示 |
| Code Lens | 修正可能な違反に対して「修正する」ボタンを表示 |

※ LSP 統合は v0.2.x の後続バージョンで実装予定です。

## 参考資料

- [tree-sitter 公式ドキュメント](https://tree-sitter.github.io/tree-sitter/)
- [tree-sitter Query Syntax](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries)
- [ADR-0017: AST ベース Lint エンジンへの移行](../../adr/ADR-0017-ast-based-lint-engine.md)
