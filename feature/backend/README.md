# Feature Backend

バックエンドサービスの配置先。

## ディレクトリ構成

```
backend/
├── rust/           # Rust バックエンドサービス
│   └── {feature_name}/
├── go/             # Go バックエンドサービス
│   └── {feature_name}/
├── csharp/         # C# バックエンドサービス
│   └── {feature_name}/
├── python/         # Python バックエンドサービス
│   └── {feature_name}/
└── kotlin/         # Kotlin バックエンドサービス
    └── {feature_name}/
```

## サービスの生成

```bash
# Rust
k1s0 new-feature --type backend-rust --name {name}

# Go
k1s0 new-feature --type backend-go --name {name}

# C#
k1s0 new-feature --type backend-csharp --name {name}

# Python
k1s0 new-feature --type backend-python --name {name}

# Kotlin
k1s0 new-feature --type backend-kotlin --name {name}
```
