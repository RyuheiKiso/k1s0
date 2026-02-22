{{/*
auth.fullname - リリース名から auth のフルネームを生成する
*/}}
{{- define "auth.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
auth.labels - 共通ラベル
*/}}
{{- define "auth.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
auth.selectorLabels - セレクタ用ラベル
*/}}
{{- define "auth.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
auth.serviceAccountName - サービスアカウント名
*/}}
{{- define "auth.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
