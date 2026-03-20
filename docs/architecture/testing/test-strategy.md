# テスト戦略

F-029: k1s0 プロジェクト全体のテスト戦略を定義する。テストピラミッド、言語別フレームワーク、カバレッジ目標、テストデータ戦略、CI 連携を含む。

---

## 基本方針

- **テストピラミッド** を遵守し、単体テストを最も厚くする
- 全言語でテストを CI に統合し、PR マージの必須条件とする
- テストカバレッジの閾値を設定し、未達の場合は CI を失敗させる
- テストデータは再現可能な方法で管理し、環境間の差異を最小化する

---

## テストピラミッド

```
          ┌─────┐
          │ E2E │          少数・高コスト・遅い
         ─┤     ├─         重要なユーザーフローのみ
        ┌─┴─────┴─┐
        │統合テスト│        中程度・DB/Kafka 等の外部依存を含む
       ─┤         ├─       サービス間連携の正常性を検証
      ┌─┴─────────┴─┐
      │  単体テスト  │      大量・低コスト・高速
      │              │      ビジネスロジックの網羅的テスト
      └──────────────┘
```

| テストレベル | 目的 | 実行頻度 | 実行時間目標 |
| --- | --- | --- | --- |
| 単体テスト | 個々の関数・モジュールの正確性を検証する | PR ごと | 5 分以内 |
| 統合テスト | 外部依存（DB、Kafka 等）を含むコンポーネント間の連携を検証する | PR ごと | 10 分以内 |
| E2E テスト | ユーザーフロー全体の正常動作を検証する | main マージ後 | 30 分以内 |

---

## 言語別テストフレームワーク

### Rust

| ツール | 用途 | 設定ファイル |
| --- | --- | --- |
| `cargo test` | 単体テスト・統合テスト | `Cargo.toml` |
| `cargo-tarpaulin` | カバレッジ計測 | `.tarpaulin.toml`（存在する場合） |
| `criterion` | ベンチマーク | `benches/` ディレクトリ |

#### テスト構成

```rust
// 単体テスト: 各モジュール内に #[cfg(test)] で配置
#[cfg(test)]
mod tests {
    use super::*;

    /// 正常系: 有効な入力で期待する結果を返すことを検証する
    #[test]
    fn test_valid_input_returns_expected_result() {
        let result = process_input("valid");
        assert_eq!(result, expected_output());
    }
}

// 統合テスト: tests/ ディレクトリに配置
// test-utils feature で統合テスト用ヘルパーを有効化する
#[cfg(feature = "test-utils")]
mod integration {
    /// DB 接続を伴う統合テスト
    #[tokio::test]
    async fn test_create_and_retrieve_entity() {
        let pool = setup_test_db().await;
        // テスト実装
    }
}
```

#### CI ワークフロー

- PR 時: `cargo test --all`（単体テスト）
- PR 時: `integration-test.yaml` で PostgreSQL + Kafka を起動し、`test-utils` feature を有効化した統合テストを実行
- PR 時: `ci.yaml` の `coverage-rust` ジョブで `cargo-tarpaulin` によるカバレッジ計測を実行

### Go

| ツール | 用途 | 設定ファイル |
| --- | --- | --- |
| `go test` | 単体テスト・統合テスト | `*_test.go` |
| `go test -cover` | カバレッジ計測 | --- |
| `golangci-lint` | 静的解析 | `.golangci.yml` |

#### テスト構成

```go
// 単体テスト: 同一パッケージ内に _test.go で配置
func TestHandleRequest_ValidInput(t *testing.T) {
    // テーブルドリブンテストを推奨する
    tests := []struct {
        name     string
        input    string
        expected string
    }{
        {"正常系: 有効な入力", "valid", "expected"},
        {"異常系: 空文字列", "", ""},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            result := HandleRequest(tt.input)
            if result != tt.expected {
                t.Errorf("got %q, want %q", result, tt.expected)
            }
        })
    }
}

// 統合テスト: ビルドタグで分離する
//go:build integration
func TestDatabaseIntegration(t *testing.T) {
    // DB 接続を伴うテスト
}
```

### TypeScript (React)

| ツール | 用途 | 設定ファイル |
| --- | --- | --- |
| Vitest | 単体テスト・コンポーネントテスト | `vitest.config.ts` |
| React Testing Library | コンポーネントテスト | --- |
| Playwright | E2E テスト | `playwright.config.ts` |
| `vitest --coverage` | カバレッジ計測（v8 プロバイダ） | `vitest.config.ts` |

#### テスト構成

