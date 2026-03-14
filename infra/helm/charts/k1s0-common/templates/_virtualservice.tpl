{{/*
k1s0-common.virtualService - Istio VirtualService リソースを生成する。
istio.enabled かつ istio.virtualService.enabled の場合のみリソースを出力する。
タイムアウト・リトライ設定を values から注入する。
*/}}
{{- define "k1s0-common.virtualService" -}}
{{- if and .Values.istio.enabled .Values.istio.virtualService.enabled }}
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  hosts:
    - {{ include "k1s0-common.fullname" . }}
  http:
    - timeout: {{ .Values.istio.virtualService.timeout | default .Values.istio.timeout | default "30s" }}
      retries:
        attempts: {{ .Values.istio.virtualService.retries.attempts | default 3 }}
        perTryTimeout: {{ .Values.istio.virtualService.retries.perTryTimeout | default "10s" }}
        retryOn: {{ .Values.istio.virtualService.retries.retryOn | default "5xx,reset,connect-failure" }}
      route:
        - destination:
            host: {{ include "k1s0-common.fullname" . }}
            port:
              number: {{ .Values.service.port }}
{{- end }}
{{- end }}
