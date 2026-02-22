{{/*
dlq-manager.fullname - リリース名から dlq-manager のフルネームを生成する
*/}}
{{- define "dlq-manager.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
dlq-manager.labels - 共通ラベル
*/}}
{{- define "dlq-manager.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
dlq-manager.selectorLabels - セレクタ用ラベル
*/}}
{{- define "dlq-manager.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
dlq-manager.serviceAccountName - サービスアカウント名
*/}}
{{- define "dlq-manager.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
