# tier3 BFF GraphQL Golden Path 完動例の Dockerfile。
FROM golang:1.22-alpine AS builder
WORKDIR /workspace
COPY go.mod go.sum ./
RUN go mod download
COPY cmd/ cmd/
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /out/portal-bff ./cmd/portal-bff

FROM gcr.io/distroless/static-debian12:nonroot
WORKDIR /
COPY --from=builder /out/portal-bff /portal-bff
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/portal-bff"]
