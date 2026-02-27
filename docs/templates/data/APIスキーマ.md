# テンプレート仕様 — APIスキーマ

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **API スキーマ定義** テンプレートの仕様を定義する。OpenAPI（REST API）、Protocol Buffers（gRPC）、GraphQL の3種類のスキーマテンプレートを提供し、`api_styles` の選択に応じてサーバーテンプレート内に自動生成する。

API 設計の全体像は [API設計](../../api/gateway/API設計.md) を参照。

## 生成対象

| kind       | OpenAPI | Proto | GraphQL |
| ---------- | ------- | ----- | ------- |
| `server`   | 条件付き | 条件付き | 条件付き |
| `bff`      | 条件付き | 条件付き | 条件付き |
| `client`   | 生成しない | 生成しない | 生成しない |
| `library`  | 生成しない | 生成しない | 生成しない |
| `database` | 生成しない | 生成しない | 生成しない |

- `api_styles` に `rest` が含まれる場合に OpenAPI を生成する
- `api_styles` に `grpc` が含まれる場合に Proto を生成する
- `api_styles` に `graphql` が含まれる場合に GraphQL を生成する

## 配置パス

生成されるスキーマファイルはサービスの `api/` 配下に API 方式ごとのディレクトリを作成して配置する。

| ファイル         | 配置パス                            |
| ---------------- | ----------------------------------- |
| OpenAPI          | `api/openapi/openapi.yaml`          |
| Proto            | `api/proto/service.proto`           |
| GraphQL          | `api/graphql/schema.graphql`        |

## テンプレートファイル一覧

テンプレートは server テンプレート内に配置する。

### Rust サーバー

Rust サーバーでも同等のスキーマファイルを生成する。gRPC は `grpc.rs.tera` 等の Rust 固有のコード生成も含む。

| テンプレートファイル                              | 生成先                       | 説明                     |
| ------------------------------------------------- | ---------------------------- | ------------------------ |
| `server/rust/api/openapi/openapi.yaml.tera`       | `api/openapi/openapi.yaml`   | OpenAPI 3.0 仕様         |
| `server/rust/api/proto/service.proto.tera`        | `api/proto/service.proto`    | Protocol Buffers 定義    |
| `server/rust/api/graphql/schema.graphql.tera`     | `api/graphql/schema.graphql` | GraphQL スキーマ         |

### ディレクトリ構成

```
CLI/
└── templates/
    └── server/
        ├── go/
        │   └── api/
        │       ├── openapi/
        │       │   └── openapi.yaml.tera
        │       ├── proto/
        │       │   └── service.proto.tera
        │       └── graphql/
        │           └── schema.graphql.tera
        └── rust/
            └── api/
                ├── openapi/
                │   └── openapi.yaml.tera
                ├── proto/
                │   └── service.proto.tera
                └── graphql/
                    └── schema.graphql.tera
```

## 使用するテンプレート変数

API スキーマテンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名                  | 型       | OpenAPI | Proto | GraphQL | 用途                                     |
| ----------------------- | -------- | ------- | ----- | ------- | ---------------------------------------- |
| `service_name`          | String   | 用      | 用    | 用      | API タイトル、パッケージ名               |
| `service_name_snake`    | String   | 用      | 用    | -       | パス・パッケージのスネークケース表記     |
| `service_name_pascal`   | String   | 用      | 用    | 用      | スキーマ名・メッセージ名・型名           |
| `service_name_camel`    | String   | -       | -     | 用      | フィールド名・クエリ名（camelCase）      |
| `api_styles`            | [String] | -       | -     | -       | 生成対象スキーマの判定（条件分岐）       |
| `go_module`             | String   | -       | 用    | -       | Go の proto option go_package             |

---

## OpenAPI テンプレート（openapi.yaml.tera）

REST API のエンドポイント定義を OpenAPI 3.0 仕様で記述する。CRUD 操作とページネーションを含むスキーマを生成する。

