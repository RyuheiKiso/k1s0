# system-server Go 共通実装リファレンス

system tier の Go サーバー（BFF Proxy）で共通する実装パターンを定義する。

---

## エラーハンドリングパターン

### エラーを黙殺しない

Go では `_ = expr` によるエラー黙殺を禁止する。全てのエラーは適切にハンドリングすること。

```go
// NG: エラーを黙殺
_ = store.Touch(ctx, sessionID, ttl)

// OK: ログ出力でエラーを記録
if err := store.Touch(ctx, sessionID, ttl); err != nil {
    slog.Warn("セッション TTL 延長に失敗", "session_id", sessionID, "error", err)
}
```

### 型アサーションの安全パターン

インターフェースからの型アサーションは必ず comma-ok パターンを使用する。

```go
// NG: パニックの可能性
cid := val.(string)

// OK: comma-ok パターンで安全に取得
cid, ok := val.(string)
if !ok {
    slog.Warn("型アサーション失敗", "key", "correlation_id")
    return
}
```

### ログライブラリ

全 Go サーバーで `log/slog` を一貫して使用する。サードパーティログライブラリは使用しない。

---

## 構造化ログ

```go
slog.Info("リクエスト処理完了",
    "method", r.Method,
    "path", r.URL.Path,
    "status", status,
    "duration_ms", elapsed.Milliseconds(),
)
```

---

## gRPC サービス定義パターン（m-16 対応）

### サービス定義とハンドラー実装

```go
// server/grpc/handler.go — gRPC サービスハンドラーの基本構造
// pb はコンパイル済みの protobuf パッケージを指す
type GrpcHandler struct {
    pb.UnimplementedServiceServer // 未実装メソッドのデフォルト実装を提供
    svc service.Service
}

// NewGrpcHandler はハンドラーを生成し、依存性を注入する
func NewGrpcHandler(svc service.Service) *GrpcHandler {
    return &GrpcHandler{svc: svc}
}

// RPC メソッドの実装パターン（エラーは gRPC Status に変換する）
func (h *GrpcHandler) GetItem(ctx context.Context, req *pb.GetItemRequest) (*pb.GetItemResponse, error) {
    item, err := h.svc.GetItem(ctx, req.GetId())
    if err != nil {
        // ドメインエラーを gRPC Status コードに変換する
        return nil, toGrpcStatus(err)
    }
    return &pb.GetItemResponse{Item: toProto(item)}, nil
}

// toGrpcStatus はドメインエラーを gRPC Status に変換するヘルパー
func toGrpcStatus(err error) error {
    var notFoundErr *domain.NotFoundError
    if errors.As(err, &notFoundErr) {
        return status.Errorf(codes.NotFound, notFoundErr.Error())
    }
    return status.Errorf(codes.Internal, "内部エラーが発生しました")
}
```

### gRPC サーバー起動パターン

```go
// main.go — gRPC サーバーの起動パターン
grpcServer := grpc.NewServer(
    grpc.ChainUnaryInterceptor(
        // 相関 ID を gRPC メタデータから取得・伝播する
        correlation.UnaryServerInterceptor(),
        // 認証インターセプター（JWT 検証）
        auth.UnaryServerInterceptor(verifier),
    ),
)
pb.RegisterServiceServer(grpcServer, grpcHandler)
reflection.Register(grpcServer) // gRPC リフレクション（開発環境用）

lis, err := net.Listen("tcp", fmt.Sprintf(":%d", cfg.GRPCPort))
if err != nil {
    slog.Error("gRPC リスナーの起動に失敗", "error", err)
    os.Exit(1)
}
if err := grpcServer.Serve(lis); err != nil {
    slog.Error("gRPC サーバーエラー", "error", err)
}
```

---

## リポジトリパターンの実装例

### インターフェース定義（domain 層）

```go
// domain/repository/item_repository.go — リポジトリインターフェース
// ドメイン層はインフラの実装詳細を知らない（依存性逆転の原則）
type ItemRepository interface {
    FindByID(ctx context.Context, id string) (*entity.Item, error)
    List(ctx context.Context, filter Filter) ([]*entity.Item, error)
    Create(ctx context.Context, item *entity.Item) error
    Update(ctx context.Context, item *entity.Item) error
    Delete(ctx context.Context, id string) error
}
```

### PostgreSQL 実装（infrastructure 層）

