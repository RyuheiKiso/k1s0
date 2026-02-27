# k1s0-migration ライブラリ設計

## 概要

DB スキーマ移行ライブラリ。sqlx Migrator（Rust）/ goose（Go）/ node-pg-migrate（TypeScript）/ sqflite_migration（Dart）/ Alembic（Python）の各言語標準ツールに共通インターフェースを被せ、マイグレーションファイルの命名規則・ディレクトリ構成・ロールバック・状態管理を標準化する。

テスト用インメモリマイグレーション（SQLite サポート）により、CI 環境での高速なスキーマ検証を可能にする。マイグレーション状態の確認（適用済み/未適用）と down migration（ロールバック）を全言語で統一 API として提供する。

**配置先**: `regions/system/library/rust/migration/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `MigrationRunner` | トレイト | マイグレーション実行の抽象インターフェース |
| `SqlxMigrationRunner` | 構造体 | sqlx Migrator 実装（PostgreSQL・SQLite 対応） |
| `MigrationConfig` | 構造体 | マイグレーションディレクトリ・DB URL・テーブル名設定 |
| `MigrationReport` | 構造体 | 適用済みマイグレーション数・所要時間・エラー情報 |
| `MigrationStatus` | 構造体 | バージョン・名前・適用日時・チェックサム |
| `PendingMigration` | 構造体 | 未適用マイグレーションのバージョン・名前 |
| `MigrationError` | enum | `ConnectionFailed`・`MigrationFailed`・`ChecksumMismatch`・`DirectoryNotFound` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-migration"
version = "0.1.0"
edition = "2021"

[features]
sqlite = ["sqlx/sqlite"]
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
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-migration = { path = "../../system/library/rust/migration" }
# SQLite（テスト用インメモリ）を有効化する場合:
k1s0-migration = { path = "../../system/library/rust/migration", features = ["sqlite"] }
```

**モジュール構成**:

```
migration/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── runner.rs       # MigrationRunner トレイト・SqlxMigrationRunner
│   ├── config.rs       # MigrationConfig（ディレクトリ・DB URL・テーブル名）
│   ├── model.rs        # MigrationReport・MigrationStatus・PendingMigration
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
use k1s0_migration::{MigrationRunner, SqlxMigrationRunner, MigrationConfig};
use std::path::PathBuf;

let config = MigrationConfig {
    migrations_dir: PathBuf::from("./migrations"),
    database_url: std::env::var("DATABASE_URL").unwrap(),
    table_name: "_migrations".to_string(),
};

let runner = SqlxMigrationRunner::new(config).await.unwrap();

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

**配置先**: `regions/system/library/go/migration/`

```
migration/
├── migration.go
├── runner.go
├── config.go
├── model.go
├── migration_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/pressly/goose/v3 v3.24.0`, `github.com/stretchr/testify v1.10.0`

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
    Version   int64
    Name      string
    AppliedAt *time.Time
    Checksum  string
}

type PendingMigration struct {
    Version int64
    Name    string
}

func NewGooseMigrationRunner(cfg MigrationConfig) (MigrationRunner, error)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/migration/`

```
migration/
├── package.json        # "@k1s0/migration", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # MigrationRunner, PgMigrationRunner, MigrationConfig, MigrationStatus, MigrationError
└── __tests__/
    └── migration.test.ts
```

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

export class PgMigrationRunner implements MigrationRunner {
  constructor(config: MigrationConfig);
  runUp(): Promise<MigrationReport>;
  runDown(steps: number): Promise<MigrationReport>;
  status(): Promise<MigrationStatus[]>;
  pending(): Promise<PendingMigration[]>;
  close(): Promise<void>;
}

export class MigrationError extends Error {
  constructor(message: string, public readonly cause?: Error);
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/migration/`

```
migration/
├── pubspec.yaml        # k1s0_migration
├── analysis_options.yaml
├── lib/
│   ├── migration.dart
│   └── src/
│       ├── runner.dart       # MigrationRunner abstract, SqliteMigrationRunner
│       ├── config.dart       # MigrationConfig
│       ├── model.dart        # MigrationReport, MigrationStatus, PendingMigration
│       └── error.dart        # MigrationError
└── test/
    └── migration_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  sqflite_common_ffi: ^2.3.4
  path: ^1.9.0
  meta: ^1.14.0
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
```

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

- [system-library-概要](../overview/概要.md) — ライブラリ一覧・テスト方針
- [system-library-test-helper設計](../testing/test-helper設計.md) — テスト用 DB 起動との組み合わせ
- [開発ルール.md](開発ルール.md) — マイグレーションファイルの命名規則・レビュー手順
- [system-auth-server設計](../../system-servers/auth/server設計.md) — DB スキーマ管理の実例
