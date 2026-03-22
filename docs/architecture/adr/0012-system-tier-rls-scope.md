# ADR-0012: system 層 DB のマルチテナント RLS 適用範囲の決定

## ステータス

承認済み

## コンテキスト

k1s0 のマルチテナント設計（`docs/architecture/multi-tenancy.md`）では、**行レベルセキュリティ（RLS）+ tenant_id カラム**を基本戦略として採用している。

Phase 1 では service 層（task-db, activity-db, board-db）に tenant_id と RLS を追加した。一方、system 層のデータベースには未対応のものが残っており、以下の状況にあった。

| データベース | 状況 | リスク |
|------------|------|--------|
| `saga-db` | tenant_id / RLS なし | Saga ワークフローのテナント間混入リスク |
| `event-store-db` | tenant_id / RLS なし | イベントストリームのテナント間混入リスク |
| `featureflag-db` | tenant_id / RLS なし | 設計上テナント横断のグローバル設定 |
| `api-registry-db` | tenant_id / RLS なし | 設計上テナント横断のシステム設定 |

特に saga-db と event-store-db はテナント固有のビジネスワークフローとドメインイベントを格納するため、テナント分離が欠如していると、テナント A のデータがテナント B から参照可能になるセキュリティリスクが生じる。

## 決定

以下のように system 層 DB の RLS 適用範囲を決定する。

**RLS を追加する DB:**

- `saga-db`: `saga.saga_states`, `saga.saga_step_logs` テーブルに tenant_id カラムと RLS ポリシーを追加する（マイグレーション 008）
- `event-store-db`: `eventstore.event_streams`, `eventstore.events`, `eventstore.snapshots` テーブルに tenant_id カラムと RLS ポリシーを追加する（マイグレーション 006）

**テナント非依存として扱う DB（RLS 対象外）:**

- `featureflag-db`: フィーチャーフラグはシステム全体のグローバル設定としてテナント横断で共有する
- `api-registry-db`: API レジストリはシステムアーキテクチャのグローバル情報としてテナント横断で管理する

RLS ポリシーは service 層と統一した以下のパターンを使用する。

```sql
-- テナント分離ポリシー（セッション変数による動的フィルタリング）
CREATE POLICY tenant_isolation ON {schema}.{table}
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
```

## 理由

### saga-db / event-store-db に RLS を追加する理由

- **ビジネスデータの分離**: Saga ワークフローとドメインイベントはテナント固有のビジネスロジックを実行・記録する。テナント間でデータが混入すると、補償トランザクションや監査に誤りが生じる
- **コンプライアンス**: マルチテナント SaaS では、テナント間のデータ分離はセキュリティ要件として不可欠
- **一貫性**: service 層と同一パターンを採用することで、開発者の認知負荷を最小化する

### featureflag-db / api-registry-db をテナント非依存とする理由

- **フィーチャーフラグの設計**: フラグの ON/OFF はシステム運用者がグローバルに管理するもので、特定テナントにのみ異なる設定を持たせることは現設計の想定外
- **API レジストリの設計**: サービスディスカバリとルーティング情報はインフラレベルのグローバル情報であり、テナントごとに異なるエンドポイントを持つ設計ではない
- **Phase 3 課題への先送り**: テナントごとのフィーチャーフラグ制御が必要になった場合は、別途設計変更として実施する

## 影響

**ポジティブな影響**:

- saga-db と event-store-db でテナント間データ混入リスクが解消される
- service 層と統一されたパターンにより、リポジトリ層の実装が簡潔になる
- 監査・コンプライアンス上の要件を満たす RLS 保護が system 層にも適用される

**ネガティブな影響・トレードオフ**:

- **データ移行が必要**: マイグレーション適用時、既存レコードには `tenant_id = 'system'` がバックフィルされる。これらのレコードに正確な tenant_id を設定するには、別途データ移行スクリプトの実行が必要
- **アプリケーション改修が必要**: saga-db / event-store-db を使用するサービスは、DB 接続時に `SET LOCAL app.current_tenant_id` を設定する処理を追加する必要がある
- **featureflag / api-registry の将来的な制限**: テナントごとのフラグ制御が必要になった場合、この決定を覆すコストが発生する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全 DB に tenant_id を追加 | featureflag-db / api-registry-db を含む全 DB に RLS を追加 | 現設計では不要な複雑性を導入する。テナント横断の管理機能が制限される。Phase 3 以降の課題として記録 |
| スキーマ分離（テナントごとに別スキーマ） | テナントごとに独立したスキーマを作成 | テナント数の増加に伴いスキーマ数が爆発する。DDL 管理コストが高い |
| アプリケーション層のみでフィルタリング | DB レベルの RLS を使わずアプリ層で tenant_id フィルタを実装 | 実装漏れによるテナント間データ露出リスクが残る。DB レベルの防御が欠如する |
| 段階適用（事後的に全 DB に適用） | Phase 1〜4 の最後に全 DB へ一括適用 | セキュリティリスクを先延ばしにする。service 層に比べて system 層の対応が遅れることで整合性が失われる |

## 参考

- [マルチテナント設計](../multi-tenancy.md)
- [ADR-0007: Saga 補償ロジックと在庫予約](0007-saga-compensation-inventory-reservations.md)
- saga-db マイグレーション: `regions/system/database/saga-db/migrations/008_add_tenant_id_rls.up.sql`
- event-store-db マイグレーション: `regions/system/database/event-store-db/migrations/006_add_tenant_id_rls.up.sql`
- service 層 RLS 参考実装: `regions/service/task/database/postgres/migrations/009_add_tenant_id_and_rls.up.sql`
