# 非推奨化ポリシー

このドキュメントでは、k1s0 における domain、API、およびその他のコンポーネントの非推奨化に関するポリシーを定義します。

## 1. 概要

非推奨化（Deprecation）は、既存の機能やAPIを段階的に廃止するためのプロセスです。適切な非推奨化ポリシーにより、依存者に十分な移行期間を提供し、システムの安定性を維持します。

---

## 2. deprecated フラグの使用方法

### 2.1 manifest.json での設定

domain を非推奨にする場合、manifest.json に `deprecated` フィールドを設定します。

```json
{
  "layer": "domain",
  "version": "1.5.0",
  "deprecated": {
    "since": "1.5.0",
    "migrate_to": "manufacturing-v2",
    "deadline": "2026-12-31",
    "reason": "新しい manufacturing-v2 domain に機能を統合しました"
  }
}
```

### 2.2 フィールドの説明

| フィールド | 必須 | 説明 |
|-----------|:----:|------|
| `since` | Yes | 非推奨化を開始したバージョン |
| `migrate_to` | No | 移行先の domain 名 |
| `deadline` | No | 非推奨期間の終了日（削除予定日） |
| `reason` | Yes | 非推奨化の理由 |

### 2.3 コマンドでの設定

```bash
# domain を非推奨化
k1s0 domain deprecate --name manufacturing \
  --migrate-to manufacturing-v2 \
  --deadline 2026-12-31 \
  --reason "新しい manufacturing-v2 domain に機能を統合しました"

# 非推奨化を解除
k1s0 domain undeprecate --name manufacturing
```

---

## 3. 非推奨化から削除までの期間

### 3.1 標準的な期間

| 種類 | 最小非推奨期間 | 推奨期間 |
|------|--------------|---------|
| domain | 6ヶ月 | 12ヶ月 |
| 公開 API | 3ヶ月 | 6ヶ月 |
| 内部 API | 1ヶ月 | 3ヶ月 |
| 設定オプション | 3ヶ月 | 6ヶ月 |

### 3.2 期間の考慮事項

非推奨期間を決定する際は以下を考慮します:

1. **依存者の数**: 多くの feature が依存している場合は長めに
2. **変更の影響度**: 破壊的変更が大きい場合は長めに
3. **移行の複雑さ**: 移行作業が複雑な場合は長めに
4. **セキュリティ**: セキュリティ問題の場合は短縮可能

### 3.3 例外的な短縮

セキュリティ脆弱性やクリティカルなバグの場合、期間を短縮できます:

```json
{
  "deprecated": {
    "since": "1.5.1",
    "deadline": "2026-02-28",
    "reason": "セキュリティ脆弱性 CVE-2026-XXXX の修正。緊急移行が必要です",
    "urgency": "high"
  }
}
```

---

## 4. migrate_to の指定方法

### 4.1 基本的な指定

```json
{
  "deprecated": {
    "migrate_to": "manufacturing-v2"
  }
}
```

### 4.2 複数の移行先

domain が複数の domain に分割された場合:

```json
{
  "deprecated": {
    "migrate_to": ["manufacturing-core", "manufacturing-scheduling"],
    "migration_guide": "https://docs.example.com/migration/manufacturing-split"
  }
}
```

### 4.3 移行先がない場合

機能自体が廃止される場合:

```json
{
  "deprecated": {
    "migrate_to": null,
    "reason": "この機能は廃止されます。代替機能はありません"
  }
}
```

---

## 5. 依存者への通知方法

### 5.1 Lint 警告（K044）

非推奨 domain を使用している feature は、`k1s0 lint` で K044 警告が表示されます。

```
[K044] WARNING: domain 'manufacturing' は非推奨です
       File: feature/backend/rust/work-order-api/.k1s0/manifest.json
       Since: 1.5.0
       Migrate to: manufacturing-v2
       Deadline: 2026-12-31
       Reason: 新しい manufacturing-v2 domain に機能を統合しました
```

### 5.2 コンパイル時警告

Rust コードでの警告:

```rust
// domain 内のコード
#[deprecated(since = "1.5.0", note = "Use manufacturing_v2::WorkOrder instead")]
pub struct WorkOrder { ... }
```

使用側での警告:
```
warning: use of deprecated struct `manufacturing::WorkOrder`: Use manufacturing_v2::WorkOrder instead
```

### 5.3 依存者の一覧表示

```bash
# 非推奨 domain の依存者を確認
k1s0 domain dependents --name manufacturing --deprecated-only

# 出力例
WARNING: domain 'manufacturing' is deprecated since 1.5.0

Features depending on deprecated 'manufacturing':
  work-order-api          ^1.2.0    feature/backend/rust/work-order-api
  manufacturing-report       ^1.0.0    feature/backend/rust/manufacturing-report
  legacy-adapter          1.5.0     feature/backend/rust/legacy-adapter

Migration deadline: 2026-12-31
Migrate to: manufacturing-v2
```

### 5.4 CI/CD での通知

`.github/workflows/deprecation-check.yml`:

```yaml
name: Deprecation Check

on:
  schedule:
    - cron: '0 9 * * 1'  # 毎週月曜 9:00

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check deprecated dependencies
        run: |
          k1s0 lint --rules K044 --json > deprecation-report.json

      - name: Notify if deprecated found
        if: ${{ contains(steps.check.outputs.result, 'K044') }}
        uses: slackapi/slack-github-action@v1
        with:
          channel-id: 'dev-alerts'
          slack-message: 'Deprecated domain usage detected. See report.'
```

---

## 6. 非推奨化のライフサイクル

### 6.1 フェーズ 1: 非推奨化の準備

