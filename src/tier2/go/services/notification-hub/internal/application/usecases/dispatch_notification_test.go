// DispatchUseCase の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

package usecases

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// 文字列比較。
	"errors"
	// JSON 検査。
	"encoding/json"
	// テスト frameworks。
	"testing"
	// 時刻取得。
	"time"

	// 設定。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/config"
	// tier2 共通エラー。
	t2errors "github.com/k1s0/k1s0/src/tier2/go/shared/errors"
)

// memBinding は BindingInvoker の in-memory mock。
type memBinding struct {
	// 呼出履歴。
	calls []bindingCall
	// 強制エラー（指定時のみ全 invoke が fail）。
	forceErr error
}

// bindingCall は 1 回分の呼出記録。
type bindingCall struct {
	// Component 名。
	name string
	// operation。
	operation string
	// payload bytes。
	data []byte
	// metadata。
	metadata map[string]string
}

// BindingInvoke は履歴に追加する。
func (m *memBinding) BindingInvoke(_ context.Context, name, operation string, data []byte, metadata map[string]string) ([]byte, map[string]string, error) {
	// 強制エラー時はそれを返す。
	if m.forceErr != nil {
		// テスト用エラー。
		return nil, nil, m.forceErr
	}
	// 履歴に追加する。
	m.calls = append(m.calls, bindingCall{name: name, operation: operation, data: data, metadata: metadata})
	// 仮レスポンス。
	return []byte("ok"), map[string]string{}, nil
}

// fixedNow は固定時刻を返す。
func fixedNow() time.Time {
	// 固定値。
	return time.Date(2026, 4, 27, 12, 0, 0, 0, time.UTC)
}

// newUseCaseForTest はテスト用に UseCase を組み立てる。
func newUseCaseForTest(binding BindingInvoker) *DispatchUseCase {
	// 構造体を組み立てる。
	return &DispatchUseCase{
		// Binding を保持する。
		binding: binding,
		// 既定の Binding 設定。
		bindings: config.BindingsConfig{
			// email チャネル。
			Email: "smtp-test",
			// slack チャネル。
			Slack: "slack-test",
			// webhook チャネル。
			Webhook: "http-test",
		},
		// 時刻を固定する。
		now: fixedNow,
	}
}

// TestExecute_EmailSuccess は email 配信の正常系を検証する。
func TestExecute_EmailSuccess(t *testing.T) {
	// in-memory binding。
	binding := &memBinding{}
	// UseCase。
	uc := newUseCaseForTest(binding)
	// 実行する。
	result, err := uc.Execute(context.Background(), DispatchInput{
		// チャネル。
		Channel: "email",
		// 受信者。
		Recipient: "user@example.com",
		// 件名。
		Subject: "Hello",
		// 本文。
		Body: "world",
	})
	// 成功するはず。
	if err != nil {
		// 失敗。
		t.Fatalf("Execute failed: %v", err)
	}
	// 成功フラグ。
	if !result.Success {
		// 失敗。
		t.Error("expected Success=true")
	}
	// Binding 名は smtp-test。
	if result.BindingName != "smtp-test" {
		// 失敗。
		t.Errorf("BindingName = %q, want smtp-test", result.BindingName)
	}
	// 履歴 1 件。
	if len(binding.calls) != 1 {
		// 失敗。
		t.Fatalf("expected 1 binding call, got %d", len(binding.calls))
	}
	// metadata に notification_id が入っていること。
	if binding.calls[0].metadata["notification_id"] == "" {
		// 失敗。
		t.Error("expected notification_id in metadata")
	}
	// payload を JSON デコードして to/subject/body を確認する。
	var p map[string]string
	// デコード。
	if err := json.Unmarshal(binding.calls[0].data, &p); err != nil {
		// 失敗。
		t.Fatalf("failed to decode payload: %v", err)
	}
	// 受信者一致。
	if p["to"] != "user@example.com" || p["subject"] != "Hello" || p["body"] != "world" {
		// 失敗。
		t.Errorf("unexpected payload: %v", p)
	}
}

// TestExecute_ValidationErrors は入力バリデーションを検証する。
func TestExecute_ValidationErrors(t *testing.T) {
	// in-memory binding。
	binding := &memBinding{}
	// UseCase。
	uc := newUseCaseForTest(binding)
	// 不正なチャネル。
	if _, err := uc.Execute(context.Background(), DispatchInput{Channel: "sms", Recipient: "x", Subject: "s", Body: "b"}); err == nil {
		// nil は失敗。
		t.Error("expected error for unknown channel")
	}
	// recipient 空。
	if _, err := uc.Execute(context.Background(), DispatchInput{Channel: "email", Recipient: "", Subject: "s", Body: "b"}); err == nil {
		// 失敗。
		t.Error("expected error for empty recipient")
	}
	// subject 空。
	if _, err := uc.Execute(context.Background(), DispatchInput{Channel: "email", Recipient: "x", Subject: "", Body: "b"}); err == nil {
		// 失敗。
		t.Error("expected error for empty subject")
	}
	// body 空。
	if _, err := uc.Execute(context.Background(), DispatchInput{Channel: "email", Recipient: "x", Subject: "s", Body: ""}); err == nil {
		// 失敗。
		t.Error("expected error for empty body")
	}
}

// TestExecute_BindingFailure は Binding 失敗を UPSTREAM カテゴリで返すことを検証する。
func TestExecute_BindingFailure(t *testing.T) {
	// 強制エラーつきの binding。
	binding := &memBinding{forceErr: errors.New("smtp connection refused")}
	// UseCase。
	uc := newUseCaseForTest(binding)
	// 実行する。
	_, err := uc.Execute(context.Background(), DispatchInput{Channel: "email", Recipient: "x", Subject: "s", Body: "b"})
	// エラーが返るはず。
	if err == nil {
		// 失敗。
		t.Fatal("expected error, got nil")
	}
	// DomainError であるはず。
	domain, ok := t2errors.AsDomainError(err)
	// 変換失敗は失敗。
	if !ok {
		// 失敗。
		t.Fatalf("expected DomainError, got %T: %v", err, err)
	}
	// UPSTREAM カテゴリのはず。
	if domain.Category != t2errors.CategoryUpstream {
		// 失敗。
		t.Errorf("Category = %q, want UPSTREAM", domain.Category)
	}
}

// TestExecute_BindingNotConfigured は Binding 未設定で INTERNAL を返すことを検証する。
func TestExecute_BindingNotConfigured(t *testing.T) {
	// in-memory binding。
	binding := &memBinding{}
	// UseCase（slack 未設定）。
	uc := &DispatchUseCase{
		// in-memory binding。
		binding: binding,
		// slack だけ未設定の Bindings。
		bindings: config.BindingsConfig{Email: "smtp", Slack: "", Webhook: "http"},
		// 時刻固定。
		now: fixedNow,
	}
	// slack を実行する。
	_, err := uc.Execute(context.Background(), DispatchInput{Channel: "slack", Recipient: "#dev", Subject: "s", Body: "b"})
	// エラーが返るはず。
	if err == nil {
		// 失敗。
		t.Fatal("expected error, got nil")
	}
	// DomainError であるはず。
	domain, ok := t2errors.AsDomainError(err)
	// 変換失敗は失敗。
	if !ok {
		// 失敗。
		t.Fatalf("expected DomainError, got %T: %v", err, err)
	}
	// INTERNAL カテゴリのはず。
	if domain.Category != t2errors.CategoryInternal {
		// 失敗。
		t.Errorf("Category = %q, want INTERNAL", domain.Category)
	}
}