```tera
openapi: "3.0.3"
info:
  title: {{ service_name_pascal }} API
  version: "1.0.0"
  description: {{ service_name_pascal }} service REST API
servers:
  - url: /api/{{ service_name_snake }}
    description: Local development
paths:
  /:
    get:
      operationId: list{{ service_name_pascal }}
      summary: List {{ service_name_pascal }} resources
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            default: 1
        - name: per_page
          in: query
          schema:
            type: integer
            default: 20
            maximum: 100
      responses:
        "200":
          description: Successful response
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ service_name_pascal }}ListResponse"
    post:
      operationId: create{{ service_name_pascal }}
      summary: Create a new {{ service_name_pascal }} resource
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/Create{{ service_name_pascal }}Request"
      responses:
        "201":
          description: Created
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ service_name_pascal }}"
  /{id}:
    get:
      operationId: get{{ service_name_pascal }}
      summary: Get a {{ service_name_pascal }} resource by ID
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        "200":
          description: Successful response
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ service_name_pascal }}"
        "404":
          description: Not found
    put:
      operationId: update{{ service_name_pascal }}
      summary: Update a {{ service_name_pascal }} resource
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/Update{{ service_name_pascal }}Request"
      responses:
        "200":
          description: Updated
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ service_name_pascal }}"
        "404":
          description: Not found
    delete:
      operationId: delete{{ service_name_pascal }}
      summary: Delete a {{ service_name_pascal }} resource
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        "204":
          description: Deleted
        "404":
          description: Not found
components:
  schemas:
    {{ service_name_pascal }}:
      type: object
      properties:
        id:
          type: string
          format: uuid
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
      required:
        - id
        - created_at
        - updated_at
    Create{{ service_name_pascal }}Request:
      type: object
      properties: {}
    Update{{ service_name_pascal }}Request:
      type: object
      properties: {}
    {{ service_name_pascal }}ListResponse:
      type: object
      properties:
        items:
          type: array
          items:
            $ref: "#/components/schemas/{{ service_name_pascal }}"
        total:
          type: integer
        page:
          type: integer
        per_page:
          type: integer
      required:
        - items
        - total
        - page
        - per_page
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
security:
  - bearerAuth: []
```

### ポイント

- CRUD の4操作（List / Create / Get / Update / Delete）のエンドポイントを定義する
- List 操作はページネーション（`page` / `per_page`）をサポートする
- ID は UUID 形式を使用する
- JWT Bearer 認証をデフォルトのセキュリティスキームとして設定する
- リクエスト/レスポンスのスキーマは雛形として定義し、生成後にサービス固有のフィールドを追加する

---

## Protocol Buffers テンプレート（service.proto.tera）

gRPC サービスの定義を Protocol Buffers で記述する。CRUD 操作の RPC メソッドとメッセージ型を生成する。

```tera
syntax = "proto3";

package {{ service_name_snake }}.v1;

option go_package = "{{ go_module }}/gen/{{ service_name_snake }}/v1;{{ service_name_snake }}v1";

import "google/protobuf/timestamp.proto";
import "google/protobuf/empty.proto";

service {{ service_name_pascal }}Service {
  rpc List{{ service_name_pascal }}(List{{ service_name_pascal }}Request) returns (List{{ service_name_pascal }}Response);
  rpc Get{{ service_name_pascal }}(Get{{ service_name_pascal }}Request) returns ({{ service_name_pascal }});
  rpc Create{{ service_name_pascal }}(Create{{ service_name_pascal }}Request) returns ({{ service_name_pascal }});
  rpc Update{{ service_name_pascal }}(Update{{ service_name_pascal }}Request) returns ({{ service_name_pascal }});
  rpc Delete{{ service_name_pascal }}(Delete{{ service_name_pascal }}Request) returns (google.protobuf.Empty);
}

message {{ service_name_pascal }} {
  string id = 1;
  google.protobuf.Timestamp created_at = 2;
  google.protobuf.Timestamp updated_at = 3;
}

message List{{ service_name_pascal }}Request {
  int32 page = 1;
  int32 per_page = 2;
}

message List{{ service_name_pascal }}Response {
  repeated {{ service_name_pascal }} items = 1;
  int32 total = 2;
  int32 page = 3;
  int32 per_page = 4;
}

message Get{{ service_name_pascal }}Request {
  string id = 1;
}

message Create{{ service_name_pascal }}Request {}

message Update{{ service_name_pascal }}Request {
  string id = 1;
}

message Delete{{ service_name_pascal }}Request {
  string id = 1;
}
```

### ポイント

- パッケージは `{{ service_name_snake }}.v1` の形式でバージョニングを導入する
- Go の `go_package` オプションで `go_module` 変数を使用し、正しい import パスを生成する
- CRUD の5つの RPC メソッド（List / Get / Create / Update / Delete）を定義する
- `google.protobuf.Timestamp` で日時フィールドを定義する
- Delete は `google.protobuf.Empty` を返す

