package presenter

// PaginationResponse はページネーション情報のレスポンス表現。
type PaginationResponse struct {
	TotalCount int  `json:"total_count"`
	Page       int  `json:"page"`
	PageSize   int  `json:"page_size"`
	HasNext    bool `json:"has_next"`
}

// NewPaginationResponse はページネーションレスポンスを作成する。
func NewPaginationResponse(totalCount, page, pageSize int) PaginationResponse {
	return PaginationResponse{
		TotalCount: totalCount,
		Page:       page,
		PageSize:   pageSize,
		HasNext:    page*pageSize < totalCount,
	}
}
