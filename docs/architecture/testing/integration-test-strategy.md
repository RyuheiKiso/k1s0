# 統合テスト戦略

H-01 対応: 統合テストにおける実 DB 使用の段階的導入計画を定義する。

---

## 現状の問題と背景（監査指摘 H-01）

### 監査指摘内容

外部監査 H-01 は以下の問題を指摘している。

- `.github/workflows/integration-test.yaml` の統合テストジョブは PostgreSQL サービスコンテナを起動しているにもかかわらず、テストコードは実際に DB への接続を行わない
- 各サーバー（auth-server, config-server, saga-server 等）の統合テストはスタブリポジトリを使用しており、DB レイヤーの正確性を検証できていない
- CI 上で `DATABASE_URL` 環境変数が渡されているが、テスト実装がこれを参照しないため、接続検証が実施されていない

### 現状の確認済み事実

`integration-test.yaml` の "Derive schema name" ステップに以下の TODO コメントが記録されている。

```
# 注: 現在の統合テストはスタブリポジトリを使用するため DB 接続は行われない
# TODO: 実 DB テストの追加
#   - スタブリポジトリを実リポジトリ実装に差し替える
#   - `#[ignore]` タグを外し、実 DB 接続テストを有効化する
#   - 対象: auth-server, config-server, saga-server 等
```

auth-server の `tests/db_integration_test.rs` には testcontainers を使用した DB 統合テストが実装済みであるが、すべてのテストに `#[ignore = "requires Docker (testcontainers)"]` タグが付与されており、CI では実行されない状態にある。

### 問題の影響

| リスク | 内容 |
| --- | --- |
| DB スキーマの回帰検知不能 | マイグレーションの変更が実際の CRUD 操作に与える影響を CI で検証できない |
| リポジトリ実装のバグ見逃し | スタブと本実装の乖離がリリース後まで発見されない |
| 監査コンプライアンス違反 | 「統合テストが実 DB を使用していない」という指摘への未対応状態が継続する |

---

## 目標

実 DB を使用した統合テストを段階的に CI へ導入し、DB レイヤーのカバレッジを 70% 以上に引き上げる。

### 達成基準

- CI の統合テストジョブが PostgreSQL サービスコンテナへ実際に接続すること
- DB 依存のリポジトリ実装テストが `#[ignore]` なしで実行されること
- 対象サーバーの DB レイヤーカバレッジが 70% 以上であること
- テスト実行時間が 10 分以内に収まること（既存の CI 目標との整合）

---

## 対象サービスと優先順位

DB 結合度・監査影響度・既存テスト整備状況を基準に以下の順序で対応する。

| 優先順位 | サーバー | 理由 | 現状 |
| --- | --- | --- | --- |
| 1 | **auth-server** | 認証基盤として最も高リスク。`db_integration_test.rs` が実装済みで `#[ignore]` 解除のみ必要 | testcontainers 実装済み・`#[ignore]` タグ付き |
| 2 | **config-server** | 全サービスが依存する設定取得。`postgres_repository_test.rs` が存在 | スタブ使用・実 DB テスト未実装 |
| 3 | **saga-server** | 分散トランザクションの整合性保証に DB が必須。`postgres_repository_test.rs` が存在 | スタブ使用・実 DB テスト未実装 |
| 4 | **session-server** | セッション管理の永続化層の検証 | スタブ使用・実 DB テスト未実装 |
| 5 | **tenant-server** | マルチテナント分離の DB レベル検証 | スタブ使用・実 DB テスト未実装 |

---

## テスト環境の設計

### アプローチ選択

実 DB テストの実行環境として以下の 2 つのアプローチを比較した。

| アプローチ | 方式 | メリット | デメリット |
| --- | --- | --- | --- |
| **testcontainers-rs** | テストコード内から Docker コンテナを起動・制御する | テストと DB ライフサイクルが一致する。並列テストで独立した DB を持てる | Docker デーモンが必要。起動オーバーヘッドが生じる |
| **GitHub Actions サービスコンテナ** | ワークフロー yaml で宣言した PostgreSQL コンテナを使用する | 既に CI に設定済み。追加の Docker 操作不要 | テスト間でスキーマを共有するためアイソレーションに注意が必要 |

