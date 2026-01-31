# Feature Frontend

フロントエンドサービスの配置先。

## ディレクトリ構成

```
frontend/
├── react/          # React フロントエンド
│   └── {feature_name}/
├── flutter/        # Flutter フロントエンド
│   └── {feature_name}/
└── android/        # Android フロントエンド
    └── {feature_name}/
```

## サービスの生成

```bash
# React
k1s0 new-feature --type frontend-react --name {name}

# Flutter
k1s0 new-feature --type frontend-flutter --name {name}

# Android
k1s0 new-feature --type frontend-android --name {name}
```
