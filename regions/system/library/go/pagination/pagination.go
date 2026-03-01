package pagination

import (
	"encoding/base64"
	"errors"
	"fmt"
	"math"
	"strings"
)

const (
	MinPerPage uint32 = 1
	MaxPerPage uint32 = 100
)

// PageRequest はページネーションリクエスト。
type PageRequest struct {
	Page    uint32
	PerPage uint32
}

// PageResponse はページネーションレスポンス。
type PageResponse[T any] struct {
	Items      []T
	Total      uint64
	Page       uint32
	PerPage    uint32
	TotalPages uint32
}

// PaginationMeta はオフセットページネーションのメタデータ。
type PaginationMeta struct {
	Total      uint64
	Page       uint32
	PerPage    uint32
	TotalPages uint32
}

// CursorRequest はカーソルベースのページネーションリクエスト。
type CursorRequest struct {
	Cursor *string
	Limit  uint32
}

// CursorMeta はカーソルベースのページネーションレスポンスメタデータ。
type CursorMeta struct {
	NextCursor *string
	HasMore    bool
}

// NewPageRequest は PageRequest を生成する。
func NewPageRequest(page, perPage uint32) PageRequest {
	return PageRequest{Page: page, PerPage: perPage}
}

// DefaultPageRequest はデフォルト値 (page: 1, perPage: 20) の PageRequest を返す。
func DefaultPageRequest() PageRequest {
	return PageRequest{Page: 1, PerPage: 20}
}

// Offset はページネーションのオフセット値を返す。
func (r PageRequest) Offset() uint64 {
	return uint64(r.Page-1) * uint64(r.PerPage)
}

// HasNext は次のページが存在するかを返す。
func (r PageRequest) HasNext(total uint64) bool {
	return uint64(r.Page)*uint64(r.PerPage) < total
}

// ValidatePerPage は per_page が 1〜100 の範囲であることを検証する。
func ValidatePerPage(perPage uint32) error {
	if perPage < MinPerPage || perPage > MaxPerPage {
		return fmt.Errorf("invalid per_page: %d (must be between %d and %d)", perPage, MinPerPage, MaxPerPage)
	}
	return nil
}

// NewPageResponse は新しい PageResponse を生成する。
func NewPageResponse[T any](items []T, total uint64, req PageRequest) PageResponse[T] {
	perPage := req.PerPage
	if perPage == 0 {
		perPage = 1
	}
	totalPages := uint32(math.Ceil(float64(total) / float64(perPage)))
	return PageResponse[T]{
		Items:      items,
		Total:      total,
		Page:       req.Page,
		PerPage:    perPage,
		TotalPages: totalPages,
	}
}

const cursorSeparator = "|"

// EncodeCursor は sort_key と id を結合して Base64 エンコードする。
func EncodeCursor(sortKey, id string) string {
	combined := sortKey + cursorSeparator + id
	return base64.RawURLEncoding.EncodeToString([]byte(combined))
}

// DecodeCursor は Base64 エンコードされたカーソルをデコードし (sortKey, id) を返す。
func DecodeCursor(cursor string) (sortKey string, id string, err error) {
	b, err := base64.RawURLEncoding.DecodeString(cursor)
	if err != nil {
		b, err = base64.URLEncoding.DecodeString(cursor)
	}
	if err != nil {
		b, err = base64.StdEncoding.DecodeString(cursor)
	}
	if err != nil {
		return "", "", errors.New("無効なカーソルです")
	}
	parts := strings.SplitN(string(b), cursorSeparator, 2)
	if len(parts) != 2 {
		return "", "", errors.New("無効なカーソル形式です")
	}
	return parts[0], parts[1], nil
}
