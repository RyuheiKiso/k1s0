# ADR-0018: メッセージング抽象層の統一

## ステータス

承認済み

## コンテキスト

k1s0 モノリポには現在、同種のメッセージング抽象を提供するライブラリが 4 つ乱立している。

| ライブラリ | 場所 | 概要 |
|-----------|------|------|
| `messaging` | `regions/system/library/*/messaging/` | Kafka を中心とした汎用メッセージング抽象 |
| `building-blocks` | `regions/system/library/*/building-blocks/` | イベントバス・メッセージバス等のビルディングブロック集 |
| `pubsub` | `regions/system/library/*/pubsub/` | Pub/Sub パターン専用の軽量実装 |
| `event-bus` | `regions/system/library/*/event-bus/` | ドメインイベントのインプロセス配信向け実装 |

この状況により、新規サービス実装時にどのライブラリを使用すべきかが不明確になっている。
また、同種の機能がバラバラに改善・バグ修正されるため、品質にばらつきが生じている。

## 決定

`messaging` ライブラリを canonical（正規）のメッセージング抽象層として統一し、
`building-blocks`・`pubsub`・`event-bus` の 3 ライブラリを deprecated（非推奨）とする。

- **新規サービス**: `messaging` ライブラリのみを使用すること
- **既存サービス**: 段階的に `messaging` へ移行する（移行完了まで deprecated ライブラリは削除しない）
- **ドメインイベント（インプロセス）**: `messaging` の同期ディスパッチャー機能を拡充して対応する

## 理由

1. **保守コスト削減**: 1 つのライブラリに集約することで、バグ修正・機能追加の影響範囲が明確になる
2. **認知負荷の軽減**: 新規開発者がどのライブラリを使うべきか迷う必要がなくなる
3. **テスト品質の向上**: `messaging` ライブラリはモック・テストユーティリティが最も充実している
4. **Kafka 対応**: 現行プロダクションユースケース（Outbox パターン）が全て Kafka を使用しており、`messaging` が最も適合する
5. **段階的移行が可能**: deprecated マークを付けた上で既存コードをそのまま残せるため、移行リスクを最小化できる

## 影響

**ポジティブな影響**:

- 新規サービス実装時の技術的意思決定が簡略化される
- メッセージング関連のバグ修正が 1 か所で済む
- `messaging` ライブラリのドキュメント・テストカバレッジに集中投資できる
- 将来的なブローカー差し替え（Kafka → NATS 等）も 1 ライブラリを変更するだけで対応可能

**ネガティブな影響・トレードオフ**:

- 既存サービスの `building-blocks`・`pubsub`・`event-bus` 依存を段階的に除去する作業が必要
- `messaging` ライブラリがインプロセスイベント配信（`event-bus` 相当）をまだカバーしていないため、機能追加が先行して必要
- deprecated ライブラリが完全削除されるまでの期間、コードベースに重複が残る

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全ライブラリを削除してゼロから設計 | 新設計で統一した抽象を実装 | 移行コストが過大。全サービスの書き直しが必要で稼働リスクが高い |
| `building-blocks` を canonical に選定 | 最も汎用的な名称のライブラリを採用 | Kafka 固有のユースケース（Outbox）での実績が `messaging` に劣る |
| 全ライブラリを並行維持 | 現状維持 | 問題が解決しないためコスト増加が継続する |

## 参考

- [可観測性設計.md](../observability/可観測性設計.md) — メッセージング基盤と可観測性の統合方針
- [ADR-0017: Kong OIDC プラグイン移行](./0017-kong-oidc-migration.md) — 依存ライブラリ統一の先行事例
- Outbox パターン実装: `regions/service/task/server/rust/task/src/infrastructure/outbox_poller.rs`

## 実装進捗

- [x] messaging ライブラリを正規実装として指定
- [x] building-blocks, event-bus, bb-pubsub を DEPRECATED マーキング（各 `Cargo.toml` の `description` フィールドに明記）
- [ ] 既存サービスの building-blocks/event-bus 依存を messaging に移行
- [ ] building-blocks, event-bus, bb-pubsub ライブラリのアーカイブ

## 移行タイムライン（HIGH-DOC-02 / MEDIUM-DOC-02 監査対応）

### Service 層 Outbox 移行状況

| サービス | 現状 | 移行先 | 状態 |
|---------|------|--------|------|
| task-server | 独自 OutboxEventPoller 実装 | `k1s0-outbox::OutboxEventPoller` | 未着手 |
| board-server | 独自 OutboxEventPoller 実装 | `k1s0-outbox::OutboxEventPoller` | 未着手 |
| activity-server | 独自 OutboxEventPoller 実装 | `k1s0-outbox::OutboxEventPoller` | 未着手 |

### Deprecated ライブラリ移行状況

| ライブラリ | 現状 | 移行先 | 状態 |
|-----------|------|--------|------|
| `building-blocks` | deprecated | `messaging` | 部分移行中 |
| `pubsub` | deprecated | `messaging` | 部分移行中 |
| `event-bus` | deprecated | `messaging` | 部分移行中 |

各サービスの Cargo.toml に `# TODO: k1s0-outbox::OutboxEventPoller に移行する（ADR-0018 参照、MEDIUM-CODE-01 監査対応）` コメントが残存している。
移行完了の基準: `k1s0-outbox` クレートの全機能が本 ADR の要件を満たし、全コンシューマーが移行済みであること。

### 2026-03-29 時点の進捗サマリー

- **messaging ライブラリ**: 正規化済み（canonical 指定完了）
- **task/board/activity サービス**: 各サービスが独自 `OutboxEventPoller` を実装中。`k1s0-outbox` クレートへの移行は未着手だが、Outbox パターン自体は ~80% のサービスで導入済み
- **business 層サービス**: messaging ライブラリへの移行中。deprecated ライブラリへの新規依存は停止済み
- **deprecated ライブラリ**: `building-blocks`・`pubsub`・`event-bus` の 3 ライブラリは deprecated マーキング済み。既存の依存箇所は段階的に除去中
