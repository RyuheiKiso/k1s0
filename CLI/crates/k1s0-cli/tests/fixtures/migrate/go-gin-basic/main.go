package main

import (
	"fmt"
	"log"
	"os"

	"github.com/example/my-service/db"
	"github.com/example/my-service/handlers"
	"github.com/gin-gonic/gin"
)

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	databaseURL := os.Getenv("DATABASE_URL")
	if databaseURL == "" {
		log.Fatal("DATABASE_URL is required")
	}

	pool, err := db.Connect(databaseURL)
	if err != nil {
		log.Fatalf("Failed to connect to database: %v", err)
	}
	defer pool.Close()

	r := gin.Default()

	r.GET("/health", handlers.HealthCheck)
	r.GET("/items", handlers.ListItems(pool))
	r.POST("/items", handlers.CreateItem(pool))
	r.GET("/items/:id", handlers.GetItem(pool))
	r.DELETE("/items/:id", handlers.DeleteItem(pool))

	addr := fmt.Sprintf(":%s", port)
	log.Printf("Listening on %s", addr)
	if err := r.Run(addr); err != nil {
		log.Fatal(err)
	}
}
