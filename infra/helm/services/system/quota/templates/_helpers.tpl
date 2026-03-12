{{/*
quota.fullname
*/}}
{{- define "quota.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
quota.labels
*/}}
{{- define "quota.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
quota.selectorLabels
*/}}
{{- define "quota.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
quota.serviceAccountName
*/}}
{{- define "quota.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
