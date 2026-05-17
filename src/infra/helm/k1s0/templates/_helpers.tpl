{{/* k1s0 Helm chart 共通ヘルパーテンプレート定義 */}}

{{/*
chart の完全名を生成するヘルパー
リリース名と chart 名を結合し、63 文字以内に収める
*/}}
{{- define "k1s0.fullname" -}}
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
chart 名を生成するヘルパー（バージョン情報を除く）
*/}}
{{- define "k1s0.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
全リソース共通の標準ラベルセットを生成するヘルパー
GitOps・可観測性ツールが使用するラベルを統一する
*/}}
{{- define "k1s0.labels" -}}
helm.sh/chart: {{ include "k1s0.chart" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: k1s0
k1s0.io/axis: infra
{{- end }}
