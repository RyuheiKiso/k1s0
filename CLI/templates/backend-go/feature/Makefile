.PHONY: all build run test clean proto lint

# Variables
BINARY_NAME={{ feature_name | snake_case }}
GO=go
PROTOC=protoc

all: build

# Build
build:
	$(GO) build -o $(BINARY_NAME) main.go

# Run
run:
	$(GO) run main.go

# Test
test:
	$(GO) test -v ./...

# Test with coverage
test-coverage:
	$(GO) test -v -coverprofile=coverage.out ./...
	$(GO) tool cover -html=coverage.out -o coverage.html

# Clean
clean:
	rm -f $(BINARY_NAME)
	rm -f coverage.out coverage.html

# Generate proto files
proto:
	$(PROTOC) --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		proto/*.proto

# Lint
lint:
	golangci-lint run

# Format
fmt:
	$(GO) fmt ./...

# Tidy dependencies
tidy:
	$(GO) mod tidy

# Docker build
docker-build:
	docker build -t $(BINARY_NAME):latest .

# Docker run
docker-run:
	docker run -p 50051:50051 $(BINARY_NAME):latest
