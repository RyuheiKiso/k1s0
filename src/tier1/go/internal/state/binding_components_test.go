// 本ファイルは FR-T1-BINDING-002 / 003 / 004 の handler / helper 単体テスト。
//
// 試験戦略:
//   - 純関数 validateBindingMetadata の type 別検証 (SMTP / HTTP / generic)
//   - handler 経由で missing metadata が adapter 呼出前に InvalidArgument に翻訳されること
//
// 検証する不変式:
//   1. SMTP Component (name="smtp-*") は emailTo / subject 必須
//   2. HTTP Component (name="http-*") は path 必須
//   3. 識別不能 name (minio-* / s3-* / cron-* / 任意) は検証 skip
//   4. handler 段で missing metadata は adapter を呼ばずに InvalidArgument を返す
//      (Dapr の codes.Internal に潰れる経路を遮断)

package state

import (
	// 標準 context。
	"context"
	// テスト fail / 報告。
	"testing"

	// adapter 型参照。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK proto stub。
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// TestValidateBindingMetadata_SMTP_HappyPath は SMTP Component に必要な
// metadata がすべて揃っていれば nil 返却することを確認する。
func TestValidateBindingMetadata_SMTP_HappyPath(t *testing.T) {
	// 必須 metadata 完備の正常入力。
	err := validateBindingMetadata("smtp-outbound", map[string]string{
		"emailTo": "ops@example.com",
		"subject": "system notice",
	})
	// 期待: nil。
	if err != nil {
		t.Fatalf("validateBindingMetadata(smtp-outbound, full) = %v, want nil", err)
	}
}

// TestValidateBindingMetadata_SMTP_MissingEmailToReturnsInvalidArg は
// emailTo 欠落で InvalidArgument を返すことを確認する。
func TestValidateBindingMetadata_SMTP_MissingEmailToReturnsInvalidArg(t *testing.T) {
	// emailTo 欠落 / subject あり。
	err := validateBindingMetadata("smtp-outbound", map[string]string{
		"subject": "x",
	})
	// gRPC code が InvalidArgument であること。
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status code = %v, want InvalidArgument (FR-T1-BINDING-002 emailTo)", got)
	}
}

// TestValidateBindingMetadata_SMTP_MissingSubjectReturnsInvalidArg は
// subject 欠落で InvalidArgument を返すことを確認する。
func TestValidateBindingMetadata_SMTP_MissingSubjectReturnsInvalidArg(t *testing.T) {
	// emailTo あり / subject 欠落。
	err := validateBindingMetadata("smtp-outbound", map[string]string{
		"emailTo": "x@y",
	})
	// gRPC code が InvalidArgument であること。
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status code = %v, want InvalidArgument (FR-T1-BINDING-002 subject)", got)
	}
}

// TestValidateBindingMetadata_HTTP_HappyPath は HTTP Component に必要な
// path metadata があれば nil 返却することを確認する。
func TestValidateBindingMetadata_HTTP_HappyPath(t *testing.T) {
	// path 入り正常入力。
	err := validateBindingMetadata("http-outbound", map[string]string{
		"path": "/api/v1/event",
	})
	// 期待: nil。
	if err != nil {
		t.Fatalf("validateBindingMetadata(http-outbound, path) = %v, want nil", err)
	}
}

// TestValidateBindingMetadata_HTTP_MissingPathReturnsInvalidArg は
// HTTP Component で path 欠落時に InvalidArgument を返すことを確認する。
func TestValidateBindingMetadata_HTTP_MissingPathReturnsInvalidArg(t *testing.T) {
	// path 欠落の不正入力。
	err := validateBindingMetadata("http-outbound", map[string]string{})
	// gRPC code が InvalidArgument であること。
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status code = %v, want InvalidArgument (FR-T1-BINDING-003 path)", got)
	}
}

// TestValidateBindingMetadata_GenericPasses は識別不能 name に対して
// 検証 skip (常に nil) を確認する。
// minio-* / s3-* / cron-* / 採用組織のカスタム binding を generic として通過させる経路。
func TestValidateBindingMetadata_GenericPasses(t *testing.T) {
	// MinIO / S3 / 任意名前で空 metadata でも通過すること。
	for _, name := range []string{"minio-archive", "s3-inbound", "cron-daily", "custom-foo"} {
		err := validateBindingMetadata(name, map[string]string{})
		// 期待: nil (Dapr に検証を委譲)。
		if err != nil {
			t.Fatalf("validateBindingMetadata(%s, empty) = %v, want nil (generic skip)", name, err)
		}
	}
}

// TestBindingHandler_Invoke_SMTPMissingEmailToBlocksAdapter は
// handler 段で missing metadata が adapter 呼出前に InvalidArgument に翻訳され、
// adapter (dapr SDK) が呼ばれないことを確認する。
//
// 失敗時の意味: Dapr SDK 越しに codes.Internal が tier2 に漏れる経路が再発した
// (FR-T1-BINDING-002 受け入れ基準と schemathesis E2 の修正経緯両方に違反)。
func TestBindingHandler_Invoke_SMTPMissingEmailToBlocksAdapter(t *testing.T) {
	// adapter は呼ばれてはならないことを fail で表明する fake。
	a := &fakeBindingAdapter{
		fn: func(_ context.Context, _ dapr.BindingRequest) (dapr.BindingResponse, error) {
			t.Fatalf("adapter must not be called when SMTP emailTo is missing (FR-T1-BINDING-002)")
			return dapr.BindingResponse{}, nil
		},
	}
	// handler を fake adapter で構築。
	h := &bindingHandler{deps: Deps{BindingAdapter: a}}
	// emailTo 欠落 + tenant 設定済の Invoke 呼出。
	_, err := h.Invoke(context.Background(), &bindingv1.InvokeBindingRequest{
		Name:      "smtp-outbound",
		Operation: "create",
		Metadata:  map[string]string{"subject": "test"},
		Context:   makeTenantCtx("T"),
	})
	// gRPC code が InvalidArgument であること。
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("handler status code = %v, want InvalidArgument", got)
	}
}
