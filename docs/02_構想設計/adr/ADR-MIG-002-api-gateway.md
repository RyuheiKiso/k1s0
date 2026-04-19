# ADR-MIG-002: レガシー統合に API Gateway 方式を採用（補完）

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: 移行チーム / API チーム / レガシー所有部門

## コンテキスト

ADR-MIG-001 でサイドカー方式を第一候補としたが、以下のケースではサイドカー方式が適用できない、または非効率である。

- **クライアント起点のルーティング分離**: 「クライアントからは一つの URL でありながら、内部で旧系 / 新系にルーティング分岐」が必要なケース
- **Windows Container 不可環境**: .NET Framework が Windows Server 2008R2 以前で Windows Container 化が困難な資産
- **サイドカー配置不可**: 旧系が Hyper-V VM / ベアメタルで Kubernetes 化不可
- **外部公開 API の段階移行**: API のスキーマ変換（v1 → v2）を含むクライアント影響最小化

業界標準の Strangler Fig Pattern では、API Gateway による段階的ルーティング切替が補完手法として有効。

候補は Envoy Gateway、Kong、Apache APISIX、KrakenD、自作 Gateway（Rust + axum）。

## 決定

**ADR-MIG-001 を補完する手法として、API Gateway 方式を Envoy Gateway（Envoy Proxy ベース、Apache 2.0）で採用する。**

- Envoy Gateway 1.2+（Kubernetes Gateway API 準拠）
- Istio Ambient（ADR-0001）と連携、L7 ルーティング
- ルーティングルールは HTTPRoute / GRPCRoute で宣言的管理
- 旧系 / 新系の段階的切替は Header / Path / Cookie 条件で実施
- スキーマ変換は Envoy の Lua Filter / WASM Filter で実装、複雑な場合は Gateway 後段に変換層
- トラフィック比率切替（5% → 25% → 50% → 100%）を Argo Rollouts（ADR-CICD-002）と連携

## 検討した選択肢

### 選択肢 A: Envoy Gateway（採用）

- 概要: Envoy Proxy ベース、Kubernetes Gateway API 公式実装
- メリット:
  - Istio Ambient（ADR-0001）と同じ Envoy 技術基盤、運用知識再利用
  - Kubernetes Gateway API 標準、将来の実装切替容易
  - Lua / WASM Filter で拡張性高
  - CNCF、コミュニティ活発
- デメリット:
  - WASM Filter 開発の学習曲線
  - Gateway API はまだ進化中、仕様変更注意

### 選択肢 B: Kong

- 概要: Lua 製 API Gateway、業界実績
- メリット: Plugin エコシステム豊富
- デメリット:
  - Lua ベースで Envoy エコシステムと別体系
  - Enterprise 版との機能差

### 選択肢 C: Apache APISIX

- 概要: Apache Foundation、NGINX/OpenResty ベース
- メリット: 高性能、Plugin 豊富
- デメリット:
  - etcd 依存で運用コンポーネント増
  - Envoy エコシステムと別技術系譜

### 選択肢 D: Istio Gateway（Istio 内蔵）

- 概要: Istio Ambient の Waypoint Proxy でルーティング
- メリット: 追加コンポーネント不要
- デメリット:
  - レガシー系との HTTP/1.0 互換等でワークアラウンド必要
  - API Gateway に求められる複雑なスキーマ変換は Envoy Filter 領域

**→ Istio Ambient の Waypoint は同一メッシュ内部用、外部レガシー統合は Envoy Gateway で役割分担**

### 選択肢 E: 自作 Gateway（Rust + axum）

- 概要: tier1 自作領域で独自 Gateway
- メリット: 完全制御
- デメリット:
  - 実装工数膨大
  - Envoy の成熟機能を再実装するコスト

## 帰結

### ポジティブな帰結

- サイドカー方式（ADR-MIG-001）が適用困難なケースに対応可能
- 段階的トラフィック切替で旧系 → 新系移行のリスク最小化
- スキーマ変換（v1 → v2）を Gateway で吸収、クライアント影響ゼロ化
- Envoy 技術基盤統一で運用知識再利用

### ネガティブな帰結

- Envoy Gateway の運用（HA 構成、バージョンアップ）が追加
- WASM Filter 開発スキルが必要、tier1 自作領域で対応
- スキーマ変換ロジックの複雑化、Filter 側のテスト重要
- ルーティングルール肥大化時の管理コスト、モジュール化戦略必要

## 実装タスク

- Envoy Gateway Helm Chart バージョン固定、Argo CD 管理
- Kubernetes Gateway API のリソース設計（GatewayClass、Gateway、HTTPRoute）
- Istio Ambient との連携設計（境界でのトラフィック委譲）
- サイドカー方式（ADR-MIG-001）と API Gateway 方式の使い分けガイドライン
- WASM Filter / Lua Filter のスキーマ変換テンプレート作成
- Argo Rollouts との連携でトラフィック比率切替自動化
- レガシー統合の Runbook: ルーティングルール追加、切替手順
- FMEA: Gateway 障害時の旧系 Direct Route フェイルバック設計

## 参考文献

- Envoy Gateway 公式: gateway.envoyproxy.io
- Kubernetes Gateway API 仕様
- Martin Fowler: Strangler Fig Application
- ADR-MIG-001 サイドカー方式（補完関係）