---

## GraphQL テンプレート（schema.graphql.tera）

GraphQL スキーマの Query / Mutation / 型定義を生成する。

```tera
type Query {
  {{ service_name_camel }}(id: ID!): {{ service_name_pascal }}
  {{ service_name_camel }}List(page: Int = 1, perPage: Int = 20): {{ service_name_pascal }}Connection!
}

type Mutation {
  create{{ service_name_pascal }}(input: Create{{ service_name_pascal }}Input!): {{ service_name_pascal }}!
  update{{ service_name_pascal }}(id: ID!, input: Update{{ service_name_pascal }}Input!): {{ service_name_pascal }}!
  delete{{ service_name_pascal }}(id: ID!): Boolean!
}

type {{ service_name_pascal }} {
  id: ID!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type {{ service_name_pascal }}Connection {
  items: [{{ service_name_pascal }}!]!
  total: Int!
  page: Int!
  perPage: Int!
}

input Create{{ service_name_pascal }}Input {
  _placeholder: String
}

input Update{{ service_name_pascal }}Input {
  _placeholder: String
}

scalar DateTime
```

### ポイント

- Query で単一取得（`{{ service_name_camel }}`）と一覧取得（`{{ service_name_camel }}List`）を定義する
- Mutation で作成・更新・削除の操作を定義する
- 一覧取得は Connection パターン（`items` / `total` / `page` / `perPage`）を使用する
- Input 型は `_placeholder` フィールドで雛形として定義し、生成後にサービス固有のフィールドを追加する
- `DateTime` カスタムスカラーを定義する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるスキーマが変わる。

| 条件                    | 選択肢          | 生成への影響                          |
| ----------------------- | --------------- | ------------------------------------- |
| API 方式 (`api_styles`) | `rest` を含む   | OpenAPI スキーマを生成する            |
| API 方式 (`api_styles`) | `grpc` を含む   | Protocol Buffers スキーマを生成する   |
| API 方式 (`api_styles`) | `graphql` を含む | GraphQL スキーマを生成する           |
| kind (`kind`)           | `client` 以下   | API スキーマを生成しない             |

複数の API 方式を選択した場合、選択された全てのスキーマが生成される。

---

## 生成例

### REST + gRPC サーバーの場合

入力:
```json
{
  "service_name": "order-service",
  "service_name_snake": "order_service",
  "service_name_pascal": "OrderService",
  "service_name_camel": "orderService",
  "api_styles": ["rest", "grpc"],
  "go_module": "github.com/example/order-service"
}
```

生成されるファイル:
- `api/openapi/openapi.yaml` -- CRUD エンドポイント、JWT 認証、ページネーション
- `api/proto/service.proto` -- 5 RPC メソッド、go_package 設定

### GraphQL サーバーの場合

入力:
```json
{
  "service_name": "product-service",
  "service_name_snake": "product_service",
  "service_name_pascal": "ProductService",
  "service_name_camel": "productService",
  "api_styles": ["graphql"]
}
```

生成されるファイル:
- `api/graphql/schema.graphql` -- Query / Mutation / Connection パターン

### REST + gRPC + GraphQL 全対応サーバーの場合

入力:
```json
{
  "service_name": "user-service",
  "service_name_snake": "user_service",
  "service_name_pascal": "UserService",
  "service_name_camel": "userService",
  "api_styles": ["rest", "grpc", "graphql"],
  "go_module": "github.com/example/user-service"
}
```

生成されるファイル:
- `api/openapi/openapi.yaml` -- REST API 定義
- `api/proto/service.proto` -- gRPC サービス定義
- `api/graphql/schema.graphql` -- GraphQL スキーマ定義

---

## 関連ドキュメント

- [API設計](../../api/gateway/API設計.md) -- API 設計の全体方針
- [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-サーバー](../server/サーバー.md) -- サーバーテンプレート仕様
- [テンプレート仕様-BFF](../client/BFF.md) -- BFF テンプレート仕様
- [テンプレート仕様-Kong](../middleware/Kong.md) -- Kong テンプレート仕様（API ルーティング連携）
- [テンプレート仕様-Keycloak](../middleware/Keycloak.md) -- Keycloak テンプレート仕様（認証連携）
- [コーディング規約](../../architecture/conventions/コーディング規約.md) -- コーディング規約