1. 移行先の domain/API を作成
2. 移行ドキュメントを作成
3. 非推奨化の日程を決定

### 6.2 フェーズ 2: 非推奨化の開始

1. `deprecated` フラグを設定
2. CHANGELOG に記載
3. 依存者への通知（メール、Slack など）

```bash
k1s0 domain deprecate --name manufacturing \
  --migrate-to manufacturing-v2 \
  --deadline 2026-12-31 \
  --reason "新しい manufacturing-v2 domain に機能を統合しました"
```

### 6.3 フェーズ 3: 移行期間

1. 依存者は段階的に移行
2. 定期的に移行状況を確認
3. サポートの提供（質問対応、ドキュメント更新）

```bash
# 移行状況の確認
k1s0 domain dependents --name manufacturing
```

### 6.4 フェーズ 4: 削除

1. 全ての依存者が移行完了を確認
2. domain を削除
3. CHANGELOG に削除を記載

```bash
# 依存者がいないことを確認
k1s0 domain dependents --name manufacturing
# => No features depend on 'manufacturing'

# domain を削除（手動）
rm -rf domain/backend/rust/manufacturing
```

---

## 7. コードレベルの非推奨化

### 7.1 Rust での非推奨化

```rust
/// 作業指示
///
/// # Deprecated
///
/// `1.5.0` から非推奨です。代わりに [`manufacturing_v2::WorkOrder`] を使用してください。
#[deprecated(since = "1.5.0", note = "Use manufacturing_v2::WorkOrder instead")]
pub struct WorkOrder {
    // ...
}

impl WorkOrder {
    /// 作業指示を開始
    ///
    /// # Deprecated
    ///
    /// `1.5.0` から非推奨です。代わりに [`manufacturing_v2::WorkOrder::start`] を使用してください。
    #[deprecated(since = "1.5.0", note = "Use manufacturing_v2::WorkOrder::start instead")]
    pub fn start(&mut self) -> Result<(), DomainError> {
        // 内部で新しい実装を呼び出すことも可能
        self.inner.start()
    }
}
```

### 7.2 Go での非推奨化

```go
// WorkOrder represents a work order.
//
// Deprecated: Use manufacturing_v2.WorkOrder instead (since 1.5.0).
type WorkOrder struct {
    // ...
}

// Start starts the work order.
//
// Deprecated: Use manufacturing_v2.WorkOrder.Start instead (since 1.5.0).
func (w *WorkOrder) Start() error {
    // ...
}
```

### 7.3 TypeScript での非推奨化

```typescript
/**
 * Work order entity.
 * @deprecated Since 1.5.0. Use WorkOrder from manufacturing-v2 instead.
 */
export interface WorkOrder {
  // ...
}

/**
 * Creates a work order.
 * @deprecated Since 1.5.0. Use createWorkOrder from manufacturing-v2 instead.
 */
export function createWorkOrder(params: CreateWorkOrderParams): WorkOrder {
  // ...
}
```

---

## 8. 移行ガイドの作成

### 8.1 移行ガイドの構成

```markdown
# manufacturing から manufacturing-v2 への移行ガイド

## 概要

manufacturing domain は 1.5.0 で非推奨となり、2026-12-31 に削除されます。
このガイドでは manufacturing-v2 への移行手順を説明します。

## 主な変更点

1. WorkOrder.quantity の型が u32 から Quantity 値オブジェクトに変更
2. SchedulingService が別 domain (manufacturing-scheduling) に分離

## 移行手順

### Step 1: 依存関係の更新

manifest.json を更新:
\`\`\`diff
{
  "dependencies": {
    "domain": {
-     "manufacturing": "^1.5.0"
+     "manufacturing-v2": "^2.0.0",
+     "manufacturing-scheduling": "^1.0.0"
    }
  }
}
\`\`\`

### Step 2: import 文の更新

\`\`\`diff
- use manufacturing::domain::entities::WorkOrder;
+ use manufacturing_v2::domain::entities::WorkOrder;
\`\`\`

### Step 3: 型の変更への対応

\`\`\`diff
- let quantity: u32 = work_order.quantity;
+ let quantity: &Quantity = work_order.quantity();
+ let quantity_value: u32 = quantity.value();
\`\`\`

## FAQ

Q: 両方の domain を同時に使用できますか？
A: はい、移行期間中は両方を依存に含めることができます。

## サポート

移行に関する質問は #dev-support チャンネルまで。
```

### 8.2 移行ガイドの配置

```
docs/
└── migration/
    ├── README.md                         # 移行ガイド一覧
    └── manufacturing-to-manufacturing-v2.md    # 個別の移行ガイド
```

---

## 9. チェックリスト

### 9.1 非推奨化時のチェックリスト

- [ ] 移行先の domain/API が準備できている
- [ ] 移行ガイドを作成した
- [ ] `deprecated` フラグを設定した
- [ ] CHANGELOG を更新した
- [ ] 依存者に通知した
- [ ] CI/CD に非推奨チェックを追加した
- [ ] サポート体制を整えた

### 9.2 削除時のチェックリスト

- [ ] 全ての依存者が移行完了した
- [ ] 非推奨期間が終了した
- [ ] 最終通知を送った
- [ ] domain/API を削除した
- [ ] CHANGELOG を更新した
- [ ] ドキュメントを更新した

---

## 10. 関連ドキュメント

- [domain バージョン管理ガイド](../guides/domain-versioning.md)
- [3層構造への移行ガイド](../guides/migration-to-three-tier.md)
- [Lint ルール K044](../design/lint.md)
- [ADR-0006: 3層アーキテクチャ](../adr/ADR-0006-three-layer-architecture.md)
