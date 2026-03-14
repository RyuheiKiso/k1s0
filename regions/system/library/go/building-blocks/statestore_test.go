package buildingblocks

import (
	"encoding/json"
	"testing"
)

// TestETagCreation は ETag の各フィールドが正しく設定されることを確認する。
func TestETagCreation(t *testing.T) {
	etag := ETag{Value: "abc123"}
	if etag.Value != "abc123" {
		t.Errorf("expected Value 'abc123', got %q", etag.Value)
	}
}

// TestETagJSONRoundtrip は ETag の JSON シリアライズ・デシリアライズが正しく動作することを確認する。
func TestETagJSONRoundtrip(t *testing.T) {
	etag := ETag{Value: "v1"}
	data, err := json.Marshal(etag)
	if err != nil {
		t.Fatalf("failed to marshal ETag: %v", err)
	}
	var decoded ETag
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal ETag: %v", err)
	}
	if decoded.Value != etag.Value {
		t.Errorf("expected %q, got %q", etag.Value, decoded.Value)
	}
}

// TestStateEntryCreation は StateEntry の各フィールドが正しく設定されることを確認する。
func TestStateEntryCreation(t *testing.T) {
	etag := &ETag{Value: "e1"}
	entry := StateEntry{
		Key:   "user:1",
		Value: []byte(`{"name":"alice"}`),
		ETag:  etag,
	}
	if entry.Key != "user:1" {
		t.Errorf("expected Key 'user:1', got %q", entry.Key)
	}
	if string(entry.Value) != `{"name":"alice"}` {
		t.Errorf("expected Value %q, got %q", `{"name":"alice"}`, string(entry.Value))
	}
	if entry.ETag.Value != "e1" {
		t.Errorf("expected ETag value 'e1', got %q", entry.ETag.Value)
	}
}

// TestStateEntryJSONRoundtrip は StateEntry の JSON シリアライズ・デシリアライズが正しく動作することを確認する。
func TestStateEntryJSONRoundtrip(t *testing.T) {
	entry := StateEntry{
		Key:   "key1",
		Value: []byte("data"),
		ETag:  &ETag{Value: "etag1"},
	}
	data, err := json.Marshal(entry)
	if err != nil {
		t.Fatalf("failed to marshal StateEntry: %v", err)
	}
	var decoded StateEntry
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal StateEntry: %v", err)
	}
	if decoded.Key != entry.Key {
		t.Errorf("expected Key %q, got %q", entry.Key, decoded.Key)
	}
	if decoded.ETag == nil || decoded.ETag.Value != "etag1" {
		t.Errorf("expected ETag value 'etag1', got %v", decoded.ETag)
	}
}

// TestStateEntryJSONOmitEmptyETag は ETag が nil のとき JSON フィールドが省略されることを確認する。
func TestStateEntryJSONOmitEmptyETag(t *testing.T) {
	entry := StateEntry{Key: "k", Value: []byte("v")}
	data, err := json.Marshal(entry)
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}
	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}
	if _, ok := raw["etag"]; ok {
		t.Error("expected 'etag' to be omitted when nil")
	}
}

// TestSetRequestCreation は SetRequest の各フィールドが正しく設定されることを確認する。
func TestSetRequestCreation(t *testing.T) {
	req := SetRequest{
		Key:   "session:abc",
		Value: []byte("session-data"),
		ETag:  &ETag{Value: "prev"},
	}
	if req.Key != "session:abc" {
		t.Errorf("expected Key 'session:abc', got %q", req.Key)
	}
	if string(req.Value) != "session-data" {
		t.Errorf("expected Value %q, got %q", "session-data", string(req.Value))
	}
	if req.ETag.Value != "prev" {
		t.Errorf("expected ETag value 'prev', got %q", req.ETag.Value)
	}
}

// TestSetRequestJSONRoundtrip は SetRequest の JSON シリアライズ・デシリアライズが正しく動作することを確認する。
func TestSetRequestJSONRoundtrip(t *testing.T) {
	req := SetRequest{
		Key:   "k",
		Value: []byte("v"),
		ETag:  &ETag{Value: "e"},
	}
	data, err := json.Marshal(req)
	if err != nil {
		t.Fatalf("failed to marshal SetRequest: %v", err)
	}
	var decoded SetRequest
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal SetRequest: %v", err)
	}
	if decoded.Key != req.Key {
		t.Errorf("expected Key %q, got %q", req.Key, decoded.Key)
	}
	if decoded.ETag == nil || decoded.ETag.Value != "e" {
		t.Errorf("expected ETag value 'e', got %v", decoded.ETag)
	}
}
