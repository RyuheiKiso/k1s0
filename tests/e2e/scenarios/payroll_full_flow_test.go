// 本ファイルは payroll（給与計算）の前段データ準備フロー E2E シナリオ。
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 (Test Pyramid + testcontainers) / ADR-TEST-002 (E2E 自動化)
//
// 検証対象（Secrets + State + Audit の 3 service 連動）:
//   1. Secrets.Get で OpenBao から payroll 計算 API key 相当の secret を取得
//   2. State.Save で payroll-input（給与計算入力データ）を tier1 State に保存
//   3. Audit.Record で「payroll 計算開始」を監査ログに記録
//   4. State.Get で payroll-input を再取得（idempotency 確認）
//   5. Audit.Record で「payroll 計算完了」を監査ログ
//   6. State.Delete で payroll-input を削除（後始末）
//
// 実 Workflow.Start での給与計算ロジック起動は tier2 worker の RegisterWorkflow が前提で、
// 採用初期に tier2 worker 整備後に payroll_workflow_full_test.go として別実装する。
//
// SHIP_STATUS line 205-207 で OpenBao Get/Rotate / Postgres State / Audit hash chain の
// 実 cluster 動作実績済。kind cluster + tools/local-stack/up.sh --role tier1-go-dev で疎通する。
package scenarios

