{{/*
event-monitor.fullname
*/}}
{{- define "event-monitor.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
event-monitor.labels
*/}}
{{- define "event-monitor.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
event-monitor.selectorLabels
*/}}
{{- define "event-monitor.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
event-monitor.serviceAccountName
*/}}
{{- define "event-monitor.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
