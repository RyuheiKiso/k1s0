# Backend Go Template

Go バックエンドサービスのテンプレート。

## ステータス

置き場のみ固定。実装は後続フェーズで行う。

## feature/ の生成物（予定）

```
feature/backend/go/{name}/
├── go.mod
├── README.md
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   └── overlays/
├── proto/
├── openapi/
├── migrations/
└── src/
    ├── application/
    ├── domain/
    ├── infrastructure/
    └── presentation/
```
