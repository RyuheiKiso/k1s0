module github.com/k1s0-platform/system-library-go-resiliency

go 1.26.1

require (
	github.com/k1s0-platform/system-library-go-bulkhead v0.0.0
	github.com/k1s0-platform/system-library-go-circuit-breaker v0.0.0
	github.com/stretchr/testify v1.11.1
)

require (
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/kr/text v0.2.0 // indirect
	github.com/pmezard/go-difflib v1.0.1-0.20181226105442-5d4384ee4fb2 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace (
	github.com/k1s0-platform/system-library-go-bulkhead => ../bulkhead
	github.com/k1s0-platform/system-library-go-circuit-breaker => ../circuit-breaker
)
