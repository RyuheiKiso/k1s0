{{/*
saga.fullname - リリース名から saga のフルネームを生成する
*/}}
{{- define "saga.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
saga.labels - 共通ラベル
*/}}
{{- define "saga.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
saga.selectorLabels - セレクタ用ラベル
*/}}
{{- define "saga.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
saga.serviceAccountName - サービスアカウント名
*/}}
{{- define "saga.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
