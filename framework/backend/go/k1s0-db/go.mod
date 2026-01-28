module github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-db

go 1.22

require (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error v0.0.0
	github.com/jackc/pgx/v5 v5.7.2
)

require (
	github.com/jackc/pgpassfile v1.0.0 // indirect
	github.com/jackc/pgservicefile v0.0.0-20240606120523-5a60cdf6a761 // indirect
	github.com/jackc/puddle/v2 v2.2.2 // indirect
	github.com/stretchr/testify v1.9.0 // indirect
	golang.org/x/crypto v0.31.0 // indirect
	golang.org/x/sync v0.10.0 // indirect
	golang.org/x/text v0.21.0 // indirect
)

replace (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-config => ../k1s0-config
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error => ../k1s0-error
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability => ../k1s0-observability
)
