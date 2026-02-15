terraform {
  backend "consul" {
    address = "consul.internal.example.com:8500"
    scheme  = "https"
    path    = "terraform/k1s0/dev"
    lock    = true
  }
}
