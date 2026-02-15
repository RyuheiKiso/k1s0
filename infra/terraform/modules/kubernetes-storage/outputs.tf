output "storage_class_names" {
  description = "Map of StorageClass names by type"
  value = {
    block      = kubernetes_storage_class.ceph_block.metadata[0].name
    filesystem = kubernetes_storage_class.ceph_filesystem.metadata[0].name
    block_fast = kubernetes_storage_class.ceph_block_fast.metadata[0].name
  }
}

output "pv_names" {
  description = "Map of PersistentVolume provisioner names by StorageClass"
  value = {
    block      = kubernetes_storage_class.ceph_block.storage_provisioner
    filesystem = kubernetes_storage_class.ceph_filesystem.storage_provisioner
    block_fast = kubernetes_storage_class.ceph_block_fast.storage_provisioner
  }
}
