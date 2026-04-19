# OPS-MIG: 移行要件

本ファイルは、k1s0 プラットフォーム上のデータストア（主に PostgreSQL）と tier1/tier2 サービス間の契約変更に伴う**スキーマ移行・データ移行・無停止更新** を要件化する。リリース戦略は [`02_release.md`](./02_release.md) に分離し、本ファイルは「データモデルが変わる時に業務を止めずにどう進めるか」に集中する。

スキーマ移行は失敗すると**データの不可逆な破壊**を招くため、失敗時の戻し方（ロールバック / ロールフォワード）と、本番稼働中の業務を止めない Expand-Contract パターンの適用を要件として固定する。

---

## 前提

- [`../30_セキュリティ_データ/04_data.md`](../30_セキュリティ_データ/04_data.md) — データモデルと保持期間
- [`../30_セキュリティ_データ/07_backup_restore.md`](../30_セキュリティ_データ/07_backup_restore.md) — マイグレーション前バックアップ
- [`02_release.md`](./02_release.md) — Feature Flag 連動による段階公開

---

## 要件本体

### OPS-MIG-001: スキーマ移行ツールの統一（flyway/atlas 等）

- 優先度: MUST（各サービスが独自にマイグレーションを実装すると手順・監査・戻し方がサービスごとに異なる）
- Phase: Phase 1a（tier1 で選定・固定）/ Phase 1b で tier2 にも展開
- 関連: OPS-MIG-002 / SEC-AUD-004

現状、起案者のローカルでは `golang-migrate` を使っているが、採用判断の記録が無く、Phase 2 以降に tier2/tier3 で他ツール（Prisma / TypeORM / EF Core のマイグレータ）が並立する恐れがある。監査観点でバラバラなマイグレーション方式を同時に管理するのは事実上不可能。

要件達成後の世界では、k1s0 標準のスキーマ移行ツールを **Atlas（Go 製、宣言的 + 命令的両対応）** に統一する（ADR で決定）。全 tier1 Rust / Go サービスと tier2 サンプル実装は Atlas の `.sql` マイグレーションを `migrations/` 配下に格納し、CI の Atlas lint でマイグレーション安全性（NOT NULL 制約追加に DEFAULT 付与、大テーブル ALTER TABLE の `CONCURRENTLY` 適用等）を自動検査する。雛形生成 CLI は `migrations/` ディレクトリと CI ステップを生成する。

崩れた時、Phase 2 で tier2 開発者が Prisma でマイグレーションを書き始め、Atlas と Prisma が同一 DB に書き込む二重管理状態に陥る。本番で「どちらが先に実行されたか」の順序保証が取れず、ロールバック手順も 2 通り必要になる。

**受け入れ基準**

- 全 tier1 サービスの `migrations/` が Atlas 形式、ADR `adr-migration-tool.md` で Atlas 採用を明記
- CI で `atlas migrate lint` が PR ブロック条件、未対応 PR は merge 不可
- `migrations/` の SQL は冪等性を必須（`IF NOT EXISTS` / `IF EXISTS`）
- マイグレーション実行ログは PostgreSQL の `schema_migrations` テーブル + OpenBao audit log に 7 年保管
- 採用ツール変更は ADR 更新必須、移行期間最低 3 か月

**検証方法**

- 四半期ごとに Atlas 以外のツールを使っている箇所がないかリポ全件 grep
- CI `atlas migrate lint` の fail 件数を月次集計

---

### OPS-MIG-002: 無停止マイグレーション（Expand-Contract パターン原則）

- 優先度: MUST（稟議承認 3 か月フローなど長期実行中の業務があるため、DB ロックを伴うマイグレーションは禁止）
- Phase: Phase 1c（業務稼働時）
- 関連: OPS-REL-004 / OPS-MIG-003

現状、dev 環境では `ALTER TABLE ADD COLUMN NOT NULL` を平気で実行しているが、PostgreSQL では大テーブルで長時間の ACCESS EXCLUSIVE ロックが発生する。稼働業務がある本番でこれをやると全 API が数分〜数十分停止する。

