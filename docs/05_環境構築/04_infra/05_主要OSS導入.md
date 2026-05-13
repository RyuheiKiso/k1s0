---
id: env.infra.infra_oss_install
axis: infra
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.infra.infra_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- Argo CD CLI / istioctl / calicoctl / OpenBao CLI / Cosign / Litmus CLI の 6 ツールを導入し、全て `--version` または `--help` が応答することを確認してから kind cluster 操作に進む。

## Argo CD CLI のインストール

```bash
curl -sSL -o argocd-linux-amd64 https://github.com/argoproj/argo-cd/releases/latest/download/argocd-linux-amd64
chmod +x argocd-linux-amd64
sudo mv argocd-linux-amd64 /usr/local/bin/argocd
argocd version --client
```

## Istio CLI (istioctl) のインストール

```bash
curl -L https://istio.io/downloadIstio | sh -
# ダウンロードされたディレクトリを確認して PATH に追加
export PATH="$PATH:$HOME/istio-<version>/bin"
istioctl version
```

永続化するには `~/.bashrc` または `~/.profile` に `export PATH` を追記する。

## Calico CLI (calicoctl) のインストール

```bash
curl -L https://github.com/projectcalico/calico/releases/latest/download/calicoctl-linux-amd64 -o calicoctl
chmod +x calicoctl
sudo mv calicoctl /usr/local/bin/calicoctl
calicoctl version
```

## OpenBao CLI のインストール

```bash
curl -LO https://github.com/openbao/openbao/releases/latest/download/bao_linux_amd64
chmod +x bao_linux_amd64
sudo mv bao_linux_amd64 /usr/local/bin/bao
bao --version
```

## Cosign のインストール

```bash
curl -LO https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
cosign version
```

Cosign は infra 軸では署名済みイメージの検証（Kyverno と連携）に使用する。鍵管理は security 軸 / k1s0 作者の責務。

## Litmus CLI のインストール

```bash
curl -LO https://github.com/litmuschaos/litmus/releases/latest/download/litmusctl-linux-amd64
chmod +x litmusctl-linux-amd64
sudo mv litmusctl-linux-amd64 /usr/local/bin/litmusctl
litmusctl version
```

Litmus は chaos engineering ツールであり、kind cluster 上で network fault / pod delete などの chaos experiment を実行するために使用する。

## 検収コマンド

```bash
argocd version --client
istioctl version --remote=false 2>/dev/null || istioctl version
calicoctl version
bao --version
cosign version
litmusctl version
```

6 コマンド全て応答することを確認する。

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
