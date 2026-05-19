// rules.go — k1s0 tier1 Library Go 実装: Rule Engine の L1+ interface
// 17_ルールエンジン適合仕様.md §RuleEngineClient（OPA / Drools L1+ 深耕）に準拠する。
// OPA（Open Policy Agent）の full API を Library 独自語彙で表現しつつ、AuthContext 伝播を強制する。
// OSS 型（opa.PreparedEvalQuery / rego.Rego 等）を公開シグネチャに一切含まない。

// パッケージ名: rules（tier1 Library の Rule Engine API を提供する）
package rules

import (
	// context: context.Context（AuthContext 伝播 + 非同期操作に使用する）
	"context"
	// cache パッケージ: CacheTTL を参照する
	"github.com/k1s0-io/k1s0/tier1/library/cache"
)

// PolicyPath は OPA ポリシーのパスを宣言する型。
// OPA の decision path（"data.k1s0.authz.allow" 等のドット記法）を表す。
type PolicyPath string

// PolicyInput は OPA ポリシーへの入力データを宣言する型。
// OPA の input document に相当する（any で表現して型安全性はスキーマで担保する）。
type PolicyInput map[string]any

// PolicyResult はポリシー評価結果を宣言する型。
// OPA の decision result を Library 独自語彙で表現する。
type PolicyResult struct {
	// Allowed: 評価結果（true = 許可 / false = 拒否）
	Allowed bool
	// Reason: 判断理由（"insufficient_scope" / "tenant_mismatch" 等）
	Reason string
	// Details: 追加詳細情報（監査ログ / デバッグ用）
	Details map[string]any
}

// PolicyBundle はポリシーのバンドル情報を宣言する型。
// OPA の Bundle（複数ポリシーファイルのアーカイブ）を Library 独自語彙で表現する。
type PolicyBundle struct {
	// BundlePath: バンドルの取得元 URL または Object Storage パス
	BundlePath string
	// Revision: バンドルのリビジョン（git commit hash 等）
	Revision string
}

// PolicyValidateOptions はポリシー検証オプションを宣言する型。
type PolicyValidateOptions struct {
	// StrictMode: 厳格モード（未定義変数参照等をエラーとして扱う）
	StrictMode bool
	// Capabilities: 使用可能な OPA 組み込み関数の制限（nil = 全組み込みを許可する）
	Capabilities []string
}

// RuleEngineClient は Rule Engine の L1+ 抽象 interface を宣言する。
// OPA（Open Policy Agent）の full API を Library 独自語彙で表現する。
// OSS 型（rego.Rego / rego.PreparedEvalQuery 等）を引数・戻り値に一切含まない。
// ctx に AuthContext が含まれることを強制する（tenant 分離 + 監査に必須）。
type RuleEngineClient interface {
	// Evaluate はポリシーを評価して結果を返す（Unary 評価）。
	// ctx には AuthContext が伝播されている前提とする（input.subject として自動注入する）。
	// path は評価するポリシーパス（"data.k1s0.authz.allow" 等）。
	// input はポリシーへの入力データ（AuthContext のフィールドは実装が自動注入する）。
	Evaluate(ctx context.Context, path PolicyPath, input PolicyInput) (*PolicyResult, error)

	// EvaluateToAny はポリシーを評価して任意型の結果を返す（ルール結果が bool 以外の場合）。
	// path のポリシー結果を any で返す（集合 / オブジェクト等）。
	EvaluateToAny(ctx context.Context, path PolicyPath, input PolicyInput) (any, error)

	// BatchEvaluate は複数のポリシーパスを一括評価する。
	// paths は評価するポリシーパスのスライス、input は全パスに共通の入力。
	// 戻り値は paths と同順の結果スライス。
	BatchEvaluate(ctx context.Context, paths []PolicyPath, input PolicyInput) ([]*PolicyResult, error)

	// LoadBundle は OPA ポリシーバンドルをロードする（hot reload 対応）。
	// bundle は取得元 URL と revision を持つバンドル情報。
	LoadBundle(ctx context.Context, bundle PolicyBundle) error

	// ValidatePolicy はポリシー文字列の構文 / 意味論的正当性を検証する（デプロイ前の CI 用）。
	// policySource は Rego ポリシーソースコード文字列。
	ValidatePolicy(ctx context.Context, policySource string, opts *PolicyValidateOptions) error

	// ListPolicies は現在ロードされているポリシーのパス一覧を返す。
	ListPolicies(ctx context.Context) ([]PolicyPath, error)
}

// CachedRuleEngineClient はポリシー評価結果をキャッシュする L1+ 拡張 interface を宣言する。
// 高頻度な認可チェックのレイテンシを改善するためのキャッシュレイヤー。
// wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
type CachedRuleEngineClient interface {
	// RuleEngineClient の全メソッドを継承する
	RuleEngineClient

	// EvaluateCached はキャッシュ付きでポリシーを評価する。
	// cacheKey はキャッシュキー（tenant_id + policy_path + input hash で構成を推奨する）。
	// ttl は HLC ベースのキャッシュ有効期限（nil = キャッシュしない）。
	EvaluateCached(ctx context.Context, path PolicyPath, input PolicyInput, cacheKey string, ttl *cache.CacheTTL) (*PolicyResult, error)

	// InvalidateCache は指定キーのキャッシュを無効化する（ポリシー更新時に呼び出す）。
	InvalidateCache(ctx context.Context, cacheKey string) error

	// InvalidateAllCache は全キャッシュを無効化する（バンドル更新時に使用する）。
	InvalidateAllCache(ctx context.Context) error
}

// PolicyAuditEntry はポリシー評価の監査ログエントリを宣言する型。
// OPA の decision log を Library 独自語彙で表現する。
type PolicyAuditEntry struct {
	// DecisionID: 評価の識別子（UUID v7 形式を推奨する）
	DecisionID string
	// Path: 評価したポリシーパス
	Path PolicyPath
	// Input: 評価に使用した入力データ（PII を除外した safe 版）
	Input PolicyInput
	// Result: 評価結果
	Result *PolicyResult
	// TenantID: 評価を行ったテナント識別子
	TenantID string
	// TimestampTick: 評価時刻（HLC tick 値: wall-clock TTL 禁止規約に準拠する）
	TimestampTick uint64
}
