# Feature Backend

バックエンドサービスの配置先。

## ディレクトリ構成

```
backend/
├── rust/           # Rust バックエンドサービス
│   └── {feature_name}/
└── go/             # Go バックエンドサービス
    └── {feature_name}/
```

## サービスの生成

```bash
# Rust
k1s0 new-feature --type backend-rust --name {name}

# Go
k1s0 new-feature --type backend-go --name {name}
```
