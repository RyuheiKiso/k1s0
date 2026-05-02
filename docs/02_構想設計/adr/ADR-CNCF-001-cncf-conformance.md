# ADR-CNCF-001: vanilla Kubernetes（CNCF Conformance 互換）を維持する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / 採用検討組織 / インフラ運用チーム

## コンテキスト

k1s0 は採用検討組織のオンプレ K8s クラスタ上に展開される PaaS であり（NFR-F-SYS-001）、採用組織が既に運用中の K8s（または将来導入する K8s）と互換であることが採用判断の前提となる。「k1s0 を採用すると K8s ディストリビューションも乗り換える必要がある」状態は、運用エンジニアの再教育・既存ツール（kubectl / kustomize / Helm / Argo CD）の動作保証・障害時の業界知識流用性のいずれにおいても採用障壁を生む。

加えて、k1s0 自身が「採用組織が世代交代しても 10 年保守できる」前提で設計されている（ADR-DEV-001）以上、Kubernetes 本体の互換性を独自方向に分岐させると将来の K8s upstream 進化（GA 機能・新規 resource・標準 API の deprecation）から段階的に取り残されるリスクを抱える。

業界では K8s の派生形態として以下 3 系統が存在し、それぞれの選択は採用組織への要請が異なる。

1. **vanilla K8s**（CNCF Conformance 互換、kubeadm / CAPI / 主要マネージド K8s が該当）
2. **OpenShift / Rancher RKE2 系派生**（vanilla 上位互換だが事業者独自 component を含む）
3. **Talos / k0s 系特化型**（K8s 用途に純化、独自 OS / 独自 API 駆動）

加えて、採用組織が CNCF Conformance 認証を取得する立場に立つ可能性（自社製品としての k8s ディストリビューション化）も将来オプションとして残しておきたい。

## 決定

**k1s0 が動作する K8s は vanilla Kubernetes（CNCF Conformance 互換、`sonobuoy` 等で Conformance テストが pass する状態）を維持する。独自 admission controller / 独自 API server 改造 / upstream-incompatible な distribution は採用しない。**

- 採用 Kubernetes は upstream 1.31 LTS（kubeadm / CAPI 経由、ADR-INFRA-001）
- CNCF Conformance テスト（sonobuoy）を kind multi-node で定期実行
- 独自 component（k1s0-scaffold / Roslyn Analyzer / 自作 sidecar 等）は K8s API server を改造せず、CRD + admission webhook の標準機構のみで実装
- 派生ディストリビューション（OpenShift / Rancher RKE2 / Talos）への積極的な対応は行わないが、「動かない」とも宣言しない（ベストエフォート互換）

## 検討した選択肢

### 選択肢 A: vanilla K8s + CNCF Conformance 維持（採用）

- 概要: upstream Kubernetes をそのまま使い、Conformance テストで互換性を継続検証する
- メリット:
  - 業界標準。採用組織が既存 K8s 知識を流用できる
  - kubectl / kustomize / Helm / Argo CD / Backstage 等の周辺エコシステムが当然に動作する
  - upstream の進化（新機能・deprecation）にそのまま追従できる
  - 採用組織が将来 CNCF Conformance 認証を取得する場合の前提が整う
- デメリット:
  - upstream のバージョン追従コスト（四半期パッチ・年次 minor upgrade）を採用組織が負う
  - 独自最適化（特殊な scheduler / network 拡張）を入れたい場合の自由度が低い

### 選択肢 B: OpenShift / Rancher RKE2 系派生

- 概要: Red Hat / SUSE の事業者ディストリビューションをベースにする
- メリット:
  - 事業者商用サポートが取れる（運用責任の一部移譲）
  - インストールが平易、HA / Operator 同梱
- デメリット:
  - 事業者独自 component（OperatorHub / Rancher UI / Project / SCC 等）に部分的にロックイン
  - 商用ライセンス費用（OpenShift）が発生する場合あり、BC-COST-003 と逆行
  - upstream 互換性は保たれるが「OpenShift / RKE2 を使う前提」の運用文書が世間に多く、k1s0 docs と二重化する

### 選択肢 C: Talos Linux / k0s 系特化型

- 概要: K8s 用途に純化された OS / 単一バイナリ K8s
- メリット:
  - 攻撃面が極小（SSH 不在 / immutable）
  - K8s 用途で運用 surface が単純
