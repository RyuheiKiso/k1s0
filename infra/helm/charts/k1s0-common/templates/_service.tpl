{{- define "k1s0-common.service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  {{/* service.type が未設定の場合は ClusterIP をデフォルトとする */}}
  type: {{ .Values.service.type | default "ClusterIP" }}
  ports:
    - name: http
      {{/* service.port が未設定の場合は 80 をデフォルトとする */}}
      port: {{ .Values.service.port | default 80 }}
      targetPort: http
      protocol: TCP
    {{- if .Values.service.grpcPort }}
    - name: grpc
      port: {{ .Values.service.grpcPort }}
      targetPort: grpc
      protocol: TCP
    {{- end }}
  selector:
    {{- include "k1s0-common.selectorLabels" . | nindent 4 }}
{{- end }}
