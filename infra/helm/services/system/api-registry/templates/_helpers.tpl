{{/*
api-registry.fullname
*/}}
{{- define "api-registry.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
api-registry.labels
*/}}
{{- define "api-registry.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
api-registry.selectorLabels
*/}}
{{- define "api-registry.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
api-registry.serviceAccountName
*/}}
{{- define "api-registry.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
