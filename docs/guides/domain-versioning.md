# domain バージョン管理ガイド

このガイドでは、k1s0 の domain 層におけるバージョン管理の方法について説明します。

## 1. Semantic Versioning（SemVer）の適用

### 1.1 バージョン形式

domain は Semantic Versioning 2.0.0 に従います。

```
MAJOR.MINOR.PATCH

例: 1.2.3
     │ │ └── PATCH: 後方互換なバグ修正
     │ └──── MINOR: 後方互換な機能追加
     └────── MAJOR: 破壊的変更（API 非互換）
```

### 1.2 バージョン更新の判断基準

| 変更内容 | 更新するバージョン | 例 |
|---------|-----------------|-----|
| バグ修正（API 変更なし） | PATCH | 1.2.3 -> 1.2.4 |
| 新機能追加（後方互換） | MINOR | 1.2.3 -> 1.3.0 |
| 既存フィールドの追加（オプショナル） | MINOR | 1.2.3 -> 1.3.0 |
| 既存 API の変更・削除 | MAJOR | 1.2.3 -> 2.0.0 |
| フィールド名の変更 | MAJOR | 1.2.3 -> 2.0.0 |
| 必須フィールドの追加 | MAJOR | 1.2.3 -> 2.0.0 |
| 型の変更 | MAJOR | 1.2.3 -> 2.0.0 |

### 1.3 pre-release バージョン

開発中のバージョンには pre-release 識別子を使用できます。

```
0.1.0-alpha.1    # アルファ版
0.1.0-beta.1     # ベータ版
0.1.0-rc.1       # リリース候補
```

---

## 2. バージョン管理コマンド

### 2.1 現在のバージョン確認

```bash
# 特定の domain のバージョンを確認
k1s0 domain version --name manufacturing

# 出力例
manufacturing: 1.2.0
```

### 2.2 バージョンの更新（bump）

```bash
# PATCH バージョンを上げる（1.2.0 -> 1.2.1）
k1s0 domain version --name manufacturing --bump patch

# MINOR バージョンを上げる（1.2.0 -> 1.3.0）
k1s0 domain version --name manufacturing --bump minor

# MAJOR バージョンを上げる（1.2.0 -> 2.0.0）
k1s0 domain version --name manufacturing --bump major
```

### 2.3 バージョンの直接指定

```bash
# 特定のバージョンを設定
k1s0 domain version --name manufacturing --set 2.0.0

# pre-release バージョンを設定
k1s0 domain version --name manufacturing --set 2.0.0-beta.1
```

### 2.4 オプション

```bash
k1s0 domain version --name manufacturing --bump major \
  --message "WorkOrder.quantity を Quantity 値オブジェクトに変更" \
  --no-changelog   # CHANGELOG.md を更新しない
```

---

## 3. breaking_changes の記録

### 3.1 manifest.json での記録

破壊的変更がある場合は、manifest.json の `breaking_changes` フィールドに記録します。

```json
{
  "version": "2.0.0",
  "breaking_changes": {
    "2.0.0": "WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更",
    "1.0.0": "初回リリース"
  }
}
```

### 3.2 コマンドでの記録

```bash
# MAJOR バージョン更新時に破壊的変更を記録
k1s0 domain version --name manufacturing --bump major \
  --message "WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更"
```

### 3.3 breaking_changes の書き方

**良い記録**:
```json
{
  "2.0.0": "WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更。移行方法: Quantity::new(old_value, QuantityUnit::Piece) を使用",
  "1.5.0": "SchedulingService.calculate_priority の引数に deadline を追加（必須）",
  "1.3.0": "WorkOrderStatus.Pending を WorkOrderStatus.Scheduled にリネーム"
}
```

**悪い記録**:
```json
{
  "2.0.0": "破壊的変更",  // 内容が不明
  "1.5.0": "API変更"      // 具体性がない
}
```

---

## 4. CHANGELOG の管理

### 4.1 CHANGELOG.md の形式