要件達成後の世界では、スキーマ変更は**必ず Expand-Contract パターン**で 3 段階に分割される。(1) **Expand**: 新カラム追加（NULL 許容）/ 新テーブル追加、(2) **Migrate**: アプリケーションを両対応版にデプロイ（OPS-REL-004 の Feature Flag で旧/新を切替）、データコピーをバックグラウンドジョブで実施、(3) **Contract**: 旧カラム / 旧テーブル削除（Migrate 完了から 2 週間後以降）。`CREATE INDEX CONCURRENTLY` / `ALTER TABLE ... SET NOT NULL` は既存データ埋め戻し後に別 PR で実施。PostgreSQL のロック取得は 5 秒以内に完了しない変更は自動 Abort（`lock_timeout = '5s'`）。

崩れた時、稟議承認 3 か月フローの最中に DB ロックが発生すると、Temporal Workflow の State 書き込みが失敗し、フローが Stuck 状態になる。再開には手動オペレーションが必要で、QUA-DR（復旧 4h）を超える可能性がある。

**受け入れ基準**

- 全 prod マイグレーションに `lock_timeout` が適用され、超過時は自動 rollback
- Expand-Contract パターン適用率が 90% 以上（1 PR で完結する破壊的変更を例外として記録）
- Contract フェーズ（旧削除）の実行は Migrate から最低 2 週間後、Feature Flag 切替完了が前提
- マイグレーション実行前の `pg_dump` バックアップが自動化、失敗時は自動 rollback
- 大テーブル（10M 行超）の ALTER はバッチ化（OPS-MIG-004）必須、単一 SQL での実行禁止

**検証方法**

- Staging で稼働業務 1 本を動かしたまま全マイグレーションをリハーサル
- `pg_locks` の ACCESS EXCLUSIVE 発生時間を Prometheus で監視、5 秒超はアラート

---

### OPS-MIG-003: ロールフォワード / ロールバック設計

- 優先度: MUST（マイグレーション失敗時に戻せない構造は本番導入不可）
- Phase: Phase 1c
- 関連: OPS-MIG-001 / OPS-MIG-002

現状、`golang-migrate` の `down` マイグレーションは起案者が書いていないケースがあり、事実上ロールフォワードしか選択肢がない。これでは CVE 修正のマイグレーションが本番で失敗した時に戻せない。

要件達成後の世界では、全マイグレーションは **ロールフォワード優先**（新しいマイグレーションを追加して修正）を既定とし、それが不可能な「Contract フェーズで消した旧カラムが必要になった」ケース向けに **ロールバック用の逆マイグレーション** を同じ PR で作成・検証する。ロールバック SQL は staging で毎週自動テスト（`atlas migrate apply` → `atlas migrate down` → integration test）され、失敗した時点で PR が blocked になる。緊急時のロールバック手順は Runbook `migration-rollback.md` に整理し、執行権限は SRE リード + tier1 リードの 2 名同時 SSH が必須。

崩れた時、マイグレーション失敗で本番データが中途半端に変更された状態で動いている時に戻せないと、業務データの整合性が恒久的に失われる。個人情報暗号化などの SEC 要件と絡むと、監査で重大インシデント扱いとなる。

**受け入れ基準**

- 全マイグレーション PR に上り SQL と下り SQL の両方が含まれる（または「ロールフォワードのみ」の明示記載）
- `atlas migrate down` の自動テストが weekly で実行、fail 時はリリース凍結
- Runbook `migration-rollback.md` に緊急時手順、四半期ドリルで手順有効性を検証
- ロールバック実行の監査ログは 7 年保管、実行者 2 名の署名必須
- Feature Flag と組み合わせて「DB は進めたがアプリは旧版に戻す」パターンを Runbook 化

**検証方法**

- 四半期ごとに staging で意図的にロールバックドリル、成功率 100%
- ロールバック SQL の CI テスト合格率を月次集計

---

### OPS-MIG-004: 大規模データ移行のバッチ戦略

- 優先度: SHOULD（数千万〜億件規模のデータ移行は単一 TX で不可能、バッチ化しないと DB 過負荷で全サービス影響）
- Phase: Phase 2（データ量が一定規模を超えた時点）
- 関連: OPS-MIG-002 / QUA-PRF-001

現状、起案者が想定しているデータ件数は tier1 監査ログで数百万件程度だが、Phase 3 以降で全社展開すると個人情報・監査ログが数億件規模になる。単一 UPDATE 文では実行できない。

