{{/*
k1s0-common.vaultAnnotations - Vault Agent Injector 用 annotation を生成する
Vault Agent Sidecar が自動的にシークレットを注入するための設定。

使用例:
  annotations:
    {{- include "k1s0-common.vaultAnnotations" . | nindent 4 }}

Values に以下を設定:
  vault:
    enabled: true
    role: "system"
    secrets:
      - path: "secret/data/k1s0/system/auth-server/database"
        template: |
          {{ "{{" }} with secret "secret/data/k1s0/system/auth-server/database" {{ "}}" }}
          DATABASE_PASSWORD={{ "{{" }} .Data.data.password {{ "}}" }}
          {{ "{{" }} end {{ "}}" }}
      - path: "database/creds/auth-server-rw"
        template: |
          {{ "{{" }} with secret "database/creds/auth-server-rw" {{ "}}" }}
          DB_USERNAME={{ "{{" }} .Data.username {{ "}}" }}
          DB_PASSWORD={{ "{{" }} .Data.password {{ "}}" }}
          {{ "{{" }} end {{ "}}" }}
*/}}
{{- define "k1s0-common.vaultAnnotations" -}}
{{- if .Values.vault.enabled }}
vault.hashicorp.com/agent-inject: "true"
vault.hashicorp.com/agent-inject-status: "update"
vault.hashicorp.com/role: {{ .Values.vault.role | quote }}
{{- if .Values.vault.serviceAccountName }}
vault.hashicorp.com/agent-inject-token: "true"
{{- end }}
{{- range $index, $secret := .Values.vault.secrets }}
vault.hashicorp.com/agent-inject-secret-{{ $secret.name | default (printf "secret-%d" $index) }}: {{ $secret.path | quote }}
{{- if $secret.template }}
vault.hashicorp.com/agent-inject-template-{{ $secret.name | default (printf "secret-%d" $index) }}: |
{{ $secret.template | indent 2 }}
{{- end }}
{{- end }}
{{- if .Values.vault.tlsSkipVerify }}
vault.hashicorp.com/tls-skip-verify: "true"
{{- end }}
{{- end }}
{{- end }}
