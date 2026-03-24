{{/*
k1s0-common.destinationRule - Istio DestinationRule リソースを生成する。
istio.enabled かつ istio.destinationRule.enabled の場合のみリソースを出力する。
H-06監査対応: mTLS（ISTIO_MUTUAL）と connectionPool 設定を追加し、
全サービスで統一されたサーキットブレーカーを実現する。
*/}}
{{- define "k1s0-common.destinationRule" -}}
{{/* istio・destinationRule オブジェクトの nil チェックを先行させ、未設定 values.yaml でのテンプレートエラーを防ぐ */}}
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
    # mTLS: サービス間通信を Istio マネージド相互 TLS で保護する
    tls:
      mode: ISTIO_MUTUAL
    # connectionPool: バックプレッシャー制御のための接続数上限を設定する
    connectionPool:
      tcp:
        maxConnections: {{ .Values.istio.destinationRule.connectionPool.tcp.maxConnections | default 100 }}
      http:
        h2UpgradePolicy: DEFAULT
        maxRequestsPerConnection: {{ .Values.istio.destinationRule.connectionPool.http.maxRequestsPerConnection | default 0 }}
    # outlierDetection: 異常なホストを Circuit Breaker でエジェクトする
    outlierDetection:
      consecutiveGatewayErrors: {{ .Values.istio.destinationRule.circuitBreaker.consecutiveGatewayErrors | default 5 }}
      consecutive5xxErrors: {{ .Values.istio.destinationRule.circuitBreaker.consecutive5xxErrors | default 5 }}
      interval: {{ .Values.istio.destinationRule.circuitBreaker.interval | default "30s" }}
      baseEjectionTime: {{ .Values.istio.destinationRule.circuitBreaker.baseEjectionTime | default "30s" }}
      maxEjectionPercent: {{ .Values.istio.destinationRule.circuitBreaker.maxEjectionPercent | default 50 }}
{{- end }}
{{- end }}
