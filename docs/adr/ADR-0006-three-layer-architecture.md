# ADR-0006: 3層アーキテクチャ（framework -> domain -> feature）

## ステータス

承認済み

## コンテキスト

k1s0 の初期実装では、2層構造（framework -> feature）を採用していた。しかし、プロジェクトの成長に伴い、以下の課題が顕在化した。

1. **ビジネスロジックの重複**: 複数の feature が同じビジネスロジック（エンティティ、値オブジェクト、ドメインサービス）を実装していた
2. **変更の影響範囲**: ビジネスルールの変更時に、複数の feature を個別に更新する必要があった
3. **一貫性の欠如**: 同じ概念に対する実装が feature 間で微妙に異なっていた
4. **コード再利用の困難**: feature 間でコードを共有する標準的な方法がなかった

## 決定

2層構造から3層構造に拡張し、中間層として「domain」層を導入する。

```
framework (技術基盤) -> domain (業務領域共通) -> feature (個別機能)
```

### 各層の責務

#### framework 層
- **位置**: `framework/backend/rust/`, `framework/backend/go/`, etc.
- **責務**: 技術的な共通基盤（ロギング、設定管理、エラーハンドリング、DB接続等）
- **特徴**: 業務ロジックを含まない純粋な技術ライブラリ
- **バージョン管理**: k1s0 CLI のバージョンと連動

#### domain 層
- **位置**: `domain/backend/rust/{domain-name}/`, etc.
- **責務**: 特定の業務領域（例: 生産管理、在庫管理）の共通ビジネスロジック
- **特徴**:
  - エンティティ、値オブジェクト、ドメインサービスを提供
  - 複数の feature から参照される
  - 独自のバージョン（SemVer）を持つ
- **バージョン管理**: 独立した SemVer（例: 1.2.0）

#### feature 層
- **位置**: `feature/backend/rust/{feature-name}/`, etc.
- **責務**: 具体的なユースケースの実装
- **特徴**:
  - 0個以上の domain に依存可能
  - プレゼンテーション層（API エンドポイント）を含む
  - 環境固有の設定、デプロイ構成を持つ
- **バージョン管理**: manifest には version フィールドを持たない（feature はリリース単位ではない）

### 依存関係のルール

1. **feature -> domain**: feature は domain に依存できる（バージョン制約付き）
2. **feature -> framework**: feature は framework に依存できる
3. **domain -> framework**: domain は framework に依存できる
4. **domain -> domain**: 許可するが循環依存は禁止
5. **framework -> domain**: 禁止（framework は最下層）
6. **framework -> feature**: 禁止

### manifest.json の拡張

```json
{
  "layer": "domain",
  "version": "1.2.0",
  "min_framework_version": "0.1.0",
  "breaking_changes": {
    "2.0.0": "EntityA のフィールド名を変更"
  },
  "deprecated": {
    "since": "1.5.0",
    "migrate_to": "domain-v2"
  },
  "dependencies": {
    "framework": ["k1s0-error", "k1s0-config"],
    "domain": {
      "other-domain": "^1.0.0"
    }
  }
}
```

### Lint ルール

以下の Lint ルールで層間依存関係を検証する。

| ID | 説明 |
|----|------|
| K040 | 層間依存の基本違反 |
| K041 | domain が見つからない |
| K042 | domain バージョン制約不整合 |
| K043 | 循環依存の検出 |
| K044 | 非推奨 domain の使用 |
| K045 | min_framework_version 違反 |
| K046 | breaking_changes の影響 |
| K047 | domain 層の version 未設定 |

### CLI コマンド

- `k1s0 new-domain`: domain の作成
- `k1s0 new-feature --domain <name>`: domain に依存する feature の作成
- `k1s0 domain version`: domain のバージョン管理
- `k1s0 domain list`: domain の一覧表示
- `k1s0 domain dependents`: domain に依存する feature の一覧
- `k1s0 domain impact`: バージョンアップの影響分析
- `k1s0 feature update-domain`: feature の domain 依存更新

## 結果

### 肯定的

1. **コード再利用の向上**: domain 層で共通ビジネスロジックを一元管理
2. **変更影響の局所化**: ビジネスルール変更は domain 層のみ
3. **バージョン管理の明確化**: domain の SemVer でAPI互換性を管理
4. **段階的移行**: 既存の feature は domain なしでも動作（後方互換性）

### 否定的

1. **複雑性の増加**: 3層構造は2層より複雑
2. **学習コストの増加**: domain の設計・管理の知識が必要
3. **初期セットアップの増加**: domain を先に作成する必要がある場合がある

### 中立

1. **循環依存リスク**: domain 間の依存を許可したため、循環依存のリスクがある（K043 で検出）
2. **バージョン管理負荷**: domain のバージョン管理が追加タスクとなる

## 参考

- Clean Architecture（Robert C. Martin）
- Domain-Driven Design（Eric Evans）
- Semantic Versioning 2.0.0
