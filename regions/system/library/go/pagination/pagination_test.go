package pagination_test

import (
	"testing"

	"github.com/k1s0-platform/system-library-go-pagination"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// NewPageResponseが総件数とページサイズから総ページ数を正しく計算することを確認する。
func TestNewPageResponse_TotalPages(t *testing.T) {
	items := []string{"a", "b", "c"}
	resp := pagination.NewPageResponse(items, 10, pagination.PageRequest{Page: 1, PerPage: 3})
	assert.Equal(t, uint32(4), resp.TotalPages)
	assert.Equal(t, uint32(1), resp.Page)
	assert.Equal(t, uint32(3), resp.PerPage)
	assert.Equal(t, uint64(10), resp.Total)
	assert.Equal(t, 3, len(resp.Items))
}

// 総件数がページサイズで割り切れる場合にTotalPagesが正確に計算されることを確認する。
func TestNewPageResponse_ExactDivision(t *testing.T) {
	items := []int{1, 2}
	resp := pagination.NewPageResponse(items, 6, pagination.PageRequest{Page: 3, PerPage: 2})
	assert.Equal(t, uint32(3), resp.TotalPages)
}

// PerPageに0を指定した場合にデフォルト値1が使われ正しくTotalPagesが計算されることを確認する。
func TestNewPageResponse_ZeroPerPage(t *testing.T) {
	resp := pagination.NewPageResponse([]int{}, 5, pagination.PageRequest{Page: 1, PerPage: 0})
	assert.Equal(t, uint32(5), resp.TotalPages)
	assert.Equal(t, uint32(1), resp.PerPage)
}

// PageResponseのMetaメソッドがページネーションメタ情報を正しく返すことを確認する。
func TestPageResponse_Meta(t *testing.T) {
	resp := pagination.NewPageResponse([]int{1, 2, 3}, 25, pagination.PageRequest{Page: 2, PerPage: 10})
	meta := resp.Meta()
	assert.Equal(t, uint64(25), meta.Total)
	assert.Equal(t, uint32(2), meta.Page)
	assert.Equal(t, uint32(10), meta.PerPage)
	assert.Equal(t, uint32(3), meta.TotalPages)
}

// EncodeCursorとDecodeCursorがカーソルのエンコードとデコードを正しく往復できることを確認する。
func TestCursor_RoundTrip(t *testing.T) {
	sortKey := "2024-01-15"
	id := "abc-123-def"
	encoded := pagination.EncodeCursor(sortKey, id)
	decodedSortKey, decodedID, err := pagination.DecodeCursor(encoded)
	require.NoError(t, err)
	assert.Equal(t, sortKey, decodedSortKey)
	assert.Equal(t, id, decodedID)
}

// 不正なbase64文字列に対してDecodeCursorがエラーを返すことを確認する。
func TestDecodeCursor_Invalid(t *testing.T) {
	_, _, err := pagination.DecodeCursor("!!!invalid!!!")
	assert.Error(t, err)
}

// パイプ区切り文字を含まないカーソルに対してDecodeCursorがエラーを返すことを確認する。
func TestDecodeCursor_MissingSeparator(t *testing.T) {
	// base64("noseparator") - contains no pipe separator
	_, _, err := pagination.DecodeCursor("bm9zZXBhcmF0b3I=")
	assert.Error(t, err)
}

// ValidatePerPageが有効な範囲の値（1〜100）に対してエラーなしを返すことを確認する。
func TestValidatePerPage_Valid(t *testing.T) {
	assert.NoError(t, pagination.ValidatePerPage(1))
	assert.NoError(t, pagination.ValidatePerPage(50))
	assert.NoError(t, pagination.ValidatePerPage(100))
}

// ValidatePerPageが0を指定した場合にエラーを返すことを確認する。
func TestValidatePerPage_Zero(t *testing.T) {
	assert.Error(t, pagination.ValidatePerPage(0))
}

// ValidatePerPageが最大値（100）を超えた場合にエラーを返すことを確認する。
func TestValidatePerPage_OverMax(t *testing.T) {
	assert.Error(t, pagination.ValidatePerPage(101))
}

// CursorRequestのCursorとLimitフィールドが正しく設定されることを確認する。
func TestCursorRequest_Fields(t *testing.T) {
	cursor := "some-cursor"
	req := pagination.CursorRequest{Cursor: &cursor, Limit: 20}
	assert.Equal(t, &cursor, req.Cursor)
	assert.Equal(t, uint32(20), req.Limit)
}

// CursorMetaのNextCursorとHasMoreフィールドが正しく設定されることを確認する。
func TestCursorMeta_Fields(t *testing.T) {
	next := "next-cursor"
	meta := pagination.CursorMeta{NextCursor: &next, HasMore: true}
	assert.Equal(t, &next, meta.NextCursor)
	assert.True(t, meta.HasMore)
}

// PaginationMetaの各フィールドが正しく設定されることを確認する。
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

// NewPageRequestが指定したページ番号とページサイズを持つPageRequestを生成することを確認する。
func TestNewPageRequest(t *testing.T) {
	req := pagination.NewPageRequest(3, 50)
	assert.Equal(t, uint32(3), req.Page)
	assert.Equal(t, uint32(50), req.PerPage)
}

// DefaultPageRequestがデフォルトのPage=1、PerPage=20を返すことを確認する。
func TestDefaultPageRequest(t *testing.T) {
	req := pagination.DefaultPageRequest()
	assert.Equal(t, uint32(1), req.Page)
	assert.Equal(t, uint32(20), req.PerPage)
}

// PageRequestのOffsetがページ番号とページサイズから正しいオフセット値を返すことを確認する。
func TestPageRequest_Offset(t *testing.T) {
	assert.Equal(t, uint64(0), pagination.NewPageRequest(1, 20).Offset())
	assert.Equal(t, uint64(20), pagination.NewPageRequest(2, 20).Offset())
	assert.Equal(t, uint64(40), pagination.NewPageRequest(3, 20).Offset())
	assert.Equal(t, uint64(0), pagination.DefaultPageRequest().Offset())
}

// PageRequestのHasNextが総件数に基づいて次ページの有無を正しく判定することを確認する。
func TestPageRequest_HasNext(t *testing.T) {
	req := pagination.NewPageRequest(1, 10)
	assert.True(t, req.HasNext(11))
	assert.False(t, req.HasNext(10))
	assert.False(t, req.HasNext(5))

	req2 := pagination.NewPageRequest(2, 10)
	assert.True(t, req2.HasNext(21))
	assert.False(t, req2.HasNext(20))
}