```go
// infrastructure/postgres/item_repository.go — PostgreSQL 実装
type itemPostgresRepository struct {
    db *sqlx.DB
}

// NewItemPostgresRepository はリポジトリを生成し、DB 接続を注入する
func NewItemPostgresRepository(db *sqlx.DB) domain.ItemRepository {
    return &itemPostgresRepository{db: db}
}

// FindByID は ID でアイテムを検索し、存在しない場合は NotFoundError を返す
func (r *itemPostgresRepository) FindByID(ctx context.Context, id string) (*entity.Item, error) {
    var row itemRow
    err := r.db.GetContext(ctx, &row, "SELECT * FROM items WHERE id = $1", id)
    if errors.Is(err, sql.ErrNoRows) {
        // ドメイン定義の NotFoundError に変換する（HTTP ステータスの決定はアダプター層で行う）
        return nil, &domain.NotFoundError{ID: id}
    }
    if err != nil {
        return nil, fmt.Errorf("items テーブルの検索に失敗: %w", err)
    }
    return row.toEntity(), nil
}
```

---

## テストパターン（モック・テーブルドリブン）

### インターフェースモック生成

Go のモックは `testify/mock` または `gomock` を使用する。本プロジェクトでは `testify/mock` を標準とする。

```go
// domain/repository/mock/item_repository_mock.go — testify/mock による自動生成
// go generate ./... で更新する
type MockItemRepository struct {
    mock.Mock
}

func (m *MockItemRepository) FindByID(ctx context.Context, id string) (*entity.Item, error) {
    args := m.Called(ctx, id)
    if args.Get(0) == nil {
        return nil, args.Error(1)
    }
    return args.Get(0).(*entity.Item), args.Error(1)
}
```

### テーブルドリブンテスト

```go
// usecase/item_usecase_test.go — テーブルドリブンテストパターン
func TestGetItemUseCase(t *testing.T) {
    // テストケースをテーブルで定義し、共通のセットアップ/検証コードを再利用する
    tests := []struct {
        name      string
        id        string
        mockSetup func(repo *mock.MockItemRepository)
        wantErr   bool
    }{
        {
            name: "正常系: アイテムが取得できる",
            id:   "item-001",
            mockSetup: func(repo *mock.MockItemRepository) {
                repo.On("FindByID", mock.Anything, "item-001").
                    Return(&entity.Item{ID: "item-001"}, nil)
            },
            wantErr: false,
        },
        {
            name: "異常系: アイテムが存在しない",
            id:   "not-exists",
            mockSetup: func(repo *mock.MockItemRepository) {
                repo.On("FindByID", mock.Anything, "not-exists").
                    Return(nil, &domain.NotFoundError{ID: "not-exists"})
            },
            wantErr: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            repo := new(mock.MockItemRepository)
            tt.mockSetup(repo)
            uc := usecase.NewGetItemUseCase(repo)

            _, err := uc.Execute(context.Background(), tt.id)
            if tt.wantErr {
                require.Error(t, err)
            } else {
                require.NoError(t, err)
            }
            repo.AssertExpectations(t)
        })
    }
}
```

---

## 共通ミドルウェアの使用方法

### net/http ミドルウェアチェーン

```go
// adapter/handler/router.go — HTTP ミドルウェアの適用順序
// 外側から順に: 相関ID → 認証 → RBAC → ハンドラー
func NewRouter(handler *Handler, verifier auth.Verifier) http.Handler {
    mux := http.NewServeMux()
    // ルート登録
    mux.HandleFunc("GET /api/v1/items", handler.ListItems)
    mux.HandleFunc("GET /api/v1/items/{id}", handler.GetItem)

    // ミドルウェアを外側から適用する（最初に実行されるものが最も外側）
    return middleware.Chain(mux,
        middleware.Correlation(), // 相関 ID の付与・伝播（最外層）
        middleware.Auth(verifier), // JWT 認証
        middleware.Logging(),      // アクセスログ記録
    )
}

// middleware/chain.go — ミドルウェアチェーン構築ヘルパー
func Chain(h http.Handler, middlewares ...func(http.Handler) http.Handler) http.Handler {
    for i := len(middlewares) - 1; i >= 0; i-- {
        h = middlewares[i](h)
    }
    return h
}
```

### 相関 ID ミドルウェアの使用

```go
// k1s0-correlation ライブラリを使用して X-Correlation-Id を自動付与する
import "github.com/k1s0/system/library/go/correlation"

// リクエストから相関 ID を取得する
cid := correlation.FromContext(ctx)
slog.InfoContext(ctx, "処理開始", "correlation_id", cid)
```

---

## 関連ドキュメント

- [Rust共通実装](Rust共通実装.md)
- [implementation.md](implementation.md)
