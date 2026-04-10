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
{{- /* LOW-011 対応: vault.role が未設定の場合は helm install/upgrade を失敗させる。
     各サービスの values.yaml に role: "<service-name>" を明示的に設定すること。 */}}
{{- if not .Values.vault.role }}
{{- fail "vault.role が設定されていません。各サービスの values.yaml に role: \"<service-name>\" を設定してください（ADR-0045 参照）。" }}
{{- end }}
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
{{- /* 本番・ステージング環境での TLS 検証スキップは中間者攻撃リスクがあるため禁止する（H-012）
     開発環境（dev/local）でのみ自己署名証明書用に使用可能。
     本番環境は vault.tlsCACert / vault.tlsServerName で正しい TLS を設定すること。 */}}
{{- $env := .Values.global.environment | default "dev" }}
{{- if or (eq $env "production") (eq $env "staging") (eq $env "prod") }}
{{- fail (printf "vault.tlsSkipVerify は %s 環境では使用できません。vault.tlsCACert で正しい TLS 設定を行ってください" $env) }}
{{- else }}
vault.hashicorp.com/tls-skip-verify: "true"
{{- end }}
{{- end }}
{{- end }}
{{- end }}
