{{/*
navigation.fullname
*/}}
{{- define "navigation.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
navigation.labels
*/}}
{{- define "navigation.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
navigation.selectorLabels
*/}}
{{- define "navigation.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
navigation.serviceAccountName
*/}}
{{- define "navigation.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
