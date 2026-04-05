# ADR-0102: graphql_handler.rs の分割延期方針

| 項目 | 内容 |
|------|------|
| ステータス | 承認済み |
| 作成日 | 2026-04-05 |
| 決定者 | アーキテクチャチーム |
| 関連 | MED-020（外部技術監査 2026-04-05） |

---

## 背景

外部技術監査（MED-020）にて、`graphql-gateway` の
`src/adapter/graphql_handler.rs`（約2347行）について
「単一ファイルが大きすぎる。責務分離が不十分であり、保守性・テスタビリティが低下している」
との指摘を受けた。

現在のファイル構成:
- `graphql_handler.rs` — GraphQL スキーマ定義・リゾルバー実装・型定義を1ファイルに集約
- 行数: 約2347行（async-graphql マクロ展開前）
- 管理対象サービス: 12サービス（auth/tenant/featureflag/config/navigation/servicecatalog/session/vault/scheduler/notification/workflow/common）

---

## 決定内容

**graphql_handler.rs の分割を段階的に実施する。ただし現時点での即時分割は延期する。**

延期の理由を以下に示す。

### 延期が適切な理由

1. **機能的安定性**: 現状コードは正常動作しており、分割自体が新たなリグレッションを生むリスクがある
2. **async-graphql マクロの制約**: `#[Object]` マクロは impl ブロック単位でスキーマを登録するため、単純なファイル分割ではなくモジュール分割が必要になる。これはリファクタリング規模が大きい
3. **依存関係の複雑性**: 全リゾルバーが共通の `AppState` や `Arc<XxxClient>` を参照しており、モジュール間でこれを受け渡すための設計変更が必要
4. **優先度の低さ**: 現時点での技術的リスクは「実行時エラー」ではなく「開発者体験の低下」に留まる

---

## 将来の分割方針

下記のタイミングで分割を実施する:

### フェーズ1: ファイル分割（次期スプリント以降）

```
src/adapter/
├── graphql_handler.rs          # MergedObject/MergedSubscription の root のみ残す
├── graphql/
│   ├── mod.rs
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── query.rs            # AuthQuery struct + #[Object] impl
│   │   └── mutation.rs         # AuthMutation struct + #[Object] impl
│   ├── tenant/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── featureflag/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── config/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── navigation/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── servicecatalog/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── session/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── vault/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── scheduler/
│   │   ├── query.rs
│   │   └── mutation.rs
│   ├── notification/
│   │   ├── query.rs
│   │   └── mutation.rs
│   └── workflow/
│       ├── query.rs
│       └── mutation.rs
```

### フェーズ2: MergedObject 統合

async-graphql の `MergedObject` を使用して各サービスの Query/Mutation を統合する:

```rust
#[derive(MergedObject, Default)]
pub struct Query(
    AuthQuery,
    TenantQuery,
    FeatureFlagQuery,
    // ...
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    AuthMutation,
    TenantMutation,
    FeatureFlagMutation,
    // ...
);
```

---

## 代替案

| 案 | 内容 | 却下理由 |
|----|------|---------|
| 即時分割 | 今すぐ12サービス×2ファイル（24ファイル）に分割 | リグレッションリスクが高く、他の監査対応作業と競合する |
| 維持 | 現状のまま変更しない | 長期的な保守性低下を招くため延期はするが維持はしない |
| GraphQL モジュール別サービス分割 | 各サービスを独立したマイクロサービスの GraphQL として分離 | アーキテクチャの大幅な変更が必要であり過剰設計 |

---

## 影響

- **開発者体験**: 分割前は IDE での検索・編集が重くなる（現状維持のリスク）
- **テスタビリティ**: 分割後はリゾルバー単体テストが容易になる
- **ビルド時間**: async-graphql マクロ展開が分散することでインクリメンタルビルドが改善する

---

## 参考資料

- [async-graphql MergedObject ドキュメント](https://async-graphql.github.io/async-graphql/en/merging_objects.html)
- 外部技術監査報告書 2026-04-05: MED-020
