{{- define "k1s0-common.service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  {{/* service オブジェクトが nil の場合も含め ClusterIP をデフォルトとする（nil-safe: C-1 対応） */}}
  type: {{ if .Values.service }}{{ .Values.service.type | default "ClusterIP" }}{{ else }}ClusterIP{{ end }}
  ports:
    - name: http
      {{/* service.port が未設定の場合は 80 をデフォルトとする（nil-safe: C-1 対応） */}}
      port: {{ if .Values.service }}{{ .Values.service.port | default 80 }}{{ else }}80{{ end }}
      targetPort: http
      protocol: TCP
    {{/* service オブジェクトと grpcPort の両方が存在する場合のみ gRPC ポートを追加する（nil-safe: C-1 対応） */}}
    {{- if and .Values.service .Values.service.grpcPort }}
    - name: grpc
      port: {{ .Values.service.grpcPort }}
      targetPort: grpc
      protocol: TCP
    {{- end }}
  selector:
    {{- include "k1s0-common.selectorLabels" . | nindent 4 }}
{{- end }}
