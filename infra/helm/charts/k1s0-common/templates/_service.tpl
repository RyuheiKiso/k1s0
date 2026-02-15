{{- define "k1s0-common.service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - name: http
      port: {{ .Values.service.port }}
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
