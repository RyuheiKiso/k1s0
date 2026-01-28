module github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-cache

go 1.22

require (
	github.com/redis/go-redis/v9 v9.7.0
	github.com/vmihailenco/msgpack/v5 v5.4.1
)

require (
	github.com/cespare/xxhash/v2 v2.3.0 // indirect
	github.com/dgryski/go-rendezvous v0.0.0-20200823014737-9f7001d12a5f // indirect
	github.com/stretchr/testify v1.9.0 // indirect
	github.com/vmihailenco/tagparser/v2 v2.0.0 // indirect
)

replace (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-config => ../k1s0-config
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error => ../k1s0-error
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability => ../k1s0-observability
)
