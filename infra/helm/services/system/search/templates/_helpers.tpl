{{/*
search.fullname
*/}}
{{- define "search.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
search.labels
*/}}
{{- define "search.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
search.selectorLabels
*/}}
{{- define "search.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
search.serviceAccountName
*/}}
{{- define "search.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
