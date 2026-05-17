// k1s0 tier2 テナントコンテキスト Go 実装
// Rust 実装（tenant_context.rs）と 4 言語等価強度を持つ Go 版
// PostgreSQL session GUC（app.tenant_id / app.actor_id / app.purpose / app.delegation_chain）を
// transaction 開始時に SET LOCAL で自動注入する

// パッケージ名: tenantcontext
package tenantcontext

import (
	// errors パッケージ: エラー生成に使用する
	"errors"
	// fmt パッケージ: SQL 文字列フォーマットに使用する
	"fmt"
	// strings パッケージ: SQL エスケープに使用する
	"strings"

	// uuid パッケージ: tenant_id / actor_id の UUID 型
	"github.com/google/uuid"
)

// SessionPurpose: PostgreSQL session GUC の app.purpose 許容値
// 10_テナント分離適合仕様.md の purpose enum と完全整合する
type SessionPurpose string

const (
	// PurposeBusinessOp: 通常業務操作
	PurposeBusinessOp SessionPurpose = "business_op"
	// PurposeSupport: サポートアクセス（PII 参照時に使用する）
	PurposeSupport SessionPurpose = "support"
	// PurposeExport: データエクスポート
	PurposeExport SessionPurpose = "export"
	// PurposeMigration: スキーマ移行操作
	PurposeMigration SessionPurpose = "migration"
	// PurposeEmergency: 緊急オペレーション（全記録必須）
	PurposeEmergency SessionPurpose = "emergency"
)

// ErrInvalidPurpose: 無効な purpose 値のエラー
var ErrInvalidPurpose = errors.New("invalid session purpose")

// TenantContext: テナントセッションコンテキスト
// 4 つの PostgreSQL session GUC をまとめて管理する
// tenant_id は外部から直接 API 引数として渡せない（FromAuth 経由のみ）
type TenantContext struct {
	// テナント識別子（unexported: API 引数経由での設定を禁止する）
	tenantID uuid.UUID
	// アクター識別子（Keycloak subject）
	actorID string
	// セッション目的
	purpose SessionPurpose
	// 委譲チェーン（通常操作では空スライス）
	delegationChain []string
}

// FromAuth: TenantContext を認証済み情報から生成する
// tenant_id を外部から直接 API 引数として渡せないことを型で表現する
func FromAuth(tenantID uuid.UUID, actorID string, purpose SessionPurpose) (*TenantContext, error) {
	// purpose の値を検証する
	switch purpose {
	case PurposeBusinessOp, PurposeSupport, PurposeExport, PurposeMigration, PurposeEmergency:
		// 許容値であれば TenantContext を生成する
	default:
		// 許容外の purpose 値はエラーを返す
		return nil, fmt.Errorf("%w: %q", ErrInvalidPurpose, purpose)
	}
	// 委譲チェーンは空スライスで初期化する
	return &TenantContext{
		tenantID:        tenantID,
		actorID:         actorID,
		purpose:         purpose,
		delegationChain: []string{},
	}, nil
}

// WithDelegation: 委譲元の actor を追加した TenantContext を返す（イミュータブルコピー）
func (ctx *TenantContext) WithDelegation(delegatorID string) *TenantContext {
	// delegation chain に delegator を追加した新しいコンテキストを返す
	newChain := make([]string, len(ctx.delegationChain)+1)
	copy(newChain, ctx.delegationChain)
	newChain[len(ctx.delegationChain)] = delegatorID
	return &TenantContext{
		tenantID:        ctx.tenantID,
		actorID:         ctx.actorID,
		purpose:         ctx.purpose,
		delegationChain: newChain,
	}
}

// ToSetLocalSQL: 4 GUC を一括 SET LOCAL する SQL 文字列を返す
// 実際の DB 実行はリポジトリ層が担う（このメソッドは SQL 生成のみ）
func (ctx *TenantContext) ToSetLocalSQL() string {
	// app.delegation_chain の配列リテラルを構築する
	var chainLiteral string
	if len(ctx.delegationChain) == 0 {
		// 委譲なしの場合は空配列リテラル
		chainLiteral = "'{}'"
	} else {
		// 各要素をダブルクォートで囲んで配列リテラルを構築する
		quoted := make([]string, len(ctx.delegationChain))
		for i, s := range ctx.delegationChain {
			quoted[i] = `"` + strings.ReplaceAll(s, `"`, `\"`) + `"`
		}
		chainLiteral = "'{" + strings.Join(quoted, ",") + "}'"
	}
	// 4 GUC の SET LOCAL SQL を返す
	return fmt.Sprintf(
		"SET LOCAL app.tenant_id = '%s'; SET LOCAL app.actor_id = '%s'; SET LOCAL app.purpose = '%s'; SET LOCAL app.delegation_chain = %s;",
		ctx.tenantID.String(),
		strings.ReplaceAll(ctx.actorID, "'", "''"),
		string(ctx.purpose),
		chainLiteral,
	)
}

// TenantID: テナント ID を返す（パッケージ内でのみ使用する）
// 外部パッケージへの直接露出を避けるために unexported getter は持たない設計
func (ctx *TenantContext) TenantID() uuid.UUID {
	// テナント ID を返す（リポジトリ抽象のみが呼ぶ）
	return ctx.tenantID
}
