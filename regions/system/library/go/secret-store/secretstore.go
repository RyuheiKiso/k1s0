// secretstore パッケージは SecretStore インターフェースと複数のバックエンド実装を提供する。
// InMemory（テスト用）、環境変数、ファイル、HashiCorp Vault の4種類をサポートする。
// building-blocks パッケージから移植した独立モジュール版。
package secretstore

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

// ErrPathTraversal はパストラバーサル攻撃を試みたキーが指定された場合に返すエラー。
// "../" や絶対パスなど、基底ディレクトリ外を参照するキーを拒否するために使用する。
var ErrPathTraversal = errors.New("path traversal detected")

// ComponentStatus はコンポーネントの現在の状態を表す文字列型。
type ComponentStatus string

// コンポーネントの状態を表す定数群。
const (
	// StatusUninitialized は初期化前の状態を示す。
	StatusUninitialized ComponentStatus = "uninitialized"
	// StatusReady は正常に動作可能な状態を示す。
	StatusReady ComponentStatus = "ready"
	// StatusDegraded は一部機能が低下している状態を示す。
	StatusDegraded ComponentStatus = "degraded"
	// StatusClosed はシャットダウン済みの状態を示す。
	StatusClosed ComponentStatus = "closed"
	// StatusError はエラーが発生している状態を示す。
	StatusError ComponentStatus = "error"
)

// Metadata はコンポーネントのメタデータを保持する構造体。
type Metadata struct {
	// Name はコンポーネント名。
	Name string `json:"name"`
	// Version はコンポーネントのバージョン。
	Version string `json:"version"`
	// Tags は追加タグ情報のマップ（省略可）。
	Tags map[string]string `json:"tags,omitempty"`
}

// Component はすべてのビルディングブロックコンポーネントの基底インターフェース。
type Component interface {
	// Name はコンポーネント識別子を返す。
	Name() string
	// Version はコンポーネントのバージョン文字列を返す。
	Version() string
	// Init はコンポーネントを初期化する。
	Init(ctx context.Context, metadata Metadata) error
	// Close はコンポーネントを終了する。
	Close(ctx context.Context) error
	// Status はコンポーネントの現在のステータスを返す。
	Status(ctx context.Context) ComponentStatus
}

// ComponentError はビルディングブロック操作からのエラーを表す構造体。
type ComponentError struct {
	// Component はエラーが発生したコンポーネント名。
	Component string
	// Operation はエラーが発生した操作名。
	Operation string
	// Message はエラーの説明メッセージ。
	Message string
	// Err は元となるエラー（省略可）。
	Err error
}

// Error はエラーメッセージを文字列として返す。
func (e *ComponentError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("[%s] %s: %s: %v", e.Component, e.Operation, e.Message, e.Err)
	}
	return fmt.Sprintf("[%s] %s: %s", e.Component, e.Operation, e.Message)
}

// Unwrap は元となるエラーを返す（errors.Is/As のための実装）。
func (e *ComponentError) Unwrap() error {
	return e.Err
}

// NewComponentError は新しい ComponentError を生成して返す。
func NewComponentError(component, operation, message string, err error) *ComponentError {
	return &ComponentError{Component: component, Operation: operation, Message: message, Err: err}
}

// Secret は取得したシークレットを表す構造体。
type Secret struct {
	// Key はシークレットのキー名。
	Key string `json:"key"`
	// Value はシークレットの値。
	Value string `json:"value"`
	// Metadata はシークレットに付随するメタデータ（省略可）。
	Metadata map[string]string `json:"metadata,omitempty"`
}

// SecretStore はシークレット取得の統合インターフェース。
// Component インターフェースを継承し、単一・複数のシークレット取得をサポートする。
type SecretStore interface {
	Component
	// Get は指定したキーのシークレットを取得する。
	Get(ctx context.Context, key string) (*Secret, error)
	// BulkGet は複数のキーのシークレットをまとめて取得する。
	BulkGet(ctx context.Context, keys []string) ([]*Secret, error)
	// Close はコンポーネントを終了する。
	Close(ctx context.Context) error
}

