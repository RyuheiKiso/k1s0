module github.com/k1s0-platform/system-library-go-resiliency

go 1.23.0

require (
	github.com/k1s0-platform/system-library-go-bulkhead v0.0.0
	github.com/k1s0-platform/system-library-go-circuit-breaker v0.0.0
	github.com/stretchr/testify v1.11.1
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace (
	github.com/k1s0-platform/system-library-go-bulkhead => ../bulkhead
	github.com/k1s0-platform/system-library-go-circuit-breaker => ../circuit-breaker
)
