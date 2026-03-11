{{/*
notification.fullname
*/}}
{{- define "notification.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
notification.labels
*/}}
{{- define "notification.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
notification.selectorLabels
*/}}
{{- define "notification.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
notification.serviceAccountName
*/}}
{{- define "notification.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
