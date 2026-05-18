// k1s0 tier2 Repository abstraction Go 実装
// Rust 実装（repository.rs）と 4 言語等価強度を持つ Go 版
// 生 SQL 文字列を受け取る public API を持たない設計で tier2 の DB アクセスを封鎖する

// パッケージ名: repository
package repository

import (
	// errors パッケージ: エラー生成に使用する
	"errors"
	// fmt パッケージ: エラーメッセージのフォーマットに使用する
	"fmt"

	// uuid パッケージ: ID の UUID 型
	"github.com/google/uuid"
	// tenantcontext パッケージ: テナントコンテキスト（GUC 注入に使用する）
	"github.com/k1s0/tier2/tenantcontext"
)

// ErrRlsBypass: RLS bypass 検出エラー
// SELECT 結果の tenant_id が TenantContext と一致しない場合に返す
// RLS が物理的に保証するが、アプリ層でも二重検証する設計
var ErrRlsBypass = errors.New("RLS bypass detected: entity tenant_id does not match context tenant_id")

// ErrNotFound: エンティティが見つからないエラー
// FindByID / FindByTenantID が 0 件の場合に返す
var ErrNotFound = errors.New("entity not found")

// TenantScopedEntity: テナントスコープ内のエンティティを表す汎用型
// PII 列は含まない（pii_segregated table は別の型で管理する）
type TenantScopedEntity struct {
	// エンティティの主キー
	ID uuid.UUID `json:"id"`
	// テナント ID（read-only、RLS が保証する）
	TenantID uuid.UUID `json:"tenant_id"`
	// エンティティバージョン（楽観的ロックに使用する）
	Version int64 `json:"version"`
	// エンティティペイロード（業界中立的な汎用 JSON）
	Payload any `json:"payload"`
}

// Repository[T]: テナントスコープ内のエンティティ操作インターフェース
// tenant_id は引数で受け取らず、実装内部で TenantContext から注入する
// Go のジェネリクスを使って型安全なリポジトリ抽象を提供する
type Repository[T any] interface {
	// FindByID: エンティティを主キーで取得する
	// tenant_id は引数で受け取らず、内部的に TenantContext から注入する
	// 見つからない場合は ErrNotFound を返す
	FindByID(id uuid.UUID) (*T, error)

	// FindByTenantID: テナント ID に紐づく全エンティティを取得する
	// tenant_id は AuthContext から取得するため引数で受け取らない
	FindByTenantID() ([]*T, error)

	// Save: エンティティを永続化する（INSERT または UPDATE）
	// tenant_id は TenantContext から注入するため entity に含めない
	Save(entity *T) error

	// Delete: エンティティを削除する
	// tenant_id は TenantContext から自動注入される（API 引数経由禁止）
	Delete(id uuid.UUID) error
}

// RepositoryContext: Repository を実行するコンテキスト
// TenantContext をラップして DB アクセス時の GUC 注入を担う
// 生 SQL 文字列は受け取らない設計を型で表現する
type RepositoryContext struct {
	// テナントコンテキスト（GUC 注入に使用する）
	tenantContext *tenantcontext.TenantContext
}

// NewRepositoryContext: RepositoryContext を生成する
// tenant_id は API 引数として渡せない（TenantContext 経由のみ）
func NewRepositoryContext(tc *tenantcontext.TenantContext) (*RepositoryContext, error) {
	// TenantContext が nil の場合はエラーを返す
	if tc == nil {
		return nil, errors.New("TenantContext must not be nil")
	}
	// RepositoryContext を生成して返す
	return &RepositoryContext{
		tenantContext: tc,
	}, nil
}

// TenantContext: テナントコンテキストを参照する
// RepositoryContext 内部の TenantContext を返す（読み取り専用）
func (rc *RepositoryContext) TenantContext() *tenantcontext.TenantContext {
	// テナントコンテキストを返す
	return rc.tenantContext
}

// SetLocalSQL: SET LOCAL SQL を取得する
// Repository 実装が DB に注入するために使用する（4 GUC を一括 SET LOCAL する）
func (rc *RepositoryContext) SetLocalSQL() string {
	// TenantContext の ToSetLocalSQL() を呼んで SQL を返す
	return rc.tenantContext.ToSetLocalSQL()
}

// VerifySelectResult: SELECT 結果の tenant_id が TenantContext と一致することを検証する
// RLS が物理的に保証するが、アプリ層でも二重検証する設計
func VerifySelectResult(entity *TenantScopedEntity, ctx *RepositoryContext) error {
	// entity が nil の場合はエラーを返す
	if entity == nil {
		return errors.New("entity must not be nil")
	}
	// RepositoryContext が nil の場合はエラーを返す
	if ctx == nil {
		return errors.New("RepositoryContext must not be nil")
	}
	// TenantContext の TenantID を取得する
	expectedTenantID := ctx.tenantContext.TenantID()
	// エンティティの tenant_id と TenantContext の tenant_id を比較する
	if entity.TenantID != expectedTenantID {
		// RLS bypass が発生した場合は即座にエラーで止める
		return fmt.Errorf(
			"%w: entity.tenant_id=%s, context.tenant_id=%s",
			ErrRlsBypass,
			entity.TenantID.String(),
			expectedTenantID.String(),
		)
	}
	// tenant_id が一致した場合は nil を返す
	return nil
}