### 採用方針

**testcontainers-rs を優先採用する。**

理由は以下のとおり。

1. `auth-server` の `dev-dependencies` に `testcontainers = "0.24"` および `testcontainers-modules = { version = "0.11", features = ["postgres"] }` が既に追加されており、実装パターンが確立されている
2. テストごとに独立した PostgreSQL コンテナを起動するため、テスト間のデータ汚染を防止できる
3. GitHub Actions のサービスコンテナは「Initialize database schemas」ステップで init-db/*.sql を全件適用するため、スキーマが混在する。testcontainers であれば各テスト専用マイグレーションを適用できる

GitHub Actions のサービスコンテナ（postgres: image: postgres:17.2）は引き続き起動するが、testcontainers テストが独自コンテナを使用するため、サービスコンテナへの直接接続は不要となる。

### testcontainers 実装パターン

auth-server の `db_integration_test.rs` で確立されたパターンを標準として採用する。

```rust
// tests/db_integration_test.rs
// testcontainers を使用して実 PostgreSQL に対してリポジトリ実装を検証する

use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

/// テスト用 PostgreSQL コンテナを起動してマイグレーション済みの接続プールを返す
async fn setup_pool() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    // PostgreSQL コンテナを起動し、動的に割り当てられたポートを取得する
    let container = Postgres::default().start().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        host_port
    );

    let pool = PgPool::connect(&connection_string).await.unwrap();

    // マイグレーションを適用してスキーマを初期化する
    sqlx::migrate!("../../../database/{server}-db/migrations")
        .run(&pool)
        .await
        .unwrap();

    (pool, container)
}

// コンテナのライフタイムはテスト関数スコープ内で管理する（_container として保持）
#[tokio::test]
async fn test_repository_crud() {
    let (pool, _container) = setup_pool().await;
    // テスト実装
}
```

### Cargo.toml の feature 設定

DB テスト有効化のために `db-tests` feature を使用する。auth-server では既に定義済み。

```toml
[features]
# DB 統合テストを有効化するフラグ。CI の db-tests ジョブでのみ有効化する
db-tests = []

[dev-dependencies]
testcontainers = "0.24"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
```

---

## Phase 1 実装計画（auth-server パイロット）

### 目標

auth-server を対象に実 DB 統合テストを CI に組み込み、以降のサーバー展開のパターンを確立する。

### 実装手順

#### Step 1: `#[ignore]` タグの解除

`tests/db_integration_test.rs` の全テスト関数から `#[ignore = "requires Docker (testcontainers)"]` を削除し、`db-tests` feature フラグによる条件コンパイルに置き換える。

```rust
// #[ignore] を削除し、feature フラグで制御する
#[cfg(feature = "db-tests")]
#[tokio::test]
async fn test_audit_log_crud_with_real_db() {
    // 既存の実装をそのまま使用する
}
```

#### Step 2: `user_postgres` および `api_key_postgres` リポジトリのテスト追加

`tests/db_integration_test.rs` に `UserPostgresRepository` と `ApiKeyPostgresRepository` の CRUD テストを追加する。

対象テストケース:
- ユーザー作成・取得・一覧
- API キー作成・検証・失効
- 監査ログ作成・ページネーション検索（既存）

#### Step 3: CI ワークフローへの組み込み

`integration-test.yaml` の `Run integration tests` ステップを拡張し、`db-tests` feature を有効にして実行する。

```yaml
- name: Run integration tests with real DB (${{ matrix.server }})
  env:
    DATABASE_URL: "postgresql://dev:dev@localhost:5432/k1s0_system?options=-c%20search_path%3D${{ steps.schema.outputs.name }},public"
    KAFKA_BROKERS: "localhost:9092"
  run: |
    # H-01 対応: db-tests feature を有効化して実 DB テストを実行する
    cd regions/system
    pkg_name=$(grep -m1 '^name' "server/rust/${{ matrix.server }}/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
    cargo test -p "$pkg_name" --test db_integration_test --features "$pkg_name/db-tests"
```

---

## Phase 2 以降の展開計画

### Phase 2: config-server, saga-server（Phase 1 完了の 2 週間後）

各サーバーで以下を実施する。

1. `Cargo.toml` に `db-tests` feature と `testcontainers` dev-dependencies を追加する
2. `tests/db_integration_test.rs` を新規作成し、リポジトリ実装のテストを記述する
3. `postgres_repository_test.rs` の既存テストを実 DB テストに移行する

### Phase 3: session-server, tenant-server（Phase 2 完了の 2 週間後）

Phase 2 と同様の手順で対応する。session-server は Redis との組み合わせも検討する。

### Phase 4: business tier / service tier への展開

system tier での確立パターンを business tier および service tier のサーバーに横展開する。

---

## カバレッジ目標

### DB レイヤーカバレッジ

| 対象 | 目標 | 計測ツール |
| --- | --- | --- |
| DB レイヤー全体（リポジトリ実装） | 70% 以上 | `cargo-tarpaulin` |
| auth-server DB レイヤー（Phase 1） | 80% 以上 | `cargo-tarpaulin` |
| 各サーバー DB レイヤー（Phase 2 以降） | 70% 以上 | `cargo-tarpaulin` |

### カバレッジの計測対象ファイル

```
adapter/repository/user_postgres.rs
adapter/repository/api_key_postgres.rs
adapter/repository/audit_log_postgres.rs
```

### カバレッジレポートの CI 統合

```yaml
# cargo-tarpaulin でカバレッジを計測してアーティファクトとして保存する
- name: Measure coverage (DB layer)
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --features db-tests \
      --include-files "src/adapter/repository/*.rs" \
      --out Xml --output-dir coverage/
- name: Upload coverage report
  uses: actions/upload-artifact@v4
  with:
    name: db-layer-coverage-${{ matrix.server }}
    path: coverage/
```

---

## CI への組み込み方法

### ワークフロー拡張方針

既存の `integration-test.yaml` を拡張し、通常の統合テスト（スタブ使用）と DB 統合テスト（実 DB 使用）を分離したステップとして実行する。

```
┌─────────────────────────────────┐
│ integration-test-system ジョブ   │
│                                 │
│ ステップ 1: Initialize database  │   ← init-db/*.sql でスキーマ初期化
│ ステップ 2: Derive schema name   │   ← スキーマ名導出
│ ステップ 3: Run integration      │   ← スタブ使用テスト（既存）
│            tests (stub)         │
│ ステップ 4: Run integration      │   ← 実 DB テスト（新規追加）
│            tests (real DB)      │     testcontainers で独自コンテナ起動
└─────────────────────────────────┘
```

### 実行時間の管理

testcontainers は Docker コンテナ起動のオーバーヘッドが生じる。以下の方針で 10 分以内に収める。

| 対策 | 内容 |
| --- | --- |
| コンテナの使い回し | 1 テストファイル内で `setup_pool()` を一度だけ呼び出し、プールを共有する |
| 並列実行の制御 | `#[serial_test]` クレートを使用してコンテナ起動の競合を防ぐ（必要な場合のみ） |
| Rust キャッシュの活用 | 既存の `Swatinem/rust-cache` アクションで依存クレートをキャッシュする |
| イメージの固定 | `Postgres::default()` の使用バージョンを workspace の postgres:17.2 と統一する |

### Docker の利用可否

GitHub Actions の `ubuntu-latest` ランナーでは Docker デーモンが利用可能であるため、testcontainers の実行に追加設定は不要。

---

## 関連ドキュメント

- [テスト戦略](./test-strategy.md) — テストピラミッド・言語別フレームワーク・カバレッジ目標
- [パフォーマンステスト戦略](./performance-strategy.md) — 負荷テスト・ベンチマーク設計
- [E2E テスト戦略](./e2e-strategy.md) — E2E テストの詳細設計
- [CI/CD 設計](../../infrastructure/cicd/CI-CD設計.md) — CI/CD パイプライン設計

---

## 改訂履歴

| 日付 | 版 | 内容 |
| --- | --- | --- |
| 2026-03-24 | 1.0 | 初版作成（外部監査 H-01 対応） |
