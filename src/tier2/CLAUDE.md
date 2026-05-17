# tier2 コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは tier2 固有の制約のみ記述する。

## 言語・workspace

- **言語**: Rust / C# (.NET 8+) / Go / TypeScript（4 言語等価強度）
- **Cargo workspace**: `src/tier1/Cargo.toml` の member（`"../tier2/rust"`）
- **go.work**: `src/tier1/go.work` の member
- **lock.yaml 配置先**: `src/tier2/lock/`（手書き禁止）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/03_tier2設計方針/README.md` を単一の真とする。

## コーディング制約

### テナント識別子

- `tenant_id` を API 引数で受け取る経路禁止（`AuthContext` からのみ取得）
- `Authorization` ヘッダ・Cookie から直接 `tenant_id` を取り出して引数に渡すことを禁止
- リポジトリ抽象が `tenant_id` 述語を自動注入するため、上位 API 引数への露出は不要

### SQL・データアクセス

- 生 SQL 文字列受付 API 禁止 → リポジトリ抽象を使う（`sqlx` の `query!` マクロ等の compile-time 型安全 API のみ）
- `sqlx prepare` で compile-time 型チェック必須（型不整合 = CI fail）

### ドメインイベント・atomic 書込

- Domain Event emit bypass 禁止（Outbox 経由必須）
- State change / Outbox / Audit は **同一 DB トランザクション**で書く（atomic 三表書込）
- Outbox 投入失敗時は rollback
- PII 含有フィールドは `field_pii` annotation + Outbox 書込時に redact

### 業界中立性（命名禁則）

- 公開 API / 公開型名に業界固有語禁止（設備 / ロット / 品目 / 拠点 / BOM 等）→ CI fail
- 業界横断層 → 業界 pack 依存禁止（`cargo-deny` / `ProjectReference` / `depguard` / `eslint-boundaries`）
- 業界 pack 相互依存禁止

### 依存方向

- tier3 から tier1 / OSS の直接 import が 1 件でも残存 → CI fail
- ドメイン層から tier1 Library / Service への直接依存禁止（クリーンアーキテクチャ）

## 関連参照

- `docs/03_概要設計/03_tier2設計方針/README.md` — 言語・設計パターン
- `docs/04_詳細設計/02_強制機構/02_tier2強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md` — テナント分離
