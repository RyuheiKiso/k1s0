// 本ファイルは tier2 Go サービス専用の業務エラー型。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/04_概要設計/30_共通機能方式設計/  （E-T2-* エラー体系）
//
// scope:
//   tier2 内部のドメインエラーを E-T2-<カテゴリ>-<番号> の体系で表す。
//   tier1 が返す k1s0.internal.errors.v1.ErrorDetail とは独立した、業務文脈固有のエラー。
//   tier3 / Web に露出させる際は API 層で OpenAPI / GraphQL のエラー型に変換する。
//
// stability: Alpha

// Package errors は tier2 Go サービス専用のエラー型と分類を提供する。
package errors

// 標準 import。
import (
	// エラーラップ用 errors.As / errors.Is サポート。
	"errors"
	// エラー文字列整形。
	"fmt"
)

// Category は tier2 業務エラーの大分類。
type Category string

// 主要カテゴリ。
const (
	// CategoryValidation は入力バリデーション違反（HTTP 400 相当）。
	CategoryValidation Category = "VALIDATION"
	// CategoryNotFound はリソース不存在（HTTP 404 相当）。
	CategoryNotFound Category = "NOT_FOUND"
	// CategoryConflict は楽観的排他違反 / 重複（HTTP 409 相当）。
	CategoryConflict Category = "CONFLICT"
	// CategoryUpstream は tier1 / 外部依存からのエラー（HTTP 502 相当）。
	CategoryUpstream Category = "UPSTREAM"
	// CategoryInternal は内部実装エラー（HTTP 500 相当、ログだけ詳細を残す）。
	CategoryInternal Category = "INTERNAL"
)

// HTTPStatus はカテゴリから推奨 HTTP ステータスを返す。
//
// API 層の middleware が DomainError を捕捉した時、本関数で HTTP status を決定する。
func (c Category) HTTPStatus() int {
	// 分岐で対応 status を返す。
	switch c {
	// 入力不備は 400。
	case CategoryValidation:
		// HTTP 400 Bad Request。
		return 400
	// 不存在は 404。
	case CategoryNotFound:
		// HTTP 404 Not Found。
		return 404
	// 衝突は 409。
	case CategoryConflict:
		// HTTP 409 Conflict。
		return 409
	// 上流エラーは 502。
	case CategoryUpstream:
		// HTTP 502 Bad Gateway。
		return 502
	// 内部エラーは 500。
	case CategoryInternal:
		// HTTP 500 Internal Server Error。
		return 500
	// 既知でないカテゴリは 500 にフォールバック。
	default:
		// 安全側に 500。
		return 500
	}
}

// DomainError は tier2 業務エラーの基底型。
//
// すべての tier2 業務エラーは E-T2-* コードを持ち、HTTP / GraphQL レスポンスへ
// 一貫した形で写像できる。
type DomainError struct {
	// カテゴリ（HTTP status の根拠）。
	Category Category
	// E-T2-* コード（例: "E-T2-RECON-001"）。
	Code string
	// 人間可読メッセージ（safe to log、PII を含めないこと）。
	Message string
	// 元エラー（あれば）。
	Cause error
}

// Error は標準 error interface 実装。
func (e *DomainError) Error() string {
	// Cause があれば連結する。
	if e.Cause != nil {
		// E-T2-XXX-NNN [CATEGORY] message: cause の形に整形する。
		return fmt.Sprintf("%s [%s] %s: %v", e.Code, e.Category, e.Message, e.Cause)
	}
	// E-T2-XXX-NNN [CATEGORY] message の形に整形する。
	return fmt.Sprintf("%s [%s] %s", e.Code, e.Category, e.Message)
}

// Unwrap は errors.As / errors.Is サポートのため Cause を返す。
func (e *DomainError) Unwrap() error {
	// 内側の error を露出する。
	return e.Cause
}

// New は DomainError を組み立てる便利コンストラクタ。
func New(cat Category, code, msg string) *DomainError {
	// シンプル版（cause なし）。
	return &DomainError{Category: cat, Code: code, Message: msg}
}

// Wrap は cause 付きで DomainError を組み立てる。
func Wrap(cat Category, code, msg string, cause error) *DomainError {
	// cause 同梱版。
	return &DomainError{Category: cat, Code: code, Message: msg, Cause: cause}
}

// AsDomainError は err を *DomainError に変換できれば true を返す。
//
// 利用側: middleware が `domain, ok := errors.AsDomainError(err)` で HTTP 変換する。
func AsDomainError(err error) (*DomainError, bool) {
	// nil 早期 return。
	if err == nil {
		// nil は変換不能。
		return nil, false
	}
	// errors.As で取り出す。
	var de *DomainError
	// 変換成功時は DomainError を返す。
	if errors.As(err, &de) {
		// 取得できた DomainError。
		return de, true
	}
	// 非 DomainError は false。
	return nil, false
}
