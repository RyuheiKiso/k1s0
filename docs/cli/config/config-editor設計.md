# Config Editor 設計

tier1 の system-client を使い、tier3 サービスの設定値をゲームのコンフィグ画面のように
編集できる GUI の設計を定義する。React・Flutter のいずれでも同一の `config-schema.yaml`
からUIを生成し、設定値の追加・変更にフロントエンドのコード変更を不要にする。

> **設計思想** — [navigation設計](navigation設計.md) と同じ SDUI アプローチを設定編集に
> 適用する。開発者が `config-schema.yaml` を編集して CLI で push するだけで、
> クライアントは**スキーマのインタープリター**として動作し、UIを自動組み立てる。

---

## 解決する課題

| 課題 | 従来のアプローチ | 本設計 |
| ---- | --------------- | ------ |
| 設定値の型情報がない | `value_json` は任意の JSON、型はない | `config-schema.yaml` で型・バリデーションを宣言 |
| 設定追加のたびにフロント変更が必要 | UI 実装・レビュー・デプロイ | `config-schema.yaml` を push するだけ |
| 入力 UI が値の型に合っていない | 開発者が個別に実装 | スキーマインタープリターが自動で UI コントロールを選択 |
| バリデーションがフロントで重複実装 | 各フロントで実装 | スキーマから自動適用 |
| キー名のタイポ | runtime エラー | CLI 生成の型定義でコンパイルエラー |

---

## アーキテクチャ全体像

```
開発者
  │ config-schema.yaml を編集
  ↓
k1s0 generate config-types （対話式 CLI）
  ├→ ① PUT /api/v1/config-schema/{service}  スキーマを config server に登録
  └→ ② 型定義ファイルを自動生成
         src/config/__generated__/config-types.ts    (React)
         lib/config/__generated__/config_types.dart  (Flutter)

管理者ユーザー
  │ /config/order-server を開く
  ↓
ConfigEditorPage  (component_id: ConfigEditorPage)
  ├─ GET /api/v1/config-schema/order-server   → スキーマ取得
  ├─ GET /api/v1/config/services/order-server → 現在値取得
  │  UIをスキーマから自動組み立て
  └─ PUT /api/v1/config/{namespace}/{key}     → 保存（楽観ロック）
```

---

## config-schema.yaml（開発者インターフェース）

各 tier3 サービスのリポジトリルートに配置する唯一の定義ファイル。
`$schema` を指定することで VS Code / IntelliJ が補完・バリデーションを提供する。

```yaml
# $schema: ./config-schema-schema.json
version: 1

service: order-server
namespace_prefix: service.order

categories:
  - id: database
    label: データベース
    icon: storage
    namespaces:
      - service.order.database
    fields:
      - key: max_connections
        label: 最大接続数
        description: DB 接続プールの上限
        type: integer
        min: 1
        max: 1000
        default: 25
        unit: connections

      - key: ssl_mode
        label: SSL モード
        type: enum
        options: [disable, require, verify-ca, verify-full]
        default: require

      - key: pool_timeout_ms
        label: 接続タイムアウト
        type: integer
        min: 100
        max: 30000
        default: 5000
        unit: ms

  - id: kafka
    label: メッセージング
    icon: message
    namespaces:
      - service.order.kafka
    fields:
      - key: consumer_group
        label: コンシューマーグループ
        type: string
        pattern: "^[a-z][a-z0-9-]+\\.[a-z][a-z0-9-]+$"
        default: order-server.default

      - key: max_poll_interval_ms
        label: 最大ポーリング間隔
        type: integer
        min: 1000
        max: 300000
        default: 30000
        unit: ms

  - id: feature_flags
    label: 機能フラグ
    icon: flag
    namespaces:
      - service.order.feature
    fields:
      - key: enable_bulk_order
        label: 一括注文機能
        description: 複数オーダーの一括処理を有効にする
        type: boolean
        default: false

      - key: max_items_per_order
        label: 注文上限アイテム数
        type: integer
        min: 1
        max: 1000
        default: 100
```

