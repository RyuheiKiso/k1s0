{{/*
k1s0-common.istioAnnotations - Istio サイドカーインジェクション用 annotation を生成する。
Istio サービスメッシュのサイドカープロキシ設定とタイムアウトを注入する。

使用例:
  annotations:
    {{- include "k1s0-common.istioAnnotations" . | nindent 4 }}

Values に以下を設定:
  istio:
    enabled: true
    sidecar:
      inject: true
      proxyConfig:
        holdApplicationUntilProxyStarts: true
    timeout: 30s
*/}}
{{- define "k1s0-common.istioAnnotations" -}}
{{- if .Values.istio.enabled }}
sidecar.istio.io/inject: {{ .Values.istio.sidecar.inject | quote }}
{{- if .Values.istio.timeout }}
proxy.istio.io/config: |
  holdApplicationUntilProxyStarts: {{ .Values.istio.sidecar.proxyConfig.holdApplicationUntilProxyStarts }}
  terminationDrainDuration: {{ .Values.istio.timeout }}
{{- end }}
{{- end }}
{{- end }}
