# data コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは data 固有の制約のみ記述する。

## 配置・構成

- **Migration tool**: sqlx-cli（forward-only）
- **Schema SoT**: `src/data/apicurio/schemas/`（GitOps）
- **lock.yaml 配置先**: `src/data/lock/`（手書き禁止）
- **PII cluster**: `src/data/pii_cluster/`（WAL chain 別分離）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/06_data設計方針/README.md` を単一の真とする。

## コーディング制約

### migration

- **forward-only migration 必須**（破壊的 rollback 禁止）
- **expand-contract pattern** + dual-write / dual-read で schema 移行
- migration ファイルは `src/data/postgresql/migrations/` に連番で追加する
- `sqlx prepare` で compile-time 型安全確保（型不整合 = CI fail）

### テナント分離

- RLS FORCE 全 table（cross-tenant 防止必須）
- session_context GUC を全クエリで設定する（tier2 が自動注入）

### 暗号化・PII

- PII column: AES-256-GCM envelope encryption + RLS + `audit_event` emit を同時に実装
- DEK は `KeyHandle` 経由（生 key bytes を SQL 内に書くことを禁止）

### Apicurio GitOps

- git が単一の真、Registry は物理 cache（直接 Registry 編集禁止）
- schema 変更は `src/data/apicurio/schemas/` へのコミットで行う
- 後方互換性チェックは Buf breaking / Apicurio compatibility rule が enforce

### Debezium CDC

- Debezium connector config は `src/data/kafka/debezium_connector.yaml` に配置
- CDC pipeline を経由せずに Kafka に直接 produce することを禁止

## 関連参照

- `docs/03_概要設計/06_data設計方針/README.md` — 設計パターン
- `docs/04_詳細設計/02_強制機構/05_data強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/14_データ保全適合仕様.md` — データ保全
- `docs/04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md` — スキーマ進化
