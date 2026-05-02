# ADR-NET-001: production CNI に Cilium、kind 検証用に Calico を使い分ける

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: ネットワーク基盤チーム / セキュリティチーム / 採用検討組織

## コンテキスト

k1s0 は tier1〜tier3 を 17 namespace に分離し、tenant 越境防止（NFR-E-AC-003）と tier 間境界の物理強制を NetworkPolicy で担保する設計（DS-NFR-* / IMP-DIR-INFRA-*）になっている。SHIP_STATUS H3a で実機検証した通り、kind の既定 CNI（kindnet）は **NetworkPolicy を ignore する**ため、default-deny 設定でも全通信が通ってしまい、設計上の境界が成立しない。production では NetworkPolicy enforcement が必須要件となる。

NetworkPolicy enforcement に加えて、k1s0 の運用には以下が要請される。

- **Istio Ambient mesh（ADR-0001）と二層防御**: Cilium L3/L4 NetworkPolicy + Istio Ambient L7 Authorization
- **Service Mesh / Ingress との互換性**（Envoy Gateway / Istio Ambient と協調動作）
- **マルチテナント観測性**（L4 / L7 トラフィックを Hubble 等で可視化）
- **kube-proxy 退化への追従**（業界は eBPF dataplane へ移行中、iptables / ipvs に縛られない選択肢を確保）
- **kind 環境での CNI 互換性**（H3a で Calico が kind で動作確認済、Cilium は kind で環境調整が複雑）

CNI の選択は data plane アーキテクチャを決める **one-way door** 寄りの判断で、後から切り替えると Pod IP / Service IP / NetworkPolicy / Ingress / observability すべてに波及する。リリース時点で確定し、採用組織の世代交代後も保守できる構造を残す。

## 決定

**production CNI には Cilium（CNCF Graduated、eBPF dataplane）を採用する。kind multi-node 検証用には Calico（CNCF Graduated）を併存させ、環境別に CNI を切り替える。**

- **production**: Cilium 1.16+、eBPF dataplane、kube-proxy replacement 有効、L7 NetworkPolicy（CiliumNetworkPolicy）+ Hubble UI 同梱
- **kind multi-node**（`infra/k8s/multinode/`）: Calico v3.29+、kindnet 無効化、NetworkPolicy enforcement 確認済（H3a）
- **kind single-node**（`tools/local-stack/kind-cluster.yaml`）: kindnet 既定（NetworkPolicy 検証は multi-node で実施）

ローカル単一ノード dev では NetworkPolicy enforcement の実機確認は不要（業務試運用なら multi-node + Calico、本番投入前には Cilium で再検証）。

## 検討した選択肢

### 選択肢 A: production = Cilium / kind = Calico（採用）

- 概要: 環境別に最適な CNI を使い分ける
- メリット:
  - production で eBPF dataplane の性能・観測性を享受（Hubble L4/L7 可視化）
  - kind では Calico の軽量性 + NetworkPolicy enforcement を享受（H3a 検証実績あり）
  - Cilium / Calico はいずれも CNCF Graduated で長期保守の信頼性が高い
- デメリット:
  - 環境間で CNI が異なるため「dev で動いたが prod で動かない」リスク（CiliumNetworkPolicy 固有 features を使うと顕在化）
  - 2 系統の CNI を理解する運用コスト
  - NetworkPolicy 標準 API 範囲内に記述を限定する規律が必要

### 選択肢 B: 全環境 Cilium 統一

- 概要: kind / production すべて Cilium で統一
- メリット:
  - 環境差分が消え、dev / prod で同一挙動
  - CiliumNetworkPolicy / ClusterMesh / Hubble 等の Cilium 固有機能を全環境で使える
- デメリット:
  - kind 環境での Cilium 起動には sysctl / cgroup / hostNetwork の調整が必要で、開発者の手元環境で頻繁にハマる
  - kind multi-node + Cilium の起動時間が Calico より大幅に長く、CI E2E の効率が低下
  - Cilium が要求する eBPF kernel 機能が dev 環境のカーネルバージョンと乖離するケースあり

### 選択肢 C: 全環境 Calico 統一

- 概要: kind / production すべて Calico で統一
- メリット:
  - 環境差分が消え、軽量で起動が速い
  - eBPF dataplane モード（VXLAN / IPIP / eBPF）を選択可能
- デメリット:
  - production の高負荷時に Cilium の eBPF 完全 dataplane（kube-proxy replacement）に比べて性能差
  - L7 observability（Hubble に相当する組込み L7 可視化）が Calico 単体では弱い
  - Service Mesh（Istio Ambient）との統合で Cilium ほどの相乗効果が出にくい

