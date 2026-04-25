# ADR-STOR-002: オンプレ LoadBalancer に MetalLB を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / ネットワーク運用チーム

## コンテキスト

k1s0 はオンプレミス Kubernetes 上で稼働する。tier1 公開 API（Dapr ファサード）、Istio Ambient の Ingress Gateway、Keycloak、Backstage、Grafana 等は、クラスタ外からのアクセスを受け付けるため Service type=LoadBalancer を要求する。

クラウド（AWS ELB / GCP LB）のマネージド LB はオンプレでは使えず、ハードウェア LB（F5、A10 等）は年間数千万〜億円規模のコストと物理調達リードタイムを要する。

Kubernetes オンプレ環境で LoadBalancer サービスを実現する選択肢は、MetalLB、Cilium LB IPAM、kube-vip、ハードウェア LB との連携（BGP スピーカ or F5 CIS）。

## 決定

**オンプレ LoadBalancer は MetalLB（CNCF Sandbox、Apache 2.0）を BGP モードで採用する。**

- MetalLB 0.14+
- **BGP モード**: コア / 境界ルータと BGP ピア、ECMP で負荷分散
- リリース時点では L2 モード（ARP）でも運用可能、採用側のスケール拡大時に BGP に切替
- IP アドレスプール（AddressPool）を tier 別・用途別に分離
- Istio Gateway、Keycloak、Backstage、Grafana 等に個別 IP 割当
- ハードウェア LB との連携は将来オプション（商用 SI 要件で必要になる場合）

## 検討した選択肢

### 選択肢 A: MetalLB BGP モード（採用）

- 概要: CNCF Sandbox、業界標準のオンプレ LB
- メリット:
  - BGP で真の冗長化、ECMP 負荷分散
  - コスト: ソフトウェアのみ、ハードウェア追加コストなし
  - 運用実績豊富、Kubernetes エコシステムでデファクト
- デメリット:
  - BGP ピアリングの設定が必要、ネットワークチームとの連携必須
  - BGP 運用知識が運用チームに要求される

### 選択肢 B: MetalLB L2 モード

- 概要: ARP / NDP でリーダー選出、冗長性は限定的
- メリット:
  - BGP 設定不要、同一 L2 ネットワーク内で即稼働
  - ネットワークチーム連携不要
- デメリット:
  - リーダー Node 障害時にフェイルオーバー時間発生
  - スケールアウト性で BGP に劣る

**→ 採用初期のみ L2 モード、採用側のスケール拡大時に BGP に移行する段階的戦略を採用**

### 選択肢 C: Cilium LB IPAM

- 概要: Cilium に内蔵された LB IPAM
- メリット: Cilium 統合で CNI と LB 一元管理
- デメリット:
  - Istio Ambient 採用（ADR-0001）で Cilium 依存は慎重に選択
  - CNI 選定が前提条件化

### 選択肢 D: kube-vip

- 概要: Control Plane 用 VIP として実績、Service LB も対応
- メリット: 軽量、Control Plane VIP と統合
- デメリット:
  - MetalLB に比べて IP Pool 管理機能が弱い
  - BGP サポートは後発

### 選択肢 E: ハードウェア LB（F5 / A10）

- 概要: 商用 LB アプライアンス
- メリット: 機能最大、商用サポート、実績
- デメリット:
  - 年間数千万〜億円規模コスト
  - 物理調達リードタイム
  - Kubernetes CSI 統合の複雑さ

### 選択肢 F: HAProxy / NGINX + keepalived 自作

- 概要: 従来型のオンプレ LB 構築
- メリット: コスト最小、柔軟性最大
- デメリット:
  - Kubernetes Service type=LoadBalancer と統合できず、ExternalIP 手運用
  - 運用工数膨大

## 帰結

### ポジティブな帰結

- ハードウェア LB コストを回避、採用側の予算内で実現
- Kubernetes ネイティブで type=LoadBalancer が標準動作
- BGP で真の冗長化、SLO 達成に寄与
- Argo CD / Helm で宣言的管理

### ネガティブな帰結

- BGP 運用知識が運用チームに必要、教育コスト
- BGP ピアリングのネットワークチーム連携、設計レビュープロセス整備
- L2 モード → BGP モード移行時の切替ダウンタイム計画
- MetalLB Controller 自体の HA 設計

## 実装タスク

- MetalLB Helm Chart バージョン固定、Argo CD 管理
- IP Pool 設計: tier1 Dapr / Istio Gateway / Keycloak / Backstage / Grafana / MinIO
- リリース時点での L2 モード設定、境界ルータと共存
- BGP モード移行計画: BGP AS 番号設計、ピア設定、検証環境での事前試験
- ネットワークチームとの連携プロトコル策定（IP Pool 追加、BGP 設定変更手順）
- MetalLB Controller / Speaker の HA 設計、FMEA
- Runbook: IP Pool 枯渇時、BGP ピアダウン時の復旧手順

## 参考文献

- MetalLB 公式: metallb.universe.tf
- CNCF MetalLB Project
- Kubernetes Service 仕様（type=LoadBalancer）
- BGP 運用ベストプラクティス
