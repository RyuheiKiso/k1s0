package searchclient_test

import (
	"context"
	"testing"

	searchclient "github.com/k1s0-platform/system-library-go-search-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// CreateIndexが新しい検索インデックスを正常に作成することを確認する。
func TestCreateIndex(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	mapping := searchclient.NewIndexMapping().WithField("name", "text")
	err := c.CreateIndex(ctx, "products", mapping)
	require.NoError(t, err)
}

// IndexDocumentがドキュメントを検索インデックスに正常に登録し、IDとバージョンを返すことを確認する。
func TestIndexDocument(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	_ = c.CreateIndex(ctx, "products", searchclient.NewIndexMapping())

	doc := searchclient.IndexDocument{
		ID:     "prod-001",
		Fields: map[string]interface{}{"name": "Rust Programming"},
	}
	result, err := c.IndexDocument(ctx, "products", doc)
	require.NoError(t, err)
	assert.Equal(t, "prod-001", result.ID)
	assert.Equal(t, int64(1), result.Version)
}

// 存在しないインデックスにドキュメントを登録しようとした際にエラーが返ることを確認する。
func TestIndexDocument_IndexNotFound(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	doc := searchclient.IndexDocument{ID: "1", Fields: map[string]interface{}{}}
	_, err := c.IndexDocument(ctx, "nonexistent", doc)
	require.Error(t, err)
}

// BulkIndexが複数のドキュメントを一括登録し、成功件数と失敗件数を正しく返すことを確認する。
func TestBulkIndex(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	_ = c.CreateIndex(ctx, "items", searchclient.NewIndexMapping())

	docs := []searchclient.IndexDocument{
		{ID: "i-1", Fields: map[string]interface{}{"name": "Item 1"}},
		{ID: "i-2", Fields: map[string]interface{}{"name": "Item 2"}},
		{ID: "i-3", Fields: map[string]interface{}{"name": "Item 3"}},
	}
	result, err := c.BulkIndex(ctx, "items", docs)
	require.NoError(t, err)
	assert.Equal(t, 3, result.SuccessCount)
	assert.Equal(t, 0, result.FailedCount)
	assert.Empty(t, result.Failures)
}

// Searchがクエリに一致するドキュメントを返し、ファセット情報も含めて返すことを確認する。
func TestSearch(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	_ = c.CreateIndex(ctx, "products", searchclient.NewIndexMapping())

	_, _ = c.IndexDocument(ctx, "products", searchclient.IndexDocument{
		ID:     "p-1",
		Fields: map[string]interface{}{"name": "Rust Programming", "category": "books"},
	})
	_, _ = c.IndexDocument(ctx, "products", searchclient.IndexDocument{
		ID:     "p-2",
		Fields: map[string]interface{}{"name": "Go Language", "category": "books"},
	})

	result, err := c.Search(ctx, "products", searchclient.SearchQuery{
		Query:  "Rust",
		Facets: []string{"category"},
		Page:   0,
		Size:   20,
	})
	require.NoError(t, err)
	assert.Equal(t, uint64(1), result.Total)
	assert.Len(t, result.Hits, 1)
	assert.Contains(t, result.Facets, "category")
}

// 存在しないインデックスに対してSearchを呼び出した際にエラーが返ることを確認する。
func TestSearch_IndexNotFound(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	_, err := c.Search(ctx, "nonexistent", searchclient.SearchQuery{Query: "test"})
	require.Error(t, err)
}

// DeleteDocumentがインデックスから指定したドキュメントを正常に削除することを確認する。
func TestDeleteDocument(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	_ = c.CreateIndex(ctx, "products", searchclient.NewIndexMapping())
	_, _ = c.IndexDocument(ctx, "products", searchclient.IndexDocument{
		ID: "p-1", Fields: map[string]interface{}{"name": "Test"},
	})

	err := c.DeleteDocument(ctx, "products", "p-1")
	require.NoError(t, err)
	assert.Equal(t, 0, c.DocumentCount("products"))
}

// 空のクエリでSearchを呼び出した際にインデックス内の全ドキュメントが返ることを確認する。
func TestSearch_EmptyQuery(t *testing.T) {
	c := searchclient.NewInMemorySearchClient()
	ctx := context.Background()
	_ = c.CreateIndex(ctx, "items", searchclient.NewIndexMapping())
	_, _ = c.IndexDocument(ctx, "items", searchclient.IndexDocument{
		ID: "i-1", Fields: map[string]interface{}{"name": "Item 1"},
	})

	result, err := c.Search(ctx, "items", searchclient.SearchQuery{Query: "", Page: 0, Size: 20})
	require.NoError(t, err)
	assert.Equal(t, uint64(1), result.Total)
}

// IndexMappingのWithFieldメソッドがフィールド定義を正しく追加することを確認する。
func TestIndexMapping_WithField(t *testing.T) {
	m := searchclient.NewIndexMapping().
		WithField("name", "text").
		WithField("price", "integer")
	assert.Len(t, m.Fields, 2)
	assert.Equal(t, "text", m.Fields["name"].FieldType)
	assert.True(t, m.Fields["name"].Indexed)
}
