## 5. 契約管理（API First）（MUST）

gRPC / REST(OpenAPI) の「契約（スキーマ）」を正本として管理し、後方互換性のルールと破壊検知を CI で強制する。

### 契約管理の実体（テンプレに組み込む・必須運用）

- 正本（source of truth）の置き場（サービス内）
	- gRPC: `proto/` 配下の `*.proto`
	- REST: `openapi/openapi.yaml`
- 生成物の置き場（サービス内）
	- gRPC: `gen/`（例: `gen/rust/`、`gen/ts/`）
	- REST: `openapi/gen/`（例: サーバスタブ/SDK/型定義）
- 生成物は原則 Git 管理対象外（`.gitignore` に含める）。ただし生成物の再現性検証を CI で必須にする。

CI の必須チェック

- gRPC: `buf lint` / `buf breaking`（比較対象は `main` もしくは直近タグ）
- REST: openapi-diff 等で `main` もしくは直近タグとの差分比較（破壊的変更は CI 失敗）
- 生成一致チェック: CI で生成を実行し、生成物差分が出る場合は失敗

テンプレが最低限持つもの（置き場は固定）

- `proto/`
- `openapi/openapi.yaml`
- `openapi/gen/`（Git 管理外）
- `gen/`（Git 管理外）
- `buf.yaml` / `buf.lock`
- `scripts/` または `Makefile` / `justfile` 等（契約 lint/breaking/生成を1コマンド化）

### 破壊的変更（MAJOR）時に必須の成果物

破壊的変更が必要な場合（例外として MAJOR で実施）は次を必須とする。

- `docs/adr/` に ADR を追加（なぜ必要か、代替案、影響範囲、移行計画）
- `UPGRADE.md`（またはリリースノート）を追加（影響一覧、移行手順、ロールバック）
- CLI にアップグレード支援を追加
	- `k1s0 upgrade --check` が破壊箇所/衝突/移行対象を検知できること
	- `k1s0 upgrade` が安全に止まれること（中途半端に壊さない）

### gRPC（Protocol Buffers）互換性

- 正本は `proto/` 配下の `*.proto`
- 後方互換性（原則）
	- フィールド追加は OK
	- フィールド削除・再採番・型変更は NG
	- 使わなくなったフィールドは削除せず `reserved`（番号/名前）を宣言
	- `oneof` の破壊的変更（既存ケースの削除/意味変更）は NG
	- サービス名/メソッド名/パッケージ名の変更は原則 NG（段階移行で実施）
	- 非推奨化は `deprecated = true` を使用し、削除は原則しない（例外は MAJOR）

### REST（OpenAPI）互換性

- 正本は `openapi/openapi.yaml`
- 後方互換性（原則）
	- エンドポイント/operation の削除・パス変更・HTTP メソッド変更は NG
	- フィールド削除・型変更は NG
	- 任意→必須（required 追加）、バリデーション強化（許容値の縮小）は原則 NG（段階移行）
	- フィールド追加は OK（任意のみ）。必須フィールド追加は原則 NG

---


