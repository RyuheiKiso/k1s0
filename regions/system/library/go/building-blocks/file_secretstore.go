package buildingblocks

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

// コンパイル時にインターフェース準拠を検証する。
var _ SecretStore = (*FileSecretStore)(nil)

// FileSecretStore はディレクトリ内のファイルからシークレットを読み取る SecretStore 実装。
// ディレクトリ内の各ファイルがシークレットとして扱われ、ファイル名がキー、
// ファイルの内容が値となる。Kubernetes の Secret をボリュームマウントした
// 構成（例: /var/secrets/db-password）と互換性がある。
type FileSecretStore struct {
	// mu は status フィールドへの並行アクセスを保護する読み書きミューテックス。
	mu sync.RWMutex
	// dir はシークレットファイルが格納されているディレクトリのパス。
	dir string
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewFileSecretStore は新しい FileSecretStore を生成して返す。
// dir はシークレットファイルが格納されたディレクトリのパス。
func NewFileSecretStore(dir string) *FileSecretStore {
	return &FileSecretStore{
		dir:    dir,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (s *FileSecretStore) Name() string { return "file-secretstore" }

// Version はコンポーネントのバージョン文字列を返す。
func (s *FileSecretStore) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (s *FileSecretStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (s *FileSecretStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (s *FileSecretStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

// Get は設定されたディレクトリ内の key という名前のファイルからシークレットを読み取る。
// 末尾の改行文字は除去する（Kubernetes のシークレットマウント動作に合わせるため）。
// ファイルが存在しない場合またはファイル読み取りに失敗した場合は ComponentError を返す。
func (s *FileSecretStore) Get(_ context.Context, key string) (*Secret, error) {
	// ディレクトリとキーを結合してファイルパスを生成する。
	path := filepath.Join(s.dir, key)
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, NewComponentError("file-secretstore", "Get",
				fmt.Sprintf("secret file %q not found", path), nil)
		}
		return nil, NewComponentError("file-secretstore", "Get",
			fmt.Sprintf("failed to read secret file %q", path), err)
	}
	// Kubernetes のシークレットマウントと互換性を保つため末尾の改行を除去する。
	value := strings.TrimRight(string(data), "\r\n")
	return &Secret{Key: key, Value: value}, nil
}

// BulkGet は複数のキーに対してファイルからシークレットをまとめて取得する。
// いずれか一つでも取得に失敗した場合は即座にエラーを返す。
func (s *FileSecretStore) BulkGet(ctx context.Context, keys []string) ([]*Secret, error) {
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