// コンパイル時にインターフェース準拠を検証する。
var _ SecretStore = (*InMemorySecretStore)(nil)
var _ SecretStore = (*EnvSecretStore)(nil)
var _ SecretStore = (*FileSecretStore)(nil)
var _ SecretStore = (*VaultSecretStore)(nil)

// ─────────────────────────────────────────
// InMemorySecretStore
// ─────────────────────────────────────────

// InMemorySecretStore はテスト用のインメモリ SecretStore 実装。
// スレッドセーフな読み書きミューテックスで secrets マップを保護する。
type InMemorySecretStore struct {
	// mu は secrets と status フィールドへの並行アクセスを保護する。
	mu sync.RWMutex
	// secrets はキーとシークレットのマップ。
	secrets map[string]*Secret
	// status はコンポーネントの現在の状態。
	status ComponentStatus
}

// NewInMemorySecretStore は新しい InMemorySecretStore を生成して返す。
func NewInMemorySecretStore() *InMemorySecretStore {
	return &InMemorySecretStore{
		secrets: make(map[string]*Secret),
		status:  StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (s *InMemorySecretStore) Name() string { return "inmemory-secretstore" }

// Version はコンポーネントのバージョン文字列を返す。
func (s *InMemorySecretStore) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (s *InMemorySecretStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (s *InMemorySecretStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (s *InMemorySecretStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

// Get はキーに対応するシークレットをインメモリマップから取得する。
// 未登録のキーを指定した場合は ComponentError を返す。
func (s *InMemorySecretStore) Get(_ context.Context, key string) (*Secret, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	secret, ok := s.secrets[key]
	if !ok {
		return nil, NewComponentError("inmemory-secretstore", "Get", fmt.Sprintf("secret %q not found", key), nil)
	}
	return secret, nil
}

// BulkGet は複数のキーに対してシークレットをまとめて取得する。
// いずれか一つでも取得に失敗した場合は即座にエラーを返す。
func (s *InMemorySecretStore) BulkGet(ctx context.Context, keys []string) ([]*Secret, error) {
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

// Put はシークレットをインメモリマップに格納する。
// SecretStore インターフェースには含まれないテスト用のヘルパーメソッド。
func (s *InMemorySecretStore) Put(key, value string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.secrets[key] = &Secret{Key: key, Value: value}
}

// ─────────────────────────────────────────
// EnvSecretStore
// ─────────────────────────────────────────

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

// ─────────────────────────────────────────
// FileSecretStore
// ─────────────────────────────────────────

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
// パストラバーサル攻撃（"../../etc/passwd" 等）を試みるキーはエラーを返す。
func (s *FileSecretStore) Get(_ context.Context, key string) (*Secret, error) {
	// パストラバーサル防止: filepath.Abs で解決した後のパスが基底ディレクトリ配下にあることを確認する。
	// "../" や絶対パスなど基底ディレクトリ外を参照するキーは ErrPathTraversal で拒否する。
	absDir, err := filepath.Abs(s.dir)
	if err != nil {
		return nil, NewComponentError("file-secretstore", "Get",
			"failed to resolve base directory path", err)
	}
	absPath, err := filepath.Abs(filepath.Join(s.dir, key))
	if err != nil {
		return nil, NewComponentError("file-secretstore", "Get",
			"failed to resolve secret file path", err)
	}
	// ディレクトリセパレータを付加して "absDir" が "absDirExtra" 等にも一致しないよう境界を正確に検証する。
	if !strings.HasPrefix(absPath, absDir+string(filepath.Separator)) && absPath != absDir {
		return nil, NewComponentError("file-secretstore", "Get",
			fmt.Sprintf("invalid key %q", key), ErrPathTraversal)
	}

	// ディレクトリとキーを結合してファイルパスを生成する。
	data, err := os.ReadFile(absPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, NewComponentError("file-secretstore", "Get",
				fmt.Sprintf("secret file %q not found", absPath), nil)
		}
		return nil, NewComponentError("file-secretstore", "Get",
			fmt.Sprintf("failed to read secret file %q", absPath), err)
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

// ─────────────────────────────────────────
// VaultSecretStore
// ─────────────────────────────────────────

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
