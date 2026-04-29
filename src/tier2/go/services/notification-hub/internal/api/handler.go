// dispatch HTTP ハンドラ。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

package api

// 標準 / 内部 import。
import (
	// JSON シリアライズ。
	"encoding/json"
	// HTTP server。
	"net/http"

	// dispatch UseCase。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/application/usecases"
	// 共通 auth middleware（context から tenant_id / subject を取り出す）。
	t2auth "github.com/k1s0/k1s0/src/tier2/go/shared/auth"
	// tier2 共通エラー型。
	t2errors "github.com/k1s0/k1s0/src/tier2/go/shared/errors"
	// k1s0 SDK の per-request tenant 上書き helper。
	"github.com/k1s0/sdk-go/k1s0"
)

// dispatchHandler は dispatch エンドポイントのハンドラ。
type dispatchHandler struct {
	// Application 層 UseCase。
	useCase *usecases.DispatchUseCase
}

// newDispatchHandler は dispatchHandler を組み立てる。
func newDispatchHandler(useCase *usecases.DispatchUseCase) *dispatchHandler {
	// 構造体を返す。
	return &dispatchHandler{useCase: useCase}
}

// dispatchRequestBody は POST /notify の JSON Body スキーマ。
type dispatchRequestBody struct {
	// チャネル文字列。
	Channel string `json:"channel"`
	// 受信者識別子。
	Recipient string `json:"recipient"`
	// 件名。
	Subject string `json:"subject"`
	// 本文。
	Body string `json:"body"`
	// 任意メタデータ。
	Metadata map[string]string `json:"metadata,omitempty"`
}

// dispatchResponseBody は成功時の JSON レスポンス。
type dispatchResponseBody struct {
	// 通知 ID。
	NotificationID string `json:"notification_id"`
	// 使用した Binding Component 名。
	BindingName string `json:"binding_name"`
	// チャネル文字列。
	Channel string `json:"channel"`
	// 配信成否。
	Success bool `json:"success"`
}

// errorResponseBody は失敗時の JSON レスポンス。
type errorResponseBody struct {
	// エラー詳細。
	Error errorDetail `json:"error"`
}

// errorDetail はエラー詳細。
type errorDetail struct {
	// E-T2-* コード。
	Code string `json:"code"`
	// 人間可読メッセージ。
	Message string `json:"message"`
	// カテゴリ。
	Category string `json:"category"`
}

// handleDispatch は POST /notify を処理する。
func (h *dispatchHandler) handleDispatch(w http.ResponseWriter, r *http.Request) {
	// Body を JSON デコードする。
	var body dispatchRequestBody
	// JSON が空 / 不正なら 400。
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		// VALIDATION で応答する。
		writeError(w, t2errors.Wrap(t2errors.CategoryValidation, "E-T2-NOTIF-100", "invalid request body", err))
		// 早期 return。
		return
	}
	// 認証済 tenant_id / subject を SDK 呼出に伝搬する（per-request 上書き）。
	// auth middleware が attach した値を使い、SDK 側 Config 既定値ではなく
	// 「実際にこのリクエストを発行したユーザのテナント」で tier1 を呼び出す。
	tenantID := t2auth.TenantIDFromContext(r.Context())
	subject := t2auth.SubjectFromContext(r.Context())
	ctx := r.Context()
	if tenantID != "" {
		ctx = k1s0.WithTenant(ctx, tenantID, subject)
	}
	// UseCase を実行する。
	result, err := h.useCase.Execute(ctx, usecases.DispatchInput{
		// チャネル文字列。
		Channel: body.Channel,
		// 受信者。
		Recipient: body.Recipient,
		// 件名。
		Subject: body.Subject,
		// 本文。
		Body: body.Body,
		// メタデータ。
		Metadata: body.Metadata,
	})
	// エラーは HTTP status に写像する。
	if err != nil {
		// DomainError → HTTP の写像。
		writeError(w, err)
		// 早期 return。
		return
	}
	// 成功レスポンスを組み立てる。
	resp := dispatchResponseBody{
		// 結果からそのままコピー。
		NotificationID: result.NotificationID,
		BindingName:    result.BindingName,
		Channel:        result.Channel,
		Success:        result.Success,
	}
	// JSON ヘッダを設定する。
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	// 200 OK。
	w.WriteHeader(http.StatusOK)
	// 本文をエンコードする。
	_ = json.NewEncoder(w).Encode(resp)
}

// writeError は error を JSON エラーレスポンスにシリアライズする。
func writeError(w http.ResponseWriter, err error) {
	// DomainError かどうかを判定する。
	domain, ok := t2errors.AsDomainError(err)
	// 非 DomainError は 500 + 汎用メッセージ。
	if !ok {
		// JSON ヘッダ。
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		// 500。
		w.WriteHeader(http.StatusInternalServerError)
		// 汎用エラーを返す（PII 漏洩防止）。
		_ = json.NewEncoder(w).Encode(errorResponseBody{
			// 詳細を組み立てる。
			Error: errorDetail{
				// 汎用コード。
				Code: "E-T2-INTERNAL",
				// 汎用メッセージ。
				Message: "internal error",
				// カテゴリ。
				Category: string(t2errors.CategoryInternal),
			},
		})
		// 早期 return。
		return
	}
	// JSON ヘッダ。
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	// HTTP status をカテゴリから決定する。
	w.WriteHeader(domain.Category.HTTPStatus())
	// エラー詳細を返す。
	_ = json.NewEncoder(w).Encode(errorResponseBody{
		// 詳細を組み立てる。
		Error: errorDetail{
			// E-T2-* コード。
			Code: domain.Code,
			// 人間可読メッセージ（PII を含めない）。
			Message: domain.Message,
			// カテゴリ。
			Category: string(domain.Category),
		},
	})
}
