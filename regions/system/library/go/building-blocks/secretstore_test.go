package buildingblocks

import (
	"encoding/json"
	"testing"
)

// Secret の Key・Value・Metadata フィールドが正しく設定されることを確認する。
func TestSecretCreation(t *testing.T) {
	s := Secret{
		Key:      "db-password",
		Value:    "s3cret",
		Metadata: map[string]string{"source": "vault"},
	}
	if s.Key != "db-password" {
		t.Errorf("expected Key 'db-password', got %q", s.Key)
	}
	if s.Value != "s3cret" {
		t.Errorf("expected Value 's3cret', got %q", s.Value)
	}
	if s.Metadata["source"] != "vault" {
		t.Errorf("expected metadata source=vault, got %q", s.Metadata["source"])
	}
}

// Secret の JSON シリアライズ・デシリアライズが全フィールドを正しく復元することを確認する。
func TestSecretJSONRoundtrip(t *testing.T) {
	s := Secret{
		Key:      "api-key",
		Value:    "abc123",
		Metadata: map[string]string{"tier": "premium"},
	}
	data, err := json.Marshal(s)
	if err != nil {
		t.Fatalf("failed to marshal Secret: %v", err)
	}

	var decoded Secret
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal Secret: %v", err)
	}
	if decoded.Key != s.Key || decoded.Value != s.Value {
		t.Errorf("roundtrip mismatch: got %+v", decoded)
	}
	if decoded.Metadata["tier"] != "premium" {
		t.Errorf("expected metadata tier=premium, got %q", decoded.Metadata["tier"])
	}
}

// Secret の Metadata が nil のとき JSON フィールドが省略されることを確認する。
func TestSecretJSONOmitEmptyMetadata(t *testing.T) {
	s := Secret{Key: "key", Value: "val"}
	data, err := json.Marshal(s)
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}
	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}
	if _, ok := raw["metadata"]; ok {
		t.Error("expected 'metadata' to be omitted when nil")
	}
}
