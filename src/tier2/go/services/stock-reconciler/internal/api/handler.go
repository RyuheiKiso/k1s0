// reconcile HTTP ハンドラ。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   `POST /reconcile/{sku}` の入出力を Application 層 UseCase に渡し、
//   結果を JSON で返す。エラーは t2errors.DomainError から HTTP status を写像する。

package api

// 標準 / 内部 import。
import (
	// JSON シリアライズ。
	"encoding/json"
	// HTTP server。
	"net/http"

	// reconcile UseCase。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/application/usecases"
	// tier2 共通エラー型。
	t2errors "github.com/k1s0/k1s0/src/tier2/go/shared/errors"
)

// reconcileHandler は reconcile エンドポイントのハンドラ。
type reconcileHandler struct {
	// Application 層 UseCase。
	useCase *usecases.ReconcileUseCase
}

// newReconcileHandler は reconcileHandler を組み立てる。
func newReconcileHandler(useCase *usecases.ReconcileUseCase) *reconcileHandler {
	// 構造体を返す。
	return &reconcileHandler{useCase: useCase}
}

// reconcileRequestBody は POST /reconcile/{sku} の JSON Body スキーマ。
type reconcileRequestBody struct {
	// authoritative 数量。
	Authoritative int64 `json:"authoritative"`
	// 同期元システム ID。
	SourceSystem string `json:"source_system"`
}

// reconcileResponseBody は成功時の JSON レスポンス。
type reconcileResponseBody struct {
	// SKU 文字列。
	SKU string `json:"sku"`
	// 反映前の数量。
	BeforeQuantity int64 `json:"before_quantity"`
	// 反映後の数量。
	AfterQuantity int64 `json:"after_quantity"`
	// 差分。
	Delta int64 `json:"delta"`
	// イベント発火成功フラグ。
	EventPublished bool `json:"event_published"`
	// イベント ID。
	EventID string `json:"event_id"`
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
	// カテゴリ（VALIDATION / UPSTREAM 等）。
	Category string `json:"category"`
}

// handleReconcile は POST /reconcile/{sku} を処理する。
func (h *reconcileHandler) handleReconcile(w http.ResponseWriter, r *http.Request) {
	// path parameter から SKU を取得する（Go 1.22 net/http の path value）。
	sku := r.PathValue("sku")
	// Body を JSON デコードする。
	var body reconcileRequestBody
	// JSON が空 / 不正なら 400。
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		// VALIDATION カテゴリで応答する。
		writeError(w, t2errors.Wrap(t2errors.CategoryValidation, "E-T2-RECON-100", "invalid request body", err))
		// 早期 return。
		return
	}
	// UseCase を実行する。
	result, err := h.useCase.Execute(r.Context(), usecases.ReconcileInput{
		// SKU 文字列を渡す。
		SKU: sku,
		// authoritative 数量を渡す。
		Authoritative: body.Authoritative,
		// 同期元システム ID を渡す。
		SourceSystem: body.SourceSystem,
	})
	// エラーは HTTP status に写像する。
	if err != nil {
		// DomainError → HTTP の写像。
		writeError(w, err)
		// 早期 return。
		return
	}
	// 成功レスポンスを組み立てる。
	resp := reconcileResponseBody{
		// 結果からそのままコピー。
		SKU:            result.SKU,
		BeforeQuantity: result.BeforeQuantity,
		AfterQuantity:  result.AfterQuantity,
		Delta:          result.Delta,
		EventPublished: result.EventPublished,
		EventID:        result.EventID,
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
		// 500 Internal Server Error。
		w.WriteHeader(http.StatusInternalServerError)
		// 汎用エラーを返す（PII 漏洩防止）。
		_ = json.NewEncoder(w).Encode(errorResponseBody{
			// エラー詳細を組み立てる。
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
			// 人間可読メッセージ（PII を含めない設計）。
			Message: domain.Message,
			// カテゴリ。
			Category: string(domain.Category),
		},
	})
}
