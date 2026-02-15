variable "namespaces" {
  description = "Map of namespaces with tier and allowed_from_tiers configuration"
  type = map(object({
    tier               = string
    allowed_from_tiers = list(string)
  }))
}
