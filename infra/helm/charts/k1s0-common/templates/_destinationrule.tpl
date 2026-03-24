{{/*
k1s0-common.destinationRule - Istio DestinationRule リソースを生成する。
istio.enabled かつ istio.destinationRule.enabled の場合のみリソースを出力する。
CircuitBreaker の outlierDetection パラメータを values から注入する。
*/}}
{{- define "k1s0-common.destinationRule" -}}
{{/* istio・destinationRule オブジェクトの nil チェックを先行させ、未設定 values.yaml でのテンプレートエラーを防ぐ（nil-safe: C-1 対応） */}}
{{- if and .Values.istio .Values.istio.enabled .Values.istio.destinationRule .Values.istio.destinationRule.enabled }}
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  host: {{ include "k1s0-common.fullname" . }}
  trafficPolicy:
    outlierDetection:
      consecutiveGatewayErrors: {{ .Values.istio.destinationRule.circuitBreaker.consecutiveGatewayErrors | default 5 }}
      consecutive5xxErrors: {{ .Values.istio.destinationRule.circuitBreaker.consecutive5xxErrors | default 5 }}
      interval: {{ .Values.istio.destinationRule.circuitBreaker.interval | default "30s" }}
      baseEjectionTime: {{ .Values.istio.destinationRule.circuitBreaker.baseEjectionTime | default "30s" }}
      maxEjectionPercent: {{ .Values.istio.destinationRule.circuitBreaker.maxEjectionPercent | default 50 }}
{{- end }}
{{- end }}