### フィールド定義

#### categories

| フィールド | 型 | 必須 | 説明 |
| ---------- | -- | ---- | ---- |
| `id` | string | ✅ | カテゴリ識別子。英小文字とアンダースコアのみ |
| `label` | string | ✅ | 左ペインに表示するラベル |
| `icon` | string | — | Material Icons のアイコン名 |
| `namespaces` | string[] | ✅ | このカテゴリが管理する config namespace のリスト |
| `fields` | Field[] | ✅ | フィールド定義のリスト |

#### categories[].fields

| フィールド | 型 | 必須 | 説明 |
| ---------- | -- | ---- | ---- |
| `key` | string | ✅ | config server のキー名 |
| `label` | string | ✅ | UI に表示するラベル |
| `description` | string | — | 補足説明。ツールチップで表示 |
| `type` | enum | ✅ | `string` / `integer` / `float` / `boolean` / `enum` / `object` / `array` |
| `min` | number | — | type=integer/float 時の最小値 |
| `max` | number | — | type=integer/float 時の最大値 |
| `options` | string[] | — | type=enum 時の選択肢 |
| `pattern` | string | — | type=string 時の正規表現バリデーション |
| `unit` | string | — | 単位ラベル（ms, px, bytes など）|
| `default` | any | — | デフォルト値。「デフォルトに戻す」で使用 |

---

## Proto スキーマ拡張

既存の `api/proto/k1s0/system/config/v1/config.proto` に ConfigEditorSchema 型を追加する。

```proto
// ============================================================
// ConfigEditorSchema（スキーマ定義）
// ============================================================

enum ConfigFieldType {
  CONFIG_FIELD_TYPE_UNSPECIFIED = 0;
  CONFIG_FIELD_TYPE_STRING      = 1;
  CONFIG_FIELD_TYPE_INTEGER     = 2;
  CONFIG_FIELD_TYPE_FLOAT       = 3;
  CONFIG_FIELD_TYPE_BOOLEAN     = 4;
  CONFIG_FIELD_TYPE_ENUM        = 5;
  CONFIG_FIELD_TYPE_OBJECT      = 6;
  CONFIG_FIELD_TYPE_ARRAY       = 7;
}

// ConfigFieldSchema はフィールドひとつの UI 定義。
message ConfigFieldSchema {
  string          key           = 1;
  string          label         = 2;
  string          description   = 3;
  ConfigFieldType type          = 4;
  int64           min           = 5;  // type=INTEGER/FLOAT 時
  int64           max           = 6;  // type=INTEGER/FLOAT 時
  repeated string options       = 7;  // type=ENUM 時
  string          pattern       = 8;  // type=STRING 時
  string          unit          = 9;  // 単位ラベル
  bytes           default_value = 10; // JSON encoded
}

// ConfigCategorySchema はカテゴリ（左ペイン一項目）の定義。
message ConfigCategorySchema {
  string                     id         = 1;
  string                     label      = 2;
  string                     icon       = 3;
  repeated string            namespaces = 4;
  repeated ConfigFieldSchema fields     = 5;
}

// ConfigEditorSchema は config-schema.yaml のサーバー表現。
message ConfigEditorSchema {
  string                        service          = 1;
  string                        namespace_prefix = 2;
  repeated ConfigCategorySchema categories       = 3;
  k1s0.system.common.v1.Timestamp updated_at    = 4;
}

// GetConfigSchemaRequest はスキーマ取得リクエスト。
message GetConfigSchemaRequest {
  string service_name = 1;
}

// GetConfigSchemaResponse はスキーマ取得レスポンス。
message GetConfigSchemaResponse {
  ConfigEditorSchema schema = 1;
}

// UpsertConfigSchemaRequest はスキーマ登録・更新リクエスト（CLI から呼ぶ）。
message UpsertConfigSchemaRequest {
  ConfigEditorSchema schema     = 1;
  string             updated_by = 2;
}

// UpsertConfigSchemaResponse はスキーマ登録・更新レスポンス。
message UpsertConfigSchemaResponse {
  ConfigEditorSchema schema = 1;
}
```

