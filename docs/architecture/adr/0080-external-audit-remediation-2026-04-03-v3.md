# ADR-0080: 外部技術監査対応 v3（2026-04-03）

## ステータス

承認済み

## コンテキスト

2026-04-03 に外部プリンシパル・エンジニアによる第三者監査が実施された（前回 ADR-0079 の v2 監査に続く第 3 回）。
総合評価は C+。前回監査対応（v1/v2）で改善した箇所が実装不備により再度問題化していた。

監査指摘の 34 件 + DOC 5 件のうち、9 件は既に対応済みを確認し、残り 25 件 + DOC 5 件を本対応で実装した。

特に以下 3 点が本番稼働に対する直接的なリスクを有していた：
1. **featureflag DB RLS 二重障害**: マイグレーション 005 は存在するが、アプリ層が `set_config()` を呼ばないため RLS が機能しない
2. **auth-rust RBAC 無効**: `auth-rust-admin` Keycloak クライアントが未登録のため静的フォールバックで稼働
3. **dlq-manager DLQ 全未処理**: `*.v1.dlq` が glob として解釈され librdkafka に一致しない

## 決定

### CRIT-001: featureflag RLS set_config() 実装
`featureflag_postgres.rs` の全 6 メソッドをトランザクション + `set_config('app.current_tenant_id', $1, true)` パターンに変更する。
lessons.md のポリシー（`SET LOCAL = $1` 禁止）を遵守し、`tenant_id.to_string()` で TEXT キャストを行う。
これにより DB レベルの RLS がアプリケーション層の WHERE 句と組み合わさった二重防御を実現する。

### CRIT-002: auth-rust-admin Keycloak クライアント追加
`infra/docker/keycloak/k1s0-realm.json` に `auth-rust-admin` クライアントを追加し、
Client Credentials Grant でトークン取得を可能にする。`serviceAccountsEnabled: true` を設定。

### CRIT-004: CLI seed SQL インジェクション修正
`psql -c &content` を `psql -f seed_file_path` に変更し、ファイル内容の直接実行を回避する。

### CRIT-005: dlq-manager DLQ トピックパターン正規表現化
`*.v1.dlq`（glob）を `^.*\.v1\.dlq$`（librdkafka 正規表現）に変更する。
librdkafka の `subscribe()` は `^` プレフィックス付きの文字列を正規表現として認識する。

### HIGH-004: PostgreSQL NOINHERIT
`CREATE ROLE k1s0` に `NOINHERIT` を追加し、権限の自動継承を防止する。

### HIGH-005: outbox_events RLS マイグレーション
activity/board/task の 3 サービスに outbox_events 用 RLS マイグレーションを追加する。
バックグラウンドパブリッシャーが set_config 未呼出しでも動作できるよう、
`current_setting('app.current_tenant_id', true) IS NULL` 条件を含むポリシーを設定する。

### HIGH-006: workflow SET LOCAL → set_config() 統一
workflow の 4 ファイル 18 箇所の `SET LOCAL app.current_tenant_id = $1` を
`SELECT set_config('app.current_tenant_id', $1, true)` に統一する。

### HIGH-011: CLI パスワードハードコード除去
`unwrap_or_else(|_| "password".to_string())` を除去し、環境変数未設定時にエラー終了する実装に変更する。

## 理由

### set_config() vs SET LOCAL の選択
- `SET LOCAL name = $1` は PostgreSQL の `SET` 文がパラメータプレースホルダーをサポートしないため実行時エラーになる（lessons.md）
- `SELECT set_config(name, $1, true)` は通常の関数呼び出しのためパラメータバインドが可能
- 第 3 引数 `true` が `SET LOCAL` と同等のトランザクションスコープを提供する

### outbox_events RLS ポリシーの設計
- 単純な `tenant_id = current_setting(...)::TEXT` ではバックグラウンドパブリッシャー（set_config 未呼出し）が全件読めなくなる
- `current_setting('app.current_tenant_id', true) IS NULL` 条件を追加することで、未設定時は全テナントアクセスを許可し、設定時は対象テナントのみに絞る
- これによりパブリッシャーの動作を維持しつつ、アプリケーション層の tenant scoping を強制できる

### librdkafka 正規表現プレフィックス
- librdkafka の `consumer.subscribe()` は `^` プレフィックス付き文字列を正規表現として扱う
- グロブパターン（`*.v1.dlq`）は存在しないリテラルトピック名として解釈されるため、実際のトピックに一致しない

## 影響

**ポジティブな影響**:
- featureflag のテナント間データ漏洩リスクを RLS 二重防御で排除
- auth-rust が Keycloak の権限変更をリアルタイムに反映できるようになる
- DLQ が正常に機能し、失敗イベントの再処理・監視が可能になる
- CLI seed の供給チェーン攻撃リスクを低減
- workflow の RLS が SET LOCAL のパラメータバインドエラーから修正される

**ネガティブな影響・トレードオフ**:
- featureflag の全クエリがトランザクション内で実行されるためわずかなオーバーヘッドが発生する
- dlq_topic_pattern の変更は既存の設定ファイルオーバーライドに影響する可能性がある
- Flutter session ID バリデーション強化により、既存の非英数字セッション ID は無効判定になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| featureflag SUPERUSER 接続 | RLS を FORCE しない設定 | 最小権限原則に反する |
| outbox FORCE RLS | バックグラウンドパブリッシャーを BYPASSRLS 付きロールで実行 | ロール設計の変更が必要で影響範囲が大きい |
| DLQ 全トピック列挙 | `subscribe(["k1s0.system.auth.audit.v1.dlq", ...])` | トピック追加のたびに設定変更が必要 |

## 参考

- [ADR-0079: 外部技術監査対応 v2](0079-external-audit-remediation-2026-04-03-v2.md)
- [ADR-0078: 外部技術監査対応](0078-audit-response-2026-04-03.md)
- [ADR-0045: Vault per-service ロール分離](0045-vault-per-service-role-isolation.md)
- [報告書.md](../../../../報告書.md)
- lessons.md: PostgreSQL set_config() パターン

## 実装ステータス（2026-04-04 更新）

本 ADR 記載の対応は全件実装完了。2026-04-04 外部監査報告書対応により確認済み。

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（外部監査 v3 対応） | kiso ryuhei |
| 2026-04-04 | 実装ステータス追記（全件完了確認） | kiso ryuhei |
