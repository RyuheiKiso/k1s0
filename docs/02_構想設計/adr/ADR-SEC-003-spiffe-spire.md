# ADR-SEC-003: ワークロード ID に SPIFFE / SPIRE を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / セキュリティチーム

## コンテキスト

k1s0 はマイクロサービス間・tier 間で頻繁に mTLS を確立する必要がある。従来の方式は、手動発行した X.509 証明書を ConfigMap/Secret に置き、アプリが読み込む構成だが、以下の運用上の限界がある。

- **証明書ライフサイクル管理が手動化**: 期限切れ事故、ローテーション漏れが発生
- **アイデンティティ貧弱**: 「この Pod は tier1-state-service」という意味論的 ID を表現しづらい
- **ポリシー記述が IP/CIDR 依存**: 動的環境で脆弱
- **監査時のトレーサビリティ不足**: 「誰が誰を呼んだか」の証跡が弱い

業界標準の解決策として、SPIFFE（Secure Production Identity Framework For Everyone）と SPIRE（SPIFFE Runtime Environment）が存在。Istio、Dapr、HashiCorp Consul、Linkerd など多くの主要プロジェクトが SPIFFE を採用・互換している。

## 決定

**ワークロード ID は SPIFFE / SPIRE（CNCF Graduated、Apache 2.0）を採用する。**

- SPIRE Server を各クラスタに配置、Trust Domain は `k1s0.jtc.example.internal`
- SPIRE Agent を DaemonSet で各 Node に配置
- SPIFFE ID は `spiffe://k1s0.jtc.example.internal/ns/<namespace>/sa/<serviceaccount>` の命名規則
- Workload API 経由で SVID（SPIFFE Verifiable Identity Document、X.509 形式）を自動取得
- Istio Ambient Mesh（ADR-0001）とは連携モード（Istio が SPIRE を CA として使う）
- tier1 内部通信の mTLS 証明書はすべて SPIRE 発行
- 認可ポリシー（AuthorizationPolicy）は SPIFFE ID ベースで記述、CIDR/IP 依存を排除

## 検討した選択肢

### 選択肢 A: SPIFFE + SPIRE（採用）

- 概要: CNCF Graduated のワークロード ID 標準
- メリット:
  - 業界標準、Istio/Dapr/Consul 等の主要プロジェクトが互換
  - ID ベースの認可ポリシーで動的環境に強い
  - 証明書自動ローテーション（TTL 1 時間等）が標準
  - 監査ログで SPIFFE ID ベースの証跡が残る
  - マルチクラスタ / マルチ Trust Domain の将来拡張も可能
- デメリット:
  - SPIRE の運用学習コスト（Server/Agent/Registration Entry の理解）
  - Istio 連携の設定が複雑（Istio 側とも整合性確保が必要）

### 選択肢 B: cert-manager のみでアプリ証明書発行

- 概要: cert-manager で X.509 証明書を発行、アプリ Pod が読込
- メリット: シンプル、cert-manager 単独で完結
- デメリット:
  - SPIFFE ID のような意味論的 ID 概念がない
  - Pod 内への証明書配布が手動（CSI Driver 等で補完可能だが標準化されていない）
  - 認可が CIDR/IP ベースに寄る

### 選択肢 C: Istio のみの mTLS（SPIRE なし）

- 概要: Istio 組込みの Citadel/istiod で mTLS 管理
- メリット: 統合シンプル
- デメリット:
  - Istio 以外のコンポーネント（Dapr、バッチ Job 等）への展開がアドホック
  - マルチクラスタで ID 体系が Istio 固有になり、他製品連携時に齟齬

### 選択肢 D: アプリ自作の mTLS

- 概要: アプリ層で mTLS 実装、証明書は ConfigMap 経由
- メリット: フレームワーク依存なし
- デメリット: 全アプリ横並びで実装 + 運用する工数が膨大

## 帰結

### ポジティブな帰結

- 証明書ライフサイクルが自動化、期限切れ事故を構造的回避
- 意味論的 ID（SPIFFE ID）で認可ポリシーが明瞭
- 業界標準で将来の他ツール（Istio、Consul、Linkerd）への移行性を保持
- マルチクラスタ DR 構成でも同一 ID 体系が使える
- 監査ログの証跡が SPIFFE ID 単位で整理される

### ネガティブな帰結

- SPIRE の運用知識が新規に必要（Runbook 整備と訓練）
- Istio Ambient + SPIRE の連携設定の複雑さ、POC が必要
- Trust Domain の命名とガバナンス（どの namespace が発行できるか）を初期に決定
- SPIRE Server 障害時の影響範囲（証明書発行停止）に備えた HA 構成と緊急手順

## 実装タスク

- SPIRE Server / Agent の Helm Chart バージョン固定、Argo CD 管理
- Trust Domain 命名規則を明文化（k1s0.jtc.example.internal）
- SPIFFE ID の命名規則ドキュメント化（ns/<namespace>/sa/<serviceaccount>）
- Istio Ambient + SPIRE の POC（ADR-0001 の POC と合流）
- Registration Entry の自動化（Kyverno + Kubernetes Workload Attestor）
- Runbook: SPIRE Server 障害時の緊急証明書発行手順
- cert-manager との役割分担（外部公開用は cert-manager + Let's Encrypt、内部は SPIRE）

## 参考文献

- SPIFFE 公式: spiffe.io
- SPIRE 公式: github.com/spiffe/spire
- NIST SP 800-207 Zero Trust Architecture
- Istio + SPIRE 統合: istio.io/latest/docs/ops/integrations/spire/
