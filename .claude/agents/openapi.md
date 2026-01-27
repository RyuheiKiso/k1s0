# OpenAPI 仕様管理エージェント

OpenAPI/Swagger 仕様の作成、lint、管理を支援するエージェント。

## 対象領域

- `**/openapi/` - OpenAPI 仕様ファイル
- `.spectral.yaml` - Linter 設定

## 主な操作

### OpenAPI lint

```bash
# OpenAPI 仕様の lint 検証
./scripts/openapi-check.sh

# または Spectral を直接実行
npx @stoplight/spectral-cli lint <openapi-file>.yaml
```

## Spectral ルール設定

`.spectral.yaml` で定義された検証ルール:

### 必須ルール（Error）

| ルール | 説明 |
|--------|------|
| `oas3-api-servers` | servers セクションが必須 |
| `operation-operationId` | 各操作に operationId が必須 |
| `operation-tags` | 各操作に tags が必須 |
| `oas3-operation-security-defined` | security が定義されている |

### 推奨ルール（Warning）

| ルール | 説明 |
|--------|------|
| `info-contact` | contact 情報の記載 |
| `info-description` | API description の記載 |
| `operation-description` | 操作の description 記載 |

### 除外パス

- `**/gen/**` - 生成ファイルは除外

## ディレクトリ構造

```
<service>/
├── openapi/
│   ├── openapi.yaml          # 仕様定義
│   ├── openapi.fingerprint   # fingerprint（変更追跡）
│   └── gen/                  # 生成コード（.gitignore 対象）
└── ...
```

## OpenAPI 仕様テンプレート

```yaml
openapi: 3.0.3
info:
  title: <Service Name> API
  version: 1.0.0
  description: <Service description>
  contact:
    name: k1s0 Team
    email: team@example.com

servers:
  - url: http://localhost:8080
    description: Development server

paths:
  /health:
    get:
      operationId: healthCheck
      tags:
        - health
      summary: Health check endpoint
      responses:
        '200':
          description: Service is healthy

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

security:
  - bearerAuth: []
```

## コード生成

OpenAPI 仕様からのクライアント/サーバーコード生成:

**Rust:**
- `openapi-generator` または `utoipa` を使用

**TypeScript:**
- `openapi-typescript-codegen` を使用
- `k1s0-api-client` パッケージで管理

## CI 検証

GitHub Actions `openapi.yml` で以下を検証:
- Spectral による lint
- 仕様の整合性
