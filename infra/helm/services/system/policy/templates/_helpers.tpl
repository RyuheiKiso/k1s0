{{/*
policy.fullname
*/}}
{{- define "policy.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
policy.labels
*/}}
{{- define "policy.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
policy.selectorLabels
*/}}
{{- define "policy.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
policy.serviceAccountName
*/}}
{{- define "policy.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
