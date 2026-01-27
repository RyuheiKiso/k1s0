# データベースマイグレーション手順

本ドキュメントは、k1s0 におけるデータベースマイグレーションの実行手順を定義する。

## 1. 概要

マイグレーションが必要となるケース:

- 新しいテーブル/カラムの追加
- インデックスの追加・変更
- データ変換・移行
- スキーマの変更

## 2. 事前準備

### 2.1 チェックリスト

- [ ] マイグレーションスクリプトがレビュー済み
- [ ] ステージング環境でテスト済み
- [ ] ロールバックスクリプトが用意されている
- [ ] 想定所要時間を見積もっている
- [ ] 本番環境の場合、メンテナンスウィンドウを確保
- [ ] バックアップが完了している

### 2.2 マイグレーションファイルの確認

```bash
# マイグレーションファイル一覧
ls -la migrations/

# 未適用のマイグレーション確認
k1s0 db migrate --status --env {env}
```

### 2.3 バックアップ

```bash
# 本番環境の場合、必ずバックアップを取得
# PostgreSQL の場合
kubectl exec -it {db_pod} -- pg_dump -Fc {database_name} > backup_$(date +%Y%m%d_%H%M%S).dump

# バックアップの確認
ls -la backup_*.dump
```

## 3. マイグレーション実行

### 3.1 ステージング環境での実行

```bash
# マイグレーション実行（dry-run）
k1s0 db migrate --env stg --dry-run

# 実際のマイグレーション
k1s0 db migrate --env stg

# 状態確認
k1s0 db migrate --status --env stg
```

### 3.2 本番環境での実行

```bash
# 1. メンテナンスモード有効化（必要な場合）
k1s0 service maintenance --env prod --enable

# 2. マイグレーション実行
k1s0 db migrate --env prod

# 3. 状態確認
k1s0 db migrate --status --env prod

# 4. メンテナンスモード解除
k1s0 service maintenance --env prod --disable
```

### 3.3 手動実行（kubectl 経由）

```bash
# マイグレーション Job の実行
kubectl apply -f deploy/jobs/migration-job.yaml

# Job の状態確認
kubectl get jobs -l app={service_name}-migration

# ログ確認
kubectl logs -l job-name={service_name}-migration

# 完了後、Job を削除
kubectl delete job {service_name}-migration
```

## 4. マイグレーション種別

### 4.1 オンラインマイグレーション（推奨）

サービス停止なしで実行可能なマイグレーション。

**適用可能なケース:**
- カラム追加（NOT NULL 制約なし、またはデフォルト値あり）
- インデックス追加（CONCURRENTLY オプション）
- 新規テーブル作成

```sql
-- カラム追加（ダウンタイムなし）
ALTER TABLE users ADD COLUMN nickname VARCHAR(100);

-- インデックス追加（PostgreSQL、ダウンタイムなし）
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);
```

### 4.2 オフラインマイグレーション

サービス停止が必要なマイグレーション。

**該当ケース:**
- カラム削除
- NOT NULL 制約の追加（大量データ）
- カラム型の変更
- テーブル名の変更

```sql
-- カラム削除（サービス停止推奨）
ALTER TABLE users DROP COLUMN deprecated_field;

-- 型変更（サービス停止推奨）
ALTER TABLE orders ALTER COLUMN amount TYPE DECIMAL(12,2);
```

### 4.3 段階的マイグレーション

大規模なスキーマ変更の場合、複数リリースに分割。

**例: カラム名変更**

1. **リリース 1:** 新カラム追加、両方に書き込み
2. **リリース 2:** データ移行、新カラムから読み込み
3. **リリース 3:** 旧カラム削除

## 5. マイグレーション後の確認

### 5.1 確認チェックリスト

- [ ] マイグレーションが正常完了
- [ ] アプリケーションが正常動作
- [ ] エラーログが出ていない
- [ ] パフォーマンスに問題がない

### 5.2 確認コマンド

