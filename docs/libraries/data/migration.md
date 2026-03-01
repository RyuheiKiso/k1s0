# k1s0-migration ライブラリ設計

## 概要

DB スキーマ移行ライブラリ。sqlx Migrator（Rust）/ goose（Go）/ node-pg-migrate（TypeScript）/ sqflite_migration（Dart）/ Alembic（Python）の各言語標準ツールに共通インターフェースを被せ、マイグレーションファイルの命名規則・ディレクトリ構成・ロールバック・状態管理を標準化する。

テスト用インメモリマイグレーション（SQLite サポート）により、CI 環境での高速なスキーマ検証を可能にする。マイグレーション状態の確認（適用済み/未適用）と down migration（ロールバック）を全言語で統一 API として提供する。

**配置先**: `regions/system/library/rust/migration/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `MigrationRunner` | トレイト | マイグレーション実行の抽象インターフェース |
| `InMemoryMigrationRunner` | 構造体 | インメモリ実装（テスト・検証用） |
| `MigrationConfig` | 構造体 | マイグレーションディレクトリ・DB URL・テーブル名設定 |
| `MigrationReport` | 構造体 | 適用済みマイグレーション数・所要時間・エラー情報 |
| `MigrationStatus` | 構造体 | バージョン・名前・適用日時・チェックサム |
| `PendingMigration` | 構造体 | 未適用マイグレーションのバージョン・名前 |
| `MigrationFile` | 構造体 | マイグレーションファイルの解析・チェックサム計算 |
| `MigrationError` | enum | `ConnectionFailed`・`MigrationFailed`・`ChecksumMismatch`・`DirectoryNotFound`・`ParseError`・`Io` |

ユーティリティ関数:

| 関数 | 説明 |
|-----|------|
| `MigrationFile::parse_filename(filename)` | ファイル名からバージョン・名前・方向を解析 |
| `MigrationFile::checksum(content)` | SQL コンテンツの SHA-256 チェックサムを計算 |
| `InMemoryMigrationRunner::from_migrations(config, up_sqls, down_sqls)` | マイグレーション SQL を直接渡してランナーを構築（テスト用途） |

PostgreSQL 直接実行ランナー（全4言語実装済み）:

| 型名 | 言語 | 説明 |
|------|------|------|
| `SqlxMigrationRunner` | Rust | sqlx + PgPool による PostgreSQL 実行（`feature = "postgres"`） |
| `PostgresMigrationRunner` | Go | `database/sql` + `lib/pq` による PostgreSQL 実行 |
| `PgMigrationRunner` | TypeScript | `pg` Pool による PostgreSQL 実行 |
| `PostgresMigrationRunner` | Dart | `postgres` パッケージの Connection による PostgreSQL 実行 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-migration"
version = "0.1.0"
edition = "2021"

[features]
sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]
cli = ["clap"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "migrate", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
clap = { version = "4", features = ["derive"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-migration = { path = "../../system/library/rust/migration" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
migration/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── runner.rs       # MigrationRunner トレイト・InMemoryMigrationRunner
│   ├── sqlx_runner.rs  # SqlxMigrationRunner（PostgreSQL 実行、#[cfg(feature = "postgres")]）
│   ├── config.rs       # MigrationConfig（ディレクトリ・DB URL・テーブル名）
│   ├── model.rs        # MigrationReport・MigrationStatus・PendingMigration・MigrationFile・parse_filename・checksum
│   └── error.rs        # MigrationError
└── Cargo.toml
```

**命名規則・ディレクトリ構成**:

```
migrations/
├── 20240101000001_create_users.up.sql
├── 20240101000001_create_users.down.sql
├── 20240101000002_add_email_index.up.sql
├── 20240101000002_add_email_index.down.sql
└── 20240201000001_add_tenant_id.up.sql
```

ファイル命名規則: `{version}_{name}.{direction}.sql`
- `version`: 14桁の数値（YYYYMMDDHHmmSS）
- `name`: スネークケースの説明的な名前
- `direction`: `up`（適用）または `down`（ロールバック）

