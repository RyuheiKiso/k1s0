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

## 関連ドキュメント

- [Rust共通実装](Rust共通実装.md)
- [implementation.md](implementation.md)
