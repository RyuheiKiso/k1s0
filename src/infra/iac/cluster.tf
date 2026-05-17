# k1s0 kind クラスタの宣言的プロビジョニング設定
# tehcyx/kind provider を使って k1s0-target と k1s0-ops-edge を定義する

# ============================================================
# k1s0-target クラスタ: メインワークロードクラスタ
# ============================================================
# k1s0-target クラスタリソースを定義する（kind provider を使用する）
resource "kind_cluster" "target" {
  # クラスタ名（kind コンテナ名のプレフィックスになる）
  name = var.target_cluster_name

  # kind クラスタの詳細設定（YAML 形式でインライン定義する）
  kind_config {
    # kind 設定の apiVersion を指定する
    api_version = "kind.x-k8s.io/v1alpha4"
    # リソース種別は Cluster を指定する
    kind = "Cluster"

    # コントロールプレーンノードを 1 台定義する
    node {
      # コントロールプレーンのロールを指定する
      role = "control-plane"
      # kubeadm 設定パッチ: NodePort のポート範囲を拡張する
      kubeadm_config_patches = [
        <<-YAML
          kind: InitConfiguration
          nodeRegistration:
            kubeletExtraArgs:
              node-labels: "ingress-ready=true"
        YAML
      ]
      # コントロールプレーンのポートマッピングを定義する
      extra_port_mappings {
        # HTTP トラフィック用ポートを 80 に割り当てる
        container_port = 80
        host_port      = 8080
        listen_address = "0.0.0.0"
        protocol       = "TCP"
      }
      extra_port_mappings {
        # HTTPS トラフィック用ポートを 443 に割り当てる
        container_port = 443
        host_port      = 8443
        listen_address = "0.0.0.0"
        protocol       = "TCP"
      }
    }

    # ワーカーノード 1（zone-a に配置する）
    node {
      # ワーカーのロールを指定する
      role = "worker"
      # kubelet の追加ラベルでゾーン情報を付与する
      kubeadm_config_patches = [
        <<-YAML
          kind: JoinConfiguration
          nodeRegistration:
            kubeletExtraArgs:
              node-labels: "topology.kubernetes.io/zone=zone-a,k1s0.io/workload-class=tier1"
        YAML
      ]
    }

    # ワーカーノード 2（zone-b に配置する）
    node {
      # ワーカーのロールを指定する
      role = "worker"
      kubeadm_config_patches = [
        <<-YAML
          kind: JoinConfiguration
          nodeRegistration:
            kubeletExtraArgs:
              node-labels: "topology.kubernetes.io/zone=zone-b,k1s0.io/workload-class=tier2"
        YAML
      ]
    }

    # ワーカーノード 3（zone-c に配置する）
    node {
      # ワーカーのロールを指定する
      role = "worker"
      kubeadm_config_patches = [
        <<-YAML
          kind: JoinConfiguration
          nodeRegistration:
            kubeletExtraArgs:
              node-labels: "topology.kubernetes.io/zone=zone-c,k1s0.io/workload-class=infra"
        YAML
      ]
    }
  }

  # クラスタ削除時に kubeconfig エントリを自動削除するか
  wait_for_ready = true
}

# ============================================================
# k1s0-ops-edge クラスタ: 運用・監視用クラスタ
# ============================================================
# k1s0-ops-edge クラスタリソースを定義する（軽量構成）
resource "kind_cluster" "ops_edge" {
  # ops-edge クラスタ名を指定する
  name = var.ops_cluster_name

  # ops-edge の kind クラスタ設定（コントロールプレーン + ワーカー 1 台）
  kind_config {
    # kind 設定の apiVersion を指定する
    api_version = "kind.x-k8s.io/v1alpha4"
    kind = "Cluster"

    # コントロールプレーンノードを定義する
    node {
      role = "control-plane"
    }

    # ops ワーカーノード（Argo CD / Grafana を実行する）
    node {
      role = "worker"
      kubeadm_config_patches = [
        <<-YAML
          kind: JoinConfiguration
          nodeRegistration:
            kubeletExtraArgs:
              node-labels: "topology.kubernetes.io/zone=zone-a,k1s0.io/workload-class=ops"
        YAML
      ]
    }
  }

  # クラスタが Ready になるまで待機する
  wait_for_ready = true
}

# ============================================================
# Namespace プロビジョニング: k1s0-target に必要な Namespace を作成する
# ============================================================
# Kubernetes provider を使って kyverno Namespace を作成する
resource "kubernetes_namespace" "kyverno" {
  metadata {
    # Namespace の名前を定義する
    name = "kyverno"
    # k1s0 管理ラベルを付与する
    labels = merge(local.common_labels, {
      "app.kubernetes.io/managed-by" = "opentofu"
      "k1s0.io/component"            = "policy-engine"
    })
  }
  # k1s0-target クラスタが起動してから Namespace を作成する
  depends_on = [kind_cluster.target]
}

# CloudNativePG operator 用 Namespace を作成する
resource "kubernetes_namespace" "cnpg_system" {
  metadata {
    name = "cnpg-system"
    labels = merge(local.common_labels, {
      "k1s0.io/component" = "database-operator"
    })
  }
  depends_on = [kind_cluster.target]
}

# Strimzi Kafka operator 用 Namespace を作成する
resource "kubernetes_namespace" "kafka_system" {
  metadata {
    name = "kafka-system"
    labels = merge(local.common_labels, {
      "k1s0.io/component" = "messaging-operator"
    })
  }
  depends_on = [kind_cluster.target]
}

# OpenBao シークレット管理用 Namespace を作成する
resource "kubernetes_namespace" "openbao" {
  metadata {
    name = "openbao"
    labels = merge(local.common_labels, {
      "k1s0.io/component" = "secret-management"
    })
  }
  depends_on = [kind_cluster.target]
}

# Harbor コンテナレジストリ用 Namespace を作成する
resource "kubernetes_namespace" "harbor" {
  metadata {
    name = "harbor"
    labels = merge(local.common_labels, {
      "k1s0.io/component" = "container-registry"
    })
  }
  depends_on = [kind_cluster.target]
}

# MetalLB LoadBalancer 用 Namespace を作成する
resource "kubernetes_namespace" "metallb_system" {
  metadata {
    name = "metallb-system"
    labels = merge(local.common_labels, {
      "k1s0.io/component" = "load-balancer"
    })
  }
  depends_on = [kind_cluster.target]
}

# cert-manager TLS 証明書管理用 Namespace を作成する
resource "kubernetes_namespace" "cert_manager" {
  metadata {
    name = "cert-manager"
    labels = merge(local.common_labels, {
      "k1s0.io/component" = "tls-management"
    })
  }
  depends_on = [kind_cluster.target]
}

# ============================================================
# 出力値: cluster モジュールが main.tf に返す値を定義する
# ============================================================
# k1s0-target クラスタの Kubernetes API エンドポイントを出力する
output "target_cluster_endpoint" {
  description = "k1s0-target クラスタの API サーバーエンドポイント"
  value       = kind_cluster.target.endpoint
}

# k1s0-ops-edge クラスタの Kubernetes API エンドポイントを出力する
output "ops_cluster_endpoint" {
  description = "k1s0-ops-edge クラスタの API サーバーエンドポイント"
  value       = kind_cluster.ops_edge.endpoint
}

# k1s0-target クラスタの kubeconfig を出力する（sensitive で保護する）
output "target_kubeconfig" {
  description = "k1s0-target クラスタの kubeconfig（機密情報）"
  value       = kind_cluster.target.kubeconfig
  sensitive   = true
}
