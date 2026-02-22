{{/*
config.fullname - リリース名から config のフルネームを生成する
*/}}
{{- define "config.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
config.labels - 共通ラベル
*/}}
{{- define "config.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
config.selectorLabels - セレクタ用ラベル
*/}}
{{- define "config.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
config.serviceAccountName - サービスアカウント名
*/}}
{{- define "config.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
