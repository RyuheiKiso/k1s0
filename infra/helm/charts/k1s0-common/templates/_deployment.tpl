{{- define "k1s0-common.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "k1s0-common.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "k1s0-common.labels" . | nindent 8 }}
      {{- if and .Values.vault .Values.vault.enabled }}
      annotations:
        vault.hashicorp.com/agent-inject: "true"
        vault.hashicorp.com/role: {{ .Values.vault.role | quote }}
        {{- range .Values.vault.secrets }}
        vault.hashicorp.com/agent-inject-secret-{{ .key }}: {{ .path | quote }}
        {{- end }}
      {{- end }}
    spec:
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ include "k1s0-common.serviceAccountName" . }}
      {{- with .Values.podSecurityContext }}
      securityContext:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      containers:
        - name: {{ .Chart.Name }}
          {{- if .Values.image.registry }}
          image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          {{- else }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          {{- end }}
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          {{- with .Values.containerSecurityContext }}
          securityContext:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.container.command }}
          command:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.container.args }}
          args:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          env:
            - name: CONFIG_PATH
              value: "{{ .Values.config.mountPath }}/config.yaml"
            {{- with .Values.container.env }}
            {{- toYaml . | nindent 12 }}
            {{- end }}
          ports:
            - name: http
              containerPort: {{ .Values.container.port }}
              protocol: TCP
            {{- if .Values.container.grpcPort }}
            - name: grpc
              containerPort: {{ .Values.container.grpcPort }}
              protocol: TCP
            {{- end }}
          {{- if and .Values.probes .Values.probes.grpcHealthCheck .Values.probes.grpcHealthCheck.enabled }}
          livenessProbe:
            grpc:
              port: {{ .Values.container.grpcPort | default 9090 }}
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            grpc:
              port: {{ .Values.container.grpcPort | default 9090 }}
            initialDelaySeconds: 5
            periodSeconds: 5
          {{- else }}
          {{- with .Values.probes.liveness }}
          livenessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.probes.readiness }}
          readinessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- end }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          volumeMounts:
            - name: config
              mountPath: {{ .Values.config.mountPath }}
              readOnly: true
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      volumes:
        - name: config
          configMap:
            name: {{ include "k1s0-common.fullname" . }}-config
{{- end }}
