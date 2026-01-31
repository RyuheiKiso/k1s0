# K060: インフラ検査

← [Lint 設計書](./)

---

## K060: Dockerfile ベースイメージ未固定

### 目的

Dockerfile の `FROM` 文でベースイメージのバージョンが固定されていない場合を検出し、ビルドの再現性を確保する。

### 検査対象

- サービスルートの `Dockerfile`

### 検査ロジック

`FROM` 行ごとに以下を検査する:

1. `scratch` イメージはスキップ（特殊イメージのため）
2. `@sha256:` ダイジェスト指定は OK
3. `:latest` タグは **違反**
4. タグなし（例: `FROM ubuntu`）は **違反**
5. 具体的なバージョンタグ（例: `FROM ubuntu:22.04`）は OK
6. `--platform=...` プレフィックスがある場合も正しく解析

### 違反例

```dockerfile
# K060 違反: タグなし
FROM ubuntu

# K060 違反: :latest タグ
FROM node:latest

# K060 違反: --platform 付きでもタグなしは NG
FROM --platform=linux/amd64 python
```

### 正しい実装

```dockerfile
# バージョンタグを指定
FROM ubuntu:22.04

# sha256 ダイジェストを指定
FROM node@sha256:abc123...

# scratch は例外的に OK
FROM scratch
```

### ヒント

具体的なバージョンタグ（例: `image:1.0.0`）または sha256 ダイジェストを指定してください。
