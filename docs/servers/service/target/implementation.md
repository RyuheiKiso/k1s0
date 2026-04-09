# service-target-server 実装設計

> **注意（MED-020 対応）**: このサービスは未実装（設計書のみ）です。実装予定が確定した場合に `regions/service/target/` に実装されます。

service-target-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## Rust 実装 (regions/service/target/server/rust/target/)

### ディレクトリ構成

board-server と同等のクリーンアーキテクチャ構成。

| パス | 説明 |
| --- | --- |
| `src/main.rs` | エントリポイント |
| `src/lib.rs` | ライブラリルート（MIGRATOR 定義） |
| `src/domain/error.rs` | TargetError（thiserror ベース） |
| `src/domain/entity/objective.rs` | Objective エンティティ（ステータス遷移チェックロジック含む） |
| `src/domain/entity/key_result.rs` | KeyResult エンティティ（進捗値バリデーション含む） |
| `src/domain/repository/objective_repository.rs` | ObjectiveRepository トレイト |
| `src/domain/repository/key_result_repository.rs` | KeyResultRepository トレイト |
| `src/usecase/create_objective.rs` | CreateObjectiveUseCase |
| `src/usecase/get_objective.rs` | GetObjectiveUseCase |
| `src/usecase/list_objectives.rs` | ListObjectivesUseCase |
| `src/usecase/update_objective.rs` | UpdateObjectiveUseCase |
| `src/usecase/close_objective.rs` | CloseObjectiveUseCase（open → closed） |
| `src/usecase/archive_objective.rs` | ArchiveObjectiveUseCase（closed → archived） |
| `src/usecase/update_key_result_progress.rs` | UpdateKeyResultProgressUseCase（進捗更新 + Objective 再計算） |
| `src/usecase/event_publisher.rs` | TargetEventPublisher トレイト + NoopTargetEventPublisher |
| `src/adapter/handler/target_handler.rs` | REST ハンドラー |
| `src/adapter/presenter/target_presenter.rs` | ObjectiveResponse, ObjectiveListResponse, KeyResultResponse |
| `src/adapter/middleware/auth.rs` | JWT 認証ミドルウェア |
| `src/infrastructure/database/objective_repository.rs` | ObjectivePostgresRepository（sqlx 実装） |
| `src/infrastructure/database/key_result_repository.rs` | KeyResultPostgresRepository（sqlx 実装） |
| `src/infrastructure/kafka/target_producer.rs` | TargetKafkaProducer（rdkafka 実装） |

---

## ドメインモデル実装（Rust）

### Objective

project_id + owner_id で管理される目標エンティティ。Key Results の進捗平均を計算するロジックをエンティティに持つ。

ObjectiveStatus の遷移ルール:

- open → closed: close 操作で遷移する
- closed → archived: archive 操作で遷移する
- archived は終端ステータス（遷移不可）

### KeyResult

progress は 0〜100 の整数値。validate_progress() で範囲外を拒否する。

Objective の progress は配下の全 KeyResult の progress の平均値として自動計算される。

### TargetError

| エラー | HTTP | 説明 |
| --- | --- | --- |
| ObjectiveNotFound(Uuid) | 404 | 目標が見つからない |
| KeyResultNotFound(Uuid) | 404 | Key Result が見つからない |
| ValidationFailed(String) | 400 | バリデーションエラー |
| InvalidStatusTransition { from, to } | 400 | 不正なステータス遷移 |
| InvalidProgress(i32) | 400 | 0〜100 範囲外の進捗値 |
| VersionConflict | 409 | 楽観的ロック競合 |
| Internal(String) | 500 | 内部エラー |

---

## ルーティング

board-server と同等の axum Router 構成。

- Public routes: /healthz, /readyz, /metrics
- API routes: /api/v1/objectives/**（JWT 認証 + RBAC ミドルウェア）

---

## インフラ実装（Rust）

### ObjectivePostgresRepository

- update_progress: UPDATE ... SET progress = , version = version + 1 WHERE id =  AND version = 
- update_status: UPDATE ... SET status = , version = version + 1 WHERE id =  AND version = 
- バージョン不一致時はエラーを返す（楽観的ロック）

### Outbox パターン

board-server と同等。update_key_result_progress / close_objective / archive_objective ユースケース内で outbox_events テーブルへ直接書き込む。
k1s0-outbox::OutboxEventPoller への移行は将来 TODO。

---

## テスト

### 単体テスト

| テスト対象 | 内容 |
| --- | --- |
| ObjectiveStatus.can_transition_to | 有効遷移（open→closed, closed→archived）、無効遷移 |
| KeyResult.validate_progress | 0, 50, 100（有効）、-1, 101（無効） |
| UpdateKeyResultProgressUseCase | 進捗更新・平均計算 |
| CloseObjectiveUseCase | 正常クローズ、無効遷移エラー |
| ArchiveObjectiveUseCase | 正常アーカイブ、無効遷移エラー |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [database.md](database.md) -- データベーススキーマ・マイグレーション
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
