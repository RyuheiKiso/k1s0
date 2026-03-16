package servercommon

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// /healthz エンドポイントが HTTP 200 と正しい JSON レスポンスを返すことを確認する。
func TestHealthzEndpoint(t *testing.T) {
	tests := []struct {
		name        string
		serviceName string
		path        string
	}{
		{
			name:        "healthz はステータス ok を返す",
			serviceName: "test-service",
			path:        "/healthz",
		},
		{
			name:        "readyz はステータス ok を返す",
			serviceName: "test-service",
			path:        "/readyz",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mux := http.NewServeMux()
			RegisterHealthHandlers(mux, tt.serviceName)

			req := httptest.NewRequest(http.MethodGet, tt.path, nil)
			w := httptest.NewRecorder()
			mux.ServeHTTP(w, req)

			if w.Code != http.StatusOK {
				t.Errorf("expected status 200, got %d", w.Code)
			}

			// Content-Type が application/json であることを確認する
			contentType := w.Header().Get("Content-Type")
			if contentType != "application/json" {
				t.Errorf("expected Content-Type application/json, got %s", contentType)
			}

			// JSON ボディの status と service フィールドを確認する
			var body HealthStatus
			if err := json.NewDecoder(w.Body).Decode(&body); err != nil {
				t.Fatalf("failed to decode response body: %v", err)
			}

			if body.Status != "ok" {
				t.Errorf("expected status 'ok', got '%s'", body.Status)
			}

			if body.Service != tt.serviceName {
				t.Errorf("expected service '%s', got '%s'", tt.serviceName, body.Service)
			}
		})
	}
}

// サービス名が空でも /healthz が正常に動作することを確認する。
func TestHealthzEmptyServiceName(t *testing.T) {
	mux := http.NewServeMux()
	RegisterHealthHandlers(mux, "")

	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	w := httptest.NewRecorder()
	mux.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected status 200, got %d", w.Code)
	}

	var body HealthStatus
	if err := json.NewDecoder(w.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response body: %v", err)
	}

	if body.Service != "" {
		t.Errorf("expected empty service, got '%s'", body.Service)
	}
}

// HealthStatus 構造体が正しく JSON シリアライズされることを確認する。
func TestHealthStatusJSON(t *testing.T) {
	status := HealthStatus{
		Status:  "ok",
		Service: "my-svc",
	}

	data, err := json.Marshal(status)
	if err != nil {
		t.Fatalf("failed to marshal HealthStatus: %v", err)
	}

	var restored HealthStatus
	if err := json.Unmarshal(data, &restored); err != nil {
		t.Fatalf("failed to unmarshal HealthStatus: %v", err)
	}

	if restored.Status != "ok" || restored.Service != "my-svc" {
		t.Errorf("roundtrip mismatch: got %+v", restored)
	}
}

// service フィールドが omitempty で、空の場合 JSON に含まれないことを確認する。
func TestHealthStatusOmitEmpty(t *testing.T) {
	status := HealthStatus{
		Status: "ok",
	}

	data, err := json.Marshal(status)
	if err != nil {
		t.Fatalf("failed to marshal HealthStatus: %v", err)
	}

	// "service" キーが JSON に含まれないことを確認する
	var m map[string]any
	if err := json.Unmarshal(data, &m); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}

	if _, exists := m["service"]; exists {
		t.Error("expected 'service' to be omitted from JSON when empty")
	}
}
