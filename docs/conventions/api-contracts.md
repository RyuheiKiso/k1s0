# API 契約管理規約

本ドキュメントは、k1s0 における API 契約（gRPC / REST）の管理規約を定義する。

## 1. 基本方針

- **API First**: 契約（スキーマ）を正本として管理する
- 後方互換性を原則とし、破壊的変更は CI で検知・拒否する
- 生成物は Git 管理外とし、再現性は CI で検証する

## 2. 正本の配置

| 種別 | 正本の置き場 | 説明 |
|------|-------------|------|
| gRPC | `{service}/proto/*.proto` | Protocol Buffers 定義 |
| REST | `{service}/openapi/openapi.yaml` | OpenAPI 3.x 定義 |

## 3. 生成物の配置

| 種別 | 生成物の置き場 | Git 管理 |
|------|---------------|----------|
| gRPC | `{service}/gen/` | 対象外（`.gitignore`） |
| REST | `{service}/openapi/gen/` | 対象外（`.gitignore`） |

## 4. buf 設定（gRPC）

各サービスに以下を配置：

```
{service}/
├── proto/
│   └── {service_name}/
│       └── v1/
│           └── {service}.proto
├── buf.yaml
└── buf.lock
```

### buf.yaml 例

```yaml
version: v2
modules:
  - path: proto
lint:
  use:
    - DEFAULT
breaking:
  use:
    - FILE
```

## 5. gRPC 互換性ルール

### 5.1 許可される変更（後方互換）

- フィールドの追加（任意フィールド）
- 新しいサービス/メソッドの追加
- `deprecated = true` の付与

### 5.2 禁止される変更（破壊的）

| 変更 | 理由 |
|------|------|
| フィールドの削除 | 既存クライアントが失敗 |
| フィールド番号の再採番 | ワイヤフォーマット破壊 |
| フィールドの型変更 | デシリアライズ失敗 |
| `oneof` の既存ケース削除 | 既存メッセージが不正に |
| サービス名/メソッド名の変更 | 既存クライアントが失敗 |
| パッケージ名の変更 | 既存クライアントが失敗 |

### 5.3 廃止の手順

1. `deprecated = true` を付与
2. 移行期間を設ける
3. 例外として MAJOR リリースで削除（ADR 必須）

### 5.4 使わなくなったフィールド

削除ではなく `reserved` を宣言：

```protobuf
message User {
  reserved 2, 15, 9 to 11;
  reserved "old_field", "temp_field";
  // ...
}
```

## 6. REST（OpenAPI）互換性ルール

### 6.1 許可される変更（後方互換）

- エンドポイントの追加
- 任意フィールドの追加
- `deprecated: true` の付与

### 6.2 禁止される変更（破壊的）

| 変更 | 理由 |
|------|------|
| エンドポイントの削除 | 既存クライアントが失敗 |
| パス/HTTP メソッドの変更 | 既存クライアントが失敗 |
| フィールドの削除 | 既存クライアントが失敗 |
| フィールドの型変更 | パース失敗 |
| 任意 → 必須（required 追加） | 既存リクエストが失敗 |
| バリデーション強化 | 既存リクエストが失敗 |
| 必須フィールドの追加 | 既存クライアントが対応できない |

## 7. CI 必須チェック

### 7.1 gRPC

```yaml
# .github/workflows/ci.yaml 例
- name: buf lint
  run: buf lint

- name: buf breaking
  run: buf breaking --against '.git#branch=main'
```

### 7.2 REST

```yaml
# openapi-diff 等を使用
- name: OpenAPI diff
  run: openapi-diff --fail-on-incompatible main...HEAD
```

### 7.3 生成一致チェック

```yaml
- name: Generate
  run: buf generate

- name: Check diff
  run: |
    if [ -n "$(git status --porcelain gen/)" ]; then
      echo "Generated files are out of sync"
      exit 1
    fi
```

## 8. 破壊的変更（MAJOR）の手順

破壊的変更が必要な場合は、同一 PR で以下を必須とする：

1. **ADR**: `docs/adr/ADR-XXXX-*.md`
   - なぜ必要か
   - 代替案
   - 影響範囲
   - 移行計画

2. **UPGRADE.md** または **リリースノート**
   - 影響一覧
   - 移行手順
   - ロールバック方法

3. **CLI 支援**
   - `k1s0 upgrade --check` が破壊箇所を検知できること
   - `k1s0 upgrade` が安全に止まれること

## 9. 生成一致の再現性指標

生成物を Git 管理外にしつつ、再現性を担保するため、以下のいずれかを採用：

### 方式 A: fingerprint ファイル

```
{service}/gen/.k1s0-gen.sha256
```

正本（proto/openapi）のハッシュを保存し、CI で一致を検証。

### 方式 B: CI での再生成 + diff

CI で毎回再生成し、差分があれば失敗。

## 関連ドキュメント

- [サービス構成規約](service-structure.md)
- [構想.md](../../work/構想.md): 全体方針（5. 契約管理）
