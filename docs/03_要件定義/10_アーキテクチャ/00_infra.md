# ARC-INF: インフラ要件

本ファイルは k1s0 プラットフォームの基盤インフラ、すなわち Kubernetes クラスタ形態・ノード構成・サービスメッシュ・クラスタ内ネットワーク・ストレージに関する要件を定義する。インフラ層の選択は tier1〜tier3 すべての実装者に波及する前提であり、ここで固めた制約がその後の tier 要件・セキュリティ要件・運用要件の出発点となる。

本ファイルを読む際は、まず ADR-0001 で確定した「Istio Ambient Mesh + Dapr サイドカー」の共存方針と、`COM-CON-001`（技術スタック制約）が提示するオンプレ閉域ネットワーク前提を頭に入れてから読むこと。これらの上に各要件が重なる構造になっている。

本カテゴリ他ファイル（tier1/2/3・integration・eventing）と比べて、本ファイルは「業務ロジックを知らない最下層」に徹する。業務要件は 20_品質特性・60_事業_契約 で扱われ、本ファイルでは扱わない。

---

## 前提

- [`../00_共通/03_constraint.md`](../00_共通/03_constraint.md) 技術スタック制約（Rust Edition 2024 / Go Dapr SDK / OSS ライセンス / データ国内保管）
- [`../00_共通/00_glossary.md`](../00_共通/00_glossary.md) 用語定義
- [`../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md`](../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md) Istio Ambient Mesh 採用
- [`../../02_構想設計/01_アーキテクチャ/01_基礎/03_配置形態.md`](../../02_構想設計/01_アーキテクチャ/01_基礎/03_配置形態.md) 配置形態
- [`../../02_構想設計/01_アーキテクチャ/02_可用性と信頼性/04_マルチクラスタ戦略.md`](../../02_構想設計/01_アーキテクチャ/02_可用性と信頼性/04_マルチクラスタ戦略.md) マルチクラスタ戦略

---

## 要件本体

### ARC-INF-001: Kubernetes 最小バージョンの固定

- 優先度: MUST（kubeadm / Istio Ambient / Dapr の全てが前提とするバージョン範囲から外れると Phase 1a を組めない）
- Phase: Phase 1a
- 関連: `COM-CON-001`, `ARC-INF-003`

現状、Kubernetes の kubeadm 公式サポートは GA 後 14 ヶ月（直近 3 マイナーバージョン）に限られ、Istio Ambient は 1.28 以降、Dapr 1.13+ は 1.26 以降を要求する。このバージョンマトリクスを明示しないまま社内情シスが古いクラスタを Phase 1a に充てると、Phase 2 に入ってから Istio Ambient ztunnel が起動しない事態が起こり得る。

本要件が満たされた世界では、Phase 1a 着手時点で Kubernetes 1.30 以上を最小バージョンとして固定し、kubeadm サポート期間内の 3 マイナーバージョン（例: 1.30 / 1.31 / 1.32）のいずれかで運用する。バージョンアップは四半期 1 度の定期メンテで実施し、サポート終了 3 ヶ月前までに次期バージョンへの移行計画を着手する。

崩れた場合、Istio Ambient の ztunnel が CNI 連携に失敗してノード間通信が断絶する、もしくは Dapr operator が CRD を更新できず tier1 全体が停止する。実運用では月次パッチ適用すら不可能になり、CVE 対応が遅延する。

**受け入れ基準**

- Phase 1a クラスタの Kubernetes バージョンが 1.30 以上である
- kubeadm サポート期限の 3 ヶ月前までに次期バージョンへの移行 Runbook が整備されている
- Istio / Dapr / CNI の互換バージョンマトリクスが文書化されている

**検証方法**

- `kubectl version` の CI 自動チェック
- バージョンマトリクスを SBOM に併記しリリース前に差分検査

---

### ARC-INF-002: ノード最小構成と専用ノードプール分離

- 優先度: MUST（tier1 と tier3 のリソース競合はレイテンシ予算 p99 500ms を食い潰す主因）
- Phase: Phase 1a
- 関連: `QUA-PRF-001`, `COM-RSK-001`

現状、単一ノードプールに tier1 / tier2 / tier3 / Kafka / Postgres を同居させると、tier3 のバッチ処理が tier1 の gRPC レイテンシを間欠的に 300ms 以上悪化させる。Phase 1a の小規模構成でもこの影響は再現する。

本要件が満たされた世界では、ノードプールを最小 3 系統（`tier1-pool` / `tier2-3-pool` / `stateful-pool`）に分離し、tier1 Pod には `nodeAffinity` で `tier1-pool` を強制する。コントロールプレーンは 3 ノード冗長、ワーカーは各プール最小 2 ノードを維持する。ステートフル系（Postgres / Kafka / etcd バックアップ）は `stateful-pool` の Local PersistentVolume 上に配置する。

