package buildingblocks

import (
	"context"
	"fmt"
	"os"
	"sync"
)

// コンパイル時にインターフェース準拠を検証する。
var _ SecretStore = (*EnvSecretStore)(nil)

// EnvSecretStore は環境変数からシークレットを読み取る SecretStore 実装。
// キーに設定済みのプレフィックスを結合して環境変数名を生成する。
// 例: prefix="APP_", key="DB_PASSWORD" → 環境変数 APP_DB_PASSWORD を参照する。
type EnvSecretStore struct {
	// mu は status フィールドへの並行アクセスを保護する読み書きミューテックス。
	mu sync.RWMutex
	// prefix は環境変数名の先頭に付与する文字列（例: "APP_"）。
	prefix string
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewEnvSecretStore は新しい EnvSecretStore を生成して返す。
// prefix は各キーの環境変数参照時に先頭へ付与する文字列（例: "APP_"）。
func NewEnvSecretStore(prefix string) *EnvSecretStore {
	return &EnvSecretStore{
		prefix: prefix,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (s *EnvSecretStore) Name() string { return "env-secretstore" }

// Version はコンポーネントのバージョン文字列を返す。
func (s *EnvSecretStore) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (s *EnvSecretStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (s *EnvSecretStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (s *EnvSecretStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

// Get は prefix+key の名前で環境変数を参照してシークレットを取得する。
// 環境変数が存在しない場合は ComponentError を返す。
func (s *EnvSecretStore) Get(_ context.Context, key string) (*Secret, error) {
	// プレフィックスとキーを結合して実際の環境変数名を生成する。
	envKey := s.prefix + key
	value, ok := os.LookupEnv(envKey)
	if !ok {
		return nil, NewComponentError("env-secretstore", "Get",
			fmt.Sprintf("environment variable %q not found", envKey), nil)
	}
	return &Secret{Key: key, Value: value}, nil
}

// BulkGet は複数のキーに対して環境変数からシークレットをまとめて取得する。
// いずれか一つでも取得に失敗した場合は即座にエラーを返す。
func (s *EnvSecretStore) BulkGet(ctx context.Context, keys []string) ([]*Secret, error) {
	results := make([]*Secret, 0, len(keys))
	for _, key := range keys {
		secret, err := s.Get(ctx, key)
		if err != nil {
			return nil, err
		}
		results = append(results, secret)
	}
	return results, nil
}
