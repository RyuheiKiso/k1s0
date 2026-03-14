package buildingblocks

import (
	"context"
	"fmt"
	"strings"
	"sync"
)

// VaultSecret はk1s0-vault-client の Secret と互換性を持つVaultシークレット構造体。
// Vault から取得したシークレットのパス、データ、バージョン情報を保持する。
type VaultSecret struct {
	// Path はVault上のシークレットパス。
	Path string
	// Data はシークレットのキーと値のマップ。
	Data map[string]string
	// Version はシークレットのバージョン番号。
	Version int64
}

// VaultClientIface はk1s0-vault-client の VaultClient と互換性を持つインターフェース。
// *vaultclient.HttpVaultClient を注入することでこのインターフェースを満たせる。
type VaultClientIface interface {
	// GetSecret は指定したパスのシークレットを Vault から取得する。
	GetSecret(ctx context.Context, path string) (VaultSecret, error)
}

// コンパイル時にインターフェース準拠を検証する。
var _ SecretStore = (*VaultSecretStore)(nil)

// VaultSecretStore は HashiCorp Vault をバックエンドとする SecretStore 実装。
// VaultClientIface をラップして SecretStore インターフェースを提供する。
// Vault シークレットのデータに単一キーしかない場合はその値を直接返す。
// 複数キーの場合は "key=value" 形式を ";" で結合した文字列として返す。
type VaultSecretStore struct {
	// mu は status フィールドへの並行アクセスを保護する読み書きミューテックス。
	mu sync.RWMutex
	// name はコンポーネントの識別子。
	name string
	// client はVaultへのアクセスを担うクライアント実装。
	client VaultClientIface
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewVaultSecretStore は新しい VaultSecretStore を生成して返す。
// name はコンポーネント識別子、client はVaultアクセスを担うクライアント実装。
func NewVaultSecretStore(name string, client VaultClientIface) *VaultSecretStore {
	return &VaultSecretStore{
		name:   name,
		client: client,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (s *VaultSecretStore) Name() string { return s.name }

// Version はコンポーネントのバージョン文字列を返す。
func (s *VaultSecretStore) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (s *VaultSecretStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (s *VaultSecretStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (s *VaultSecretStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

// Get は指定されたパス（key）のシークレットを Vault から取得する。
// シークレットデータが単一キーの場合はその値を直接返し、
// 複数キーの場合は "key=value;key=value" 形式の文字列として返す。
// メタデータとしてシークレットのバージョン番号も返す。
func (s *VaultSecretStore) Get(ctx context.Context, key string) (*Secret, error) {
	vs, err := s.client.GetSecret(ctx, key)
	if err != nil {
		return nil, NewComponentError(s.name, "Get",
			fmt.Sprintf("failed to get secret %q from Vault", key), err)
	}
	var value string
	if len(vs.Data) == 1 {
		// 単一キーの場合は値をそのまま使用する。
		for _, v := range vs.Data {
			value = v
		}
	} else {
		// 複数キーの場合は "key=value" ペアを ";" で結合して返す。
		parts := make([]string, 0, len(vs.Data))
		for k, v := range vs.Data {
			parts = append(parts, k+"="+v)
		}
		value = strings.Join(parts, ";")
	}
	return &Secret{
		Key:   key,
		Value: value,
		// バージョン情報をメタデータとして格納する。
		Metadata: map[string]string{
			"version": fmt.Sprintf("%d", vs.Version),
		},
	}, nil
}

// BulkGet は複数のパスに対して Vault からシークレットをまとめて取得する。
// いずれか一つでも取得に失敗した場合は即座にエラーを返す。
func (s *VaultSecretStore) BulkGet(ctx context.Context, keys []string) ([]*Secret, error) {
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
