package db

import (
	"context"
	"fmt"
	"os"

	"github.com/jackc/pgx/v5/pgxpool"
)

func Connect(databaseURL string) (*pgxpool.Pool, error) {
	secretKey := os.Getenv("SECRET_KEY")
	if secretKey == "" {
		fmt.Println("WARNING: SECRET_KEY not set")
	}

	pool, err := pgxpool.New(context.Background(), databaseURL)
	if err != nil {
		return nil, fmt.Errorf("unable to connect to database: %w", err)
	}

	if err := pool.Ping(context.Background()); err != nil {
		return nil, fmt.Errorf("unable to ping database: %w", err)
	}

	return pool, nil
}
