package handlers

import (
	"net/http"
	"strconv"

	"github.com/example/my-service/models"
	"github.com/gin-gonic/gin"
	"github.com/jackc/pgx/v5/pgxpool"
)

func HealthCheck(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"status": "ok"})
}

func ListItems(pool *pgxpool.Pool) gin.HandlerFunc {
	return func(c *gin.Context) {
		rows, err := pool.Query(c, "SELECT id, name, description FROM items")
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		defer rows.Close()

		var items []models.Item
		for rows.Next() {
			var item models.Item
			if err := rows.Scan(&item.ID, &item.Name, &item.Description); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			items = append(items, item)
		}

		c.JSON(http.StatusOK, items)
	}
}

func CreateItem(pool *pgxpool.Pool) gin.HandlerFunc {
	return func(c *gin.Context) {
		var input models.Item
		if err := c.ShouldBindJSON(&input); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		err := pool.QueryRow(c,
			"INSERT INTO items (name, description) VALUES ($1, $2) RETURNING id",
			input.Name, input.Description,
		).Scan(&input.ID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		c.JSON(http.StatusCreated, input)
	}
}

func GetItem(pool *pgxpool.Pool) gin.HandlerFunc {
	return func(c *gin.Context) {
		id, err := strconv.Atoi(c.Param("id"))
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
			return
		}

		var item models.Item
		err = pool.QueryRow(c, "SELECT id, name, description FROM items WHERE id = $1", id).
			Scan(&item.ID, &item.Name, &item.Description)
		if err != nil {
			c.JSON(http.StatusNotFound, gin.H{"error": "item not found"})
			return
		}

		c.JSON(http.StatusOK, item)
	}
}

func DeleteItem(pool *pgxpool.Pool) gin.HandlerFunc {
	return func(c *gin.Context) {
		id, err := strconv.Atoi(c.Param("id"))
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
			return
		}

		_, err = pool.Exec(c, "DELETE FROM items WHERE id = $1", id)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		c.Status(http.StatusNoContent)
	}
}
