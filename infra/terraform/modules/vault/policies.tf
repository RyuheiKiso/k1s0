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

# ============================================================
# H-010 監査対応: system ティア サービス個別ポリシー
# 最小権限原則に従い、各サービスが自サービスのシークレットにのみアクセスできるよう
# 個別の HCL ポリシーを定義する。
# "system" 共通ポリシーへの依存を排除し、爆発半径を最小化する。
# ============================================================

# auth サービス専用ポリシー（JWT 署名、Transit、auth DB のみ）
resource "vault_policy" "auth" {
  name   = "auth"
  policy = file("${path.module}/policies/auth.hcl")
}

# session サービス専用ポリシー（session DB、Redis のみ）
resource "vault_policy" "session" {
  name   = "session"
  policy = file("${path.module}/policies/session.hcl")
}

# tenant サービス専用ポリシー（tenant DB、Kafka のみ）
resource "vault_policy" "tenant" {
  name   = "tenant"
  policy = file("${path.module}/policies/tenant.hcl")
}

# featureflag サービス専用ポリシー（featureflag DB のみ）
resource "vault_policy" "featureflag" {
  name   = "featureflag"
  policy = file("${path.module}/policies/featureflag.hcl")
}

# ratelimit サービス専用ポリシー（ratelimit DB、Redis のみ）
resource "vault_policy" "ratelimit" {
  name   = "ratelimit"
  policy = file("${path.module}/policies/ratelimit.hcl")
}

# rule-engine サービス専用ポリシー（rule-engine DB、Kafka のみ）
resource "vault_policy" "rule_engine" {
  name   = "rule-engine"
  policy = file("${path.module}/policies/rule-engine.hcl")
}

# policy サービス専用ポリシー（policy DB のみ）
resource "vault_policy" "policy_svc" {
  name   = "policy"
  policy = file("${path.module}/policies/policy.hcl")
}

# workflow サービス専用ポリシー（workflow DB、Kafka のみ）
resource "vault_policy" "workflow" {
  name   = "workflow"
  policy = file("${path.module}/policies/workflow.hcl")
}

# scheduler サービス専用ポリシー（scheduler DB、Kafka のみ）
resource "vault_policy" "scheduler" {
  name   = "scheduler"
  policy = file("${path.module}/policies/scheduler.hcl")
}

# quota サービス専用ポリシー（quota DB、Redis のみ）
resource "vault_policy" "quota" {
  name   = "quota"
  policy = file("${path.module}/policies/quota.hcl")
}

# notification サービス専用ポリシー（notification DB、Kafka、SMTP/FCM シークレットのみ）
resource "vault_policy" "notification" {
  name   = "notification"
  policy = file("${path.module}/policies/notification.hcl")
}

# file サービス専用ポリシー（file DB、S3 クレデンシャルのみ）
resource "vault_policy" "file_svc" {
  name   = "file"
  policy = file("${path.module}/policies/file.hcl")
}

# service-catalog サービス専用ポリシー（service-catalog DB のみ）
resource "vault_policy" "service_catalog" {
  name   = "service-catalog"
  policy = file("${path.module}/policies/service-catalog.hcl")
}

# event-monitor サービス専用ポリシー（event-monitor DB、Kafka のみ）
resource "vault_policy" "event_monitor" {
  name   = "event-monitor"
  policy = file("${path.module}/policies/event-monitor.hcl")
}

# api-registry サービス専用ポリシー（api-registry DB のみ）
resource "vault_policy" "api_registry" {
  name   = "api-registry"
  policy = file("${path.module}/policies/api-registry.hcl")
}

# app-registry サービス専用ポリシー（app-registry DB のみ）
resource "vault_policy" "app_registry" {
  name   = "app-registry"
  policy = file("${path.module}/policies/app-registry.hcl")
}

# graphql-gateway サービス専用ポリシー（ゲートウェイ設定、PKI のみ）
resource "vault_policy" "graphql_gateway" {
  name   = "graphql-gateway"
  policy = file("${path.module}/policies/graphql-gateway.hcl")
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