```typescript
// 単体テスト: *.test.ts / *.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

// コンポーネントの描画結果を検証する
describe('UserCard', () => {
  it('ユーザー名を表示する', () => {
    render(<UserCard name="テストユーザー" />);
    expect(screen.getByText('テストユーザー')).toBeInTheDocument();
  });
});
```

### Dart (Flutter)

| ツール | 用途 | 設定ファイル |
| --- | --- | --- |
| `flutter test` | 単体テスト・Widget テスト | `pubspec.yaml` |
| `flutter test --coverage` | カバレッジ計測 | --- |
| `integration_test` | 統合テスト | `integration_test/` ディレクトリ |

#### テスト構成

```dart
// 単体テスト: test/ ディレクトリに配置
// ビジネスロジックの正確性を検証する
void main() {
  group('OrderCalculator', () {
    test('合計金額を正しく計算する', () {
      final calculator = OrderCalculator();
      final total = calculator.calculate([
        OrderItem(price: 100, quantity: 2),
        OrderItem(price: 200, quantity: 1),
      ]);
      expect(total, equals(400));
    });
  });
}

// Widget テスト: コンポーネントの振る舞いを検証する
void main() {
  testWidgets('ボタン押下でカウンターが増加する', (tester) async {
    await tester.pumpWidget(const MyApp());
    await tester.tap(find.byIcon(Icons.add));
    await tester.pump();
    expect(find.text('1'), findsOneWidget);
  });
}
```

---

## テストカバレッジ目標

### 言語別カバレッジ閾値

| 言語 | カバレッジ目標 | 計測ツール | CI での閾値チェック |
| --- | --- | --- | --- |
| Rust | **70%** | cargo-tarpaulin | `ci.yaml` の `coverage-rust` ジョブで計測、PR コメントにレポート |
| Go | **70%** | go test -coverprofile | CI ジョブ内で閾値チェック |
| TypeScript | **80%** | vitest --coverage (v8) | CI ジョブ内で閾値チェック |
| Dart | **80%** | flutter test --coverage | CI ジョブ内で閾値チェック |

### カバレッジ除外対象

以下のコードはカバレッジ計測から除外する。

| 除外対象 | 理由 |
| --- | --- |
| 自動生成コード（proto 生成ファイル等） | 手動テストの対象外であるため |
| `main.rs` / `main.go` のエントリポイント | 起動ロジックは統合テストで担保するため |
| テストヘルパー・フィクスチャ | テスト支援コード自体はカバレッジ対象外とするため |
| データベースマイグレーションスクリプト | 統合テストで実行されるが、行カバレッジの対象外とするため |

### カバレッジレポート

- Rust: `ci.yaml` の `coverage-rust` ジョブで JSON + HTML レポートをアーティファクトとしてアップロードする
- 全言語: PR コメントにカバレッジサマリーを自動投稿する（将来計画）

---

## テストデータ戦略

### テストデータの種類

| 種類 | 管理方法 | 用途 |
| --- | --- | --- |
| フィクスチャ | リポジトリ内の JSON / SQL ファイル | 単体テスト・統合テスト |
| ファクトリ | コード内のビルダーパターン | 単体テスト |
| シードデータ | SQL マイグレーション + シードスクリプト | 統合テスト・E2E テスト |
| モック / スタブ | テストコード内で定義 | 外部依存の分離 |

### テストデータの原則

1. **再現可能性**: テストデータは常に同一の初期状態から開始する
2. **独立性**: 各テストケースは他のテストケースに依存しない
3. **最小性**: テストに必要な最小限のデータのみを用意する
4. **現実性**: 本番データに近い形式・制約を持つテストデータを使用する

### Rust でのテストデータパターン

```rust
// ビルダーパターンによるテストデータ生成
#[cfg(test)]
mod tests {
    /// テスト用ユーザーデータを生成するビルダー
    struct UserBuilder {
        name: String,
        email: String,
        role: Role,
    }

    impl UserBuilder {
        fn new() -> Self {
            Self {
                name: "テストユーザー".to_string(),
                email: "test@example.com".to_string(),
                role: Role::User,
            }
        }

        fn with_role(mut self, role: Role) -> Self {
            self.role = role;
            self
        }

        fn build(self) -> User {
            User {
                name: self.name,
                email: self.email,
                role: self.role,
            }
        }
    }
}
```

### 統合テストの DB セットアップ

統合テストでは、テストごとにトランザクションを開始し、テスト終了後にロールバックする。

