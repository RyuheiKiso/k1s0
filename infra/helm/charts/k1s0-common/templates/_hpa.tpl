{{- define "k1s0-common.hpa" -}}
{{/* autoscaling オブジェクトと enabled フラグの両方が存在する場合のみ HPA リソースを生成する（nil-safe: C-1 対応） */}}
{{- if and .Values.autoscaling .Values.autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "k1s0-common.fullname" . }}
  minReplicas: {{ .Values.autoscaling.minReplicas }}
  maxReplicas: {{ .Values.autoscaling.maxReplicas }}
  metrics:
    {{- /* HIGH-007 監査対応: hasKey で存在確認してから出力する */}}
    {{- /* lessons.md: default フィルタは 0 を falsy 扱いするため hasKey を使う */}}
    {{- if hasKey .Values.autoscaling "targetCPUUtilizationPercentage" }}
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetCPUUtilizationPercentage }}
    {{- end }}
    {{- if hasKey .Values.autoscaling "targetMemoryUtilizationPercentage" }}
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetMemoryUtilizationPercentage }}
    {{- end }}
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Pods
          value: 2
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Pods
          value: 1
          periodSeconds: 120
{{- end }}
{{- end }}
