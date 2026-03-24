{{- define "k1s0-common.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  {{/* autoscaling が未設定または無効の場合に replicas を明示的に指定する（nil-safe: C-1 対応） */}}
  {{- if not (and .Values.autoscaling .Values.autoscaling.enabled) }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "k1s0-common.selectorLabels" . | nindent 6 }}
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  template:
    metadata:
      annotations:
        {{- include "k1s0-common.istioAnnotations" . | nindent 8 }}
        {{- with .Values.podAnnotations }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
      labels:
        {{- include "k1s0-common.labels" . | nindent 8 }}
        version: {{ .Values.image.tag | default "stable" | quote }}
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
          image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          {{- with .Values.containerSecurityContext }}
          securityContext:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{/* container オブジェクトが nil の場合は command/args/ports を安全にスキップする（nil-safe: C-1 対応） */}}
          {{- if and .Values.container .Values.container.command }}
          command:
            {{- toYaml .Values.container.command | nindent 12 }}
          {{- end }}
          {{- if and .Values.container .Values.container.args }}
          args:
            {{- toYaml .Values.container.args | nindent 12 }}
          {{- end }}
          ports:
            - name: http
              containerPort: {{ if .Values.container }}{{ .Values.container.port | default 8080 }}{{ else }}8080{{ end }}
              protocol: TCP
            {{- if and .Values.container .Values.container.grpcPort }}
            - name: grpc
              containerPort: {{ .Values.container.grpcPort }}
              protocol: TCP
            {{- end }}
          {{- with .Values.env }}
          env:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.envFrom }}
          envFrom:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- if and .Values.probes .Values.probes.grpcHealthCheck .Values.probes.grpcHealthCheck.enabled }}
          {{/* gRPC ヘルスチェックが有効な場合: startup/liveness/readiness すべてを gRPC プローブとして展開する */}}
          {{- with .Values.probes }}
          {{- with .startup }}
          startupProbe:
            grpc:
              port: {{ $.Values.container.grpcPort }}
            failureThreshold: {{ .failureThreshold }}
            periodSeconds: {{ .periodSeconds }}
          {{- end }}
          {{- end }}
          livenessProbe:
            grpc:
              port: {{ .Values.container.grpcPort }}
            initialDelaySeconds: {{ .Values.probes.liveness.initialDelaySeconds }}
            periodSeconds: {{ .Values.probes.liveness.periodSeconds }}
            failureThreshold: {{ .Values.probes.liveness.failureThreshold }}
          readinessProbe:
            grpc:
              port: {{ .Values.container.grpcPort }}
            initialDelaySeconds: {{ .Values.probes.readiness.initialDelaySeconds }}
            periodSeconds: {{ .Values.probes.readiness.periodSeconds }}
            failureThreshold: {{ .Values.probes.readiness.failureThreshold }}
          {{- else }}
          {{/* probes オブジェクト自体が nil の場合のチェックを先行させ HTTP プローブを安全に展開する（nil-safe: C-1 対応） */}}
          {{- with .Values.probes }}
          {{/* startupProbe: コンテナ初回起動時の猶予期間を確保し、liveness の誤検知を防ぐ */}}
          {{- with .startup }}
          startupProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .liveness }}
          livenessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .readiness }}
          readinessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- end }}
          {{- end }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          volumeMounts:
            - name: config
              {{/* config オブジェクトが nil の場合は /etc/app をデフォルトとする（nil-safe: C-1 対応） */}}
              mountPath: {{ if .Values.config }}{{ .Values.config.mountPath | default "/etc/app" }}{{ else }}/etc/app{{ end }}
              readOnly: true
            {{- with .Values.extraVolumeMounts }}
            {{- toYaml . | nindent 12 }}
            {{- end }}
      volumes:
        - name: config
          configMap:
            name: {{ include "k1s0-common.fullname" . }}-config
        {{- with .Values.extraVolumes }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
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
{{- end }}
