// 本ファイルは FR-T1-FEATURE-004 A/B テスト基盤の variant validation テスト。
//
// 受け入れ基準:
//   - variant 数は最大 10
//   - variant 配分の合計は 100%

package state

import (
	"strings"
	"testing"

	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	"google.golang.org/protobuf/types/known/structpb"
)

// makeFractionalDef は fractional split 付き FlagDefinition を組み立てる helper。
func makeFractionalDef(variants []string, weights []int32) *featurev1.FlagDefinition {
	vs := make(map[string]*structpb.Value, len(variants))
	for _, v := range variants {
		vs[v] = structpb.NewStringValue(v)
	}
	splits := make([]*featurev1.FractionalSplit, 0, len(variants))
	for i, v := range variants {
		splits = append(splits, &featurev1.FractionalSplit{Variant: v, Weight: weights[i]})
	}
	return &featurev1.FlagDefinition{
		FlagKey:        "tenant.app.feature",
		ValueType:      featurev1.FlagValueType_FLAG_VALUE_STRING,
		State:          featurev1.FlagState_FLAG_STATE_ENABLED,
		DefaultVariant: variants[0],
		Variants:       vs,
		Targeting: []*featurev1.TargetingRule{
			{Fractional: splits},
		},
	}
}

// TestValidateFlagDefinition_RejectsTooManyVariants は 11 variants を InvalidArgument で弾く。
func TestValidateFlagDefinition_RejectsTooManyVariants(t *testing.T) {
	vs := make(map[string]*structpb.Value, 11)
	for i := 0; i < 11; i++ {
		vs[string(rune('a'+i))] = structpb.NewStringValue("v")
	}
	def := &featurev1.FlagDefinition{
		FlagKey:        "tenant.app.feature",
		ValueType:      featurev1.FlagValueType_FLAG_VALUE_STRING,
		State:          featurev1.FlagState_FLAG_STATE_ENABLED,
		DefaultVariant: "a",
		Variants:       vs,
	}
	err := validateFlagDefinition(def, "")
	if err == nil {
		t.Fatal("expected error for 11 variants")
	}
	if !strings.Contains(err.Error(), "exceeds maximum") {
		t.Errorf("error message = %q, want contains 'exceeds maximum'", err.Error())
	}
}

// TestValidateFlagDefinition_AcceptsExactly10Variants は 10 variants は通ることを確認する。
func TestValidateFlagDefinition_AcceptsExactly10Variants(t *testing.T) {
	vs := make(map[string]*structpb.Value, 10)
	for i := 0; i < 10; i++ {
		vs[string(rune('a'+i))] = structpb.NewStringValue("v")
	}
	def := &featurev1.FlagDefinition{
		FlagKey:        "tenant.app.feature",
		ValueType:      featurev1.FlagValueType_FLAG_VALUE_STRING,
		State:          featurev1.FlagState_FLAG_STATE_ENABLED,
		DefaultVariant: "a",
		Variants:       vs,
	}
	if err := validateFlagDefinition(def, ""); err != nil {
		t.Errorf("10 variants should pass: %v", err)
	}
}

// TestValidateFlagDefinition_FractionalSumMustBe100 は weight 合計 != 100 を弾く。
func TestValidateFlagDefinition_FractionalSumMustBe100(t *testing.T) {
	def := makeFractionalDef([]string{"control", "treatment"}, []int32{40, 40})
	err := validateFlagDefinition(def, "")
	if err == nil {
		t.Fatal("expected error for sum=80")
	}
	if !strings.Contains(err.Error(), "sum to 100") {
		t.Errorf("error message = %q, want contains 'sum to 100'", err.Error())
	}
}

// TestValidateFlagDefinition_FractionalAcceptsExactly100 は weight 合計 = 100 が通ることを確認する。
func TestValidateFlagDefinition_FractionalAcceptsExactly100(t *testing.T) {
	def := makeFractionalDef([]string{"control", "treatment"}, []int32{50, 50})
	if err := validateFlagDefinition(def, ""); err != nil {
		t.Errorf("sum=100 should pass: %v", err)
	}
}

// TestValidateFlagDefinition_FractionalRejectsUnknownVariant は variant が variants map に
// 存在しない場合に InvalidArgument を返すことを確認する。
func TestValidateFlagDefinition_FractionalRejectsUnknownVariant(t *testing.T) {
	def := &featurev1.FlagDefinition{
		FlagKey:        "tenant.app.feature",
		ValueType:      featurev1.FlagValueType_FLAG_VALUE_STRING,
		State:          featurev1.FlagState_FLAG_STATE_ENABLED,
		DefaultVariant: "control",
		Variants: map[string]*structpb.Value{
			"control": structpb.NewStringValue("c"),
		},
		Targeting: []*featurev1.TargetingRule{
			{Fractional: []*featurev1.FractionalSplit{
				{Variant: "control", Weight: 50},
				{Variant: "ghost", Weight: 50}, // variants 未定義
			}},
		},
	}
	err := validateFlagDefinition(def, "")
	if err == nil {
		t.Fatal("expected error for unknown variant")
	}
	if !strings.Contains(err.Error(), "unknown variant") {
		t.Errorf("error message = %q, want contains 'unknown variant'", err.Error())
	}
}

// TestValidateFlagDefinition_FractionalRejectsNegativeWeight は負 weight を弾く。
func TestValidateFlagDefinition_FractionalRejectsNegativeWeight(t *testing.T) {
	def := makeFractionalDef([]string{"control", "treatment"}, []int32{-1, 101})
	err := validateFlagDefinition(def, "")
	if err == nil {
		t.Fatal("expected error for negative weight")
	}
}

// TestValidateFlagDefinition_FractionalSkipsEmptyRules は fractional 無し（pure if/else）の
// rule は検証 skip されることを確認する。
func TestValidateFlagDefinition_FractionalSkipsEmptyRules(t *testing.T) {
	def := &featurev1.FlagDefinition{
		FlagKey:        "tenant.app.feature",
		ValueType:      featurev1.FlagValueType_FLAG_VALUE_STRING,
		State:          featurev1.FlagState_FLAG_STATE_ENABLED,
		DefaultVariant: "v",
		Variants:       map[string]*structpb.Value{"v": structpb.NewStringValue("v")},
		Targeting: []*featurev1.TargetingRule{
			{VariantIfMatch: "v"}, // fractional なし
		},
	}
	if err := validateFlagDefinition(def, ""); err != nil {
		t.Errorf("rule with no fractional should pass: %v", err)
	}
}
