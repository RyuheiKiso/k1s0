# Backend Rust Template

Rust バックエンドサービスのテンプレート。

## ディレクトリ構成

```
backend-rust/
├── project/      # リポジトリ初期化テンプレ（k1s0 init 用）
└── feature/      # 機能雛形（k1s0 new-feature 用）
```

## feature/ の生成物

`k1s0 new-feature --type backend-rust --name {name}` で生成される構成：

```
feature/backend/rust/{name}/
├── Cargo.toml
├── README.md
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   └── overlays/
│       ├── dev/
│       ├── stg/
│       └── prod/
├── proto/              # gRPC 正本（必要な場合）
├── openapi/
│   └── openapi.yaml    # REST 正本
├── migrations/
└── src/
    ├── application/
    ├── domain/
    ├── infrastructure/
    ├── presentation/
    └── main.rs
```

## 技術スタック

- Web フレームワーク: axum
- 非同期ランタイム: tokio
- gRPC: tonic
- 設定: serde + yaml
- ログ/トレース: tracing + OpenTelemetry