**使用例**:

```rust
use k1s0_migration::{MigrationRunner, InMemoryMigrationRunner, MigrationConfig};
use std::path::PathBuf;

let config = MigrationConfig::new(PathBuf::from("./migrations"), "postgres://...".to_string());

// ディスク上のマイグレーションファイルを読み込んで実行
let runner = InMemoryMigrationRunner::new(config.clone()).unwrap();

// または、マイグレーション SQL を直接渡して構築（テスト用途に便利）
let runner = InMemoryMigrationRunner::from_migrations(
    config,
    vec![("20240101000001".into(), "create_users".into(), "CREATE TABLE users (id INT);".into())],
    vec![("20240101000001".into(), "create_users".into(), "DROP TABLE users;".into())],
);

// up マイグレーション（全件適用）
let report = runner.run_up().await.unwrap();
println!("Applied {} migrations in {:?}", report.applied_count, report.elapsed);

// ステータス確認
let statuses = runner.status().await.unwrap();
for s in &statuses {
    println!(
        "{} {} [{}]",
        s.version,
        s.name,
        if s.applied_at.is_some() { "applied" } else { "pending" }
    );
}

// 未適用マイグレーション一覧
let pending = runner.pending().await.unwrap();
println!("{} pending migrations", pending.len());

// down マイグレーション（2ステップロールバック）
let report = runner.run_down(2).await.unwrap();
println!("Rolled back {} migrations", report.applied_count);
```

## Go 実装

**配置先**: `regions/system/library/go/migration/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/lib/pq v1.11.2`、`github.com/stretchr/testify v1.11.1`（goose 不使用）

**主要インターフェース**:

```go
type MigrationRunner interface {
    RunUp(ctx context.Context) (*MigrationReport, error)
    RunDown(ctx context.Context, steps int) (*MigrationReport, error)
    Status(ctx context.Context) ([]*MigrationStatus, error)
    Pending(ctx context.Context) ([]*PendingMigration, error)
}

type MigrationConfig struct {
    MigrationsDir string
    DatabaseURL   string
    TableName     string   // default: _migrations
    Driver        string   // postgres, sqlite3
}

type MigrationReport struct {
    AppliedCount int
    Elapsed      time.Duration
    Errors       []error
}

type MigrationStatus struct {
    Version   string
    Name      string
    AppliedAt *time.Time
    Checksum  string
}

type PendingMigration struct {
    Version string
    Name    string
}

func NewMigrationConfig(migrationsDir, databaseURL string) MigrationConfig
func NewInMemoryRunner(cfg MigrationConfig) (*InMemoryMigrationRunner, error)
func NewInMemoryRunnerFromMigrations(cfg MigrationConfig, ups, downs []struct{ Version, Name, Content string }) *InMemoryMigrationRunner
func NewPostgresMigrationRunner(db *sql.DB, config MigrationConfig) (*PostgresMigrationRunner, error)
type MigrationDirection int

const (
	DirectionUp   MigrationDirection = iota
	DirectionDown
)

func ParseFilename(filename string) (version, name string, direction MigrationDirection, ok bool)
func Checksum(content string) string
```

`PostgresMigrationRunner` は `MigrationRunner` インターフェースを実装し、PostgreSQL に対してトランザクション付きでマイグレーションを実行する。`_migrations` テーブルを自動作成し、適用済みバージョンを管理する。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/migration/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface MigrationStatus {
  version: string;
  name: string;
  appliedAt: Date | null;
  checksum: string;
}

export interface PendingMigration {
  version: string;
  name: string;
}

export interface MigrationReport {
  appliedCount: number;
  elapsedMs: number;
  errors: Error[];
}

export interface MigrationConfig {
  migrationsDir: string;
  databaseUrl: string;
  tableName?: string;  // default: "_migrations"
}

export interface MigrationRunner {
  runUp(): Promise<MigrationReport>;
  runDown(steps: number): Promise<MigrationReport>;
  status(): Promise<MigrationStatus[]>;
  pending(): Promise<PendingMigration[]>;
}

