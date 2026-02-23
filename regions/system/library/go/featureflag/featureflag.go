package featureflag

import (
	"context"
	"fmt"
)

// FlagVariant はフラグのバリアント。
type FlagVariant struct {
	Name   string `json:"name"`
	Value  string `json:"value"`
	Weight int    `json:"weight"`
}

// FeatureFlag はフィーチャーフラグ。
type FeatureFlag struct {
	ID          string        `json:"id"`
	FlagKey     string        `json:"flag_key"`
	Description string        `json:"description"`
	Enabled     bool          `json:"enabled"`
	Variants    []FlagVariant `json:"variants"`
}

// EvaluationContext はフラグ評価のコンテキスト。
type EvaluationContext struct {
	UserID     *string
	TenantID   *string
	Attributes map[string]string
}

// NewEvaluationContext は新しい EvaluationContext を作成する。
func NewEvaluationContext() *EvaluationContext {
	return &EvaluationContext{
		Attributes: make(map[string]string),
	}
}

// WithUserID はユーザー ID を設定する。
func (c *EvaluationContext) WithUserID(userID string) *EvaluationContext {
	c.UserID = &userID
	return c
}

// WithTenantID はテナント ID を設定する。
func (c *EvaluationContext) WithTenantID(tenantID string) *EvaluationContext {
	c.TenantID = &tenantID
	return c
}

// WithAttribute は属性を追加する。
func (c *EvaluationContext) WithAttribute(key, value string) *EvaluationContext {
	c.Attributes[key] = value
	return c
}

// EvaluationResult はフラグ評価の結果。
type EvaluationResult struct {
	FlagKey string
	Enabled bool
	Variant *string
	Reason  string
}

// FeatureFlagClient はフィーチャーフラグクライアントのインターフェース。
type FeatureFlagClient interface {
	// Evaluate はフラグを評価する。
	Evaluate(ctx context.Context, flagKey string, evalCtx *EvaluationContext) (*EvaluationResult, error)
	// GetFlag はフラグを取得する。
	GetFlag(ctx context.Context, flagKey string) (*FeatureFlag, error)
	// IsEnabled はフラグが有効かどうかを返す。
	IsEnabled(ctx context.Context, flagKey string, evalCtx *EvaluationContext) (bool, error)
}

// FeatureFlagError はフィーチャーフラグ操作のエラー。
type FeatureFlagError struct {
	Code    string
	Message string
}

func (e *FeatureFlagError) Error() string {
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// NewFlagNotFoundError はフラグが見つからないエラーを生成する。
func NewFlagNotFoundError(key string) *FeatureFlagError {
	return &FeatureFlagError{Code: "FLAG_NOT_FOUND", Message: fmt.Sprintf("フラグが見つかりません: %s", key)}
}

// NewConnectionError は接続エラーを生成する。
func NewConnectionError(msg string) *FeatureFlagError {
	return &FeatureFlagError{Code: "CONNECTION_ERROR", Message: msg}
}
