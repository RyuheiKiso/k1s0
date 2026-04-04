// noop_store.go は Redis 接続不可時に使用するno-opセッションストアを提供する。
// ALLOW_REDIS_SKIP=true かつ dev 環境で Redis に接続できない場合、
// broken な redis.Client を下流に渡すことによる panic を防止するために使用する（H-002 監査対応）。
// 全操作は成功を返すが実際にはデータを永続化しない。
// 本番環境では使用しないこと。
package session

import (
	"context"
	"time"
)

// NoOpStore は全操作が no-op の FullStore 実装。
// ALLOW_REDIS_SKIP=true かつ dev 環境で Redis 接続不可の場合に使用し、
// broken な redis クライアントが下流に渡ることを防ぐ（H-002 監査対応）。
// セッションデータは保持されないため、ログイン状態は維持されない。
type NoOpStore struct{}

// NewNoOpStore は NoOpStore を生成する。
// Redis 接続をスキップする場合（ALLOW_REDIS_SKIP=true, dev 環境）にのみ使用すること。
func NewNoOpStore() *NoOpStore {
	return &NoOpStore{}
}

// Create はセッションの作成を no-op で処理し、固定のダミーセッション ID を返す。
func (n *NoOpStore) Create(_ context.Context, _ *SessionData, _ time.Duration) (string, error) {
	// データを永続化しないため、ダミー ID を返す
	return "noop-session", nil
}

// Get はセッションの取得を no-op で処理し、常に nil を返す。
func (n *NoOpStore) Get(_ context.Context, _ string) (*SessionData, error) {
	// 常に未発見として扱う
	return nil, nil
}

// Update はセッションの更新を no-op で処理し、常に成功を返す。
func (n *NoOpStore) Update(_ context.Context, _ string, _ *SessionData, _ time.Duration) error {
	return nil
}

// Delete はセッションの削除を no-op で処理し、常に成功を返す。
func (n *NoOpStore) Delete(_ context.Context, _ string) error {
	return nil
}

// Touch はセッション TTL の延長を no-op で処理し、常に成功を返す。
func (n *NoOpStore) Touch(_ context.Context, _ string, _ time.Duration) error {
	return nil
}

// CreateExchangeCode は交換コードの作成を no-op で処理し、固定のダミーコードを返す。
func (n *NoOpStore) CreateExchangeCode(_ context.Context, _ *ExchangeCodeData, _ time.Duration) (string, error) {
	// データを永続化しないため、ダミーコードを返す
	return "noop-exchange-code", nil
}

// GetExchangeCode は交換コードの取得を no-op で処理し、常に nil を返す。
func (n *NoOpStore) GetExchangeCode(_ context.Context, _ string) (*ExchangeCodeData, error) {
	// 常に未発見として扱う
	return nil, nil
}

// DeleteExchangeCode は交換コードの削除を no-op で処理し、常に成功を返す。
func (n *NoOpStore) DeleteExchangeCode(_ context.Context, _ string) error {
	return nil
}
