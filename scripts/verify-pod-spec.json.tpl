{
  "spec": {
    "serviceAccountName": "backup-verify-sa",
    "containers": [{
      "name": "${POD_NAME}",
      "image": "${IMAGE}",
      "command": ["/bin/sh", "-c", "${COMMAND}"],
      "volumeMounts": [{
        "name": "backup-volume",
        "mountPath": "/backup",
        "readOnly": true
      }]
    }],
    "volumes": [{
      "name": "backup-volume",
      "persistentVolumeClaim": {
        "claimName": "${BACKUP_PVC}",
        "readOnly": true
      }
    }]
  }
}