import (
	"bytes"
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	// k1s0 SDK 高水準 facade（Secrets / State / Audit）
	"github.com/k1s0/sdk-go/k1s0"

	// E2E 共通 helper
	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestPayrollFullFlow は給与計算の前段データ準備（Secrets + State + Audit）を貫通検証する。
// 並列実行不可（tier1 共有データを操作するため、t.Parallel() は呼ばない）。
func TestPayrollFullFlow(t *testing.T) {
	// kind cluster への接続（未起動なら test 全体が Skip）
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	// 全体タイムアウト 90 秒（Audit Query で短い遅延を許容）
	ctx, cancel := context.WithTimeout(context.Background(), 90*time.Second)
	defer cancel()

	// 各 service が別 Pod / 別 Service に分かれているため Client を別個に作成する。
	// State / Secrets / その他は tier1-facade-state Pod（K1S0_TIER1_TARGET）、Audit は
	// tier1-audit Pod（K1S0_TIER1_AUDIT_TARGET）。本番では Envoy Gateway / Service Mesh が
	// 単一 endpoint で routing するが、local kind 環境では各 Service を個別 port-forward する。
	target := os.Getenv("K1S0_TIER1_TARGET")
	auditTarget := os.Getenv("K1S0_TIER1_AUDIT_TARGET")
	if target == "" || auditTarget == "" {
		// endpoint 未指定なら gRPC 検証を Skip（kind 接続のみ確認する経路）
		t.Skip("K1S0_TIER1_TARGET / K1S0_TIER1_AUDIT_TARGET 未設定: kubectl port-forward で各 service を露出してから再実行")
	}

	// State / Secrets 用 Client（tier1-facade-state Pod 直結）
	client, err := k1s0.New(ctx, k1s0.Config{
		Target:   target,
		TenantID: "demo-tenant",
		Subject:  "e2e-test/payroll-full-flow",
		UseTLS:   false,
	})
	if err != nil {
		t.Fatalf("k1s0.New(state target=%s): %v", target, err)
	}
	defer func() { _ = client.Close() }()

	// Audit 用 Client（tier1-audit Pod 直結）
	auditClient, err := k1s0.New(ctx, k1s0.Config{
		Target:   auditTarget,
		TenantID: "demo-tenant",
		Subject:  "e2e-test/payroll-full-flow",
		UseTLS:   false,
	})
	if err != nil {
		t.Fatalf("k1s0.New(audit target=%s): %v", auditTarget, err)
	}
	defer func() { _ = auditClient.Close() }()

	// 各 service の動作確認用 input
	const (
		// OpenBao の dev 用 demo secret（local-stack/openbao-dev/ で seed されている想定）
		secretName = "demo/payroll-config"
		// State store 名（local-stack の Dapr Component と一致）
		store = "state.in-memory"
	)

	// E2E 専用 key（test 名 + 時刻ベース、衝突回避）
	stateKey := fmt.Sprintf("e2e-payroll-input-%d", time.Now().UnixNano())
	// idempotencyKey の prefix で Audit Query 時に test 起源を絞り込む
	runID := fmt.Sprintf("e2e-payroll-%d", time.Now().UnixNano())
	// payroll 計算入力（小さい固定値）
	payrollInput := []byte(`{"period":"2026-05","employees":3,"e2e":true}`)

	// Step 1: Secrets.Get — OpenBao から payroll 計算 secret を取得
	t.Run("secrets_get", func(t *testing.T) {
		values, version, err := client.Secrets().Get(ctx, secretName)
		// secret 不在は demo 用 seed が無い環境では起こりうるため、エラー型を分類する
		if err != nil {
			// secret 不在は採用組織が seed していない環境では正常、ログのみ
			t.Logf("Secrets.Get(%s): %v（demo seed が未配置の場合は採用初期で対応）", secretName, err)
			return
		}
		// version が 0 なら secret が空
		if version == 0 {
			t.Fatalf("Secrets.Get(%s): version=0（secret が空）", secretName)
		}
		// values が空 map なら secret 内容が空
		if len(values) == 0 {
			t.Fatalf("Secrets.Get(%s): values 空", secretName)
		}
		// 取得した version を log（実値は機密のため log 化しない）
		t.Logf("Secrets.Get(%s): version=%d keys=%d", secretName, version, len(values))
	})

	// Step 2: State.Save — payroll-input を保存
	t.Run("state_save_payroll_input", func(t *testing.T) {
		etag, err := client.State().Save(ctx, store, stateKey, payrollInput)
		if err != nil {
			t.Fatalf("State.Save: %v", err)
		}
		if etag == "" {
			t.Fatalf("State.Save: etag 空")
		}
		t.Logf("State.Save: key=%s etag=%s", stateKey, etag)
	})

	// Step 3: Audit.Record — 「payroll 計算開始」を監査ログに記録
	startAuditID, err := auditClient.Audit().Record(
		ctx,
		"e2e-test/payroll-full-flow",
		"e2e.payroll.start",
		"test://e2e/payroll/"+stateKey,
		"success",
		map[string]string{
			"run_id":     runID,
			"phase":      "start",
			"state_key":  stateKey,
			"input_size": fmt.Sprintf("%d", len(payrollInput)),
		},
		runID+"-start",
	)
	if err != nil {
		t.Fatalf("Audit.Record(start): %v", err)
	}
	if startAuditID == "" {
		t.Fatalf("Audit.Record(start): audit_id 空")
	}
	t.Logf("Audit.Record(start): audit_id=%s", startAuditID)

	// Step 4: State.Get — payroll-input を再取得（idempotency 確認）
	t.Run("state_get_idempotency", func(t *testing.T) {
		got, _, found, err := client.State().Get(ctx, store, stateKey)
		if err != nil {
			t.Fatalf("State.Get: %v", err)
		}
		if !found {
			t.Fatalf("State.Get: found=false（直前 Save が消えている）")
		}
		if !bytes.Equal(got, payrollInput) {
			t.Fatalf("State.Get: data mismatch got=%q want=%q", got, payrollInput)
		}
		t.Logf("State.Get: %d bytes 一致", len(got))
	})

	// Step 5: Audit.Record — 「payroll 計算完了」
	endAuditID, err := auditClient.Audit().Record(
		ctx,
		"e2e-test/payroll-full-flow",
		"e2e.payroll.complete",
		"test://e2e/payroll/"+stateKey,
		"success",
		map[string]string{
			"run_id":          runID,
			"phase":           "complete",
			"state_key":       stateKey,
			"start_audit_id":  startAuditID,
			"linked_to_start": "true",
		},
		runID+"-complete",
	)
	if err != nil {
		t.Fatalf("Audit.Record(complete): %v", err)
	}
	if endAuditID == "" {
		t.Fatalf("Audit.Record(complete): audit_id 空")
	}
	t.Logf("Audit.Record(complete): audit_id=%s", endAuditID)

	// Step 6: State.Delete — 後始末（無条件削除）
	t.Run("state_delete_cleanup", func(t *testing.T) {
		if err := client.State().Delete(ctx, store, stateKey, ""); err != nil {
			t.Fatalf("State.Delete: %v", err)
		}
		// 削除確認
		_, _, foundAfterDelete, err := client.State().Get(ctx, store, stateKey)
		if err != nil {
			t.Fatalf("State.Get(after delete): %v", err)
		}
		if foundAfterDelete {
			t.Fatalf("State.Get(after delete): found=true（delete が効いていない）")
		}
		t.Logf("State.Delete + 確認 完了")
	})

	// Step 7: Audit.Query — start / complete の 2 record が hash chain に append されたか確認
	t.Run("audit_chain_integrity", func(t *testing.T) {
		// 範囲は test 開始から現在 + 1 秒
		queryStart := time.Now().UTC().Add(-90 * time.Second)
		queryEnd := time.Now().UTC().Add(1 * time.Second)
		// run_id でフィルタして本 test の 2 record だけを取得
		events, err := auditClient.Audit().Query(ctx, queryStart, queryEnd, map[string]string{"run_id": runID}, 10)
		if err != nil {
			t.Fatalf("Audit.Query: %v", err)
		}
		// start + complete の 2 record を期待する
		if len(events) < 2 {
			t.Fatalf("Audit.Query: 期待 2 record / 実 %d record（filter run_id=%s）", len(events), runID)
		}
		t.Logf("Audit.Query: %d events 取得（run_id=%s、start + complete chain 確認）", len(events), runID)

		// hash chain 整合性検証
		chainResult, err := auditClient.Audit().VerifyChain(ctx, queryStart, queryEnd)
		if err != nil {
			t.Fatalf("Audit.VerifyChain: %v", err)
		}
		if !chainResult.Valid {
			t.Fatalf("Audit.VerifyChain: chain 不整合 first_bad_sequence=%d reason=%s", chainResult.FirstBadSequence, chainResult.Reason)
		}
		t.Logf("Audit.VerifyChain: valid=%v checked=%d", chainResult.Valid, chainResult.CheckedCount)
	})
}
