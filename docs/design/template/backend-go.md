# backend-go テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/backend/go/{service_name}/
├── .k1s0/
│   └── manifest.json
├── go.mod
├── go.sum
├── README.md
├── cmd/
│   └── main.go.tera
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   │   ├── configmap.yaml
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   └── kustomization.yaml
│   └── overlays/
│       ├── dev/
│       ├── stg/
│       └── prod/
├── internal/
│   ├── domain/
│   │   ├── entities/
│   │   └── errors/
│   ├── application/
│   │   ├── services/
│   │   └── usecases/
│   ├── presentation/
│   └── infrastructure/
└── proto/
    └── service.proto
```

## go.mod テンプレート

```go
module github.com/your-org/{{ feature_name }}

go {{ "1.22" }}

require (
    // Framework packages
    github.com/your-org/k1s0-go/config v0.1.0
    github.com/your-org/k1s0-go/observability v0.1.0
    github.com/your-org/k1s0-go/validation v0.1.0
{% if with_grpc %}
    google.golang.org/grpc v1.64.0
    google.golang.org/protobuf v1.34.0
{% endif %}
{% if with_rest %}
    github.com/labstack/echo/v4 v4.12.0
{% endif %}
{% if with_db %}
    github.com/jackc/pgx/v5 v5.6.0
{% endif %}
)
```