```bash
# マイグレーション状態
k1s0 db migrate --status --env {env}

# テーブル構造確認（PostgreSQL）
kubectl exec -it {db_pod} -- psql -c "\d+ {table_name}"

# アプリケーションログ確認
kubectl logs -l app={service_name} --tail=100 | jq 'select(.level=="ERROR")'

# クエリ実行確認
kubectl exec -it {db_pod} -- psql -c "SELECT COUNT(*) FROM {table_name}"
```

## 6. ロールバック手順

### 6.1 自動ロールバック

```bash
# 直前のマイグレーションをロールバック
k1s0 db rollback --env {env}

# 特定のバージョンまでロールバック
k1s0 db rollback --env {env} --to-version {version}
```

### 6.2 手動ロールバック

```bash
# ロールバックスクリプトの実行
kubectl exec -it {db_pod} -- psql -f /migrations/rollback/{version}.sql
```

### 6.3 バックアップからの復元

最終手段としてバックアップから復元。

```bash
# PostgreSQL の場合
kubectl exec -it {db_pod} -- pg_restore -d {database_name} /backup/backup_YYYYMMDD_HHMMSS.dump
```

## 7. 大規模マイグレーション

### 7.1 事前準備

- [ ] 影響範囲の特定（テーブルサイズ、ロック時間）
- [ ] メンテナンスウィンドウの確保
- [ ] ステークホルダーへの事前通知
- [ ] 詳細な実行計画書の作成
- [ ] ロールバック計画の策定

### 7.2 実行時の監視

```bash
# PostgreSQL: 実行中のクエリ確認
kubectl exec -it {db_pod} -- psql -c "SELECT pid, now() - pg_stat_activity.query_start AS duration, query FROM pg_stat_activity WHERE state = 'active'"

# ロック状況確認
kubectl exec -it {db_pod} -- psql -c "SELECT * FROM pg_locks WHERE NOT granted"
```

### 7.3 パフォーマンス考慮

| 操作 | 推奨事項 |
|------|---------|
| 大量 INSERT | バッチ処理（1000 行ずつ） |
| 大量 UPDATE | 小分けにして実行、コミット間隔を設定 |
| インデックス作成 | CONCURRENTLY オプション使用 |
| カラム追加 | デフォルト値を設定 |

## 8. トラブルシューティング

### 8.1 マイグレーションが途中で失敗

```bash
# 失敗したマイグレーションの確認
k1s0 db migrate --status --env {env}

# ログの確認
kubectl logs -l job-name={service_name}-migration

# 手動でロールバック
k1s0 db rollback --env {env}
```

### 8.2 ロックタイムアウト

```bash
# 長時間実行クエリの確認
kubectl exec -it {db_pod} -- psql -c "SELECT pid, now() - pg_stat_activity.query_start AS duration, query FROM pg_stat_activity WHERE state = 'active' AND now() - pg_stat_activity.query_start > interval '5 minutes'"

# 必要に応じてクエリをキャンセル
kubectl exec -it {db_pod} -- psql -c "SELECT pg_cancel_backend({pid})"
```

### 8.3 ディスク容量不足

```bash
# 容量確認
kubectl exec -it {db_pod} -- df -h

# 一時ファイルのクリーンアップ
kubectl exec -it {db_pod} -- psql -c "VACUUM FULL"
```

## 9. 本番環境での実行

### 9.1 追加チェックリスト

- [ ] #ops チャンネルでマイグレーション開始を通知
- [ ] 2 名以上で実施
- [ ] バックアップが完了していることを再確認
- [ ] ロールバック手順を手元に用意

### 9.2 通知テンプレート

**開始時:**
```
[DBマイグレーション開始] {service_name}
環境: prod
マイグレーション: {migration_name}
予定所要時間: 約{時間}分
影響: {影響内容（ダウンタイムの有無等）}
担当: {担当者名}
```

**完了時:**
```
[DBマイグレーション完了] {service_name}
環境: prod
結果: 正常完了
所要時間: {実際の時間}分
確認項目: スキーマ変更OK, アプリ動作OK
```

## 関連ドキュメント

- [デプロイメント手順](../deployment.md)
- [トラブルシューティング](../troubleshooting.md)
- [インシデント対応](incident-response.md)
