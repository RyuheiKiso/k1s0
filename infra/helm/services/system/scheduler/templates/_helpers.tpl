{{/*
scheduler.fullname
*/}}
{{- define "scheduler.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
scheduler.labels
*/}}
{{- define "scheduler.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
scheduler.selectorLabels
*/}}
{{- define "scheduler.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
scheduler.serviceAccountName
*/}}
{{- define "scheduler.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
