{{/*
master-maintenance.fullname
*/}}
{{- define "master-maintenance.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
master-maintenance.labels
*/}}
{{- define "master-maintenance.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
master-maintenance.selectorLabels
*/}}
{{- define "master-maintenance.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
master-maintenance.serviceAccountName
*/}}
{{- define "master-maintenance.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
