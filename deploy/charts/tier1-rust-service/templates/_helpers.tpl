{{/*
tier1-rust-service chart の helper template。
共通 label / fullname / Pod 単位 fullname / serviceAccountName を生成する。
*/}}

{{- define "tier1-rust-service.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "tier1-rust-service.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}

{{- define "tier1-rust-service.podFullname" -}}
{{- $root := .Release.Name -}}
{{- $chart := .Chart.Name -}}
{{- printf "%s-%s-%s" $root $chart .podName | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "tier1-rust-service.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: k1s0
k1s0.io/tier: tier1
k1s0.io/lang: rust
{{- end -}}

{{- define "tier1-rust-service.podSelectorLabels" -}}
app.kubernetes.io/name: {{ printf "tier1-%s" .podName }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: tier1-{{ .podName }}
k1s0.io/component: tier1-{{ .podName }}
{{- end -}}

{{- define "tier1-rust-service.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "tier1-rust-service.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}