`ConfigService` に追加するメソッド:

```proto
// スキーマ取得（sys_auditor 以上）
rpc GetConfigSchema(GetConfigSchemaRequest) returns (GetConfigSchemaResponse);

// スキーマ登録・更新（sys_admin のみ、CLI 専用）
rpc UpsertConfigSchema(UpsertConfigSchemaRequest) returns (UpsertConfigSchemaResponse);
```

---

## config server エンドポイント拡張

既存の `regions/system/server/rust/config/` に以下のエンドポイントを実装する。

| Method | Path | Description | 認可 |
| ------ | ---- | ----------- | ---- |
| GET | `/api/v1/config-schema` | スキーマ一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/config-schema/:service_name` | スキーマ取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/config-schema/:service_name` | スキーマ登録・更新 | `sys_admin` のみ |

### GET /api/v1/config-schema/:service_name レスポンス例

```json
{
  "service": "order-server",
  "namespace_prefix": "service.order",
  "categories": [
    {
      "id": "database",
      "label": "データベース",
      "icon": "storage",
      "namespaces": ["service.order.database"],
      "fields": [
        {
          "key": "max_connections",
          "label": "最大接続数",
          "description": "DB 接続プールの上限",
          "type": "integer",
          "min": 1,
          "max": 1000,
          "default": 25,
          "unit": "connections"
        }
      ]
    }
  ]
}
```

### PUT /api/v1/config-schema/:service_name リクエスト例

`service_name` はパスパラメータで指定する。リクエストボディは以下の形式。
`updated_by` は JWT クレームから自動取得するため、リクエストボディには不要。

```json
{
  "namespace_prefix": "service.order",
  "schema_json": {
    "categories": [
      {
        "id": "database",
        "label": "データベース",
        "icon": "storage",
        "namespaces": ["service.order.database"],
        "fields": [...]
      }
    ]
  }
}
```

> **注意**: gRPC の `UpsertConfigSchemaRequest` とは異なるフィールド構成。REST では `service_name` をパスパラメータで渡し、`updated_by` はトークンから自動取得する。

---

## system-client SDK（ConfigInterpreter）

`system-client` に `ConfigInterpreter` と `ConfigEditorPage` を追加する。
クライアントアプリは `ConfigEditorPage` を `component-registry` に登録するだけでよい。

### React

```typescript
// system-client/src/config/ConfigInterpreter.ts

export class ConfigInterpreter {
  constructor(private apiClient: ApiClient) {}

  /** スキーマと現在値を取得して ConfigEditorConfig を構築する */
  async build(serviceName: string): Promise<ConfigEditorConfig> {
    const [schema, values] = await Promise.all([
      this.apiClient.get<ConfigEditorSchema>(`/api/v1/config-schema/${serviceName}`),
      this.apiClient.get<ServiceConfigResult>(`/api/v1/config/services/${serviceName}`),
    ]);
    return mergeSchemaWithValues(schema, values);
  }
}
```

```typescript
// system-client/src/config/ConfigEditorPage.tsx

export function ConfigEditorPage({ serviceName }: { serviceName: string }) {
  const { config, isDirty, save, reset, hasConflict } = useConfigEditor(serviceName);

  return (
    <div className="config-editor">
      <ConfigEditorHeader
        title={config.service}
        isDirty={isDirty}
        dirtyCount={config.dirtyCount}
        onSave={save}
        onReset={reset}
      />
      <div className="config-editor__body">
        <CategoryNav categories={config.categories} />
        <ConfigFieldList />
      </div>
      {hasConflict && <ConflictDialog />}
    </div>
  );
}
```

型ごとの UI コントロール:

