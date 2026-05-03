// tests/e2e/owner/helpers/tempo_client.go
//
// observability/ の検証 1 (OTLP trace 貫通) で使う Tempo HTTP API client。
//
// 設計正典:
//   ADR-TEST-009（観測性 E2E 検証 1: trace propagation）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/04_観測性5検証.md
package helpers

import (
	// context は HTTP request の cancellation で使う
	"context"
	// encoding/json は Tempo API response の decode で使う
	"encoding/json"
	// fmt は URL 組み立てとエラー format で使う
	"fmt"
	// io は response body 読み込み
	"io"
	// net/http は Tempo HTTP API client
	"net/http"
	// time は HTTP timeout で使う
	"time"
)

// TempoClient は Tempo HTTP API への薄い wrapper。
// owner suite の cluster 内で port-forward 経由で叩く想定（base URL = http://localhost:3200 等）。
type TempoClient struct {
	// BaseURL は Tempo の HTTP listen address（例: http://localhost:3200）
	BaseURL string
	// HTTPClient は default 30 秒 timeout の net/http.Client
	HTTPClient *http.Client
}

// NewTempoClient は base URL を受け取って TempoClient を生成する。
// timeout は 30 秒 default、長時間 trace 検索には不適なので呼び出し側で context cancel を使う。
func NewTempoClient(baseURL string) *TempoClient {
	return &TempoClient{
		BaseURL:    baseURL,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// SpanTree は Tempo /api/traces/<trace-id> response の JSON 構造（簡略版）。
// 検証 1 では batches 数 + service name の集合を assert に使う。
type SpanTree struct {
	// Batches は trace を構成する batch 一覧（service ごとに 1 batch 想定）
	Batches []SpanBatch `json:"batches"`
}

// SpanBatch は 1 service 分の span 集合
type SpanBatch struct {
	// Resource は service.name 等の attribute
	Resource SpanResource `json:"resource"`
	// ScopeSpans は scope（library 単位）ごとの span 群
	ScopeSpans []SpanScope `json:"scopeSpans"`
}

// SpanResource は OpenTelemetry resource attribute
type SpanResource struct {
	Attributes []SpanAttribute `json:"attributes"`
}

// SpanScope は instrumentation scope ごとの span 集合
type SpanScope struct {
	Spans []SpanItem `json:"spans"`
}

// SpanAttribute は key-value attribute
type SpanAttribute struct {
	Key   string         `json:"key"`
	Value SpanAttrValue  `json:"value"`
}

// SpanAttrValue は protobuf any-of の simplified subset
type SpanAttrValue struct {
	StringValue string `json:"stringValue,omitempty"`
}

// SpanItem は 1 span 分（簡略版、検証 1 では parent / span name のみ参照）
type SpanItem struct {
	Name         string `json:"name"`
	ParentSpanId string `json:"parentSpanId,omitempty"`
}

// GetTraceByID は Tempo の /api/traces/<trace-id> を叩いて SpanTree を取得する。
// 404（trace 不在）は error にせず nil を返す（caller が retry 判定する）。
func (c *TempoClient) GetTraceByID(ctx context.Context, traceID string) (*SpanTree, error) {
	// URL 組み立て
	url := fmt.Sprintf("%s/api/traces/%s", c.BaseURL, traceID)
	// HTTP GET request 構築
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("Tempo request 構築失敗: %w", err)
	}
	// HTTP 実行
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("Tempo HTTP 失敗: %w", err)
	}
	defer resp.Body.Close()
	// 404 は trace 不在として nil 返却（caller が wait + retry）
	if resp.StatusCode == http.StatusNotFound {
		return nil, nil
	}
	// 200 以外は error
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("Tempo unexpected status: %d", resp.StatusCode)
	}
	// body 読み込み
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("Tempo response body 読み込み失敗: %w", err)
	}
	// JSON decode
	var tree SpanTree
	if err := json.Unmarshal(body, &tree); err != nil {
		return nil, fmt.Errorf("Tempo response decode 失敗: %w", err)
	}
	return &tree, nil
}

// ServiceNames は SpanTree から service.name 値の集合を取り出す。
// 検証 1 で「tier1-state / tier2-service / tier3-bff の 3 service が span を持つ」を assert する。
func (t *SpanTree) ServiceNames() []string {
	names := []string{}
	for _, batch := range t.Batches {
		for _, attr := range batch.Resource.Attributes {
			if attr.Key == "service.name" && attr.Value.StringValue != "" {
				names = append(names, attr.Value.StringValue)
				break
			}
		}
	}
	return names
}