export class InMemoryMigrationRunner implements MigrationRunner {
  constructor(
    config: MigrationConfig,
    ups: Array<{version: string, name: string, content: string}>,
    downs: Array<{version: string, name: string, content: string}>,
  );
  runUp(): Promise<MigrationReport>;
  runDown(steps: number): Promise<MigrationReport>;
  status(): Promise<MigrationStatus[]>;
  pending(): Promise<PendingMigration[]>;
}

export function parseFilename(filename: string): ParsedMigration | null;
export function checksum(content: string): string;

export class MigrationError extends Error {
  constructor(message: string, public readonly cause?: Error);
}

export class PgMigrationRunner implements MigrationRunner {
  constructor(pool: Pool, config: MigrationConfig);
  runUp(): Promise<MigrationReport>;
  runDown(steps: number): Promise<MigrationReport>;
  status(): Promise<MigrationStatus[]>;
  pending(): Promise<PendingMigration[]>;
  close(): Promise<void>;
}
```

`PgMigrationRunner` は `pg` パッケージの `Pool` を使用して PostgreSQL に対してトランザクション付きでマイグレーションを実行する。`close()` メソッドで接続プールを終了できる。

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/migration/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  crypto: ^3.0.3
  path: ^1.9.0
  meta: ^1.14.0
  postgres: ^3.1.0
```

**主要インターフェース**:

```dart
abstract class MigrationRunner {
  Future<MigrationReport> runUp();
  Future<MigrationReport> runDown(int steps);
  Future<List<MigrationStatus>> status();
  Future<List<PendingMigration>> pending();
}

class MigrationConfig {
  final String migrationsDir;
  final String databaseUrl;
  final String tableName;

  const MigrationConfig({
    required this.migrationsDir,
    required this.databaseUrl,
    this.tableName = '_migrations',
  });
}

class MigrationStatus {
  final String version;
  final String name;
  final DateTime? appliedAt;
  final String checksum;
}

class MigrationFile {
  static ({String version, String name, MigrationDirection direction})?
      parseFilename(String filename);
  static String computeChecksum(String content);  // SHA-256
}

class PostgresMigrationRunner implements MigrationRunner {
  PostgresMigrationRunner({
    required Connection connection,
    required MigrationConfig config,
    required List<({String version, String name, String content})> ups,
    required List<({String version, String name, String content})> downs,
  });
}

class MigrationError implements Exception {
  MigrationError(String message, {String? code, Object? cause});
  factory MigrationError.connectionFailed(String message, {Object? cause});
  factory MigrationError.migrationFailed(String version, String message, {Object? cause});
  factory MigrationError.checksumMismatch(String version, String expected, String actual);
  factory MigrationError.directoryNotFound(String path);
}
```

`PostgresMigrationRunner` は `postgres` パッケージの `Connection` を使用して PostgreSQL に対してマイグレーションを実行する。`MigrationFile.computeChecksum()` は Dart の命名規則に従い `checksum` ではなく `computeChecksum` を使用する。

**カバレッジ目標**: 85%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | ファイル名パース・チェックサム計算・バージョン順ソート | tokio::test |
| インメモリテスト | SQLite インメモリ DB での up/down/status ラウンドトリップ | tokio::test + SQLite feature |
| PostgreSQL 統合テスト | testcontainers + PostgreSQL コンテナでの全操作検証 | testcontainers |
| チェックサム検証テスト | 適用済みマイグレーションファイル改ざん時の `ChecksumMismatch` エラー確認 | tokio::test |
| ロールバックテスト | down マイグレーションでのスキーマ巻き戻し確認 | testcontainers |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-test-helper設計](../testing/test-helper.md) — テスト用 DB 起動との組み合わせ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) — マイグレーションファイルの命名規則・レビュー手順
- [system-auth-server設計](../../servers/auth/server.md) — DB スキーマ管理の実例
