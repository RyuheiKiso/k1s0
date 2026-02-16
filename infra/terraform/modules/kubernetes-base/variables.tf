variable "namespaces" {
  description = "Map of namespaces with tier and allowed_from_tiers configuration"
  type = map(object({
    tier               = string
    allowed_from_tiers = list(string)
  }))
}

variable "resource_quotas" {
  description = "Per-namespace resource quota overrides"
  type = map(object({
    requests_cpu    = string
    requests_memory = string
    limits_cpu      = string
    limits_memory   = string
    pods            = string
    pvcs            = string
  }))
  default = {}
}

variable "default_resource_quota" {
  description = "Default resource quota for namespaces without explicit overrides"
  type = object({
    requests_cpu    = string
    requests_memory = string
    limits_cpu      = string
    limits_memory   = string
    pods            = string
    pvcs            = string
  })
  default = {
    requests_cpu    = "8"
    requests_memory = "16Gi"
    limits_cpu      = "16"
    limits_memory   = "32Gi"
    pods            = "50"
    pvcs            = "20"
  }
}
