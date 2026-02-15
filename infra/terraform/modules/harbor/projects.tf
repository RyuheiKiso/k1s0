resource "harbor_project" "k1s0" {
  for_each = toset(["k1s0-system", "k1s0-business", "k1s0-service", "k1s0-infra"])

  name   = each.key
  public = false

  depends_on = [helm_release.harbor]
}

resource "harbor_robot_account" "ci_push" {
  for_each = harbor_project.k1s0

  name        = "ci-push-${each.key}"
  description = "CI/CD push account for ${each.key}"
  level       = "project"

  permissions {
    access {
      action   = "push"
      resource = "repository"
    }
    access {
      action   = "pull"
      resource = "repository"
    }
    kind      = "project"
    namespace = each.value.name
  }

  depends_on = [harbor_project.k1s0]
}