| type | UI コントロール |
| ---- | -------------- |
| `integer` | 数値 input + スライダー（min/max 定義時）+ 単位ラベル |
| `float` | 数値 input + スライダー（min/max 定義時）+ 単位ラベル |
| `boolean` | トグルスイッチ |
| `enum` | ドロップダウン |
| `string` | テキスト input（pattern バリデーション付き）|
| `object` | 展開可能な JSON エディタ |
| `array` | タグ入力 |

### Flutter

```dart
// system_client/lib/src/config/config_interpreter.dart

class ConfigInterpreter {
  const ConfigInterpreter({required this.apiClient});
  final ApiClient apiClient;

  Future<ConfigEditorConfig> build(String serviceName) async {
    final results = await Future.wait([
      apiClient.get('/api/v1/config-schema/$serviceName'),
      apiClient.get('/api/v1/config/services/$serviceName'),
    ]);
    return mergeSchemaWithValues(
      ConfigEditorSchema.fromJson(results[0]),
      ServiceConfigResult.fromJson(results[1]),
    );
  }
}
```

---

## CLI コマンド

既存の `generate` / `validate` コマンドに追加する。対話式フローに従う。

### generate config-types（スキーマ登録 + 型定義生成）

```
? 操作を選択してください
> ひな形生成
  → 設定スキーマ型ファイル生成

? config-schema.yaml のパス: ./config-schema.yaml

? 対象フレームワーク（複数選択可）
> [x] React (TypeScript)
  [x] Flutter (Dart)

? config server に push しますか？（スキーマをサーバーに登録する）
> はい
  いいえ（型定義ファイルのみ生成）

[確認] 以下の内容で実行します。よろしいですか？
    スキーマ:  ./config-schema.yaml (order-server)
    push:      https://config.k1s0-system.svc.cluster.local
    React   →  src/config/__generated__/config-types.ts
    Flutter →  lib/config/__generated__/config_types.dart

> はい
  いいえ

Pushing schema...
  ✅ schema registered: order-server (3 categories, 8 fields)
Generating type definitions...
  ✅ src/config/__generated__/config-types.ts
  ✅ lib/config/__generated__/config_types.dart
```

#### 生成ファイル（React）

```typescript
// src/config/__generated__/config-types.ts
// このファイルは CLI が自動生成する。直接編集しないこと。
// config-schema.yaml から生成: 2026-02-24

export const ConfigKeys = {
  DATABASE: {
    MAX_CONNECTIONS: 'max_connections',
    SSL_MODE:        'ssl_mode',
    POOL_TIMEOUT_MS: 'pool_timeout_ms',
  },
  KAFKA: {
    CONSUMER_GROUP:       'consumer_group',
    MAX_POLL_INTERVAL_MS: 'max_poll_interval_ms',
  },
  FEATURE_FLAGS: {
    ENABLE_BULK_ORDER:   'enable_bulk_order',
    MAX_ITEMS_PER_ORDER: 'max_items_per_order',
  },
} as const;

export type ConfigValues = {
  'database.max_connections':   number;
  'database.ssl_mode':          'disable' | 'require' | 'verify-ca' | 'verify-full';
  'database.pool_timeout_ms':   number;
  'kafka.consumer_group':       string;
  'kafka.max_poll_interval_ms': number;
  'feature_flags.enable_bulk_order':   boolean;
  'feature_flags.max_items_per_order': number;
};
```

#### 生成ファイル（Flutter）

```dart
// lib/config/__generated__/config_types.dart
// このファイルは CLI が自動生成する。直接編集しないこと。

enum DatabaseConfigKey {
  maxConnections,
  sslMode,
  poolTimeoutMs;

  String get key => switch (this) {
    DatabaseConfigKey.maxConnections => 'max_connections',
    DatabaseConfigKey.sslMode        => 'ssl_mode',
    DatabaseConfigKey.poolTimeoutMs  => 'pool_timeout_ms',
  };
}

enum SslMode { disable, require, verifyCa, verifyFull }
```

### validate config-schema（バリデーション）

