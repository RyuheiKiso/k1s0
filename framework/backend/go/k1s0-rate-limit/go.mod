module github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-rate-limit

go 1.22

require (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error v0.0.0
)

replace (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error => ../k1s0-error
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-config => ../k1s0-config
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability => ../k1s0-observability
)
