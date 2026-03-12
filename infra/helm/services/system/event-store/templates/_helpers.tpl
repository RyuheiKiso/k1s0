{{/*
event-store.fullname
*/}}
{{- define "event-store.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
event-store.labels
*/}}
{{- define "event-store.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
event-store.selectorLabels
*/}}
{{- define "event-store.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
event-store.serviceAccountName
*/}}
{{- define "event-store.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