```rust
// 統合テスト用の DB セットアップ
#[cfg(feature = "test-utils")]
pub async fn setup_test_db() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("DB 接続に失敗");

    // マイグレーションを実行する
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("マイグレーションに失敗");

    pool
}
```

---

## CI 連携

### テスト実行フロー

```
PR 作成 / 更新
    │
    ├─► detect-changes（変更検出）
    │       │
    │       ├─► lint-rust → test-rust（cargo test --all）
    │       ├─► lint-go → test-go（go test ./...）
    │       ├─► lint-ts → test-ts（npm test）
    │       └─► lint-dart → test-dart（flutter test）
    │
    ├─► integration-test.yaml
    │       PostgreSQL + Kafka を起動 → test-utils feature 有効化 → 統合テスト
    │
    ├─► `ci.yaml` の `coverage-rust` ジョブ
    │       cargo-tarpaulin でカバレッジ計測 → アーティファクトアップロード
    │
    └─► security-scan（Trivy）
```

### CI ワークフローとテストの対応

| ワークフロー | テストレベル | 実行条件 |
| --- | --- | --- |
| `ci.yaml` (test-rust) | Rust 単体テスト | PR 時（Rust ファイル変更） |
| `ci.yaml` (test-go) | Go 単体テスト | PR 時（Go ファイル変更） |
| `ci.yaml` (test-ts) | TypeScript 単体テスト | PR 時（TS ファイル変更） |
| `ci.yaml` (test-dart) | Dart 単体テスト | PR 時（Dart ファイル変更） |
| `integration-test.yaml` | Rust 統合テスト | PR 時（system tier Rust ファイル変更） |
| `ci.yaml` (`coverage-rust` ジョブ) | Rust カバレッジ計測 | PR 時（Rust ファイル変更） |
| サービス別 CI | サービス単位の lint + test + build | PR 時（各サービスのファイル変更） |

### テスト失敗時の対応

- **単体テスト失敗**: PR マージをブロックする。開発者が修正する
- **統合テスト失敗**: PR マージをブロックする。テスト環境の問題か実装の問題かを切り分ける
- **カバレッジ未達**: PR コメントで警告する（将来的にマージブロックに変更予定）

---

## テスト命名規約

### 共通ルール

テスト名は **テスト対象_条件_期待結果** の形式で命名する。日本語コメントで目的を補足する。

| 言語 | 命名例 |
| --- | --- |
| Rust | `fn test_create_user_with_valid_input_returns_ok()` |
| Go | `func TestCreateUser_ValidInput_ReturnsOK(t *testing.T)` |
| TypeScript | `it('有効な入力でユーザーを作成する')` |
| Dart | `test('有効な入力でユーザーを作成する')` |

---

## サービス間統合テスト設計

### 概要

Docker Compose ベースのサービス間統合テストにより、複数サービスが連携するフローの正常性を検証する。本番環境の Kubernetes デプロイ前に、ローカルおよび CI 環境で高速にフィードバックを得ることを目的とする。

### テスト用 Docker Compose 構成

テスト専用の `docker-compose.test.yaml` を作成し、テスト実行に必要なサービスのみを起動する。
（設計方針：ファイルは未実装。実際は `docker compose` の `--profile infra` フラグを使用する。）

```yaml
# docker-compose.test.yaml（設計方針・未実装）
# テスト専用のオーバーライド設定
# 使用方法: docker compose -f docker-compose.yaml -f docker-compose.test.yaml --profile infra --profile system up -d
services:
  # テストランナー: テスト完了後に自動終了する
  test-runner:
    build:
      context: ./tests
      dockerfile: Dockerfile
    depends_on:
      auth-rust:
        condition: service_healthy
      config-rust:
        condition: service_healthy
      bff-proxy:
        condition: service_healthy
    environment:
      - BASE_URL=http://bff-proxy:8080
      - AUTH_URL=http://auth-rust:8080
      - KEYCLOAK_URL=http://keycloak:8080
```

### サービス起動順序

サービス間の依存関係に基づき、以下の順序で起動する。`depends_on` の `condition: service_healthy` で起動順序を保証する。

```
1. インフラ層:     PostgreSQL → Redis → Kafka → Keycloak → Vault
2. System 層:      auth → config → saga → その他 system サービス
3. Gateway 層:     graphql-gateway → bff-proxy
4. Business 層:    domain-master
5. テストランナー:  全サービス healthy 後に起動
```

### テストデータのセットアップ方針