```
? 操作を選択してください
> バリデーション
  → 設定スキーマ

? config-schema.yaml のパス: ./config-schema.yaml

Checking config-schema.yaml...

  ✅ JSON Schema バリデーション OK
  ✅ namespace prefix 整合性 OK（全 namespaces が service.order で始まる）
  ✅ field key の重複なし
  ❌ category 'database' の field 'max_retry' に type が未指定
       → type を指定してください (string | integer | float | boolean | enum | object | array)

1 error found.
```

チェック項目:

| チェック | 内容 |
| -------- | ---- |
| JSON Schema バリデーション | `config-schema.yaml` が JSON Schema に準拠しているか |
| namespace prefix 整合性 | `namespaces` が `namespace_prefix` で始まっているか |
| field key の重複 | 同カテゴリ内に同一 key が存在しないか |
| type 必須チェック | 全フィールドに type が指定されているか |
| enum options チェック | type=enum 時に options が空でないか |
| default 型整合性 | default 値が type と一致しているか |

CI パイプラインへの統合:

```yaml
# .github/workflows/ci.yaml
- name: Validate config schema
  run: k1s0 validate config-schema
```

---

## navigation.yaml との統合

Config Editor は navigation.yaml の `ConfigEditorPage` コンポーネントとして統合される。
tier3 サービス側は `component-registry` に1行追加するだけでよい。

```yaml
# navigation.yaml（tier3 service 側）
routes:
  - id: config
    path: /config/:service
    component_id: ConfigEditorPage
    guards: [auth_required, admin_only]
    params:
      - name: service
        type: string
```

```typescript
// component-registry に追加するだけで設定画面が完成する
import { ConfigEditorPage } from 'system-client';

export const componentRegistry: ComponentRegistry = {
  ConfigEditorPage,  // ← これだけ
  DashboardPage:  () => import('../../pages/DashboardPage'),
  // ...
};
```

---

## UI レイアウト

```
┌──────────────────────────────────────────────────────────────┐
│  ⚙ order-server 設定         [変更 2件あり]  [破棄] [保存]  │
├───────────────┬──────────────────────────────────────────────┤
│               │                                              │
│  > データベース│  データベース                                │
│    メッセージ  │  ──────────────────────────────────────────  │
│    機能フラグ  │                                              │
│               │  最大接続数                                  │
│               │  DB接続プールの上限                          │
│               │  [ ██████░░░░░░░░░░ ]  [ 25 ] connections   │
│               │  (1 〜 1000)                                  │
│               │                                              │
│               │  SSL モード                                  │
│               │  [ require        ▼ ]                        │
│               │                                              │
│               │  接続タイムアウト                            │
│               │  [ ████░░░░░░░░░░░ ]  [ 5000 ] ms           │
│               │  (100 〜 30000)                               │
│               │                                              │
│               │                     [デフォルトに戻す]       │
└───────────────┴──────────────────────────────────────────────┘
```

### UX 仕様

| 動作 | 仕様 |
| ---- | ---- |
| 変更検知 | 元の値との差分をリアルタイムに検知し「変更 N件あり」表示 |
| まとめて保存 | カテゴリ内の全変更を 1 回の操作でまとめて PUT |
| 楽観ロック | 409 Conflict 時「他のユーザーが更新しました」ダイアログ表示 |
| デフォルトに戻す | `default` 値を入力欄に適用（未保存状態） |
| バリデーション | min/max/pattern を入力中にリアルタイムチェック |
| リアルタイム反映 | gRPC WatchConfig ストリームで他クライアントにも即時反映 |

---

## 開発フロー

```
1. config-schema.yaml を編集
   （新しい設定値を追加・型定義を変更）

2. k1s0 validate config-schema
   → エラーがあれば修正

3. k1s0 generate config-types
   → ① config server にスキーマ登録
   → ② 型定義ファイル自動生成（React / Flutter）

4. git commit / PR
   （フロントエンドのコード変更は不要）

5. デプロイ後、/config/order-server を開くと新しい設定項目が自動表示
```

---

## ディレクトリ構成