### 選択肢 D: Weave Net

- 概要: Weave Net（旧 Weaveworks 製）
- メリット: 過去に広く使われた、シンプル
- デメリット:
  - **Weaveworks 解散（2024）後、メンテナンスが事実上停止**
  - 新規採用は 10 年保守の観点で不可

### 選択肢 E: Flannel

- 概要: 最古典の K8s CNI
- メリット: 単純、軽量
- デメリット:
  - **NetworkPolicy enforcement なし**（Flannel 単体では別 CNI と組み合わせる必要）
  - Service Mesh / observability の統合が薄い
  - eBPF dataplane 非対応

## 決定理由

選択肢 A（環境別使い分け）を採用する根拠は以下。

- **production の物理制約への適合**: Cilium の eBPF dataplane は kube-proxy replacement / L7 NetworkPolicy / Hubble 観測性を一体提供し、tier1 の高 RPS（NFR-B-PERF-002 で 150 RPS 持続、production multi-replica で更に拡大）と Service Mesh（Istio Ambient）との二層防御に最も適合する
- **kind での実機検証実績**: SHIP_STATUS H3a で kind multi-node + Calico v3.29.2 で NetworkPolicy default-deny enforcement が実機確認済。Cilium を kind で動作させるには sysctl / cgroup の追加調整が必要で、dev 環境の起動安定性を損ねる
- **業界の dataplane 進化への追従**: eBPF dataplane への業界移行（kube-proxy iptables 退化）を前提にすると、production は Cilium 一択。Calico も eBPF モードを持つが、Cilium ほど成熟していない
- **CNCF Graduated の信頼性**: Cilium / Calico はいずれも CNCF Graduated で 10 年保守の前提で安心して採用可能。Weave（D、メンテ停止）/ Flannel（E、NetworkPolicy 非対応）は除外
- **環境差分のリスク管理**: dev / prod で CNI が異なる弊害は「NetworkPolicy 標準 API のみ使う」「Cilium 固有 features（CiliumNetworkPolicy / ClusterMesh）を docs で明示分離する」運用規律で軽減できる。`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/` 系の設計書で対応分離を明文化する

## 帰結

### ポジティブな帰結

- production で eBPF dataplane の性能・観測性を享受、Service Mesh（Istio Ambient）と二層防御
- kind で NetworkPolicy enforcement を実機検証可能、設計上の境界が dev / prod で整合
- Cilium / Calico いずれも CNCF Graduated で長期保守可能
- Hubble UI で L4 / L7 トラフィックを LGTM 観測スタック（ADR-OBS-001）に補完

### ネガティブな帰結 / リスク

- 環境間 CNI 差分のリスク。「dev で動いたが prod で動かない」を avoid するため NetworkPolicy 標準 API 範囲内に記述を限定する規律が必要
- 2 系統の CNI 運用知識が要る（採用組織の学習コスト）
- Cilium 固有 features（CiliumNetworkPolicy / ClusterMesh / L7 policy）を活用する場合、kind では検証できないため staging cluster での実機検証が必須

### 移行・対応事項

- production cluster の CNI install を `infra/k8s/networking/cilium-values.yaml` で管理（ADR-INFRA-001 の bootstrap と整合）
- kind multi-node の Calico 構成を `infra/k8s/multinode/` で固定（H3a で実機検証済、再現可能）
- NetworkPolicy 記述の規約として「標準 NetworkPolicy API のみ使い、CiliumNetworkPolicy 固有 features は別ファイルに分離」を `docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/` で明文化
- Hubble UI の LGTM スタック統合方針を `infra/observability/` で別途設計
- kind single-node では NetworkPolicy 検証ができない旨を `tools/local-stack/README.md` に明記

## 関連

- ADR-INFRA-001（K8s クラスタ）— bootstrap 後の CNI 適用
- ADR-CNCF-001（CNCF Conformance）— vanilla K8s 上の CNI 選定
- ADR-0001（Istio Ambient）— Service Mesh との二層防御
- ADR-CICD-003（Kyverno）— NetworkPolicy 適用の admission 検証
- IMP-DIR-INFRA-* — infra/k8s/networking/ 配置
- ADR-TEST-002（E2E 自動化）— L4 standard E2E は kind multi-node + Calico
- ADR-TEST-003（CNCF Conformance）— L5 conformance も kind multi-node + Calico

## 参考文献

- Cilium 公式: cilium.io
- Calico 公式: tigera.io / projectcalico.docs.tigera.io
- CNCF Project Maturity Levels
- SHIP_STATUS.md §H3a（kind multi-node + Calico 検証実績）
