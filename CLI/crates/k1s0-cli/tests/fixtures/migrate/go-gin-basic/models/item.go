package models

type Item struct {
	ID          int     `json:"id"`
	Name        string  `json:"name" binding:"required"`
	Description *string `json:"description"`
}
