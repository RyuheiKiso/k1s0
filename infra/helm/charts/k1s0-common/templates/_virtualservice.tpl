{{/*
k1s0-common.virtualService - Istio VirtualService リソースを生成する。
istio.enabled かつ istio.virtualService.enabled の場合のみリソースを出力する。
タイムアウト・リトライ設定を values から注入する。
*/}}
{{- define "k1s0-common.virtualService" -}}
{{/* istio・virtualService オブジェクトの nil チェックを先行させ、未設定 values.yaml でのテンプレートエラーを防ぐ（nil-safe: C-1 対応） */}}
{{- if and .Values.istio .Values.istio.enabled .Values.istio.virtualService .Values.istio.virtualService.enabled }}
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
        {{- $retries := .Values.istio.virtualService.retries }}
        {{/* M-07監査対応: `default` フィルタは 0 を falsy として扱うため attempts: 0（リトライ無効）が機能しないバグを修正。hasKey で値の有無を判定する。 */}}
        attempts: {{ if (hasKey $retries "attempts") }}{{ $retries.attempts }}{{ else }}3{{ end }}
        perTryTimeout: {{ .Values.istio.virtualService.retries.perTryTimeout | default "10s" }}
        retryOn: {{ .Values.istio.virtualService.retries.retryOn | default "5xx,reset,connect-failure" }}
      route:
        - destination:
            host: {{ include "k1s0-common.fullname" . }}
            port:
              number: {{ .Values.service.port }}
{{- end }}
{{- end }}