```
regions/system/
├── client/
│   ├── react/
│   │   └── system-client/
│   │       └── src/
│   │           └── config/
│   │               ├── ConfigInterpreter.ts
│   │               ├── ConfigInterpreter.test.ts
│   │               ├── ConfigEditorPage.tsx
│   │               ├── ConfigEditorPage.test.tsx
│   │               ├── components/
│   │               │   ├── CategoryNav.tsx
│   │               │   ├── ConfigFieldList.tsx
│   │               │   └── fields/
│   │               │       ├── IntegerField.tsx
│   │               │       ├── FloatField.tsx
│   │               │       ├── BooleanField.tsx
│   │               │       ├── EnumField.tsx
│   │               │       ├── StringField.tsx
│   │               │       ├── ObjectField.tsx
│   │               │       └── ArrayField.tsx
│   │               ├── hooks/
│   │               │   └── useConfigEditor.ts
│   │               └── types.ts
│   └── flutter/
│       └── system_client/
│           └── lib/src/config/
│               ├── config_interpreter.dart
│               ├── config_editor_page.dart
│               └── widgets/
│                   ├── category_nav.dart
│                   ├── config_field_list.dart
│                   └── fields/
│                       ├── integer_field.dart
│                       ├── boolean_field.dart
│                       ├── enum_field.dart
│                       └── string_field.dart
└── server/
    └── rust/
        └── config/
            └── src/
                └── adapter/
                    └── handler/
                        └── config_schema_handler.rs  ← 新規追加

api/
└── proto/
    └── k1s0/
        └── system/
            └── config/
                └── v1/
                    └── config.proto  ← ConfigEditorSchema を追加

regions/service/
└── {service_name}/
    └── config-schema.yaml  ← 各サービスの定義ファイル（新規）
```

---

## 実装ロードマップ

| Phase | 内容 | 優先度 |
| ----- | ---- | ------ |
| 1 | proto 拡張（ConfigEditorSchema 型追加） | 高 |
| 2 | config server に GET/PUT `/api/v1/config-schema/:service_name` 追加 | 高 |
| 3 | CLI `validate config-schema` コマンド | 高 |
| 4 | CLI `generate config-types` コマンド（スキーマ登録 + 型定義生成） | 高 |
| 5 | React `ConfigInterpreter` + `ConfigEditorPage` | 高 |
| 6 | React フィールドコンポーネント（integer / boolean / enum / string） | 高 |
| 7 | navigation.yaml との統合（ConfigEditorPage をコンポーネント登録） | 中 |
| 8 | Flutter `ConfigInterpreter` + `ConfigEditorPage` ウィジェット | 中 |
| 9 | JSON Schema 生成（`config-schema.yaml` の IDE 補完用） | 中 |
| 10 | React フィールドコンポーネント（object / array） | 低 |
| 11 | gRPC WatchConfig によるリアルタイム更新反映 | 低 |

---

## 設計上の補足

- **config-schema.yaml の配置** — 各 tier3 サービスのリポジトリルートに配置する。navigation.yaml と同じ扱い
- **Local-first 開発** — `generate config-types` 実行後は config server なしで型チェックだけ動く。UI の確認は mock データで対応
- **後方互換性** — フィールドの削除は `deprecated: true` でマークし、1 バージョン猶予を設ける
- **権限** — スキーマ取得は `sys_auditor` 以上、スキーマ登録は `sys_admin` のみ。値の編集は既存の config server の権限（`sys_operator`）を踏襲する
- **audit log** — 設定値の変更は既存の監査ログに記録される。スキーマ変更も同様に記録する

---

## 関連ドキュメント

- [system-config-server設計](../../servers/config/server.md)
- [navigation設計](navigation設計.md)
- [system-client設計](../../servers/_common/client.md)
- [proto設計](../../architecture/api/proto設計.md)
- [CLIフロー](../flow/CLIフロー.md)
- [RBAC設計](../../architecture/auth/RBAC設計.md)
- [tier-architecture](../../architecture/overview/tier-architecture.md)
