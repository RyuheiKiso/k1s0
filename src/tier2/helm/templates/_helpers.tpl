{{/*
_helpers.tpl — k1s0-tier2 Helm チャートヘルパーテンプレート定義
Chart.yaml / values.yaml と組み合わせて使用する共通テンプレート群
*/}}

{{/*
k1s0-tier2.fullname: Helm release 名とチャート名を組み合わせてフルリソース名を生成する
63 文字上限（Kubernetes リソース名制約）を超えた場合はトリムする
*/}}
{{- define "k1s0-tier2.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- /* fullnameOverride が設定されている場合は上書き値を使用する */ -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- /* release 名とチャート名を組み合わせてフルネームを生成する */ -}}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- /* release 名がチャート名を既に含む場合は重複を避けてチャート名のみ使用する */ -}}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- /* release 名とチャート名をハイフンで結合してフルネームを生成する */ -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
k1s0-tier2.chart: チャート名とバージョンを組み合わせたラベル値を生成する
Helm 管理ラベル chart: の値として使用する
*/}}
{{- define "k1s0-tier2.chart" -}}
{{- /* チャート名とバージョンをハイフンで結合して 63 文字に切り詰める */ -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
k1s0-tier2.labels: Helm 管理ラベルセットを生成する
全リソースに共通して付与する標準 Kubernetes ラベル群
*/}}
{{- define "k1s0-tier2.labels" -}}
{{- /* Helm チャートラベル */ -}}
helm.sh/chart: {{ include "k1s0-tier2.chart" . }}
{{- /* セレクタラベルを含める */ -}}
{{ include "k1s0-tier2.selectorLabels" . }}
{{- /* アプリケーションバージョンラベル（Chart.yaml の appVersion から取得する） */ -}}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
{{- /* 管理ツールラベル（Helm によって管理されていることを示す） */ -}}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- /* k1s0 軸識別ラベル */ -}}
k1s0.axis: tier2
{{- end }}

{{/*
k1s0-tier2.selectorLabels: Pod セレクタラベルセットを生成する
Deployment の selector.matchLabels と Pod の labels に共通して使用する
*/}}
{{- define "k1s0-tier2.selectorLabels" -}}
{{- /* アプリケーション名ラベル（チャート名を使用する） */ -}}
app.kubernetes.io/name: {{ include "k1s0-tier2.fullname" . }}
{{- /* Helm release 名ラベル（同一チャートの複数インスタンスを区別する） */ -}}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
k1s0-tier2.serviceAccountName: ServiceAccount 名を返す
values.yaml の serviceAccount.name が設定されている場合はその値を使用し、
設定されていない場合はフルネームを使用する
*/}}
{{- define "k1s0-tier2.serviceAccountName" -}}
{{- /* serviceAccount.create が true の場合は名前を解決する */ -}}
{{- if .Values.serviceAccount.create }}
{{- /* serviceAccount.name が設定されている場合はその値を使用する */ -}}
{{- default (include "k1s0-tier2.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- /* serviceAccount.create が false の場合は "default" を使用する */ -}}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
