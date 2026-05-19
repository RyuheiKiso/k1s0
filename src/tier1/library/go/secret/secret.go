// secret.go — k1s0 tier1 Library Go 実装: Secret Management L3 facade interface
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および §SecretStore 抽象 に準拠する。
// Rust core/secret.rs の SecretStore trait と 4 言語等価強度を保つ。
// OpenBao / AWS Secrets Manager 等のバックエンドを実装で切り替えられる interface を宣言する。
// 公開 API シグネチャに生 key bytes / 生シークレット値を露出しない。

// パッケージ名: secret（tier1 Library の Secret Management 公開 API を提供する）
package secret

import (
	// context: context.Context（SecretStore の非同期操作に使用する）
	"context"
	// keyhandle パッケージ: KeyClass を参照する
	"github.com/k1s0-io/k1s0/tier1/library/keyhandle"
)

// SecretMetadata はシークレットのメタデータを宣言する型。
// 生のシークレット値は含まない（SecretStore.WithSecret の callback パターンで別管理する）。
// Rust core/secret.rs の SecretMetadata struct と 1:1 対応する。
type SecretMetadata struct {
	// SecretID: シークレットの識別子（UUID v7 形式）
	SecretID string
	// Version: シークレットのバージョン番号（ローテーション追跡用）
	Version uint32
	// KeyClass: このシークレットを保護している鍵のクラス（05_鍵管理適合仕様 §v1 5 class）
	KeyClass keyhandle.KeyClass
	// TenantID: シークレットが属するテナントの識別子
	TenantID string
	// IsActive: シークレットが有効かどうか（revoke / rotate 後 false になる）
	IsActive bool
}

// SecretRotationKind はシークレットローテーション方式を宣言する型。
// Rust core/secret.rs の SecretRotationPolicy enum と 4 言語等価強度を保つ。
type SecretRotationKind string

const (
	// SecretRotationManual: 手動ローテーション（自動ローテーションなし）
	SecretRotationManual SecretRotationKind = "manual"
	// SecretRotationOnDemand: 要求時ローテーション（呼び出し元が Rotate を呼ぶ）
	SecretRotationOnDemand SecretRotationKind = "on_demand"
	// SecretRotationScheduled: スケジュールローテーション（間隔秒数で指定）
	SecretRotationScheduled SecretRotationKind = "scheduled"
)

// SecretRotationPolicy はシークレットローテーションポリシーを宣言する型。
// Kind で方式を、IntervalSeconds で Scheduled 時の間隔秒数を保持する。
type SecretRotationPolicy struct {
	// Kind: ローテーション方式（manual / on_demand / scheduled）
	Kind SecretRotationKind
	// IntervalSeconds: Scheduled 時のローテーション間隔秒数（Kind=Scheduled のみ使用する）
	IntervalSeconds int64
}

// SecretStore は Secret Management の L3 抽象 interface。
// OpenBao / AWS Secrets Manager 等の OSS / サービスを実装で切り替えられる。
// Rust core/secret.rs の SecretStore trait と 4 言語等価強度を保つ。
// 公開 API に生シークレット値を返さない（WithSecret の callback パターンを使う）。
type SecretStore interface {
	// GetMetadata はシークレットのメタデータのみを返す（値は返さない）。
	// Rust の get_metadata(&self, secret_id, tenant_id) -> Result<Option<SecretMetadata>> に対応する。
	// 存在しない場合は nil を返す。
	GetMetadata(ctx context.Context, secretID string, tenantID string) (*SecretMetadata, error)

	// WithSecret はシークレット値を callback に渡して処理させる。
	// callback 外にシークレット値が漏れない設計（値は返さない: callback の戻り値を interface{} で返す）。
	// Rust の with_secret(&self, secret_id, tenant_id, f: FnOnce(&SecretValue) -> T) -> Result<T> に対応する。
	// callback の引数 secretBytes は呼び出し後に zeroize するため外部で保持してはならない。
	WithSecret(
		ctx context.Context,
		secretID string,
		tenantID string,
		// callback: シークレットのバイト列を受け取り任意の値を返す処理（バイト列を漏洩させない）
		callback func(secretBytes []byte) (any, error),
	) (any, error)

	// Rotate はシークレットを新しいバージョンにローテーションする。
	// 返した SecretMetadata に新しい Version が含まれる。
	// Rust の rotate(&self, secret_id, tenant_id) -> Result<SecretMetadata> に対応する。
	Rotate(ctx context.Context, secretID string, tenantID string) (*SecretMetadata, error)

	// Revoke はシークレットを無効化する（IsActive=false にする）。
	// Revoke 後の WithSecret 呼び出しはエラーを返す。
	// Rust の revoke(&self, secret_id, tenant_id) -> Result<()> に対応する。
	Revoke(ctx context.Context, secretID string, tenantID string) error
}
