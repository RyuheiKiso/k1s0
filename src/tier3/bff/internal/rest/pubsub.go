// PubSub の REST エンドポイント。
//
//	POST /api/pubsub/publish — k1s0 PubSub にメッセージを発行する

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// pubsubPublishRequest は POST /api/pubsub/publish の入力。
type pubsubPublishRequest struct {
	Topic          string            `json:"topic"`
	Data           string            `json:"data"`
	ContentType    string            `json:"content_type,omitempty"`
	IdempotencyKey string            `json:"idempotency_key,omitempty"`
	Metadata       map[string]string `json:"metadata,omitempty"`
}

// pubsubPublishResponse は POST /api/pubsub/publish の出力。
type pubsubPublishResponse struct {
	Offset int64 `json:"offset"`
}

// registerPubSub は PubSub 系 endpoint を mux に登録する。
func (r *Router) registerPubSub(mux *http.ServeMux) {
	// PubSub.Publish。
	mux.HandleFunc("POST /api/pubsub/publish", r.handlePubSubPublish)
}

// handlePubSubPublish は POST /api/pubsub/publish を処理する。
func (r *Router) handlePubSubPublish(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body pubsubPublishRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Topic == "" {
		writeBadRequest(w, "E-T3-BFF-PUBSUB-100", "topic is required")
		return
	}
	// content_type の既定は application/json。
	contentType := body.ContentType
	if contentType == "" {
		contentType = "application/json"
	}
	// facade 経由で SDK を呼ぶ。
	offset, err := r.facade.PubSubPublish(req.Context(), body.Topic, []byte(body.Data), contentType, body.IdempotencyKey, body.Metadata)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-PUBSUB-200", "publish failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, pubsubPublishResponse{Offset: offset})
}
