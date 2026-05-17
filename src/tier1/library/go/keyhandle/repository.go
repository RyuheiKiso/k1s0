// repository.go — k1s0 tier1 Library Go 実装: Repository[T any] interface
// 04_認証適合仕様.md §CI 不変条件 整合 5「生 SQL 文字列受付 API 禁止」に準拠する。
// tenant_id を必須引数として受け取る（SQL 文字列受付 API は提供しない）。
// Go 1.18+ generics を使用して型安全な Repository 抽象を実装する。

// パッケージ名: keyhandle（tier1 Library のリポジトリ抽象 API を提供する）
package keyhandle

// context: context.Context（非同期操作に使用する）
import (
	"context"
	// errors: エラー生成に使用する
	"errors"
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
)

// Repository[T any] は生 SQL 文字列を受け取らない DB アクセス抽象 interface。
// tenant_id は必須引数として受け取る（RLS と二重で tenant 境界を保証する）。
// T は DB に永続化されるドメイン型（any: 制約なし、実装側で具体型を指定する）。
type Repository[T any] interface {
	// FindByID は id と tenantID を受け取り、エンティティを返す。
	// tenantID は必須引数（RLS と二重で tenant 境界を保証する）。
	// エンティティが見つからない場合は nil を返す。
	FindByID(ctx context.Context, id string, tenantID string) (*T, error)

	// Save はエンティティと tenantID を受け取り、DB に保存する。
	// tenantID は必須引数（RLS と二重で tenant 境界を保証する）。
	Save(ctx context.Context, entity *T, tenantID string) error
}

// ErrRLSBypass: RLS bypass 検出時のエラー
// RLS が物理的に保証するが、アプリ層でも二重検証する設計。
var ErrRLSBypass = errors.New("RLS bypass detected")

// NewRLSBypassError は RLS bypass 検出エラーを生成するヘルパー関数。
// entity.tenantID と context.tenantID の不一致を報告する。
func NewRLSBypassError(entityTenantID, contextTenantID string) error {
	// RLS bypass 検出エラーを生成する
	return fmt.Errorf("%w: entity.tenant_id=%s != context.tenant_id=%s",
		ErrRLSBypass, entityTenantID, contextTenantID)
}

// VerifyTenantID は entity の tenantID が context の tenantID と一致することを検証する。
// RLS が物理的に保証するが、アプリ層でも二重検証する設計。
func VerifyTenantID(entityTenantID, contextTenantID string) error {
	// テナント ID が一致しない場合は RLS bypass 検出エラーを返す
	if entityTenantID != contextTenantID {
		return NewRLSBypassError(entityTenantID, contextTenantID)
	}
	// テナント ID が一致した場合は nil を返す（正常）
	return nil
}