要件達成後の世界では、10M 行を超えるテーブルのデータ変更（バックフィル、カラム値再計算、暗号化キーローテーション）は**バッチ化**を必須とする。バッチサイズは 10,000 行、バッチ間に 100ms のスリープを挟む（PostgreSQL の replication lag を監視し、1 秒以上で自動一時停止）。バッチ実行は Kubernetes CronJob または Temporal Workflow（長時間実行）で管理し、進捗は Prometheus の `migration_batch_progress` でリアルタイム可視化。失敗時は中断位置から再開可能（チェックポイント記録）。

崩れた時、単一 TX で数億件の UPDATE を流すと、PostgreSQL のレプリケーションラグが数時間単位に膨らみ、readonly レプリカ参照の tier1 API が古いデータを返す期間が発生、P99 レイテンシ違反と業務影響が同時発生する。

**受け入れ基準**

- 10M 行超のデータ変更はバッチ化必須、Runbook `large-scale-migration.md` に手順
- バッチ実行中の `pg_replication_slots` の遅延を Prometheus で監視、1 秒超で自動 pause
- 進捗 dashboard `migration-batch-progress` が Grafana に常設
- 中断 / 再開が可能（チェックポイントテーブル `migration_checkpoints`）
- 大規模移行の計画は SRE リード + DBA の 2 者レビュー必須、計画書は Confluence に 180 日保管

**検証方法**

- Phase 2 で実施する本番相当データ（3,000 名 × 監査ログ数年分）でバッチ動作を staging リハーサル
- バッチ失敗時の再開テストを四半期ドリル

---

### OPS-MIG-005: Schema Registry によるイベント契約の前方互換

- 優先度: SHOULD（Kafka イベントの breaking change は非同期に連鎖し、発覚が遅れる）
- Phase: Phase 2（Kafka + Apicurio 導入後）
- 関連: ARC-EVT-001 / OPS-CID-001

現状、Kafka 導入前のため本要件は直接の実害はないが、Phase 2 での Kafka + Apicurio 稼働時にイベントスキーマの後方互換性を CI で検査しないと、Producer/Consumer の連携が本番で壊れる。

要件達成後の世界では、Apicurio Registry に登録された Avro / JSON Schema を `BACKWARD` 互換モードで必須運用し、PR の CI ステージで `apicurio-cli compatibility-check` を実行する。互換性違反時は merge ブロック。スキーマ変更はバージョン番号を自動採番し、旧バージョンは最低 6 か月保管。

崩れた時、イベントスキーマの破壊的変更が Consumer 側で Deserialize エラーを起こし、個人情報削除フローや稟議承認フローが非同期に失敗する。発覚は Consumer ログの監視まで遅れる。

**受け入れ基準**

- 全 Kafka プロデューサは Apicurio Registry 登録済みスキーマのみ使用
- CI `apicurio compatibility-check` が PR ブロック条件
- BACKWARD 互換モード必須、FULL 互換が望ましい（ADR で定義）
- 旧バージョンスキーマの保持期間 6 か月以上
- 互換性違反件数を月次集計、0 件を維持

**検証方法**

- Apicurio Registry の API でバージョン履歴を四半期監査
- Kafka Consumer のデシリアライズエラー率を Prometheus で監視

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| OPS-MIG-001 | スキーマ移行ツール統一（Atlas） | MUST | 1a/1b |
| OPS-MIG-002 | Expand-Contract 無停止原則 | MUST | 1c |
| OPS-MIG-003 | ロールフォワード / ロールバック | MUST | 1c |
| OPS-MIG-004 | 大規模バッチ戦略 | SHOULD | 2 |
| OPS-MIG-005 | Schema Registry 前方互換 | SHOULD | 2 |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 3 | OPS-MIG-001, 002, 003 |
| SHOULD | 2 | OPS-MIG-004, 005 |

### Phase 達成度

| Phase | 必達件数 | 未達影響 |
|---|---|---|
| 1a | 1 | tier1 開発開始時にマイグレーション方針が空白 |
| 1c | 3 | 本番稼働時の無停止更新が不可、稼働業務停止リスク |
| 2 | 5 | 大規模展開時のデータ移行・イベント契約互換が未整備 |
