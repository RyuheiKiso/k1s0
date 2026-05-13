---
id: env.ops.platform_operator_oss_install
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.platform_operator_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- OpenBao CLI / ykman / Cosign / kubectl / Grafana CLI の 5 ツールをそれぞれのインストール手順に従い導入し、各コマンドが正常応答することを確認してから次のステップへ進む。

## 1. OpenBao CLI（bao）

OpenBao は HashiCorp Vault の OSS fork。KEK shamir ceremony に使用する。

```bash
OPENBAO_VER="2.0.2"
curl -L "https://github.com/openbao/openbao/releases/download/v${OPENBAO_VER}/bao_${OPENBAO_VER}_linux_amd64.zip" \
  -o /tmp/bao.zip
unzip /tmp/bao.zip -d /tmp/bao-bin/
sudo mv /tmp/bao-bin/bao /usr/local/bin/bao
bao --version
```

## 2. ykman（yubikey-manager）

Yubikey の PIV スロット管理ツール。pip で導入する。

```bash
sudo apt update
sudo apt install -y python3-pip swig libpcsclite-dev pcscd
pip install yubikey-manager
# pcscd（スマートカードデーモン）を起動しないと Yubikey が認識されない
sudo systemctl enable --now pcscd
ykman --version
```

または apt で直接インストールできる場合もある。

```bash
sudo apt install -y yubikey-manager
ykman --version
```

## 3. Cosign

artifact 署名検証ツール。GitHub Releases から binary を取得する。

```bash
curl -LO https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
cosign version
```

## 4. kubectl

production cluster の操作に使用する。

```bash
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/kubectl
kubectl version --client
```

## 5. Grafana CLI

監視ダッシュボードの管理に使用する。

```bash
sudo apt install -y apt-transport-https
sudo mkdir -p /etc/apt/keyrings/
wget -q -O - https://apt.grafana.com/gpg.key | gpg --dearmor | sudo tee /etc/apt/keyrings/grafana.gpg > /dev/null
echo "deb [signed-by=/etc/apt/keyrings/grafana.gpg] https://apt.grafana.com stable main" | sudo tee /etc/apt/sources.list.d/grafana.list
sudo apt update
sudo apt install -y grafana
grafana-cli --version
```

## 6. audit-trail-tool（自製スクリプト）

audit hash chain の管理には `src/ops/audit-trail/` 配下の自製スクリプトを参照する。

```bash
ls src/ops/audit-trail/
# audit-hash-chain.sh / replay-audit.sh が存在することを確認
bash src/ops/audit-trail/replay-audit.sh --dry-run
```

スクリプトが存在しない場合は infra 軸エンジニアに依頼して配置してもらう。

## 検収コマンド

```bash
bao --version
ykman --version
cosign version
kubectl version --client
grafana-cli --version
```

全コマンドが正常応答すれば OSS 導入完了。

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
