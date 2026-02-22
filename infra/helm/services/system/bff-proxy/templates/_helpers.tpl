{{/*
bff-proxy.fullname - generate fullname from release
*/}}
{{- define "bff-proxy.fullname" -}}
{{- include "k1s0-common.fullname" . }}
{{- end }}

{{/*
bff-proxy.labels - common labels
*/}}
{{- define "bff-proxy.labels" -}}
{{- include "k1s0-common.labels" . }}
{{- end }}

{{/*
bff-proxy.selectorLabels - selector labels
*/}}
{{- define "bff-proxy.selectorLabels" -}}
{{- include "k1s0-common.selectorLabels" . }}
{{- end }}

{{/*
bff-proxy.serviceAccountName - service account name
*/}}
{{- define "bff-proxy.serviceAccountName" -}}
{{- include "k1s0-common.serviceAccountName" . }}
{{- end }}
