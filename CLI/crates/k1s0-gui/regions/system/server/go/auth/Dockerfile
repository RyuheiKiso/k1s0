FROM golang:1.21-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /bin/auth ./cmd/

FROM alpine:3.19
COPY --from=builder /bin/auth /bin/auth
ENTRYPOINT ["/bin/auth"]