Keep a Changelog 形式に従います。

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- WorkOrderService に cancel_work_order メソッドを追加

## [2.0.0] - 2026-01-28

### Changed
- **BREAKING**: WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更

### Migration
- `work_order.quantity` を `Quantity::new(old_value, QuantityUnit::Piece)` に置き換えてください

## [1.5.0] - 2026-01-15

### Added
- SchedulingService に calculate_priority メソッドを追加

### Changed
- **BREAKING**: calculate_priority に deadline 引数を追加（必須）

## [1.4.0] - 2026-01-10

### Added
- WorkOrderStatus に Cancelled ステータスを追加

### Fixed
- WorkOrder.complete() でステータス検証が正しく動作しない問題を修正

## [1.0.0] - 2026-01-01

### Added
- WorkOrder エンティティ
- Quantity 値オブジェクト
- WorkOrderRepository トレイト
- SchedulingService
```

### 4.2 自動更新

`k1s0 domain version --bump` コマンドは自動的に CHANGELOG.md を更新します。

```bash
# バージョン更新と CHANGELOG 更新
k1s0 domain version --name manufacturing --bump minor \
  --message "WorkOrderService に cancel_work_order メソッドを追加"

# CHANGELOG が以下のように更新される
## [1.6.0] - 2026-01-28

### Added
- WorkOrderService に cancel_work_order メソッドを追加
```

---

## 5. feature からの依存バージョン制約

### 5.1 バージョン制約の形式

feature の manifest.json でバージョン制約を指定します。

```json
{
  "layer": "feature",
  "domain": "manufacturing",
  "domain_version": "^1.2.0",
  "dependencies": {
    "domain": {
      "manufacturing": "^1.2.0",
      "inventory": "~2.0.0"
    }
  }
}
```

### 5.2 制約の種類

| 制約 | 意味 | 例 | 許可されるバージョン |
|------|------|-----|-------------------|
| `^1.2.3` | 互換性のある変更を許可 | ^1.2.3 | >=1.2.3, <2.0.0 |
| `~1.2.3` | パッチレベルの変更のみ | ~1.2.3 | >=1.2.3, <1.3.0 |
| `1.2.3` | 完全一致 | 1.2.3 | =1.2.3 |
| `>=1.2.0` | 以上 | >=1.2.0 | >=1.2.0 |
| `>=1.2.0, <2.0.0` | 範囲指定 | | >=1.2.0, <2.0.0 |
| `*` | 任意のバージョン | * | 全て |

### 5.3 推奨される制約

```json
{
  "dependencies": {
    "domain": {
      // 推奨: ^（キャレット）を使用
      // MINOR/PATCH の更新を自動的に受け入れる
      "manufacturing": "^1.2.0",

      // 厳格な制御が必要な場合: ~（チルダ）を使用
      "inventory": "~2.0.0",

      // 避けるべき: 完全一致
      // "legacy": "1.0.0"  // 更新が困難になる
    }
  }
}
```

### 5.4 依存の更新

```bash
# domain 依存のバージョンを更新
k1s0 feature update-domain --name work-order-api --domain manufacturing --version "^2.0.0"

# 全ての domain 依存を最新に更新
k1s0 feature update-domain --name work-order-api --all --latest
```

---

## 6. バージョン整合性の検証

### 6.1 Lint ルール

| ルール | 説明 |
|--------|------|
| K042 | domain バージョン制約と実際のバージョンの不整合を検出 |
| K046 | breaking_changes の影響を警告 |
| K047 | domain 層に version が未設定の場合にエラー |

### 6.2 検証コマンド

```bash
# バージョン整合性をチェック
k1s0 lint

# 出力例
[K042] ERROR: domain 'manufacturing' のバージョン 1.5.0 が制約 ^2.0.0 を満たしません
       File: feature/backend/rust/work-order-api/.k1s0/manifest.json
       Hint: domain_version を更新するか、domain のバージョンを更新してください

