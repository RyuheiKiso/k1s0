package cache

import (
	"context"
	"fmt"
	"time"
)

// CacheClient はキャッシュクライアントのインターフェース。
type CacheClient interface {
	// Get はキーに対応する値を取得する。見つからない場合は nil を返す。
	Get(ctx context.Context, key string) (*string, error)
	// Set はキーと値をキャッシュに格納する。ttl が nil の場合は無期限。
	Set(ctx context.Context, key string, value string, ttl *time.Duration) error
	// Delete はキーを削除する。削除した場合 true を返す。
	Delete(ctx context.Context, key string) (bool, error)
	// Exists はキーが存在するかどうかを返す。
	Exists(ctx context.Context, key string) (bool, error)
	// SetNX はキーが存在しない場合のみ値をセットする。セットできた場合 true を返す。
	SetNX(ctx context.Context, key string, value string, ttl time.Duration) (bool, error)
	// Expire はキーの有効期限を更新する。キーが存在しない場合 false を返す。
	Expire(ctx context.Context, key string, ttl time.Duration) (bool, error)
}

// CacheError はキャッシュ操作のエラー。
type CacheError struct {
	Code    string
	Message string
}

func (e *CacheError) Error() string {
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// NewNotFoundError はキーが見つからないエラーを生成する。
func NewNotFoundError(key string) *CacheError {
	return &CacheError{Code: "NOT_FOUND", Message: fmt.Sprintf("キャッシュキーが見つかりません: %s", key)}
}

// NewConnectionError は接続エラーを生成する。
func NewConnectionError(msg string) *CacheError {
	return &CacheError{Code: "CONNECTION_ERROR", Message: msg}
}
