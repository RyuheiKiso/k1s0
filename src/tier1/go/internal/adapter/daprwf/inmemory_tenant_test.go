// 本ファイルは daprwf in-memory backend のテナント越境防止（NFR-E-AC-003）動作確認テスト。
//
// 検証ポイント:
//   1. tenant-A の workflow を tenant-B が GetStatus すると ErrNotFound
//   2. tenant-A の workflow を tenant-B が Cancel しても他 tenant に影響しない
//   3. Signal / Query / Terminate も同様に tenant 境界を守る
//   4. tenantID が不一致の場合 NotFound を返し、PermissionDenied は使わない
//      （他 tenant の存在を漏らさない設計）

package daprwf

import (
	"context"
	"errors"
	"testing"
)

// 1. GetStatus: tenant-A の run を tenant-B が見ても ErrNotFound。
func TestInMemoryWorkflow_GetStatus_TenantIsolation(t *testing.T) {
	// 空 backend を作る。
	m := NewInMemoryWorkflow()
	// tenant-A で wf-001 を起動する。
	resp, err := m.Start(context.Background(), StartRequest{
		WorkflowID:   "wf-001",
		WorkflowType: "test",
		TenantID:     "tenant-A",
	})
	// Start 失敗は backend バグ。
	if err != nil {
		t.Fatalf("Start: %v", err)
	}
	// tenant-A 自身は GetStatus できる。
	if _, err := m.GetStatus(context.Background(), GetStatusRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-A",
	}); err != nil {
		t.Fatalf("GetStatus(A): %v", err)
	}
	// tenant-B は ErrNotFound 期待（PermissionDenied ではなく、存在自体を漏らさない設計）。
	_, err = m.GetStatus(context.Background(), GetStatusRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-B",
	})
	// nil error は越境バグ。
	if err == nil {
		t.Fatalf("NFR-E-AC-003 violation: tenant-B got tenant-A workflow status")
	}
	// errors.Is で ErrNotFound を確認する。
	if !errors.Is(err, ErrNotFound) {
		t.Fatalf("want ErrNotFound, got %v", err)
	}
}

// 2. Cancel: tenant-B の Cancel は tenant-A の workflow に影響しない。
func TestInMemoryWorkflow_Cancel_TenantIsolation(t *testing.T) {
	// 空 backend を作る。
	m := NewInMemoryWorkflow()
	// tenant-A で wf-001 を起動する。
	resp, err := m.Start(context.Background(), StartRequest{
		WorkflowID:   "wf-001",
		WorkflowType: "test",
		TenantID:     "tenant-A",
	})
	// Start 失敗は backend バグ。
	if err != nil {
		t.Fatalf("Start: %v", err)
	}
	// tenant-B が Cancel を試みる → ErrNotFound（影響なし）。
	err = m.Cancel(context.Background(), CancelRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-B",
	})
	// errors.Is で ErrNotFound を確認する。
	if !errors.Is(err, ErrNotFound) {
		t.Fatalf("Cancel(B): want ErrNotFound got %v", err)
	}
	// tenant-A の status は依然 Running 期待。
	statusResp, err := m.GetStatus(context.Background(), GetStatusRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-A",
	})
	// err nil 期待。
	if err != nil {
		t.Fatalf("GetStatus(A): %v", err)
	}
	// status は StatusRunning 期待（B の Cancel に影響されていない）。
	if statusResp.Status != StatusRunning {
		t.Fatalf("status changed by other tenant: want %v got %v", StatusRunning, statusResp.Status)
	}
}

// 3. Signal / Query / Terminate も tenant 境界を守る。
func TestInMemoryWorkflow_AllOps_TenantIsolation(t *testing.T) {
	// 空 backend を作る。
	m := NewInMemoryWorkflow()
	// tenant-A で wf-001 を起動する。
	resp, err := m.Start(context.Background(), StartRequest{
		WorkflowID:   "wf-001",
		WorkflowType: "test",
		TenantID:     "tenant-A",
	})
	// Start 失敗は backend バグ。
	if err != nil {
		t.Fatalf("Start: %v", err)
	}

	// Signal: tenant-B から signal を送ると ErrNotFound。
	if err := m.Signal(context.Background(), SignalRequest{
		WorkflowID: resp.WorkflowID, SignalName: "x", TenantID: "tenant-B",
	}); !errors.Is(err, ErrNotFound) {
		t.Fatalf("Signal(B): want ErrNotFound got %v", err)
	}
	// Query: 同様。
	if _, err := m.Query(context.Background(), QueryRequest{
		WorkflowID: resp.WorkflowID, QueryName: "x", TenantID: "tenant-B",
	}); !errors.Is(err, ErrNotFound) {
		t.Fatalf("Query(B): want ErrNotFound got %v", err)
	}
	// Terminate: 同様。
	if err := m.Terminate(context.Background(), TerminateRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-B",
	}); !errors.Is(err, ErrNotFound) {
		t.Fatalf("Terminate(B): want ErrNotFound got %v", err)
	}
	// tenant-A は依然として Running（B の操作で影響されていない）。
	statusResp, err := m.GetStatus(context.Background(), GetStatusRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-A",
	})
	// err nil 期待。
	if err != nil {
		t.Fatalf("GetStatus(A): %v", err)
	}
	// status は StatusRunning 期待。
	if statusResp.Status != StatusRunning {
		t.Fatalf("status changed: want %v got %v", StatusRunning, statusResp.Status)
	}
}

// 4. tenant-A 内では Cancel が正常に動く（テナント越境ではない）。
func TestInMemoryWorkflow_Cancel_WithinTenant(t *testing.T) {
	// 空 backend を作る。
	m := NewInMemoryWorkflow()
	// tenant-A で wf-001 を起動する。
	resp, err := m.Start(context.Background(), StartRequest{
		WorkflowID:   "wf-001",
		WorkflowType: "test",
		TenantID:     "tenant-A",
	})
	// Start 失敗は backend バグ。
	if err != nil {
		t.Fatalf("Start: %v", err)
	}
	// tenant-A 自身の Cancel は成功。
	if err := m.Cancel(context.Background(), CancelRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-A",
	}); err != nil {
		t.Fatalf("Cancel(A): %v", err)
	}
	// status は StatusCanceled 期待。
	statusResp, err := m.GetStatus(context.Background(), GetStatusRequest{
		WorkflowID: resp.WorkflowID, TenantID: "tenant-A",
	})
	// err nil 期待。
	if err != nil {
		t.Fatalf("GetStatus(A): %v", err)
	}
	// 同 tenant 内では Cancel が反映される。
	if statusResp.Status != StatusCanceled {
		t.Fatalf("status: want %v got %v", StatusCanceled, statusResp.Status)
	}
}
