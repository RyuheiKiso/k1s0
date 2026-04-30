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
