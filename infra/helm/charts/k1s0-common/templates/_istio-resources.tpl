{{/*
k1s0-common.istioResources - VirtualService と DestinationRule をまとめて生成する統合ヘルパー。
消費側チャートは templates/istio.yaml に1行 include するだけで両リソースを有効化できる。

使用例（消費側チャートの templates/istio.yaml）:
  {{- include "k1s0-common.istioResources" . }}

生成される manifest:
  - VirtualService（istio.virtualService.enabled=true の場合）
  - DestinationRule（istio.destinationRule.enabled=true の場合）
  両方が有効な場合は "---" セパレータで区切られる。

消費側チャートでの有効化手順:
  1. 消費側チャートの dependencies に k1s0-common を追加済みであること
  2. 以下の内容で templates/istio.yaml を作成する:

     {{- include "k1s0-common.istioResources" . }}

  3. values.yaml に以下を設定する:
     istio:
       enabled: true
       virtualService:
         enabled: true
       destinationRule:
         enabled: true
*/}}
{{- define "k1s0-common.istioResources" -}}
{{- $vs := include "k1s0-common.virtualService" . -}}
{{- $dr := include "k1s0-common.destinationRule" . -}}
{{- if and $vs $dr }}
{{ $vs }}
---
{{ $dr }}
{{- else if $vs }}
{{ $vs }}
{{- else if $dr }}
{{ $dr }}
{{- end }}
{{- end }}
