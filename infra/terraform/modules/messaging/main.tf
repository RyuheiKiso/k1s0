resource "helm_release" "strimzi_operator" {
  name       = "strimzi-kafka-operator"
  namespace  = "messaging"
  repository = "https://strimzi.io/charts/"
  chart      = "strimzi-kafka-operator"
  version    = var.strimzi_operator_version

  create_namespace = true

  set {
    name  = "replicas"
    value = "1"
  }
}