崩れた場合、tier3 のバッチが tier1 の CPU を吸い取り p99 SLO 違反が月次 3% を超える。ノード障害時に tier1 と tier3 が同時停止し、業務 UI と基盤 API が同時に落ちる。

**受け入れ基準**

- 最小ノードプール構成が 3 系統以上で定義されている
- tier1 Pod に `nodeAffinity` / `taints / tolerations` が設定されている
- コントロールプレーン 3 ノード冗長構成が稟議提出時点の設計図に含まれる

**検証方法**

- `kubectl get nodes -L node-role` での pool ラベル検証
- k6 による tier3 バッチ同時実行時の tier1 p99 計測

---

### ARC-INF-003: Istio Ambient Mesh の必須化

- 優先度: MUST（ADR-0001 で確定済み。Sidecar モードへ後退する意思決定は新規 ADR が必要）
- Phase: Phase 2（Phase 1a では未導入、Phase 2 着手直後に 1 週間の POC を経て本番投入）
- 関連: `COM-CON-003`, `ARC-T1-002`, ADR-0001

現状、tier1 の Dapr サイドカー注入と Istio Sidecar の二重注入は mTLS 二重掛け・ポートキャプチャ競合・分散トレース断絶を引き起こす。これらは ADR-0001 の「コンテキスト」節で構造的衝突として整理されている。

本要件が満たされた世界では、サービスメッシュは Istio Ambient Mesh モードに固定し、L4 は ztunnel（DaemonSet）が HBONE で担い、L7 は namespace または service 単位の waypoint proxy が担う。アプリ Pod 内には Istio 由来のサイドカーを注入しない。Phase 1a では Ambient を未導入とし、North-South は Envoy Gateway、East-West は Dapr-mTLS で完結させる。Phase 2 で Ambient を導入する際は POC 1 週間を必ず実施する。

崩れた場合、サイドカー二重注入の既知問題に踏み込むことになり、p99 500ms 目標の達成余地が縮退する。Dapr 撤退時の切り離しコストが ADR-0001 の試算を超え、Phase 3 のマルチクラスタ移行で east-west gateway の再検証が二度手間になる。

**受け入れ基準**

- Phase 2 着手直後に Ambient POC（ztunnel HBONE + Dapr-mTLS 共存、W3C Trace Context 継承、p99 影響）を実施し結果を ADR 更新として記録する
- POC 不成立時の Phase 2 着手 1 ヶ月以内のフォールバック判断プロセスが定義されている
- 全 namespace で Istio Sidecar 注入ラベル（`istio-injection=enabled`）が禁止されている

**検証方法**

- namespace ラベルの CI 自動チェック
- POC 結果のレイテンシ実測レポートを Phase 2 ゲートレビューで確認

---

### ARC-INF-004: CNI 選定と NetworkPolicy デフォルト deny

- 優先度: MUST（マルチテナント分離の最下層。CNI を誤ると `SEC-IAM-*` の前提が崩れる）
- Phase: Phase 1a
- 関連: `SEC-IAM-*`, `ARC-INF-003`

現状、Kubernetes デフォルトでは全 Pod 間通信が許可されており、NetworkPolicy 未導入のままマルチテナント運用に入ると、テナント間でパケットが素通しになる。

本要件が満たされた世界では、CNI は Cilium または Calico（いずれも Istio Ambient 互換）に固定し、NetworkPolicy はデフォルト deny を全 namespace で有効化する。tier1 / tier2 / tier3 / ingress / egress の各 namespace について、明示的に許可した通信のみが通る構成とする。MTU は Istio Ambient の HBONE オーバーヘッドを考慮し 1450 バイト以下を実測根拠として設定する。

崩れた場合、テナント間のサイドチャネル通信が物理的に可能となり、GDPR / 個人情報保護法上の分離義務違反として監査指摘を受ける。

**受け入れ基準**

- CNI が Cilium または Calico で固定されている
- 全 namespace で NetworkPolicy デフォルト deny が有効化されている
- MTU 設定と HBONE オーバーヘッドを考慮した計算根拠が文書化されている

**検証方法**

- `kubectl get networkpolicy -A` で deny-all が全 namespace に存在することを確認
- Cilium Hubble / Calico Flow Logs で未許可通信の 0 件化を観測

---

### ARC-INF-005: クラスタ DNS と社内 DNS の連携

- 優先度: MUST（社内 IdP / 監視基盤 / SFTP サーバへの到達性が断絶すると Phase 1a は動作不可）
- Phase: Phase 1a
- 関連: `COM-RSK-003`, `ARC-INT-002`

現状、JTC 環境では corporate proxy と社内 DNS が介在し、CoreDNS のデフォルト forward 設定では内部ホスト名（例: `ldap.corp.jtc.local`）が解決されない。MVP-0 期間過小見積もりリスク（`COM-RSK-003`）の主因でもある。

