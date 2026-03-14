package buildingblocks

import (
	"testing"
)

// BindingData の Data と Metadata フィールドが正しく設定されることを確認する。
func TestBindingDataCreation(t *testing.T) {
	bd := BindingData{
		Data:     []byte("input-data"),
		Metadata: map[string]string{"source": "kafka"},
	}
	if string(bd.Data) != "input-data" {
		t.Errorf("expected Data 'input-data', got %q", string(bd.Data))
	}
	if bd.Metadata["source"] != "kafka" {
		t.Errorf("expected metadata source=kafka, got %q", bd.Metadata["source"])
	}
}

// BindingData をゼロ値で生成すると Data と Metadata が nil であることを確認する。
func TestBindingDataEmpty(t *testing.T) {
	bd := BindingData{}
	if bd.Data != nil {
		t.Errorf("expected nil Data, got %v", bd.Data)
	}
	if bd.Metadata != nil {
		t.Errorf("expected nil Metadata, got %v", bd.Metadata)
	}
}

// BindingResponse の Data と Metadata フィールドが正しく設定されることを確認する。
func TestBindingResponseCreation(t *testing.T) {
	br := BindingResponse{
		Data:     []byte("response-data"),
		Metadata: map[string]string{"status": "ok"},
	}
	if string(br.Data) != "response-data" {
		t.Errorf("expected Data 'response-data', got %q", string(br.Data))
	}
	if br.Metadata["status"] != "ok" {
		t.Errorf("expected metadata status=ok, got %q", br.Metadata["status"])
	}
}

// BindingResponse をゼロ値で生成すると Data と Metadata が nil であることを確認する。
func TestBindingResponseEmpty(t *testing.T) {
	br := BindingResponse{}
	if br.Data != nil {
		t.Errorf("expected nil Data, got %v", br.Data)
	}
	if br.Metadata != nil {
		t.Errorf("expected nil Metadata, got %v", br.Metadata)
	}
}
