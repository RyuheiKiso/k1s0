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
	original := "abc-123-def"
	encoded := pagination.EncodeCursor(original)
	decoded, err := pagination.DecodeCursor(encoded)
	require.NoError(t, err)
	assert.Equal(t, original, decoded)
}

func TestDecodeCursor_Invalid(t *testing.T) {
	_, err := pagination.DecodeCursor("!!!invalid!!!")
	assert.Error(t, err)
}

func TestCursor_EmptyString(t *testing.T) {
	encoded := pagination.EncodeCursor("")
	decoded, err := pagination.DecodeCursor(encoded)
	require.NoError(t, err)
	assert.Equal(t, "", decoded)
}