- デメリット:
  - 採用組織の運用エンジニアが「Talos の machined API」「k0s の独自 CLI」を別途学習する必要
  - upstream K8s 知識の流用性が低下する
  - 既存 RHEL / Ubuntu 標準で運用したい採用組織と整合しない

### 選択肢 D: 独自 K8s 派生（fork / patch）

- 概要: upstream を fork して k1s0 用に最適化したディストリビューションを配布
- メリット: k1s0 専用の最適化が可能
- デメリット:
  - 10 年保守で fork 維持コストが破綻する
  - upstream 進化からの断絶リスク
  - 採用組織にとって「k1s0 専用 K8s」を運用する必要があり、既存 K8s 知識の流用性が皆無
  - 完全な one-way door、撤退困難

## 決定理由

選択肢 A（vanilla K8s 維持）を採用する根拠は以下。

- **採用組織のスキル流用性**: 採用検討組織が既に持つ / これから採用する K8s 運用エンジニアのスキルがそのまま流用できる。Talos（C）の独自 API 学習、RKE2（B）の事業者依存学習はいずれも採用障壁になる
- **エコシステム互換性**: kubectl / kustomize / Helm / Argo CD / Backstage / Kyverno 等、k1s0 が前提とする周辺 OSS はすべて vanilla K8s を前提に設計されている。派生ディストリビューションでは「OpenShift では SCC が必要」「RKE2 では default StorageClass が異なる」等の差分対応が散発的に発生する
- **業界標準への乗り続け権**: CNCF Conformance を維持することで、採用組織が将来「自社製品としての k8s ディストリビューション化」「マネージド K8s への移行」を選択する際の退路が残る。独自 fork（D）はこの権利を完全に放棄する
- **保守コスト**: 10 年保守を前提とすると、独自 fork（D）の維持コストは破綻する。upstream 進化に追従する作業の方が独自 fork 維持より圧倒的に低コスト
- **CRD + Admission Webhook での十分性**: k1s0 が必要とする独自挙動（依存方向強制 / コメント規約 / Backstage catalog 検証 / SoT drift block）は、API server 改造ではなく CRD（CustomResourceDefinition）+ admission webhook（Kyverno、ADR-CICD-003）で実装可能。vanilla 維持と機能要件は両立する

## 帰結

### ポジティブな帰結

- 採用組織の K8s 運用エンジニアが標準スキルで k1s0 を保守できる
- upstream Kubernetes の機能進化（Gateway API GA / Volume Populators / Admission Policies 等）に自動追従できる
- マネージド K8s（EKS / GKE / AKS）への将来移行時にも互換性で動作する見込み
- 採用組織が CNCF Conformance 認証を取得する選択肢を残せる

### ネガティブな帰結 / リスク

- upstream バージョン追従の運用コスト（四半期パッチ、年次 minor upgrade、deprecation 対応）を採用組織が負う必要がある。Runbook 化で軽減（NFR-C-NOP-003）
- 独自最適化したい場面（特殊な scheduler / network 拡張）で自由度が制約される
- 派生ディストリビューション固有機能（OpenShift Routes / RKE2 の組み込み Helm Controller 等）には依存できない

### 移行・対応事項

- CNCF Conformance テスト（sonobuoy）を kind multi-node で定期実行する CI を整備（IMP-CI-* 系）
- upstream Kubernetes のバージョン追従手順を Runbook 化（NFR-C-NOP-003）
- 派生ディストリビューション動作確認（best-effort）の方針を README に明記、商用サポートは別途
- 独自 admission webhook（k1s0 specific）は Kyverno ClusterPolicy として実装し、API server 改造を避ける（ADR-CICD-003）

## 関連

- ADR-INFRA-001（K8s クラスタ ブートストラップ）— vanilla K8s を kubeadm + CAPI で構築
- ADR-NET-001（CNI 選定）— vanilla K8s 上の CNI 選定
- ADR-DEV-001（Paved Road）— 採用組織のスキル流用性思想
- ADR-CICD-003（Kyverno）— admission webhook の標準実装
- IMP-CI-CONF-* — CNCF Conformance テスト CI

## 参考文献

- CNCF Certified Kubernetes Conformance Program: cncf.io/training/certification/software-conformance
- sonobuoy: sonobuoy.io
- Kubernetes Release Cycle: kubernetes.io/releases/
