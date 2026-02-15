resource "kubernetes_storage_class" "ceph_block" {
  metadata {
    name = "ceph-block"
    annotations = {
      "storageclass.kubernetes.io/is-default-class" = "true"
    }
  }

  storage_provisioner    = "rbd.csi.ceph.com"
  reclaim_policy         = var.reclaim_policy   # dev: Delete, prod: Retain
  allow_volume_expansion = true

  parameters = {
    clusterID = var.ceph_cluster_id
    pool      = var.ceph_pool
  }
}

resource "kubernetes_storage_class" "ceph_filesystem" {
  metadata {
    name = "ceph-filesystem"
  }

  storage_provisioner    = "cephfs.csi.ceph.com"
  reclaim_policy         = var.reclaim_policy
  allow_volume_expansion = true

  parameters = {
    clusterID = var.ceph_cluster_id
    fsName    = var.ceph_filesystem_name
  }
}

resource "kubernetes_storage_class" "ceph_block_fast" {
  metadata {
    name = "ceph-block-fast"
  }

  storage_provisioner    = "rbd.csi.ceph.com"
  reclaim_policy         = var.reclaim_policy
  allow_volume_expansion = true

  parameters = {
    clusterID = var.ceph_cluster_id
    pool      = var.ceph_pool_fast   # SSD-backed pool
  }
}
