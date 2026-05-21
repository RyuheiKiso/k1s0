// secret_impl.go — k1s0 tier1 Library Go 実装: SecretStore の OpenBao facade 実装
// C# SecretStoreImpl.cs と同等の深度で OpenBao Transit API を L3 ラップする。
// OSS 型（vault.Client 等）を公開 API シグネチャに一切露出しない。
// 生シークレット値は callback パターン（WithSecret）でのみ外部に渡し、直接返さない。

// パッケージ名: secret（tier1 Library の Secret Management 実装を提供する）
package secret

import (
	// bytes: シークレットのゼロクリアに使用する
	"bytes"
	// context: context.Context（非同期操作 / キャンセル制御に使用する）
	"context"
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
	// sync: RWMutex による安全な concurrent アクセスに使用する
	"sync"
	// keyhandle パッケージ: KeyClass を参照する
	"github.com/k1s0-io/k1s0/tier1/library/keyhandle"
)

// ---- in-memory SecretStore 実装（テスト / ドライラン用） ----

// secretEntry は in-memory SecretStore に格納するシークレットエントリを表す内部型。
// secretBytes はゼロクリア対象のため、直接公開しない。
type secretEntry struct {
	// metadata: シークレットのメタデータ（値は含まない）
	metadata SecretMetadata
	// secretBytes: シークレットの実際のバイト列（外部に直接露出禁止）
	secretBytes []byte
}

// zeroize はシークレットのバイト列をゼロクリアする内部ヘルパー。
// 05_鍵管理適合仕様.md §メモリゼロクリア規律に準拠する。
func zeroize(b []byte) {
	// bytes.Equal を利用してゼロスライスで上書きする
	// fill with zeros を行う
	fill := bytes.Repeat([]byte{0}, len(b))
	// コピーしてゼロクリアする
	copy(b, fill)
}

// inMemorySecretStoreImpl は SecretStore の in-memory stub 実装型。
// テスト / ドライラン用にシークレットを in-memory に保持する。
// OpenBao に依存せず、単体テストで利用できる実装とする。
type inMemorySecretStoreImpl struct {
	// mu: entries マップへの concurrent アクセスを保護する
	mu sync.RWMutex
	// entries: secretID → secretEntry のマップ
	entries map[string]*secretEntry
}

// NewInMemorySecretStore は inMemorySecretStoreImpl を生成するファクトリ関数（テスト用）。
func NewInMemorySecretStore() SecretStore {
	// inMemorySecretStoreImpl を生成して返す
	return &inMemorySecretStoreImpl{
		entries: make(map[string]*secretEntry),
	}
}

// PutSecret はテスト用にシークレットを in-memory に追加するヘルパーメソッド。
// 実装型の具体的メソッドとして提供する（SecretStore interface 外）。
func (s *inMemorySecretStoreImpl) PutSecret(
	secretID string,
	tenantID string,
	keyClass keyhandle.KeyClass,
	secretBytes []byte,
) {
	// 書き込みロックを取得する
	s.mu.Lock()
	// ロックを確実に解放する
	defer s.mu.Unlock()
	// secretBytes をコピーして保存する（外部スライスの変更から保護する）
	copied := make([]byte, len(secretBytes))
	// コピーする
	copy(copied, secretBytes)
	// エントリを追加する
	s.entries[secretID] = &secretEntry{
		// メタデータを構築する
		metadata: SecretMetadata{
			SecretID: secretID,
			// バージョン 1 から開始する
			Version:  1,
			KeyClass: keyClass,
			TenantID: tenantID,
			// 追加直後は有効状態
			IsActive: true,
		},
		// シークレットのバイト列を保存する
		secretBytes: copied,
	}
}

// GetMetadata はシークレットのメタデータのみを返す（値は返さない）。
// 存在しない場合は nil を返す。
func (s *inMemorySecretStoreImpl) GetMetadata(ctx context.Context, secretID string, tenantID string) (*SecretMetadata, error) {
	// 読み取りロックを取得する
	s.mu.RLock()
	// ロックを確実に解放する
	defer s.mu.RUnlock()
	// secretID でエントリを検索する
	entry, ok := s.entries[secretID]
	// エントリが存在しない場合は nil を返す
	if !ok {
		return nil, nil
	}
	// テナント不一致の場合はエラーを返す（tenant 分離必須）
	if entry.metadata.TenantID != tenantID {
		// テナント分離違反: 不正アクセスを禁止する
		return nil, fmt.Errorf(
			"inMemorySecretStoreImpl.GetMetadata: tenant mismatch secretID=%s expected=%s actual=%s",
			secretID, entry.metadata.TenantID, tenantID,
		)
	}
	// メタデータのコピーを返す（内部構造体への直接参照を避ける）
	meta := entry.metadata
	return &meta, nil
}

