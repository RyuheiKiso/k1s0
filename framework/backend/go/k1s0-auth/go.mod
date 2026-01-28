module github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-auth

go 1.22.7

toolchain go1.24.1

require (
	github.com/coreos/go-oidc/v3 v3.11.0
	github.com/golang-jwt/jwt/v5 v5.2.1
	google.golang.org/grpc v1.68.0
)

require (
	github.com/go-jose/go-jose/v4 v4.0.2 // indirect
	github.com/stretchr/testify v1.9.0 // indirect
	golang.org/x/crypto v0.31.0 // indirect
	golang.org/x/net v0.29.0 // indirect
	golang.org/x/oauth2 v0.23.0 // indirect
	golang.org/x/sys v0.28.0 // indirect
	golang.org/x/text v0.21.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240903143218-8af14fe29dc1 // indirect
	google.golang.org/protobuf v1.35.1 // indirect
)

replace (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-cache => ../k1s0-cache
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-config => ../k1s0-config
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error => ../k1s0-error
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability => ../k1s0-observability
)
