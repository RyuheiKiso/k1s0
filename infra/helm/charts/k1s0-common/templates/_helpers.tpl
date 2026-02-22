{{/*
k1s0-common.fullname - リリース名とChart名からフルネームを生成する
63文字に切り詰め、末尾のハイフンを除去する
*/}}
{{- define "k1s0-common.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
k1s0-common.name - Chart名を返す
*/}}
{{- define "k1s0-common.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
k1s0-common.labels - 共通ラベルを生成する
6標準ラベル: name / instance / version / component / part-of / managed-by
Values.labels で追加ラベル（tier 等）を動的に付与する
*/}}
{{- define "k1s0-common.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
app.kubernetes.io/name: {{ include "k1s0-common.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
{{- if .Values.component }}
app.kubernetes.io/component: {{ .Values.component | quote }}
{{- end }}
app.kubernetes.io/part-of: {{ default "k1s0" .Values.partOf | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- range $key, $value := .Values.labels }}
{{ $key }}: {{ $value | quote }}
{{- end }}
{{- end }}

{{/*
k1s0-common.selectorLabels - セレクタ用ラベルを生成する
*/}}
{{- define "k1s0-common.selectorLabels" -}}
app.kubernetes.io/name: {{ include "k1s0-common.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
k1s0-common.serviceAccountName - サービスアカウント名を返す
*/}}
{{- define "k1s0-common.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "k1s0-common.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