[K046] WARNING: domain 'manufacturing' v2.0.0 に破壊的変更があります: WorkOrder.quantity の型を変更
       File: feature/backend/rust/work-order-api/.k1s0/manifest.json
       Hint: CHANGELOG を確認し、必要に応じてコードを更新してください
```

---

## 7. バージョンアップの影響分析

### 7.1 影響範囲の確認

```bash
# domain のバージョンアップによる影響を分析
k1s0 domain impact --name manufacturing --from 1.5.0 --to 2.0.0

# 出力例
Domain: manufacturing
Version change: 1.5.0 -> 2.0.0 (MAJOR)

Breaking changes:
  - 2.0.0: WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更

Affected features (5):
  - work-order-api (constraint: ^1.2.0) - INCOMPATIBLE
  - work-order-dashboard (constraint: ^1.5.0) - INCOMPATIBLE
  - manufacturing-report (constraint: ^1.0.0) - INCOMPATIBLE
  - inventory-sync (constraint: ~1.5.0) - INCOMPATIBLE
  - legacy-adapter (constraint: 1.5.0) - INCOMPATIBLE

Recommendation:
  Update all affected features to use ^2.0.0 and apply migration steps.
```

### 7.2 依存する feature の一覧

```bash
# domain に依存する feature を一覧表示
k1s0 domain dependents --name manufacturing

# 出力例
Features depending on 'manufacturing':
  work-order-api          ^1.2.0    feature/backend/rust/work-order-api
  work-order-dashboard    ^1.5.0    feature/frontend/react/work-order-dashboard
  manufacturing-report       ^1.0.0    feature/backend/rust/manufacturing-report
  inventory-sync          ~1.5.0    feature/backend/go/inventory-sync
  legacy-adapter          1.5.0     feature/backend/rust/legacy-adapter
```

---

## 8. バージョン管理のベストプラクティス

### 8.1 リリースプロセス

1. **開発ブランチで変更を実装**
2. **テストを実行し、全てパスすることを確認**
3. **CHANGELOG.md を更新**
4. **バージョンを更新**
   ```bash
   k1s0 domain version --name manufacturing --bump minor --message "..."
   ```
5. **依存する feature でテストを実行**
6. **main ブランチにマージ**
7. **タグを作成**
   ```bash
   git tag domain/manufacturing/v1.3.0
   ```

### 8.2 破壊的変更の導入

1. **非推奨化フェーズ**
   - 古い API に `#[deprecated]` を付ける
   - 新しい API を追加（MINOR 更新）
   - 移行ドキュメントを作成

2. **移行期間**（推奨: 2-3 マイナーバージョン）
   - 両方の API を提供
   - 警告を出力

3. **削除フェーズ**
   - 古い API を削除（MAJOR 更新）
   - breaking_changes に記録

```rust
// 1. 非推奨化フェーズ（v1.5.0）
#[deprecated(since = "1.5.0", note = "Use new_quantity() instead")]
pub fn quantity(&self) -> u32 {
    self.quantity_obj.value()
}

pub fn new_quantity(&self) -> &Quantity {
    &self.quantity_obj
}

// 2. 削除フェーズ（v2.0.0）
pub fn quantity(&self) -> &Quantity {
    &self.quantity_obj
}
```

### 8.3 pre-release の活用

```bash
# ベータ版をリリース
k1s0 domain version --name manufacturing --set 2.0.0-beta.1

# feature でテスト（オプトイン）
# manifest.json
{
  "dependencies": {
    "domain": {
      "manufacturing": "2.0.0-beta.1"
    }
  }
}

# 正式リリース
k1s0 domain version --name manufacturing --set 2.0.0
```

---

## 9. 関連ドキュメント

- [domain 開発ガイド](domain-development.md)
- [3層構造への移行ガイド](migration-to-three-tier.md)
- [非推奨化ポリシー](../conventions/deprecation-policy.md)
- [Lint ルール K042, K046, K047](../design/lint.md)
- [Semantic Versioning 2.0.0](https://semver.org/lang/ja/)