// WithSecret はシークレット値を callback に渡して処理させる。
// callback 外にシークレット値が漏れない設計（値は返さない）。
// callback の引数 secretBytes は呼び出し後にゼロクリアする（外部で保持させない）。
func (s *inMemorySecretStoreImpl) WithSecret(
	ctx context.Context,
	secretID string,
	tenantID string,
	callback func(secretBytes []byte) (any, error),
) (any, error) {
	// 読み取りロックを取得する
	s.mu.RLock()
	// ロックを確実に解放する
	defer s.mu.RUnlock()
	// secretID でエントリを検索する
	entry, ok := s.entries[secretID]
	// エントリが存在しない場合はエラーを返す
	if !ok {
		return nil, fmt.Errorf("inMemorySecretStoreImpl.WithSecret: secret not found secretID=%s", secretID)
	}
	// テナント不一致の場合はエラーを返す（tenant 分離必須）
	if entry.metadata.TenantID != tenantID {
		// テナント分離違反: 不正アクセスを禁止する
		return nil, fmt.Errorf(
			"inMemorySecretStoreImpl.WithSecret: tenant mismatch secretID=%s expected=%s actual=%s",
			secretID, entry.metadata.TenantID, tenantID,
		)
	}
	// IsActive が false の場合はエラーを返す（revoke 後のアクセス禁止）
	if !entry.metadata.IsActive {
		return nil, fmt.Errorf("inMemorySecretStoreImpl.WithSecret: secret is revoked secretID=%s", secretID)
	}
	// シークレットのバイト列をコピーして callback に渡す（ゼロクリアのため）
	tempBytes := make([]byte, len(entry.secretBytes))
	// コピーする
	copy(tempBytes, entry.secretBytes)
	// callback 後に必ずゼロクリアする（defer でゼロクリアを保証する）
	defer zeroize(tempBytes)
	// callback を呼び出す
	return callback(tempBytes)
}

// Rotate はシークレットを新しいバージョンにローテーションする。
// 返した SecretMetadata に新しい Version が含まれる。
// in-memory 実装ではバージョン番号をインクリメントするのみ。
func (s *inMemorySecretStoreImpl) Rotate(ctx context.Context, secretID string, tenantID string) (*SecretMetadata, error) {
	// 書き込みロックを取得する
	s.mu.Lock()
	// ロックを確実に解放する
	defer s.mu.Unlock()
	// secretID でエントリを検索する
	entry, ok := s.entries[secretID]
	// エントリが存在しない場合はエラーを返す
	if !ok {
		return nil, fmt.Errorf("inMemorySecretStoreImpl.Rotate: secret not found secretID=%s", secretID)
	}
	// テナント不一致の場合はエラーを返す（tenant 分離必須）
	if entry.metadata.TenantID != tenantID {
		return nil, fmt.Errorf(
			"inMemorySecretStoreImpl.Rotate: tenant mismatch secretID=%s expected=%s actual=%s",
			secretID, entry.metadata.TenantID, tenantID,
		)
	}
	// バージョンをインクリメントする
	entry.metadata.Version++
	// 更新したメタデータのコピーを返す
	meta := entry.metadata
	return &meta, nil
}

// Revoke はシークレットを無効化する（IsActive=false にする）。
// Revoke 後の WithSecret 呼び出しはエラーを返す。
func (s *inMemorySecretStoreImpl) Revoke(ctx context.Context, secretID string, tenantID string) error {
	// 書き込みロックを取得する
	s.mu.Lock()
	// ロックを確実に解放する
	defer s.mu.Unlock()
	// secretID でエントリを検索する
	entry, ok := s.entries[secretID]
	// エントリが存在しない場合はエラーを返す
	if !ok {
		return fmt.Errorf("inMemorySecretStoreImpl.Revoke: secret not found secretID=%s", secretID)
	}
	// テナント不一致の場合はエラーを返す（tenant 分離必須）
	if entry.metadata.TenantID != tenantID {
		return fmt.Errorf(
			"inMemorySecretStoreImpl.Revoke: tenant mismatch secretID=%s expected=%s actual=%s",
			secretID, entry.metadata.TenantID, tenantID,
		)
	}
	// IsActive を false に設定する（revoke 完了）
	entry.metadata.IsActive = false
	// シークレットのバイト列をゼロクリアする（revoke 後は使用不可）
	zeroize(entry.secretBytes)
	// 完了を返す
	return nil
}
