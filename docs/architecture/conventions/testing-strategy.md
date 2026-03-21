# テスト戦略・モック統一ガイドライン

k1s0 プロジェクト全体のテスト戦略と、言語別のモック実装方針を定義する。
本ドキュメントはコーディング規約（[コーディング規約.md](コーディング規約.md)）と合わせて参照すること。

---

## テストピラミッド方針

```
          /\
         /E2E\        比率: 少（5%）
        /------\
       /Integra-\     比率: 中（25%）
      /  tion    \
     /------------\
    /    Unit      \  比率: 多（70%）
   /--------------/
```

| レベル | 目的 | 実行タイミング |
|--------|------|----------------|
| Unit | ドメインロジック・ユースケースの正確性検証 | PR ごとに全実行 |
| Integration | DB・Kafka・gRPC などの外部依存を含む動作検証 | PR ごとに実行（`--include-ignored` 指定） |
| E2E | ユーザーシナリオ全体の動作検証（Playwright） | リリースブランチマージ時 |

**原則**: ユニットテストを最大化し、インテグレーションテストは外部依存が絡む境界のみを対象とする。
E2E テストは本番相当環境でのスモークテスト相当に限定し、テスト数を最小化する。

---

## カバレッジ要件

| 言語 | 最低カバレッジ | 計測方法 |
|------|---------------|---------|
| Rust | 70% | `cargo llvm-cov` |
| Go | 70% | `go test -coverprofile` |
| TypeScript | 80% | `vitest --coverage` / `jest --coverage` |
| Dart | 80% | `flutter test --coverage` |

カバレッジはドメイン層・ユースケース層を重点的に計測する。
インフラ層（DB アダプタ、gRPC クライアント等）はモックによるユニットテストで補完する。

---

## Rust モック戦略

### 推奨ライブラリ: `mockall::automock`

```toml
# Cargo.toml の [dev-dependencies] に追加
mockall = "0.13"
```

#### 使用方針

- **`#[automock]` を推奨** — トレイトに `#[automock]` を付与し、`MockXxx` を自動生成する。
  手動での `struct MockXxx` 実装は禁止（重複・保守性の低下を防ぐ）。
- **`async_trait` との混在を避ける** — `async_trait` と `mockall` を組み合わせる場合は
  `#[async_trait]` を `#[automock]` の後に記述し、順序を固定する。
- **内部ロジックはモック不要** — ドメインサービス・値オブジェクト・ユースケースの純粋関数は
  実装をそのままテストする。モック対象は外部依存（DB, Kafka, gRPC）のみ。

#### コード例

```rust
// リポジトリトレイトに automock を付与する
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<User>;
}

// テスト内での利用例
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn ユーザー取得が成功する() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .with(eq("user-1"))
            .times(1)
            .returning(|_| Ok(User { id: "user-1".to_string(), ..Default::default() }));

        let uc = GetUserUseCase::new(Arc::new(mock));
        let result = uc.execute("user-1").await;
        assert!(result.is_ok());
    }
}
```

#### 禁止パターン

```rust
// NG: 手動での Mock struct 実装（automock で代替する）
struct ManualMockUserRepository;
impl UserRepository for ManualMockUserRepository { ... }

// NG: async_trait の前に automock を付与しない（順序を固定する）
#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock)]  // 逆順は不可
pub trait UserRepository { ... }
```

---

## Go モック戦略

### 推奨ライブラリ: `github.com/stretchr/testify/mock`

- インターフェースに対する手動 Mock 実装（`testify/mock` の埋め込み）を採用する。
- `mockgen`（gomock）は生成ファイルの管理コストを考慮し、新規コードでは使用しない。
  ただし既存コードで使用済みの場合はそのまま継続する。

#### コード例

```go
// リポジトリのモック実装
type MockUserRepository struct {
    mock.Mock
}

func (m *MockUserRepository) FindByID(ctx context.Context, id string) (*User, error) {
    args := m.Called(ctx, id)
    return args.Get(0).(*User), args.Error(1)
}

// テスト内での利用例
func TestGetUser_Success(t *testing.T) {
    repo := new(MockUserRepository)
    repo.On("FindByID", mock.Anything, "user-1").Return(&User{ID: "user-1"}, nil)

    uc := NewGetUserUseCase(repo)
    user, err := uc.Execute(context.Background(), "user-1")
    assert.NoError(t, err)
    assert.Equal(t, "user-1", user.ID)
}
```

