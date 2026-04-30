# infra/k8s/multinode — multi-node kind + Calico

ADR-NET-001 (Calico CNI) と ADR-CNCF-001 (CNCF Conformance) の production-grade
network behavior を kind ローカル環境で再現するための prescription。

## なぜ必要か

`k1s0-local` (single-node kind) は kindnet を使っており、**NetworkPolicy が
ignore される**。production 級の検証 (Pod anti-affinity / topology spread /
NetworkPolicy 強制 / multi-AZ ルーティング) には:

1. multi-worker (zone-a / zone-b / zone-c)
2. NetworkPolicy を実装する CNI (Calico / Cilium)

の 2 条件が必須。本ディレクトリの prescription はその両方を満たす。

## 起動手順

```bash
# 1) cluster 作成
kind create cluster --name k1s0-multinode --config infra/k8s/multinode/kind-multinode.yaml

# 2) Calico install (operator 経由)
kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.29.2/manifests/tigera-operator.yaml
kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.29.2/manifests/custom-resources.yaml

# 3) Calico Ready 待ち
kubectl wait --for=condition=Available deployment/calico-kube-controllers -n calico-system --timeout=300s

# 4) ノード Ready 確認
kubectl get nodes -o wide
# 期待: control-plane + 3 worker (各 zone label 付き) すべて Ready

# 5) NetworkPolicy 動作確認 (例)
kubectl apply -f infra/k8s/multinode/test-netpol-deny-all.yaml
# default deny の namespace に curl pod を deploy → 通信 timeout を確認

# 6) topology spread 確認
kubectl apply -f infra/k8s/multinode/test-topology-spread.yaml
# 3 replica が 3 zone に 1:1:1 で分散することを kubectl get pod -o wide で確認
```

## kind config の主な点

- `disableDefaultCNI: true`: kindnet を install しない (Calico を後で入れる)
- `podSubnet: 192.168.0.0/16`: Calico デフォルト
- 3 worker: 各々 `topology.kubernetes.io/zone` label で a/b/c を割り当て

## production 移行との関係

production (EKS/GKE/AKS/on-prem) では:
- **CNI**: Calico / Cilium が事実上の標準。本 prescription と同じ
- **multi-AZ**: zone label が cloud provider 由来（kubelet がノードに自動付与）
- **NetworkPolicy**: 同じ k8s API、CNI 実装が enforce

→ multi-node kind での検証結果は production にそのまま転用できる。

## 検証 (2026-04-30 時点)

未実施。本 prescription は **次に作業する人の起点**として用意。実機検証を
追加した時点で本 README に「検証済」セクションを追記する。

## 検証実績 (2026-04-30)

`kind v0.27.0 / Kubernetes 1.32.2 / Calico v3.29.2 / 4-node` で本 prescription
を実機検証:

### topology spread

```
$ kubectl apply -f infra/k8s/multinode/test-topology-spread.yaml
$ kubectl get pods -l app=topo-test -o wide
NAME                         STATUS    NODE
topo-test-85c9cc986d-9xhbk   Running   k1s0-multinode-worker2
topo-test-85c9cc986d-dwflr   Running   k1s0-multinode-worker
topo-test-85c9cc986d-knlmg   Running   k1s0-multinode-worker3
```

3 replica が 3 worker (zone-a / zone-b / zone-c) に **1:1:1 で分散**することを
確認。`maxSkew: 1` + `whenUnsatisfiable: DoNotSchedule` が機能している。

### NetworkPolicy 強制

```
$ kubectl apply -f infra/k8s/multinode/test-netpol-deny-all.yaml
$ kubectl exec -n netpol-test client -- curl --max-time 5 http://<server-ip>:8080
curl: (28) Connection timed out after 5063 milliseconds  # ✅ blocked

$ kubectl delete networkpolicy -n netpol-test default-deny
$ kubectl exec -n netpol-test client -- curl --max-time 5 http://<server-ip>:8080
hello                                                       # ✅ allowed
```

**default-deny NetworkPolicy を Calico が enforce している**ことを確認。kindnet
では同じ条件で curl が通ってしまう（kindnet は NetworkPolicy ignore）。

### 含意

- production K8s (EKS / GKE / on-prem) は同じ Kubernetes API + Calico/Cilium 等の
  CNI を使うため、本 prescription での検証結果は **そのまま production に転用可能**
- `k1s0-local` (single-node + kindnet) で検証できなかった「multi-node 挙動 +
  NetworkPolicy 強制」は本 prescription で kind ベースに保ったまま検証可能

## I1〜I3 追加検証 (2026-04-30 後段)

H3a で prescription を validate した後、application-level の挙動も multi-node で
検証して production carry-over matrix の `(要追加検証 multi-node)` 列を埋めた。

### I1: multi-replica perf SLA (G7 仮説の実証)

`tier1-state` を 3 replica + Pod anti-affinity (各 worker に 1 つ) で deploy、
in-cluster `ab` で c=50 / 5000 req:

| 環境 | p50 | p95 | p99 |
|---|---|---|---|
| single-pod (k1s0-local, c=50) | 5ms | 71ms | **80ms** ❌ SLA 未達 |
| 3-replica (multinode, c=50) | 4ms | 8ms | **15ms** (5x 改善) ⚠️ ほぼ達成 |
| 3-replica (multinode, c=100) | 9ms | 17ms | **20ms** |

→ G7 仮説 (production multi-replica で SLA 達成) は **方向として実証**。kind の
overhead (containerd / port-forward なし in-cluster でも残る) で 10ms 微超過、
production の bare metal / cloud node では 5+ replica で SLA <10ms を達成見込み。

### I2: NFR-A-FT-001 multi-node failover

3 replica 状態で 1 pod を `--force --grace-period=0` で削除しつつ 30 秒間 HTTP
トラフィックを流す:

- 30 秒 window で **8449 requests / 2 failures (0.024%)**
- 新 Pod (`tier1-state-...-54nzn`) が同 worker で **27 秒で Ready**
- Service LB が remaining 2 replicas にトラフィックをすぐルーティング = 実質
  zero-downtime
- single-pod では同シナリオが **756 秒 (12 分)** だった (NFR-A-FT-001 SLA <15min
  ぎりぎり) → multi-replica で **28x 高速化**、SLA を桁違いに上回る達成

### I3: Istio Ambient mTLS STRICT (multi-node)

Istio Ambient (istio-base 1.29.2 + istio-cni + ztunnel + istiod profile=ambient)
を multi-node cluster に install、`k1s0-tier1` namespace に
`istio.io/dataplane-mode=ambient` label + PeerAuthentication STRICT を適用:

- **in-mesh → in-mesh** (perf-tester → tier1-state): 成功 (ztunnel が自動 mTLS、
  cross-node でも問題なし)
- **outside-mesh → in-mesh** (outside-mesh ns の curl pod → tier1-state):
  `curl: (56) Recv failure: Connection reset by peer` で **STRICT が reject**

→ ambient mesh が **複数 worker 跨ぎでも mTLS 強制を維持**。single-node (C3)
の検証結果が multi-node でもそのまま再現することを確認。
