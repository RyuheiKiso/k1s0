{{/*
k1s0-common.fullname - リリース名とChart名からフルネームを生成する
63文字に切り詰め、末尾のハイフンを除去する
*/}}
{{- define "k1s0-common.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
k1s0-common.name - Chart名を返す
*/}}
{{- define "k1s0-common.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
k1s0-common.labels - 共通ラベルを生成する
6標準ラベル: name / instance / version / component / part-of / managed-by
Values.labels で追加ラベル（tier 等）を動的に付与する
*/}}
{{- define "k1s0-common.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
app.kubernetes.io/name: {{ include "k1s0-common.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
{{- if .Values.component }}
app.kubernetes.io/component: {{ .Values.component | quote }}
{{- end }}
app.kubernetes.io/part-of: {{ default "k1s0" .Values.partOf | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- range $key, $value := .Values.labels }}
{{ $key }}: {{ $value | quote }}
{{- end }}
{{- end }}

{{/*
k1s0-common.selectorLabels - セレクタ用ラベルを生成する
*/}}
{{- define "k1s0-common.selectorLabels" -}}
app.kubernetes.io/name: {{ include "k1s0-common.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
k1s0-common.validateValues - 必須 values のバリデーションを行う（L-8 / L-9 監査対応）

L-8 対応: alertmanager.webhookUrl が空の場合に本番環境（prod）では必須エラーを発生させる。
  空のまま本番デプロイすると Microsoft Teams への障害通知が欠落し、インシデント対応が遅延するリスクがある。
  開発・staging 環境では空を許容するが、本番では環境別 values.yaml で必ず設定すること。

L-9 対応: redis.enabled が true の場合に redis.host が空文字でないことを検証する。
  redis.host が空だと REDIS_HOST 環境変数に空文字が注入され、Redis 接続先未指定で起動するリスクがある。
*/}}
{{- define "k1s0-common.validateValues" -}}
{{/* L-8: alertmanager.required=true の場合に webhookUrl を必須チェックする。
     本番環境の values.yaml では alertmanager.required: true を設定すること。
     空のまま本番デプロイすると Microsoft Teams への障害通知が欠落し、インシデント対応が遅延するリスクがある。 */}}
{{- if and .Values.alertmanager .Values.alertmanager.required (not .Values.alertmanager.webhookUrl) -}}
{{- fail "alertmanager.required が true の場合、alertmanager.webhookUrl は必須です。本番環境の values.yaml に alertmanager.webhookUrl を設定してください。" -}}
{{- end -}}
{{/* L-9: redis.enabled=true の場合に redis.host を必須チェックする */}}
{{- if and .Values.redis .Values.redis.enabled (not .Values.redis.host) -}}
{{- fail "redis.enabled が true の場合、redis.host は必須です。values.yaml に redis.host を設定してください。" -}}
{{- end -}}
{{- end -}}

{{/*
k1s0-common.serviceAccountName - サービスアカウント名を返す
*/}}
{{- define "k1s0-common.serviceAccountName" -}}
{{/* serviceAccount オブジェクトが nil の場合は "default" を返す（nil-safe: C-1 対応） */}}
{{- if and .Values.serviceAccount .Values.serviceAccount.create }}
{{- default (include "k1s0-common.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" ((.Values.serviceAccount).name) }}
{{- end }}
{{- end }}
