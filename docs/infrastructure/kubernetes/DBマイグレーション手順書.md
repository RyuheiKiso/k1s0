# DB 本番マイグレーション手順書

データベーススキーマ変更を本番環境に適用する手順と、ロールバック方法を定義する。

関連設計: [プログレッシブデリバリー設計](../../architecture/deployment/プログレッシブデリバリー設計.md)

---

## 1. マイグレーション戦略の選択

まずスキーマ変更の種類に応じて適用戦略を選択する。

```
DB スキーマ変更あり
  │
  ├─ 後方互換あり（カラム追加、新テーブル追加等）
  │    └─→ Canary デプロイ（マイグレーション先行実行）
  │
  ├─ 後方互換なし（カラム削除、型変更、リネーム等）
  │    └─→ Blue-Green デプロイ（メンテナンスウィンドウ設定）
  │
  └─ API の破壊的変更も伴う
       └─→ Blue-Green + Feature Flag
```

**後方互換ありの判断基準:**
- 既存のアプリケーションコードが、マイグレーション後の DB で正常動作するか
- カラム追加: `NOT NULL` 制約 + デフォルト値なしは互換性なし
- インデックス追加: 互換性あり（ただし大規模テーブルは LOCK 注意）

---

## 2. 事前チェックリスト

```
マイグレーション実行前に以下をすべて確認する:

[ ] バックアップが直近24時間以内に成功していること
    → kubectl get jobs -n k1s0-system | grep postgres-backup

[ ] staging 環境で同じマイグレーションが成功していること

[ ] マイグレーションファイルのレビューが完了していること
    → PR レビュー + CI の migration-check が PASS

[ ] ロールバック用の down マイグレーションが用意されていること

[ ] 影響テーブルの行数を確認していること（10万行超は LOCK 注意）
    → psql -c "SELECT count(*) FROM <table_name>;"

[ ] メンテナンスウィンドウが設定されていること（Blue-Green の場合）

[ ] インシデント対応担当者が待機していること（prod の場合）
```

---

## 3. Canary デプロイ（後方互換あり）

```bash
# 1. マイグレーションを先行実行する（アプリデプロイ前）
# sqlx migrate を使用する（justfile の migrate レシピ）
just migrate
# または直接実行:
# DATABASE_URL=postgres://<user>:<pass>@postgresql.k1s0-system.svc.cluster.local/<db> \
#   sqlx migrate run

# 2. マイグレーションが正常に適用されたことを確認する
# DATABASE_URL=... sqlx migrate info

# 3. Canary デプロイを開始する（通常の GitHub Actions デプロイ）
# main マージ → build-and-push → deploy-dev → deploy-staging → deploy-prod（手動承認）

# 4. カナリア段階でエラーが発生していないことを確認する
# Grafana → SLO ダッシュボード → エラーレート確認
```

---

## 4. Blue-Green デプロイ（後方互換なし）

### 4-1. メンテナンスウィンドウの設定

```bash
# Kong でメンテナンス応答を返すように設定する
# infra/kong/plugins/ の maintenance-mode プラグインを有効化する
kubectl patch kongplugin maintenance-mode -n k1s0-system \
  --type=merge -p '{"config":{"enabled":true}}'

# または、Istio で 503 を返す VirtualService を適用する
kubectl apply -f infra/istio/maintenance-virtualservice.yaml
```

### 4-2. マイグレーション実行

```bash
# 1. 現在の接続数を確認する（接続が0になるまで待つ）
psql -h postgresql.k1s0-system.svc.cluster.local -U k1s0_user \
  -c "SELECT count(*) FROM pg_stat_activity WHERE datname='k1s0_system' AND state='active';"

# 2. マイグレーションを実行する
just migrate

# 3. マイグレーション結果を確認する
# DATABASE_URL=... sqlx migrate info
```

### 4-3. Blue-Green 切り替え

```bash
# 1. Green 環境（新バージョン）を起動する
helm upgrade --install <service>-green \
  ./infra/helm/services/<path> \
  -n <namespace> \
  -f ./infra/helm/services/<path>/values-prod.yaml \
  --set image.tag=<new-version> \
  --set nameOverride=<service>-green

# 2. Green 環境のヘルスチェックを確認する
kubectl rollout status deployment/<service>-green -n <namespace>

# 3. Istio VirtualService でトラフィックを切り替える
kubectl patch virtualservice <service> -n <namespace> \
  --type=merge -p '{"spec":{"http":[{"route":[{"destination":{"host":"<service>-green"}}]}]}}'

# 4. メンテナンスモードを解除する
kubectl patch kongplugin maintenance-mode -n k1s0-system \
  --type=merge -p '{"config":{"enabled":false}}'

# 5. 動作確認後、Blue 環境（旧バージョン）を削除する
helm uninstall <service>-blue -n <namespace>
```

---

## 5. ロールバック手順

### 5-1. Canary デプロイのロールバック（アプリのみ）

アプリのロールバックは [デプロイ手順書](./デプロイ手順書.md) の「2. ロールバック手順」を参照。

マイグレーション（後方互換あり）は原則としてロールバックしない。安定後に不要カラムを削除する。

### 5-2. マイグレーションの down（後方互換なし、緊急時のみ）

```bash
# 特定バージョンまでダウンする
DATABASE_URL=postgres://<user>:<pass>@postgresql.k1s0-system.svc.cluster.local/<db> \
  sqlx migrate revert

# 確認
DATABASE_URL=... sqlx migrate info
```

**注意**: down マイグレーションでデータが失われる可能性がある。必ずバックアップを確認してから実行すること。

---

## 6. 大規模テーブルのマイグレーション

対象テーブルの行数が **10万行を超える** 場合は、テーブルロックを回避する手法を使用する。

```sql
-- カラム追加（NOT NULLはLOCKが長い。NULLABLEで追加後にバックフィルすること）
ALTER TABLE large_table ADD COLUMN new_col TEXT;  -- まず NULLABLE で追加
UPDATE large_table SET new_col = 'default' WHERE new_col IS NULL;  -- バックフィル
ALTER TABLE large_table ALTER COLUMN new_col SET NOT NULL;  -- 最後にNOT NULL化

-- インデックス追加（CONCURRENT でロックを回避する）
CREATE INDEX CONCURRENTLY idx_large_table_new_col ON large_table(new_col);
```

> **注意**: `sqlx migrate run` は `CONCURRENTLY` を含むマイグレーションをトランザクション外で実行する必要がある。`-- +migrate StatementBegin` / `-- +migrate StatementEnd` アノテーションを活用すること。

---

## 関連ドキュメント

- [バックアップリストア手順書](./バックアップリストア手順書.md)
- [デプロイ手順書](./デプロイ手順書.md)
- [プログレッシブデリバリー設計](../../architecture/deployment/プログレッシブデリバリー設計.md)