| フェーズ | 方法 | 備考 |
| --- | --- | --- |
| DB スキーマ | `infra/docker/init-db/*.sql` | PostgreSQL の初期化スクリプトで自動実行 |
| Keycloak Realm | `infra/docker/keycloak/` の Realm JSON | コンテナ起動時に `--import-realm` で自動インポート |
| テストユーザー | Realm JSON に定義済み | admin / user / order-manager の 3 ロール |
| テストデータ | テストケース内で API 経由で作成 | テスト終了後にクリーンアップ |
| Kafka トピック | `infra/messaging/kafka/create-topics.sh` | kafka-init コンテナで自動作成 |

### CI での実行

`integration-test.yaml` で PostgreSQL + Kafka をサービスコンテナとして起動し、パッケージ単位で並列実行する。サービス間統合テストはサービスコンテナの起動コストが高いため、PR ごとではなく main マージ後に実行することを検討する。

---

## セキュリティテスト

### OWASP Top 10 ベースのセキュリティテストチェックリスト

API サーバーおよび BFF プロキシに対して、以下のセキュリティテストを実施する。

#### A01: アクセス制御の不備

| テスト項目 | 検証内容 | 対象サービス |
| --- | --- | --- |
| 認証バイパス | 未認証リクエストが 401 を返すこと | 全サービス |
| 認可バイパス | 権限不足のリクエストが 403 を返すこと | auth, bff-proxy |
| IDOR | 他ユーザーのリソースにアクセスできないこと | 全 CRUD API |
| JWT 改ざん | 署名が無効な JWT が拒否されること | auth |
| 期限切れトークン | 有効期限切れの JWT が拒否されること | auth |

#### A02: 暗号化の失敗

| テスト項目 | 検証内容 | 対象サービス |
| --- | --- | --- |
| TLS 強制 | HTTP リクエストが HTTPS にリダイレクトされること | Kong / Ingress |
| パスワードハッシュ | Argon2id でハッシュ化されていること | auth |
| シークレット管理 | API レスポンスにシークレットが含まれないこと | vault |

#### A03: インジェクション

| テスト項目 | 検証内容 | 対象サービス |
| --- | --- | --- |
| SQL インジェクション | パラメータ化クエリにより SQL インジェクションが防止されること | 全 DB アクセスサービス |
| XSS | HTML エスケープによりスクリプト挿入が防止されること | bff-proxy, graphql-gateway |
| コマンドインジェクション | シェルコマンド実行入力がサニタイズされること | 全サービス |

#### A05: セキュリティの設定ミス

| テスト項目 | 検証内容 | 対象サービス |
| --- | --- | --- |
| セキュリティヘッダー | `X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security` が設定されていること | bff-proxy, Kong |
| CORS | 許可されたオリジンのみアクセスできること | bff-proxy |
| エラー情報漏洩 | スタックトレースやDB情報がレスポンスに含まれないこと | 全サービス |

#### A08: ソフトウェアとデータの整合性の不備

| テスト項目 | 検証内容 | 対象サービス |
| --- | --- | --- |
| CSRF | CSRF トークンが検証されること | bff-proxy |
| Content-Type 検証 | 不正な Content-Type が拒否されること | 全 API |

#### API レート制限テスト

| テスト項目 | 検証内容 | 対象サービス |
| --- | --- | --- |
| レート超過 | 制限超過時に 429 Too Many Requests が返ること | ratelimit |
| レート回復 | ウィンドウ経過後にリクエストが許可されること | ratelimit |
| IP ベース制限 | 同一 IP からの連続リクエストが制限されること | ratelimit |

### CI でのセキュリティテスト

| ツール | 実行タイミング | 目的 |
| --- | --- | --- |
| Trivy (filesystem) | PR 時 + 日次 | 依存関係の脆弱性スキャン |
| Trivy (image) | 日次 | コンテナイメージの脆弱性スキャン |
| Trivy (IaC) | PR 時 + 日次 | Terraform / K8s マニフェストの構成ミス検出 |
| cargo-audit | PR 時 + 日次 | Rust 依存関係の脆弱性監査 |
| govulncheck | PR 時 + 日次 | Go 依存関係の脆弱性検出 |
| npm audit | PR 時 + 日次 | npm パッケージの脆弱性監査 |
| gitleaks | pre-commit | シークレットのコミット防止 |

---

## 関連ドキュメント

- [E2Eテスト戦略](./e2e-strategy.md) -- E2E テストの詳細設計
- [パフォーマンステスト戦略](./performance-strategy.md) -- パフォーマンステストの詳細設計
- [CI-CD設計.md](../../infrastructure/cicd/CI-CD設計.md) -- CI/CD パイプライン設計
- [コーディング規約.md](../conventions/コーディング規約.md) -- Linter・Formatter 設定
