package pagination

import (
	"encoding/base64"
	"errors"
	"math"
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

// EncodeCursor はカーソルをBase64エンコードする。
func EncodeCursor(id string) string {
	return base64.StdEncoding.EncodeToString([]byte(id))
}

// DecodeCursor はBase64エンコードされたカーソルをデコードする。
func DecodeCursor(cursor string) (string, error) {
	b, err := base64.StdEncoding.DecodeString(cursor)
	if err != nil {
		return "", errors.New("無効なカーソルです")
	}
	return string(b), nil
}
