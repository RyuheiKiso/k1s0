{{/* tier2-go-service helper */}}

{{- define "tier2-go-service.fullname" -}}
{{- printf "%s-%s" .Release.Name .Values.service.name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "tier2-go-service.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
app.kubernetes.io/name: {{ .Values.service.name }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: k1s0
k1s0.io/tier: tier2
k1s0.io/lang: go
k1s0.io/component: {{ .Values.service.name }}
{{- end -}}

{{- define "tier2-go-service.selectorLabels" -}}
app.kubernetes.io/name: {{ .Values.service.name }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "tier2-go-service.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "tier2-go-service.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}
