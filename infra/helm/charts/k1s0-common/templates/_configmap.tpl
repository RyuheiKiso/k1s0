{{- define "k1s0-common.configmap" -}}
{{- if .Values.config.data }}
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "k1s0-common.fullname" . }}-config
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
data:
  {{- range $key, $value := .Values.config.data }}
  {{ $key }}: |
    {{- $value | nindent 4 }}
  {{- end }}
{{- else }}
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "k1s0-common.fullname" . }}-config
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
data: {}
{{- end }}
{{- end }}
