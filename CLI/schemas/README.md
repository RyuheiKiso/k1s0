# k1s0 Schemas

k1s0 CLI が使用する JSON Schema 定義。

## スキーマ一覧

| ファイル | 説明 |
|---------|------|
| `manifest.schema.json` | `.k1s0/manifest.json` のスキーマ定義 |
| `manifest.example.json` | manifest.json のサンプル |

## manifest.json

### 概要

`k1s0 new-feature` で生成されたサービスごとに `.k1s0/manifest.json` が作成される。

**配置先:**
```
feature/backend/rust/{service_name}/.k1s0/manifest.json
```

### 用途

1. **テンプレートの出自の記録**: どのテンプレートから生成されたかを記録
2. **upgrade の差分計算**: 旧テンプレート → 新テンプレートの差分を計算
3. **managed/protected の境界**: CLI が自動更新してよい領域の定義
4. **衝突検知**: チェックサムによる手動変更の検知

### CLI の使用方法

```rust
// スキーマのバリデーション例（概念）
use jsonschema::JSONSchema;

let schema = include_str!("schemas/manifest.schema.json");
let manifest = read_manifest(".k1s0/manifest.json")?;
let compiled = JSONSchema::compile(&schema)?;
compiled.validate(&manifest)?;
```

### 手動編集について

manifest.json は **CLI が管理** するファイルであり、手動編集は非推奨。

手動編集が必要な場合:
1. `k1s0 lint` でバリデーションを確認
2. 変更理由をコメントやコミットメッセージに記録

## 関連ドキュメント

- [ADR-0002](../../docs/adr/ADR-0002-versioning-and-manifest.md): バージョニングと manifest の型の固定
