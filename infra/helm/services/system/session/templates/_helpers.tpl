{{/*
session.fullname
*/}}
{{- define "session.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
session.labels
*/}}
{{- define "session.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
session.selectorLabels
*/}}
{{- define "session.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
session.serviceAccountName
*/}}
{{- define "session.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
