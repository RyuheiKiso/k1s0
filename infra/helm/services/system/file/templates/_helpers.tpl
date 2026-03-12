{{/*
file.fullname
*/}}
{{- define "file.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
file.labels
*/}}
{{- define "file.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
file.selectorLabels
*/}}
{{- define "file.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
file.serviceAccountName
*/}}
{{- define "file.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