本要件が満たされた世界では、CoreDNS の `forward` プラグインで社内 DNS（zone: `*.corp.jtc.local`）を明示的に登録し、それ以外のゾーンはクラスタ内 `svc.cluster.local` を優先する。NodeLocal DNS Cache を DaemonSet で配置し、CoreDNS 単一障害点を排除する。corporate proxy 経由の外部名前解決は egress 専用 namespace で例外ルートを設ける。

崩れた場合、Phase 1a で Keycloak が社内 LDAP に到達できず IdP 連携が機能しない、あるいはコンテナレジストリへの pull が社内 proxy 不通でタイムアウトする。

**受け入れ基準**

- CoreDNS の forward 設定に社内 DNS ゾーンが含まれている
- NodeLocal DNS Cache が DaemonSet として配置されている
- corporate proxy 経由の DNS 解決パスが egress namespace で完結している

**検証方法**

- クラスタ内 `nslookup` テストで社内ホスト名が解決されることを確認
- CoreDNS / NodeLocal DNS Cache のメトリクスを Prometheus で監視

---

### ARC-INF-006: ストレージ StorageClass と PodSecurityStandard

- 優先度: MUST（PVC 発行失敗は Phase 1a の Postgres / Kafka 起動失敗に直結）
- Phase: Phase 1a
- 関連: `COM-CON-006`, `SEC-DAT-*`

現状、StorageClass が未定義のまま StatefulSet をデプロイすると PVC が Pending 状態で止まり、根本原因が運用に伝わらない。さらに PodSecurityStandard を有効化していないと privileged コンテナが混入する。

本要件が満たされた世界では、StorageClass は 3 種（`fast-local` / `standard-block` / `backup-object`）を明示定義し、用途を `stateful-pool` 配置 / 通常 PVC / バックアップオブジェクトストレージに分ける。PodSecurityStandard は全 namespace で `restricted` を enforce し、特権昇格・hostPath マウント・rootユーザー起動を禁止する。国内保管要件（`COM-CON-006`）を満たすため、外部オブジェクトストレージは国内リージョンのみを許可する。

崩れた場合、Postgres の WAL が Local PV ではなく低 IOPS のネットワークストレージに配置され、tier1 監査ログ同期が p99 を食い潰す。privileged コンテナ混入で脅威モデルの信頼境界が破られる。

**受け入れ基準**

- 3 種以上の StorageClass が定義されている
- 全 namespace で PodSecurityStandard `restricted` が enforce されている
- バックアップオブジェクトストレージが国内リージョンに固定されている

**検証方法**

- `kubectl get storageclass` の定義確認
- CI での PodSecurityStandard 違反検出

---

### ARC-INF-007: シングルクラスタ / マルチクラスタ判断基準

- 優先度: SHOULD（Phase 1a〜2 はシングル運用で十分。Phase 3 の DR 要件で判断が必要）
- Phase: Phase 3
- 関連: `QUA-DR-*`, `ARC-INF-003`

現状、Phase 1a でマルチクラスタを前提に組むと、2 名運用制約（ADR-0001）と衝突する。一方で Phase 3 以降の DR / リージョン分散要件ではシングルでは不足する可能性がある。

本要件が満たされた世界では、シングル / マルチの判断基準を「テナント数 20 を超えるか」「RPO 要件が 15 分未満か」「リージョン冗長が契約必須か」の 3 条件のいずれかに該当した Phase で初めてマルチ移行を検討する。Phase 3 着手前に Istio Ambient の east-west gateway 方式を ADR として追補記録し、データプレーンの再検証を行う。

崩れた場合、Phase 1a の段階でマルチクラスタを先行導入して運用工数が 2 名枠を超過するか、逆に Phase 3 で DR 要件が発覚してから急ぎマルチ化して ADR-0001 のフォールバック経路（案 D）との整合性が崩れる。

**受け入れ基準**

- シングル / マルチの判断基準が 3 条件で明示されている
- Phase 3 着手前に Ambient east-west gateway の検証計画が ADR 追補として記録される

**検証方法**

- Phase ゲートレビューで判断基準との照合を実施

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| ARC-INF-001 | Kubernetes 最小バージョンの固定 | MUST | 1a |
| ARC-INF-002 | ノード最小構成と専用ノードプール分離 | MUST | 1a |
| ARC-INF-003 | Istio Ambient Mesh の必須化 | MUST | 2 |
| ARC-INF-004 | CNI 選定と NetworkPolicy デフォルト deny | MUST | 1a |
| ARC-INF-005 | クラスタ DNS と社内 DNS の連携 | MUST | 1a |
| ARC-INF-006 | ストレージ StorageClass と PodSecurityStandard | MUST | 1a |
| ARC-INF-007 | シングル / マルチクラスタ判断基準 | SHOULD | 3 |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 6 | ARC-INF-001, 002, 003, 004, 005, 006 |
| SHOULD | 1 | ARC-INF-007 |

---

## 関連図

本カテゴリの構造図（tier 境界とインフラ層の積み上げ）は後続タスクで drawio を追加する。
