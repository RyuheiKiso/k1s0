{{- define "k1s0-common.pdb" -}}
{{/* pdb オブジェクトと enabled フラグの両方が存在する場合のみ PDB リソースを生成する（nil-safe: C-1 対応） */}}
{{- if and .Values.pdb .Values.pdb.enabled }}
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  minAvailable: {{ .Values.pdb.minAvailable }}
  selector:
    matchLabels:
      {{- include "k1s0-common.selectorLabels" . | nindent 6 }}
{{- end }}
{{- end }}
