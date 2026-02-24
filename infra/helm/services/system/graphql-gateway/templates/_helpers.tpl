{{/*
graphql-gateway.fullname - リリース名から graphql-gateway のフルネームを生成する
*/}}
{{- define "graphql-gateway.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
graphql-gateway.labels - 共通ラベル
*/}}
{{- define "graphql-gateway.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
graphql-gateway.selectorLabels - セレクタ用ラベル
*/}}
{{- define "graphql-gateway.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
graphql-gateway.serviceAccountName - サービスアカウント名
*/}}
{{- define "graphql-gateway.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
