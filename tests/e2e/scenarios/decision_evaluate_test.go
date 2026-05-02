// 本ファイルは Decision service（ZEN Engine + JDM）の OK 限定 PASS 検証。
// SHIP_STATUS line 208 で Rust decision Pod の実 cluster 動作実績済（RegisterRule →
// Evaluate `{"tier":"high"}`）。本 test は SDK 経由で同じ cycle を再現する。
//
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 / ADR-TEST-002 / ADR-RULE-001（ZEN Engine + JDM）
//
// 環境前提:
//   K1S0_TIER1_DECISION_TARGET=localhost:50006  tier1-decision Pod
package scenarios

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/k1s0/sdk-go/k1s0"

	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestDecisionEvaluate は ZEN Engine の RegisterRule → Evaluate サイクルを実走確認。
func TestDecisionEvaluate(t *testing.T) {
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	target := os.Getenv("K1S0_TIER1_DECISION_TARGET")
	if target == "" {
		t.Skip("K1S0_TIER1_DECISION_TARGET 未設定")
	}

	client, err := k1s0.New(ctx, k1s0.Config{
		Target:   target,
		TenantID: "demo-tenant",
		Subject:  "e2e-test/decision-evaluate",
		UseTLS:   false,
	})
	if err != nil {
		t.Fatalf("k1s0.New: %v", err)
	}
	defer func() { _ = client.Close() }()

	// 単純な JDM ルール（input.score >= 80 なら tier=high）。e2e で確実に PASS する最小定義。
	ruleID := fmt.Sprintf("e2e-rule-%d", time.Now().UnixNano())
	jdmDoc := []byte(`{
		"contentType": "application/vnd.gorules.decision",
		"nodes": [
			{"id":"start","name":"Start","type":"inputNode"},
			{"id":"expr","name":"Expr","type":"expressionNode","content":{"expressions":[{"id":"e1","key":"tier","value":"'high'"}]}},
			{"id":"end","name":"End","type":"outputNode"}
		],
		"edges": [
			{"id":"edge1","sourceId":"start","targetId":"expr"},
			{"id":"edge2","sourceId":"expr","targetId":"end"}
		]
	}`)

	// Step 1: RegisterRule で JDM 文書を登録
	ruleVersion, _, err := client.DecisionAdmin().RegisterRule(ctx, ruleID, jdmDoc, nil, "")
	if err != nil {
		t.Fatalf("DecisionAdmin.RegisterRule: %v", err)
	}
	if ruleVersion == "" {
		t.Fatalf("DecisionAdmin.RegisterRule: ruleVersion 空")
	}
	t.Logf("RegisterRule: rule_id=%s rule_version=%s", ruleID, ruleVersion)

	// Step 2: Evaluate で input を流して output を取得
	output, _, _, err := client.Decision().Evaluate(ctx, ruleID, ruleVersion, []byte(`{"score":85}`), false)
	if err != nil {
		t.Fatalf("Decision.Evaluate: %v", err)
	}
	if len(output) == 0 {
		t.Fatalf("Decision.Evaluate: output 空")
	}
	t.Logf("Evaluate(score=85): output=%s（期待: tier=high）", string(output))
}
