{{/*
本ファイルは tier1-facade chart の Helm テンプレートヘルパ。
リソース名・ラベル・ServiceAccount 名の導出ロジックを集約する。

設計: docs/05_実装/70_リリース設計/
関連 ID: ADR-CICD-001
*/}}

{{/*
chart 名（最大 63 文字、k8s name 規約に従う）。
*/}}
{{- define "tier1-facade.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
fullname（リリース名 + chart 名、最大 63 文字）。
*/}}
{{- define "tier1-facade.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{/*
Pod 別の fullname（fullname + Pod 名、例: release-tier1-facade-state）。
引数: dict "Release" .Release "Chart" .Chart "Values" .Values "podName" "state"
*/}}
{{- define "tier1-facade.podFullname" -}}
{{- printf "%s-%s" (include "tier1-facade.fullname" .) .podName | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
共通 labels（Recommended Labels）。
*/}}
{{- define "tier1-facade.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{ include "tier1-facade.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: k1s0
k1s0.io/tier: tier1
{{- end -}}

{{/*
selector labels（Deployment.spec.selector / Service.spec.selector で使う）。
*/}}
{{- define "tier1-facade.selectorLabels" -}}
app.kubernetes.io/name: {{ include "tier1-facade.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Pod 別 selector labels（state / secret / workflow を区別する）。
引数: dict "Release" .Release "Chart" .Chart "Values" .Values "podName" "state"
*/}}
{{- define "tier1-facade.podSelectorLabels" -}}
{{ include "tier1-facade.selectorLabels" . }}
k1s0.io/tier1-pod: {{ .podName }}
{{- end -}}

{{/*
ServiceAccount 名の導出（serviceAccount.create=true なら fullname、false なら明示 name）。
*/}}
{{- define "tier1-facade.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "tier1-facade.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}