---

## TypeScript / Dart モック戦略

### TypeScript: `vi.fn()` / `jest.fn()`

- Vitest では `vi.fn()` でモック関数を生成する。
- 型安全を確保するために `as jest.Mock` / `as MockedFunction<T>` で型アサーションを付与する。

### Dart: `mocktail` / `mockito`

- `mocktail` を推奨（`build_runner` 不要でシンプル）。
- `when(() => mock.method()).thenAnswer(...)` パターンで記述する。

---

## `#[ignore]` 属性の使用基準（Rust）

### 付与必須の対象

以下のテストには全て `#[ignore]` を付与する:

| 対象 | 理由 |
|------|------|
| 実 PostgreSQL / MySQL への接続テスト | CI 環境で DB が常時起動しているとは限らない |
| 実 Kafka ブローカーへの接続テスト | CI 環境で Kafka が常時起動しているとは限らない |
| 実 Redis への接続テスト | 同上 |
| 外部 HTTP エンドポイントへの接続テスト | ネットワーク依存でフレーキーになりやすい |
| 処理時間が 1 秒を超えるテスト | CI のレスポンスタイムを悪化させる |

#### コード例

```rust
// 実 DB テストには #[ignore] を必ず付与する
#[tokio::test]
#[ignore = "実 PostgreSQL が必要（CI では --include-ignored で実行）"]
async fn ユーザーをDBに保存して取得できる() {
    // 実装...
}
```

### CI での実行方法

```yaml
# .github/workflows/_test.yaml（統合テスト実行ステップ）
- name: Run integration tests (with DB)
  run: cargo test --test '*' -- --include-ignored
  env:
    DATABASE_URL: postgres://postgres:password@localhost:5432/testdb
```

### 付与不要の対象

- モックを使用したユニットテスト（インメモリ実装を使うテストを含む）
- `tempfile::tempdir()` を使うファイル I/O テスト
- 決定論的なロジックのみを検証するテスト

---

## モック設計原則

### 外部依存はモック化する

| 依存種別 | モック対象 | 備考 |
|----------|-----------|------|
| PostgreSQL / MySQL | Repository トレイト/インターフェース | インメモリ実装または mockall |
| Kafka Producer/Consumer | Producer トレイト/インターフェース | インメモリ実装または mockall |
| gRPC クライアント | クライアントトレイト/インターフェース | mockall の automock |
| HTTP 外部 API | クライアントトレイト/インターフェース | mockall の automock |
| Redis | キャッシュトレイト/インターフェース | インメモリ HashMap 実装 |

### 内部ロジックはモック不要

- ドメインエンティティ（値オブジェクト・集約）
- ドメインサービス（純粋関数）
- ユースケース（外部依存をコンストラクタインジェクションで差し替え済みの場合）

### テストダブルの優先順位

1. **Fake（インメモリ実装）** — シンプルで可読性が高い。Repository のインメモリ版など
2. **Mock（`mockall` / `testify/mock`）** — 呼び出し回数・引数の検証が必要な場合
3. **Stub（固定値を返す実装）** — 呼び出し検証不要で戻り値だけ固定したい場合

---

## テスト命名規約

### Rust

```rust
// 日本語で「何が・どうなる」を表現する
#[test]
fn 有効なトークンであれば検証に成功する() { ... }

#[test]
fn 期限切れトークンであれば検証エラーを返す() { ... }
```

### Go

```go
// TestXxx_Condition_Expected パターンを使用する
func TestValidateToken_ValidToken_ReturnsSuccess(t *testing.T) { ... }
func TestValidateToken_ExpiredToken_ReturnsError(t *testing.T) { ... }
```

### TypeScript / Dart

```typescript
// describe + it/test のネスト構造を使用する
describe("validateToken", () => {
  it("有効なトークンであれば成功する", () => { ... });
  it("期限切れトークンであればエラーを返す", () => { ... });
});
```

---

## 関連ドキュメント

- [コーディング規約.md](コーディング規約.md)
- [エラーハンドリング方針.md](エラーハンドリング方針.md)
- [docs/infrastructure/rate-limiting-strategy.md](../../infrastructure/rate-limiting-strategy.md)
