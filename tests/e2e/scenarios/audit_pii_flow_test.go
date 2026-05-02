// 本ファイルは Audit + PII の貫通 E2E シナリオ。
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 (Test Pyramid + testcontainers) / ADR-TEST-002 (E2E 自動化)
//
// 検証対象:
//   1. Pii.Classify による PII 検出（EMAIL 識別）
//   2. Pii.Mask による [EMAIL] 等のマスキング
//   3. Audit.Record で監査ログ書き込み（hash chain WORM）
//   4. Audit.Query で直前の record 取得（fan-in 整合）
//   5. Audit.VerifyChain で hash chain 整合性検証
//
// SHIP_STATUS line 207-209 で Rust audit / pii Pod の実 cluster 動作実績あり、
// kind cluster + tools/local-stack/up.sh --role tier1-go-dev で疎通する設計。
package scenarios

import (
	"context"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	// k1s0 SDK 高水準 facade（Pii / Audit）
	"github.com/k1s0/sdk-go/k1s0"

	// E2E 共通 helper
	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestAuditPiiFlow は Audit + PII service の最小貫通検証。
// PII 検出 → マスキング → 監査ログ書込 → 取得 → hash chain 整合性の 5 段階を試す。
func TestAuditPiiFlow(t *testing.T) {
	// kind cluster への接続（未起動なら test 全体が Skip）
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	// 全体タイムアウト 60 秒（Audit Query で 5 秒程度の整合性遅延を見込む）
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	// tier1 公開 gRPC endpoint を env var から取得
	target := os.Getenv("K1S0_TIER1_TARGET")
	if target == "" {
		// endpoint 未指定なら gRPC 検証部を Skip（kind 接続のみ検証）
		t.Skip("K1S0_TIER1_TARGET 未設定: kubectl port-forward で tier1 endpoint を露出してから K1S0_TIER1_TARGET=localhost:50001 で再実行")
	}

	// k1s0 SDK Client を初期化（dev 用に平文 gRPC、テスト用 tenant_id を付与）
	client, err := k1s0.New(ctx, k1s0.Config{
		// gRPC 接続先
		Target: target,
		// E2E 専用 tenant_id（本番 namespace を汚染しない）
		TenantID: "e2e-tenant-audit-pii",
		// 主体識別子（監査ログで test 起源を識別）
		Subject: "e2e-test/audit-pii-flow",
		// dev 用に平文（kind 内部、TLS は採用初期で SPIRE 経由）
		UseTLS: false,
	})
	// 接続失敗は Fatal（endpoint があったのに接続できないのは bug）
	if err != nil {
		t.Fatalf("k1s0.New(target=%s): %v", target, err)
	}
	// test 終了時に gRPC connection を解放
	defer func() { _ = client.Close() }()

	// テスト用 PII を含む文字列（実在の email では衝突するため example.com を使う）
	const piiText = "contact: alice@example.com please reach out"

	// Step 1: PII Classify — EMAIL カテゴリの検出を assert
	findings, containsPii, err := client.Pii().Classify(ctx, piiText)
	if err != nil {
		t.Fatalf("Pii.Classify: %v", err)
	}
	// PII を含むのに containsPii=false なら detector が壊れている
	if !containsPii {
		t.Fatalf("Pii.Classify: containsPii=false（EMAIL を検出できていない）: findings=%v", findings)
	}
	// 検出件数が 0 なら detector が空配列を返している（containsPii と矛盾）
	if len(findings) == 0 {
		t.Fatalf("Pii.Classify: findings=[] だが containsPii=true（実装の不整合）")
	}
	// 検出された PII カテゴリを log に残す
	t.Logf("Pii.Classify: containsPii=%v findings=%d", containsPii, len(findings))

	// Step 2: PII Mask — 元のメールアドレスが masked text から消えていることを assert
	maskedText, maskFindings, err := client.Pii().Mask(ctx, piiText)
	if err != nil {
		t.Fatalf("Pii.Mask: %v", err)
	}
	// 元のメールが残っていれば mask が効いていない
	if strings.Contains(maskedText, "alice@example.com") {
		t.Fatalf("Pii.Mask: 元のメール残存 maskedText=%q", maskedText)
	}
	// findings が空なら detector が動いていない
	if len(maskFindings) == 0 {
		t.Fatalf("Pii.Mask: findings=[] だが mask 対象テキストを渡した")
	}
	// マスク後テキストを log に残す
	t.Logf("Pii.Mask: masked=%q findings=%d", maskedText, len(maskFindings))

	// Step 3: Audit.Record — 監査ログ書き込み（hash chain WORM、append-only）
	queryStart := time.Now().UTC().Add(-1 * time.Second) // Query の from に使う基準時刻
	idempotencyKey := fmt.Sprintf("e2e-audit-pii-%d", time.Now().UnixNano())
	auditID, err := client.Audit().Record(
		ctx,
		"e2e-test/audit-pii-flow", // actor
		"e2e.audit_pii_flow",      // action
		"test://e2e/audit-pii",    // resource
		"success",                 // outcome
		map[string]string{
			"test_run": idempotencyKey,
		},
		idempotencyKey,
	)
	if err != nil {
		t.Fatalf("Audit.Record: %v", err)
	}
	// audit_id が空なら record が失敗している（resp が空）
	if auditID == "" {
		t.Fatalf("Audit.Record: audit_id 空（record 失敗の疑い）")
	}
	// 取得した audit_id を log
	t.Logf("Audit.Record: audit_id=%s", auditID)

	// Step 4: Audit.Query — 直前の record を取得して fan-in 整合性を確認
	// hash chain への append が完了するまで短い待機（実装依存、最大 3 秒で諦める）
	var queriedEvents int
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		events, qerr := client.Audit().Query(
			ctx,
			queryStart,
			time.Now().UTC().Add(1*time.Second),
			map[string]string{"test_run": idempotencyKey},
			10,
		)
		if qerr != nil {
			t.Fatalf("Audit.Query: %v", qerr)
		}
		queriedEvents = len(events)
		if queriedEvents > 0 {
			break
		}
		time.Sleep(500 * time.Millisecond)
	}
	// 直前に Record した event が見つからない場合は WORM への append が遅延 / 失敗
	if queriedEvents == 0 {
		t.Fatalf("Audit.Query: 直前の record が取得できない（filter test_run=%s）", idempotencyKey)
	}
	// 取得件数を log
	t.Logf("Audit.Query: %d events（filter test_run=%s）", queriedEvents, idempotencyKey)

	// Step 5: Audit.VerifyChain — hash chain 整合性を test 範囲で検証
	chainResult, err := client.Audit().VerifyChain(ctx, queryStart, time.Now().UTC().Add(1*time.Second))
	if err != nil {
		t.Fatalf("Audit.VerifyChain: %v", err)
	}
	// hash chain が壊れていれば WORM の改ざん検知が壊れている（致命的）
	if !chainResult.Valid {
		t.Fatalf("Audit.VerifyChain: chain 不整合 first_bad_sequence=%d reason=%s", chainResult.FirstBadSequence, chainResult.Reason)
	}
	// 検証件数を log
	t.Logf("Audit.VerifyChain: valid=%v checked=%d", chainResult.Valid, chainResult.CheckedCount)
}
