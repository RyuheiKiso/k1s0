# Vault Module - Policy Configuration
# Loads tier-based access policies from HCL files.

# system Tier policy - access to secret/data/k1s0/system/*, database/creds/system-*, pki/issue/system
resource "vault_policy" "system" {
  name   = "system"
  policy = file("${path.module}/policies/system.hcl")
}

# business Tier policy - access to secret/data/k1s0/business/*, database/creds/business-*, kafka SASL
resource "vault_policy" "business" {
  name   = "business"
  policy = file("${path.module}/policies/business.hcl")
}

# service Tier policy - access to secret/data/k1s0/service/*, database/creds/service-*, kafka SASL
resource "vault_policy" "service" {
  name   = "service"
  policy = file("${path.module}/policies/service.hcl")
}

# ドメイン分離ポリシー（I-5 対応）
# business/service の tier レベルポリシーはそのまま維持し、ドメイン単位で細粒度のアクセス制御を追加する

# business-project-master ドメインポリシー - project-master ドメイン専用シークレットへのアクセス
resource "vault_policy" "business_project_master" {
  name   = "business-project-master"
  policy = file("${path.module}/policies/business-project-master.hcl")
}

# service-task ドメインポリシー - task ドメイン専用シークレットへのアクセス
resource "vault_policy" "service_task" {
  name   = "service-task"
  policy = file("${path.module}/policies/service-task.hcl")
}

# service-board ドメインポリシー - board ドメイン専用シークレットへのアクセス
resource "vault_policy" "service_board" {
  name   = "service-board"
  policy = file("${path.module}/policies/service-board.hcl")
}

# service-activity ドメインポリシー - activity ドメイン専用シークレットへのアクセス
resource "vault_policy" "service_activity" {
  name   = "service-activity"
  policy = file("${path.module}/policies/service-activity.hcl")
}

# CI/CD pipeline policy - limited access for AppRole auth
# セキュリティ: CI/CDパイプラインはデプロイに必要な cicd 配下のシークレットのみアクセス可能
resource "vault_policy" "cicd" {
  name = "cicd"
  policy = <<-EOT
    # CI/CD pipeline policy
    # CI/CDパイプラインに必要なシークレットのみに限定（最小権限の原則）

    # CI/CD用シークレットへの読み取りアクセス
    path "secret/data/k1s0/cicd/*" {
      capabilities = ["read", "list"]
    }

    # CI/CD用シークレットのメタデータ参照
    path "secret/metadata/k1s0/cicd/*" {
      capabilities = ["read", "list"]
    }

    # デプロイ時の証明書発行（CI/CDパイプライン用のロールに限定）
    path "pki/issue/cicd" {
      capabilities = ["create", "update"]
    }
  EOT
}
