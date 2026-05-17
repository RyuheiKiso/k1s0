#!/usr/bin/env bash
# kind クラスタをブートストラップし、CNI・cert-manager・Kyverno を導入する
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

# 必要ツールの存在確認
check_prerequisites() {
  local missing=0
  for cmd in kind kubectl helm; do
    if ! command -v "${cmd}" &>/dev/null; then
      echo "[ERROR] ${cmd} が見つかりません。インストールしてください。" >&2
      missing=1
    fi
  done
  if [[ "${missing}" -eq 1 ]]; then
    exit 1
  fi
  echo "[OK] 必要ツール（kind / kubectl / helm）を確認しました。"
}

# kind クラスタ作成
create_clusters() {
  echo "[INFO] k1s0-target クラスタを作成します..."
  kind create cluster \
    --name k1s0-target \
    --config "${SCRIPT_DIR}/kind-config.yaml"

  echo "[INFO] k1s0-ops-edge クラスタを作成します..."
  kind create cluster \
    --name k1s0-ops-edge \
    --config "${SCRIPT_DIR}/kind-config-ops-edge.yaml"
}

# Cilium CNI インストール
install_cilium() {
  helm repo add cilium https://helm.cilium.io --force-update

  for ctx in kind-k1s0-target kind-k1s0-ops-edge; do
    echo "[INFO] ${ctx} に Cilium をインストールします..."
    helm install cilium cilium/cilium \
      --version 1.16.0 \
      --namespace kube-system \
      --kube-context "${ctx}" \
      --set kubeProxyReplacement=true \
      --set k8sServiceHost=localhost \
      --set k8sServicePort=6443 \
      --wait \
      --timeout 5m
    echo "[OK] ${ctx} への Cilium インストール完了。"
  done
}

# cert-manager インストール（target クラスタのみ）
install_cert_manager() {
  echo "[INFO] cert-manager をインストールします..."
  kubectl --context kind-k1s0-target apply \
    -f https://github.com/cert-manager/cert-manager/releases/download/v1.16.0/cert-manager.yaml
  kubectl --context kind-k1s0-target wait \
    --for=condition=Ready pod \
    -l app.kubernetes.io/instance=cert-manager \
    -n cert-manager \
    --timeout=120s
  echo "[OK] cert-manager インストール完了。"
}

# Kyverno インストール（target クラスタのみ）
install_kyverno() {
  local install_manifest="${REPO_ROOT}/src/security/kyverno/install/install.yaml"
  if [[ ! -f "${install_manifest}" ]]; then
    echo "[WARN] Kyverno install.yaml が見つかりません: ${install_manifest}" >&2
    echo "[WARN] Kyverno のインストールをスキップします。" >&2
    return
  fi
  echo "[INFO] Kyverno をインストールします..."
  kubectl --context kind-k1s0-target apply -f "${install_manifest}"
  kubectl --context kind-k1s0-target wait \
    --for=condition=Ready pod \
    -l app.kubernetes.io/name=kyverno \
    -n kyverno \
    --timeout=180s
  echo "[OK] Kyverno インストール完了。"
}

main() {
  check_prerequisites
  create_clusters
  install_cilium
  install_cert_manager
  install_kyverno
  echo ""
  echo "========================================="
  echo "[SUCCESS] ブートストラップが完了しました。"
  echo "  クラスタ: k1s0-target / k1s0-ops-edge"
  echo "  CNI: Cilium 1.16.0"
  echo "  cert-manager: v1.16.0"
  echo "  Kyverno: インストール済み（install.yaml 存在時）"
  echo "========================================="
}

main "$@"
