{{/*
workflow.fullname
*/}}
{{- define "workflow.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
workflow.labels
*/}}
{{- define "workflow.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
workflow.selectorLabels
*/}}
{{- define "workflow.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
workflow.serviceAccountName
*/}}
{{- define "workflow.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
