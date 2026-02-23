package searchclient

import (
	"context"
	"fmt"
	"strings"
	"sync"
)

// Filter はフィルター条件。
type Filter struct {
	Field    string      `json:"field"`
	Operator string      `json:"operator"` // "eq", "lt", "gt", "range", "in"
	Value    interface{} `json:"value"`
}

// FacetBucket はファセット集計バケット。
type FacetBucket struct {
	Value string `json:"value"`
	Count uint64 `json:"count"`
}

// SearchQuery は検索クエリ。
type SearchQuery struct {
	Query   string   `json:"query"`
	Filters []Filter `json:"filters,omitempty"`
	Facets  []string `json:"facets,omitempty"`
	Page    uint32   `json:"page"`
	Size    uint32   `json:"size"`
}

// SearchResult は検索結果。
type SearchResult struct {
	Hits   []map[string]interface{}    `json:"hits"`
	Total  uint64                      `json:"total"`
	Facets map[string][]FacetBucket    `json:"facets"`
	TookMs uint64                      `json:"took_ms"`
}

// IndexDocument はインデックス対象ドキュメント。
type IndexDocument struct {
	ID     string                 `json:"id"`
	Fields map[string]interface{} `json:"fields"`
}

// IndexResult はインデックス結果。
type IndexResult struct {
	ID      string `json:"id"`
	Version int64  `json:"version"`
}

// BulkFailure はバルクインデックスの個別失敗。
type BulkFailure struct {
	ID    string `json:"id"`
	Error string `json:"error"`
}

// BulkResult はバルクインデックス結果。
type BulkResult struct {
	SuccessCount int           `json:"success_count"`
	FailedCount  int           `json:"failed_count"`
	Failures     []BulkFailure `json:"failures"`
}

// FieldMapping はフィールドのマッピング定義。
type FieldMapping struct {
	FieldType string `json:"field_type"`
	Indexed   bool   `json:"indexed"`
}

// IndexMapping はインデックスマッピング。
type IndexMapping struct {
	Fields map[string]FieldMapping `json:"fields"`
}

// NewIndexMapping は新しい IndexMapping を生成する。
func NewIndexMapping() IndexMapping {
	return IndexMapping{Fields: make(map[string]FieldMapping)}
}

// WithField はフィールドを追加する。
func (m IndexMapping) WithField(name, fieldType string) IndexMapping {
	m.Fields[name] = FieldMapping{FieldType: fieldType, Indexed: true}
	return m
}

// SearchClient は検索クライアントのインターフェース。
type SearchClient interface {
	IndexDocument(ctx context.Context, index string, doc IndexDocument) (IndexResult, error)
	BulkIndex(ctx context.Context, index string, docs []IndexDocument) (BulkResult, error)
	Search(ctx context.Context, index string, query SearchQuery) (SearchResult, error)
	DeleteDocument(ctx context.Context, index, id string) error
	CreateIndex(ctx context.Context, name string, mapping IndexMapping) error
}

// InMemorySearchClient はメモリ内の検索クライアント。
type InMemorySearchClient struct {
	mu       sync.Mutex
	indexes  map[string][]IndexDocument
	mappings map[string]IndexMapping
}

// NewInMemorySearchClient は新しい InMemorySearchClient を生成する。
func NewInMemorySearchClient() *InMemorySearchClient {
	return &InMemorySearchClient{
		indexes:  make(map[string][]IndexDocument),
		mappings: make(map[string]IndexMapping),
	}
}

func (c *InMemorySearchClient) CreateIndex(_ context.Context, name string, mapping IndexMapping) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.indexes[name] = []IndexDocument{}
	c.mappings[name] = mapping
	return nil
}

func (c *InMemorySearchClient) IndexDocument(_ context.Context, index string, doc IndexDocument) (IndexResult, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	docs, ok := c.indexes[index]
	if !ok {
		return IndexResult{}, fmt.Errorf("index not found: %s", index)
	}
	c.indexes[index] = append(docs, doc)
	return IndexResult{
		ID:      doc.ID,
		Version: int64(len(c.indexes[index])),
	}, nil
}

func (c *InMemorySearchClient) BulkIndex(_ context.Context, index string, docs []IndexDocument) (BulkResult, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	existing, ok := c.indexes[index]
	if !ok {
		return BulkResult{}, fmt.Errorf("index not found: %s", index)
	}
	c.indexes[index] = append(existing, docs...)
	return BulkResult{
		SuccessCount: len(docs),
		FailedCount:  0,
		Failures:     []BulkFailure{},
	}, nil
}

func (c *InMemorySearchClient) Search(_ context.Context, index string, query SearchQuery) (SearchResult, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	docs, ok := c.indexes[index]
	if !ok {
		return SearchResult{}, fmt.Errorf("index not found: %s", index)
	}

	var hits []map[string]interface{}
	for _, doc := range docs {
		if query.Query == "" || matchesQuery(doc, query.Query) {
			hit := make(map[string]interface{})
			hit["id"] = doc.ID
			for k, v := range doc.Fields {
				hit[k] = v
			}
			hits = append(hits, hit)
		}
	}

	start := int(query.Page) * int(query.Size)
	if start > len(hits) {
		start = len(hits)
	}
	end := start + int(query.Size)
	if end > len(hits) {
		end = len(hits)
	}
	paged := hits[start:end]

	facets := make(map[string][]FacetBucket)
	for _, f := range query.Facets {
		facets[f] = []FacetBucket{{Value: "default", Count: uint64(len(paged))}}
	}

	return SearchResult{
		Hits:   paged,
		Total:  uint64(len(paged)),
		Facets: facets,
		TookMs: 1,
	}, nil
}

func matchesQuery(doc IndexDocument, query string) bool {
	for _, v := range doc.Fields {
		if s, ok := v.(string); ok && strings.Contains(s, query) {
			return true
		}
	}
	return false
}

func (c *InMemorySearchClient) DeleteDocument(_ context.Context, index, id string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	docs, ok := c.indexes[index]
	if !ok {
		return nil
	}
	filtered := make([]IndexDocument, 0, len(docs))
	for _, doc := range docs {
		if doc.ID != id {
			filtered = append(filtered, doc)
		}
	}
	c.indexes[index] = filtered
	return nil
}

// DocumentCount は指定インデックスのドキュメント数を返す。
func (c *InMemorySearchClient) DocumentCount(index string) int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return len(c.indexes[index])
}
