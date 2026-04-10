# ローカル開発環境 — Docker Volume 管理

ローカル開発環境の Docker Volume に関する運用手順を定義する。
特に DB スキーマ変更時の対応手順を記載する（MED-06 監査対応）。

## 起動コマンドの注意事項（CRIT-002 / CRIT-003 / HIGH-003 対応）

### 常に `just local-up` を使用すること

`docker compose up` を直接実行すると、以下の問題が発生する（外部技術監査 CRIT-002/003/HIGH-003）：

| 問題 | 症状 | 原因 |
|------|------|------|
| CRIT-002 | config-rust が `Restarting` 状態で起動ループ | `--build` 省略で `dev-auth-bypass` 未コンパイルイメージが使用される |
| CRIT-003 | featureflag-rust が `unhealthy` になる | `--build` 省略で migration 006（UUID→TEXT）対応前のイメージが使用される |
| HIGH-003 | 複数サービスが DB スキーマ不在で unhealthy | `migrate-all` が実行されず DB スキーマが未初期化のまま起動する |

**正しい起動方法**:

```bash
# ✅ 推奨: --build と migrate-all が自動適用される
just local-up

# ❌ 非推奨: --build が省略され古いイメージが使われる可能性がある
docker compose up -d
```

`just local-up` の内部処理（3フェーズ）:
1. **Phase 1**: `docker compose up --build --profile infra` でインフラ起動
2. **Phase 1.5**: `just migrate-all` で全 DB マイグレーション実行
3. **Phase 2**: `docker compose up --build --profile system ...` で全サービス起動

`docker compose up` を直接使用する場合は、以下の全オプションを明示すること:

```bash
# 手動起動（非推奨）: 全オプションを明示する必要がある
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  --profile infra --profile system up -d --build

# マイグレーションも手動で実行が必要
just migrate-all
```

---

## データベーススキーマ変更後の Volume 再作成手順（MED-06 監査対応）

### 背景

PostgreSQL の Docker Volume が既に存在する場合、`init-db` スクリプト（`infra/docker/init-db/` 配下）は再実行されない。
Docker の仕様として、Volume が空の場合のみ `initdb` フックが走るため、既存の Volume にはスキーマ変更が反映されない。

### いつ Volume 再作成が必要か

以下の変更を `git pull` した場合、既存の Docker Volume には変更が適用されないため Volume を削除して再起動する必要がある:

- `infra/docker/init-db/` 配下のスクリプト変更
- 新規データベースの追加（例: ADR-0060 での `k1s0_saga` DB 追加）
- テーブルスキーマの大幅な変更

### 手順

```bash
# データを保存する必要がある場合は事前にバックアップを取ること
# docker exec k1s0-postgres-1 pg_dumpall -U dev > backup.sql

# 1. コンテナを停止し Volume を削除する
just local-down-clean
# または手動で実行する場合:
# docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml down -v

# 2. 再起動（init-db スクリプトが再実行される）
just local-up
```

### 注意事項

- `-v` オプションで全ての Volume が削除されるため、ローカルのデータは全て失われる
- **本番環境では絶対に使用しないこと**
- `just local-down`（`-v` なし）は Volume を保持したままコンテナを停止するため、通常の停止にはこちらを使用する

## 通常操作との比較

| コマンド | Volume 削除 | 用途 |
| --- | --- | --- |
| `just local-up` | なし | 通常の開発環境起動 |
| `just local-down` | なし | 通常の開発環境停止（データ保持） |
| `just local-down-clean` | **あり** | DB スキーマ変更後のリセット |

## 関連ドキュメント

- [docker-compose 設計](./docker-compose設計.md) — プロファイル構成と使用例
- [compose-インフラサービス設計](./compose-インフラサービス設計.md) — PostgreSQL 設定詳細
- ADR-0060 — `k1s0_saga` DB 追加の設計決定
