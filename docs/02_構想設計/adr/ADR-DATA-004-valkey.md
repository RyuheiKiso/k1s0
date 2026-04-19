# ADR-DATA-004: KV ストア／キャッシュに Valkey を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / データ基盤チーム / 運用チーム / 法務部

## コンテキスト

tier1 の State API の KV ストア、Workflow API の idempotency key 管理、Valkey 前提のキャッシュ（セッション、ルール評価結果、認可決定）、ratelimit カウンタなど、k1s0 全体で高速な KV ストア／キャッシュが必要。

従来この用途は Redis が業界デファクトだったが、Redis Labs が 2024 年 3 月に Redis 7.4 以降を RSALv2/SSPL ライセンスに変更したことで、商用利用可能性に重大な変化が生じた。RSALv2 は「同じ領域でのサービス提供」を禁じる条項を含み、k1s0 のような PaaS での組込みが法務リスクになる可能性がある。

Linux Foundation が 2024 年 3 月に Redis のフォーク「Valkey」を設立、BSD-3-Clause ライセンスで開発継続することを発表。AWS、Google Cloud、Oracle、Ericsson 等の大手が支持表明。2024 年 4 月に Valkey 7.2.5 リリース、以降も活発に開発継続。

## 決定

**KV ストア／キャッシュは Valkey（BSD-3-Clause、Linux Foundation）を採用する。**

- Redis 7.2 系互換 API、Redis クライアントがそのまま使える
- 3 ノード以上の Sentinel 構成で HA（将来 Cluster モード検討）
- Kubernetes StatefulSet で管理、Valkey Operator または汎用 Helm Chart
- TLS 転送時暗号化、ACL で認可
- 保管時データは原則非永続（RDB/AOF を使う場合は LUKS 下位暗号化）
- tier1 State API / Workflow API からの利用は Dapr State Component / 自作ラッパー経由

将来 Redis が OSS 回帰した場合、または Valkey が方向性を失った場合の切替え可能性を維持するため、アプリ側は Redis 互換 API（基本コマンド）のみ使用、Redis/Valkey 固有拡張機能は避ける。

## 検討した選択肢

### 選択肢 A: Valkey（採用）

- 概要: Linux Foundation 傘下の Redis フォーク、BSD-3-Clause
- メリット:
  - 真の OSS ライセンス（BSD-3-Clause）、ベンダーロックなし
  - Redis 7.2 完全互換、既存クライアント・ツールがそのまま動く
  - AWS/GCP/Oracle/Ericsson 等大手の支援、長期継続性が期待できる
  - CNCF プロジェクトとしての成熟度（Redis のコードベース継承）
- デメリット:
  - 2024 年 3 月発足で運用実績がまだ積上げ段階
  - Redis との API 互換性は将来的に乖離する可能性
  - 大規模事例（業界での Valkey 単独採用）はこれから増えるフェーズ

### 選択肢 B: Redis 7.4+（RSALv2/SSPL）

- 概要: Redis 本家の最新版
- メリット: 既存ノウハウそのまま、ドキュメント豊富
- デメリット:
  - RSALv2/SSPL ライセンスが k1s0 の商用利用と抵触する可能性
  - 法務リスクを抱えた運用になる
  - 将来のライセンス変更リスク（BUSL への移行等）

### 選択肢 C: Redis 7.2 の最終 OSS 版（BSD）で凍結

- 概要: ライセンス変更前の最終版を使い続ける
- メリット: ライセンス問題なし
- デメリット:
  - セキュリティパッチ供給が停止する
  - 数年後には脆弱性リスクが蓄積
  - 実質的に Valkey と同じ状態を自力維持することになる

### 選択肢 D: KeyDB（Snapchat 由来、BSD-3）

- 概要: Redis マルチスレッドフォーク
- メリット: ライセンスクリア、マルチスレッドで性能有利
- デメリット:
  - コミュニティ規模が小さい
  - 2022 年以降のコミット頻度が低下傾向

### 選択肢 E: DragonflyDB（Business Source License）

- 概要: モダンな Redis 互換、メモリ効率優秀
- メリット: 性能が Redis の数倍
- デメリット:
  - Business Source License で商用利用制限
  - ADR-0003 の AGPL 分離より厳しい商用制約

## 帰結

### ポジティブな帰結

- ライセンスリスクの構造的回避（BSD-3-Clause）
- ベンダーロックなし、Linux Foundation ガバナンスで中立性
- 既存 Redis ノウハウ・ツール資産をそのまま活用可能
- 将来 Redis 回帰・他選択肢への切替え可能性を維持

### ネガティブな帰結

- Valkey 運用実績が Redis より薄い（2026 年時点で 2 年程度）
- コミュニティ・ドキュメントが Redis から移行中の過渡期
- Redis 7.4 以降の新機能を使えない（互換性維持のため）

## 実装タスク

- Valkey Helm Chart バージョン固定、Argo CD で管理
- Sentinel 構成の運用 Runbook 整備
- Redis クライアントライブラリの設定を Valkey エンドポイントに向ける（アプリ側変更はほぼ不要）
- ACL ポリシーテンプレートを Backstage Software Template 化
- バージョンアップ手順書（Valkey 7.2 → 8.x への移行）を Runbook 化（将来）

## 参考文献

- Valkey 公式: valkey.io
- Linux Foundation Valkey Project Announcement (2024-03)
- Redis ライセンス変更の影響分析（OSI、SFLC）
- BSD-3-Clause 本文
