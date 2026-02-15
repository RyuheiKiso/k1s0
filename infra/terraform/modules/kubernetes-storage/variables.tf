variable "ceph_cluster_id" {
  description = "Ceph cluster ID"
  type        = string
}

variable "ceph_pool" {
  description = "Ceph RBD pool name for block storage"
  type        = string
}

variable "ceph_pool_fast" {
  description = "Ceph RBD SSD-backed pool name for fast block storage"
  type        = string
}

variable "ceph_filesystem_name" {
  description = "CephFS filesystem name for shared file storage"
  type        = string
}

variable "reclaim_policy" {
  description = "StorageClass reclaim policy (Delete or Retain)"
  type        = string
  default     = "Delete"

  validation {
    condition     = contains(["Delete", "Retain"], var.reclaim_policy)
    error_message = "reclaim_policy must be either 'Delete' or 'Retain'."
  }
}
