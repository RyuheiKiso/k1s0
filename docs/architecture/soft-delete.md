# ソフトデリート設計

k1s0 におけるソフトデリート（論理削除）の方針と実装ガイドラインを定義する。

## ソフトデリートの方針

### 基本方針: deleted_at カラム

`is_deleted` フラグではなく **`deleted_at TIMESTAMPTZ`** カラムを採用する。

| 方式 | メリット | デメリット | 採用 |
| --- | --- | --- | --- |
| `deleted_at TIMESTAMPTZ` | 削除時刻が記録される。NULL 判定で未削除を表現 | カラムが NULL 許容 | **採用** |
| `is_deleted BOOLEAN` | シンプル | 削除時刻が不明。別途カラムが必要 | 不採用 |

### deleted_at を選定した理由

- 削除時刻の記録により、データ保持期間の管理とクリーンアップが容易
- `WHERE deleted_at IS NULL` で未削除データを効率的にフィルタリング可能
- 部分インデックス（`WHERE deleted_at IS NULL`）で未削除データのクエリ性能を確保

### カラム定義

```sql
-- ソフトデリート用カラム（NULL = 未削除、値あり = 削除済み）
deleted_at TIMESTAMPTZ DEFAULT NULL
```

## クエリ時のフィルタリング戦略

### リポジトリ層での統一フィルタ

全てのクエリにデフォルトで `WHERE deleted_at IS NULL` を付与する。
削除済みデータを含む検索が必要な場合は、明示的にオプトインする。

### パフォーマンス最適化

```sql
-- 未削除データに限定した部分インデックス（頻出クエリの高速化）
CREATE INDEX idx_{table}_active ON {schema}.{table} ({columns})
    WHERE deleted_at IS NULL;
```

### ソフトデリートの実行

```sql
-- 論理削除: deleted_at に現在時刻を設定する
UPDATE {schema}.{table}
SET deleted_at = NOW(), updated_at = NOW()
WHERE id = $1 AND deleted_at IS NULL;
```

### 復元

```sql
-- 論理削除の取り消し: deleted_at を NULL に戻す
UPDATE {schema}.{table}
SET deleted_at = NULL, updated_at = NOW()
WHERE id = $1 AND deleted_at IS NOT NULL;
```

## データ保持期間とクリーンアップ方針

### 保持期間のデフォルト

| データ種別 | 保持期間 | 根拠 |
| --- | --- | --- |
| 業務データ（orders, payments） | 90 日 | 業務要件・法令要件に応じて延長可能 |
| 監査ログ（audit_logs） | 24 ヶ月 | コンプライアンス要件 |
| セッション・一時データ | 30 日 | 運用効率 |
| マスタデータ | 無期限 | 参照整合性の維持 |

### クリーンアップジョブ

scheduler サーバーのジョブとして定期実行する。

```sql
-- 保持期間を超えた論理削除データを物理削除する
DELETE FROM {schema}.{table}
WHERE deleted_at IS NOT NULL
  AND deleted_at < NOW() - INTERVAL '{retention_period}';
```

### クリーンアップの実行ルール

1. **バッチサイズの制限**: 1 回のジョブで削除する行数を制限し、ロック競合を防ぐ
2. **低負荷時間帯の実行**: scheduler のスケジュール設定で深夜帯に実行する
3. **ドライラン**: 本番環境での初回実行時は `SELECT COUNT(*)` で対象件数を確認する
4. **監査ログの例外**: audit_logs は pg_partman によるパーティション管理で自動デタッチする（ソフトデリートではなくパーティション単位で管理）

## 関連テーブルのカスケード処理

### カスケード方針

親テーブルがソフトデリートされた場合、子テーブルの処理方針を以下のように定める。

| 関係 | 方針 | 例 |
| --- | --- | --- |
| 強い所有関係 | 子も同時にソフトデリート | order → order_items |
| 弱い参照関係 | 子はそのまま残す | user → audit_logs |
| 外部キー参照 | FK 制約を維持、参照先が削除済みかはアプリ層で判定 | notification_logs → templates |

### カスケードソフトデリートの実装

アプリケーション層のサービスメソッドで明示的に実装する。DB トリガーでの自動カスケードは使用しない。

```
// 疑似コード: 注文のソフトデリート
fn soft_delete_order(order_id) {
    // 1. 注文本体をソフトデリート
    order_repo.soft_delete(order_id);
    // 2. 関連する注文明細もソフトデリート
    order_item_repo.soft_delete_by_order_id(order_id);
    // 3. Outbox イベントを発行（他サービスへの通知）
    outbox.publish(OrderDeleted { order_id });
}
```

### カスケードをアプリ層で実装する理由

- DB トリガーはデバッグが困難で、暗黙的な副作用を生む
- サービス層で明示的に制御することで、ビジネスルールの適用が容易
- Outbox パターンとの統合が自然に行える

## ソフトデリート対象外のテーブル

以下のテーブルはソフトデリートを適用しない。

| テーブル | 理由 |
| --- | --- |
| `auth.audit_logs` | パーティション管理で保持期間を制御 |
| `saga.saga_states` | ワークフロー状態は完了後にアーカイブ |
| `*.outbox_events` | 処理済みイベントは物理削除 |
| 設定系テーブル | バージョン管理で履歴を保持 |
