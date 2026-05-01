// 本ファイルは LoadFeatureCBRulesFromFile / PrometheusMetricSource の単体テスト。
//
// 検証観点:
//   - 空 path / "off" は no-op
//   - 必須フィールド（flag_key / promql）欠落は error
//   - comparator 既定 / 不正値の扱い
//   - recover_after_seconds 既定 5 分
//   - PrometheusMetricSource: vector / scalar / 結果なし / 5xx を正しく扱う

package state

import (
	"context"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func writeFile(t *testing.T, content string) string {
	t.Helper()
	dir := t.TempDir()
	p := filepath.Join(dir, "rules.json")
	if err := os.WriteFile(p, []byte(content), 0o600); err != nil {
		t.Fatalf("write tempfile: %v", err)
	}
	return p
}

func TestLoadFeatureCBRules_EmptyOrOffReturnsNil(t *testing.T) {
	for _, p := range []string{"", "off"} {
		rules, err := LoadFeatureCBRulesFromFile(p)
		if err != nil {
			t.Errorf("path %q: unexpected err %v", p, err)
		}
		if len(rules) != 0 {
			t.Errorf("path %q: expected empty rules, got %d", p, len(rules))
		}
	}
}

func TestLoadFeatureCBRules_ValidFile(t *testing.T) {
	p := writeFile(t, `[
  {"flag_key":"feature.alpha","promql":"sum(rate(errors[1m]))","threshold":0.5,"comparator":"gt","recover_after_seconds":120,"forced_false":true},
  {"flag_key":"feature.beta","promql":"vector(0)","threshold":0,"comparator":"lt"}
]`)
	rules, err := LoadFeatureCBRulesFromFile(p)
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	if len(rules) != 2 {
		t.Fatalf("len = %d; want 2", len(rules))
	}
	if rules[0].RecoverAfter != 2*time.Minute {
		t.Errorf("rules[0].RecoverAfter = %v; want 2m", rules[0].RecoverAfter)
	}
	if rules[1].RecoverAfter != 5*time.Minute {
		// recover_after_seconds 省略時は 5 分既定。
		t.Errorf("rules[1].RecoverAfter = %v; want 5m default", rules[1].RecoverAfter)
	}
	if rules[1].Comparator != "lt" {
		t.Errorf("rules[1].Comparator = %q; want lt", rules[1].Comparator)
	}
}

func TestLoadFeatureCBRules_RejectsMissingFields(t *testing.T) {
	cases := map[string]string{
		"missing flag_key": `[{"promql":"x","threshold":0}]`,
		"missing promql":   `[{"flag_key":"feature.x","threshold":0}]`,
		"unknown comparator": `[{"flag_key":"feature.x","promql":"x","threshold":0,"comparator":"eq"}]`,
		"bad json":         `[not-json`,
	}
	for name, content := range cases {
		t.Run(name, func(t *testing.T) {
			p := writeFile(t, content)
			_, err := LoadFeatureCBRulesFromFile(p)
			if err == nil {
				t.Errorf("expected error for %q", content)
			}
		})
	}
}

func TestPrometheusMetricSource_VectorResult(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Prometheus instant query 応答。先頭ベクタ値 = 0.7。
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{
			"status":"success",
			"data":{"resultType":"vector","result":[
				{"metric":{"job":"errors"},"value":[1700000000.123,"0.7"]}
			]}
		}`))
	}))
	defer srv.Close()
	src := NewPrometheusMetricSource(srv.URL, nil)
	v, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: "x"})
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	if v != 0.7 {
		t.Errorf("value = %v; want 0.7", v)
	}
}

func TestPrometheusMetricSource_EmptyResultReturnsZero(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"status":"success","data":{"resultType":"vector","result":[]}}`))
	}))
	defer srv.Close()
	src := NewPrometheusMetricSource(srv.URL, nil)
	v, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: "x"})
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	if v != 0 {
		t.Errorf("empty result should yield 0, got %v", v)
	}
}

func TestPrometheusMetricSource_5xxReturnsError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "boom", http.StatusInternalServerError)
	}))
	defer srv.Close()
	src := NewPrometheusMetricSource(srv.URL, nil)
	_, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: "x"})
	if err == nil {
		t.Error("expected error on 5xx")
	}
}

func TestPrometheusMetricSource_StatusError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"status":"error","error":"parse error: bad query"}`))
	}))
	defer srv.Close()
	src := NewPrometheusMetricSource(srv.URL, nil)
	_, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: "bad"})
	if err == nil {
		t.Error("expected error on status=error")
	}
}

func TestPrometheusMetricSource_ScalarResult(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{
			"status":"success",
			"data":{"resultType":"scalar","result":[{"metric":{},"value":[1700000000,"42"]}]}
		}`))
	}))
	defer srv.Close()
	src := NewPrometheusMetricSource(srv.URL, nil)
	v, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: "x"})
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	if v != 42 {
		t.Errorf("scalar = %v; want 42", v)
	}
}

func TestPrometheusMetricSource_QueryURLEscaping(t *testing.T) {
	// PromQL は URL エンコードされて送信される必要がある。"+" や " " が壊れないことを確認。
	gotQuery := ""
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotQuery = r.URL.Query().Get("query")
		_, _ = w.Write([]byte(`{"status":"success","data":{"resultType":"vector","result":[]}}`))
	}))
	defer srv.Close()
	src := NewPrometheusMetricSource(srv.URL, nil)
	expected := `sum by (tenant_id) (rate(errors[1m])) > 0.5`
	_, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: expected})
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	if gotQuery != expected {
		t.Errorf("server received query %q; want %q", gotQuery, expected)
	}
}

func TestPrometheusMetricSource_RejectsEmptyURL(t *testing.T) {
	src := NewPrometheusMetricSource("", nil)
	_, err := src.Evaluate(context.Background(), FeatureCBRule{FlagKey: "f", PromQL: "x"})
	if err == nil {
		t.Error("expected error for empty BaseURL")
	}
}
