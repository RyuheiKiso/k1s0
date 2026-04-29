// 本ファイルは tier1 facade HTTP/JSON gateway の fuzz target（Go 1.18+ 標準 fuzzing）。
//
// docs 正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」: protojson 直列化
//   docs/03_要件定義/30_非機能要件/H_完全性.md（NFR-H-INT-*）
//
// 目的:
//   tier1 facade の HTTP/JSON gateway は外部から JSON bytes を受け取る経路があるため、
//   decoder の crash / DoS は SEV1 直結。代表的な request 型 4 種について
//   protojson.Unmarshal が任意の入力で panic / OOM / 無限ループしないことを保証する。
//
// 起動:
//   go test -fuzz=FuzzStateSetJSON -fuzztime=5m ./targets/
//   go test -fuzz=FuzzAuditRecordJSON -fuzztime=5m ./targets/

package targets

import (
	"testing"

	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"

	"google.golang.org/protobuf/encoding/protojson"
)

// FuzzStateSetJSON は State.Set の SetRequest JSON を protojson.Unmarshal で
// 任意 byte 列に対し crash しないことを fuzz で確認する。
func FuzzStateSetJSON(f *testing.F) {
	// 正常 seed（共通規約 §「HTTP/JSON 互換」の minimum 例）。
	f.Add([]byte(`{"store":"kv","key":"k","data":"aGVsbG8=","expectedEtag":"","ttlSec":0,"idempotencyKey":"k1","context":{"tenantId":"T1","subject":"u1"}}`))
	f.Add([]byte(`{}`))
	f.Add([]byte(`{"store":"","key":"","data":""}`))
	f.Fuzz(func(t *testing.T, data []byte) {
		var req statev1.SetRequest
		// 失敗は OK（仕様外入力）。panic は NG。defer recover で観測する。
		defer func() {
			if r := recover(); r != nil {
				t.Errorf("protojson.Unmarshal SetRequest panic: %v / data=%q", r, data)
			}
		}()
		_ = protojson.Unmarshal(data, &req)
	})
}

// FuzzAuditRecordJSON は Audit.Record の RecordAuditRequest JSON を fuzz する。
func FuzzAuditRecordJSON(f *testing.F) {
	f.Add([]byte(`{"event":{"actor":"u","action":"R","resource":"r","outcome":"SUCCESS"},"context":{"tenantId":"T1"},"idempotencyKey":"k1"}`))
	f.Add([]byte(`{"event":{}}`))
	f.Add([]byte(`{"event":{"timestamp":"2026-01-01T00:00:00Z"}}`))
	f.Fuzz(func(t *testing.T, data []byte) {
		var req auditv1.RecordAuditRequest
		defer func() {
			if r := recover(); r != nil {
				t.Errorf("protojson.Unmarshal RecordAuditRequest panic: %v / data=%q", r, data)
			}
		}()
		_ = protojson.Unmarshal(data, &req)
	})
}

// FuzzPubSubPublishJSON は PubSub.Publish の PublishRequest JSON を fuzz する。
func FuzzPubSubPublishJSON(f *testing.F) {
	f.Add([]byte(`{"topic":"orders","data":"aGVsbG8=","contentType":"application/json","context":{"tenantId":"T1"}}`))
	f.Add([]byte(`{"topic":""}`))
	f.Fuzz(func(t *testing.T, data []byte) {
		var req pubsubv1.PublishRequest
		defer func() {
			if r := recover(); r != nil {
				t.Errorf("protojson.Unmarshal PublishRequest panic: %v / data=%q", r, data)
			}
		}()
		_ = protojson.Unmarshal(data, &req)
	})
}

// FuzzWorkflowStartJSON は Workflow.Start の StartRequest JSON を fuzz する。
func FuzzWorkflowStartJSON(f *testing.F) {
	f.Add([]byte(`{"workflowType":"OrderProcess","workflowId":"order-1","input":"e30=","idempotent":true,"context":{"tenantId":"T1"},"backend":1}`))
	f.Add([]byte(`{"backend":99}`))
	f.Fuzz(func(t *testing.T, data []byte) {
		var req workflowv1.StartRequest
		defer func() {
			if r := recover(); r != nil {
				t.Errorf("protojson.Unmarshal StartRequest panic: %v / data=%q", r, data)
			}
		}()
		_ = protojson.Unmarshal(data, &req)
	})
}
