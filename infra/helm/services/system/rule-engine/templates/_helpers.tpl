{{/*
rule-engine.fullname
*/}}
{{- define "rule-engine.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
rule-engine.labels
*/}}
{{- define "rule-engine.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
rule-engine.selectorLabels
*/}}
{{- define "rule-engine.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
rule-engine.serviceAccountName
*/}}
{{- define "rule-engine.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
