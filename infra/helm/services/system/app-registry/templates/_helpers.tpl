{{/*
app-registry.fullname - リリース名から app-registry のフルネームを生成する
*/}}
{{- define "app-registry.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
app-registry.labels - 共通ラベル
*/}}
{{- define "app-registry.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
app-registry.selectorLabels - セレクタ用ラベル
*/}}
{{- define "app-registry.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
app-registry.serviceAccountName - サービスアカウント名
*/}}
{{- define "app-registry.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
