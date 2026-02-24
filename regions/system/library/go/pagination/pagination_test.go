package pagination_test

import (
	"testing"

	"github.com/k1s0-platform/system-library-go-pagination"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewPageResponse_TotalPages(t *testing.T) {
	items := []string{"a", "b", "c"}
	resp := pagination.NewPageResponse(items, 10, pagination.PageRequest{Page: 1, PerPage: 3})
	assert.Equal(t, uint32(4), resp.TotalPages)
	assert.Equal(t, uint32(1), resp.Page)
	assert.Equal(t, uint32(3), resp.PerPage)
	assert.Equal(t, uint64(10), resp.Total)
	assert.Equal(t, 3, len(resp.Items))
}

func TestNewPageResponse_ExactDivision(t *testing.T) {
	items := []int{1, 2}
	resp := pagination.NewPageResponse(items, 6, pagination.PageRequest{Page: 3, PerPage: 2})
	assert.Equal(t, uint32(3), resp.TotalPages)
}

func TestNewPageResponse_ZeroPerPage(t *testing.T) {
	resp := pagination.NewPageResponse([]int{}, 5, pagination.PageRequest{Page: 1, PerPage: 0})
	assert.Equal(t, uint32(5), resp.TotalPages)
	assert.Equal(t, uint32(1), resp.PerPage)
}

func TestCursor_RoundTrip(t *testing.T) {
	sortKey := "2024-01-15"
	id := "abc-123-def"
	encoded := pagination.EncodeCursor(sortKey, id)
	decodedSortKey, decodedID, err := pagination.DecodeCursor(encoded)
	require.NoError(t, err)
	assert.Equal(t, sortKey, decodedSortKey)
	assert.Equal(t, id, decodedID)
}

func TestDecodeCursor_Invalid(t *testing.T) {
	_, _, err := pagination.DecodeCursor("!!!invalid!!!")
	assert.Error(t, err)
}

func TestDecodeCursor_MissingSeparator(t *testing.T) {
	// base64("noseparator") - contains no pipe separator
	_, _, err := pagination.DecodeCursor("bm9zZXBhcmF0b3I=")
	assert.Error(t, err)
}

func TestValidatePerPage_Valid(t *testing.T) {
	assert.NoError(t, pagination.ValidatePerPage(1))
	assert.NoError(t, pagination.ValidatePerPage(50))
	assert.NoError(t, pagination.ValidatePerPage(100))
}

func TestValidatePerPage_Zero(t *testing.T) {
	assert.Error(t, pagination.ValidatePerPage(0))
}

func TestValidatePerPage_OverMax(t *testing.T) {
	assert.Error(t, pagination.ValidatePerPage(101))
}

func TestCursorRequest_Fields(t *testing.T) {
	cursor := "some-cursor"
	req := pagination.CursorRequest{Cursor: &cursor, Limit: 20}
	assert.Equal(t, &cursor, req.Cursor)
	assert.Equal(t, uint32(20), req.Limit)
}

func TestCursorMeta_Fields(t *testing.T) {
	next := "next-cursor"
	meta := pagination.CursorMeta{NextCursor: &next, HasMore: true}
	assert.Equal(t, &next, meta.NextCursor)
	assert.True(t, meta.HasMore)
}

func TestPaginationMeta_Fields(t *testing.T) {
	meta := pagination.PaginationMeta{
		Total:      100,
		Page:       2,
		PerPage:    10,
		TotalPages: 10,
	}
	assert.Equal(t, uint64(100), meta.Total)
	assert.Equal(t, uint32(10), meta.TotalPages)
}
