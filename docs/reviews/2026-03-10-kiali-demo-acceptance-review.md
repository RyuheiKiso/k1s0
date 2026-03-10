# Kiali Demo 検収レビュー

実施日: 2026-03-10

判定: 不合格

対象:
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md`
- `docs/architecture/observability/可観測性設計.md`
- `docs/architecture/observability/ログ設計.md`
- `infra/demo/kiali/`

実施した確認:
- `infra/demo/kiali/ui` で `npm run build` 成功
- `infra/demo/kiali/setup.sh` を再実行し正常終了を確認
- 実クラスタ `kind-k1s0-demo` 上で Kiali / Grafana / Jaeger / Prometheus 応答を確認
- Kubernetes リソース、Prometheus 指標、Jaeger API、Loki API を直接確認

## 不適合一覧

### 1. 重大: トラフィック生成が成立せず、主要デモが再現不能

仕様根拠:
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:24`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:71`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:275`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:533`

実装根拠:
- [traffic-gen.sh](C:\work\github\k1s0\infra\demo\kiali\traffic-gen.sh#L40)
- [traffic-gen.sh](C:\work\github\k1s0\infra\demo\kiali\traffic-gen.sh#L46)
- [traffic-gen.sh](C:\work\github\k1s0\infra\demo\kiali\traffic-gen.sh#L72)
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L155)

再現:
- `C:\Program Files\Git\bin\bash.exe infra/demo/kiali/traffic-gen.sh 1 5`
  - 1ラウンド目で終了コード `1`
- `kubectl exec order-bff-... -n k1s0-service -c istio-proxy -- curl -s -o /dev/null -w "%{http_code}" http://order-server.k1s0-service.svc.cluster.local/`
  - `000` / exit code `56`
- 同一 Pod の `stub` コンテナからは `wget` で `HTTP/1.1 200 OK`

問題:
- 送信元に `istio-proxy` コンテナを使っており、実クラスタで疎通に失敗する
- `demo.sh` と React UI のシナリオ実行はこのスクリプト依存のため、Normal / Canary / Header / Mirror / Fault / Tracing / Logs のデモ基盤が崩れる
- さらに `send_req()` が内部で毎回新しい `span_id` を生成するため、コメントで意図した親子関係のトレースになっていない

是正要求:
- トラフィック生成元を `stub` 側に移すか、送信用サイドカーを別途配置する
- `traffic-gen.sh` を `set -e` 前提で成功する形に直す
- B3 親子関係を実際の送信 span に一致させる

再検収条件:
- `traffic-gen.sh 1 5` が正常終了する
- `sum(rate(istio_requests_total{reporter="destination"}[5m]))` が非ゼロになる
- canary / mirror / fault シナリオ適用後に Kiali で対応する edge が確認できる

### 2. 重大: 分散トレースのデモが実トレースではなく偽装データ

仕様根拠:
- `docs/architecture/observability/可観測性設計.md:491`
- `docs/architecture/observability/可観測性設計.md:505`
- `docs/architecture/observability/可観測性設計.md:793`

実装根拠:
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L354)
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L399)
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L431)
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L465)

再現:
- `http://localhost:16686/api/traces?service=order-bff.k1s0-service&limit=1&lookback=1h`
  - `order-bff.k1s0-service -> graphql-gateway.k1s0-system`
  - `graphql-gateway.k1s0-system handler`
  - という `server.ts` 生成文字列そのままの span 名が返る

問題:
- UI バックエンドが 30 秒ごとに OTLP で人工 span を Jaeger へ投入している
- 実トレース経路が壊れていても、画面上は成功しているように見える
- 検収用デモとして信用できない

是正要求:
- `injectTraces()` と定期投入を削除する
- 実トラフィックから得たトレースのみ表示する
- 必要なら stub アプリ側に最小の OpenTelemetry 実装を入れる

再検収条件:
- 人工 trace 投入コードが除去されている
- トラフィック生成後に Jaeger API で実通信由来の span のみが返る

### 3. 重大: Loki / Promtail のログ集約が機能していない

仕様根拠:
- `docs/architecture/observability/可観測性設計.md:448`
- `docs/architecture/observability/可観測性設計.md:450`
- `docs/architecture/observability/ログ設計.md:118`
- `docs/architecture/observability/ログ設計.md:128`

実装根拠:
- [08-promtail.yaml](C:\work\github\k1s0\infra\demo\kiali\manifests\08-promtail.yaml#L21)
- [08-promtail.yaml](C:\work\github\k1s0\infra\demo\kiali\manifests\08-promtail.yaml#L25)
- [08-promtail.yaml](C:\work\github\k1s0\infra\demo\kiali\manifests\08-promtail.yaml#L37)

再現:
- Grafana Loki datasource proxy 経由で `{namespace="k1s0-service"}` を query
  - 結果 0 件
- Loki ログでも query `returned_lines=0`

問題:
- 18 時間稼働済みのクラスタでもアプリログが 1 行も取得できない
- `ScenarioPanel` の `Log Aggregation` は成立していない
- Promtail 設定は pod discovery しているが、収集対象ファイルへ結びつける設定がない

是正要求:
- kind/containerd 前提の pod log path を正しく収集する設定へ修正する
- Grafana から namespace 単位でログが見えることを確認する
- `trace_id` を含むログを 1 本でも実際に流す

再検収条件:
- `{namespace="k1s0-service"}` で Loki からログが返る
- `trace_id` の derived field から Jaeger 遷移できる

### 4. 重大: Kafka デモの案内先が誤っており、Topology タブでは TCP フローを可視化できない

仕様根拠:
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:456`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:691`

実装根拠:
- [ScenarioPanel.tsx](C:\work\github\k1s0\infra\demo\kiali\ui\src\components\ScenarioPanel.tsx#L58)
- [DashboardViewer.tsx](C:\work\github\k1s0\infra\demo\kiali\ui\src\components\DashboardViewer.tsx#L76)
- [TopologyView.tsx](C:\work\github\k1s0\infra\demo\kiali\ui\src\components\TopologyView.tsx#L73)
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L285)

再現:
- Prometheus では `istio_tcp_sent_bytes_total` に
  - `kafka-demo-producer -> kafka`
  - `kafka-demo-consumer -> kafka`
  - が存在
- 一方で `server.ts` の `/api/topology` は `istio_requests_total` のみ参照
- HTTP request 系クエリ結果は 0 件

問題:
- Kafka シナリオの推奨タブは `topology` だが、その API は HTTP メトリクスしか見ていない
- Kafka は TCP フローなので、案内した画面に必要情報が出ない

是正要求:
- Kafka シナリオでは Kiali Graph を直接案内するか、Topology API に TCP 指標を追加する
- 表示単位を `destination_service_name` ベースでも扱えるよう修正する

再検収条件:
- Kafka シナリオ選択時に、推奨画面で producer / consumer から kafka への edge が見える

### 5. 重大: 仕様で定義された VirtualService の timeout / retry が未実装

仕様根拠:
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:24`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:103`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:282`
- `docs/infrastructure/service-mesh/サービスメッシュ設計.md:533`

実装根拠:
- [02-istio.yaml](C:\work\github\k1s0\infra\demo\kiali\manifests\02-istio.yaml#L102)
- `kubectl get virtualservice -A` の結果は `0`
- `infra/demo/kiali` 配下に baseline 用 VirtualService manifest が存在しない

問題:
- DestinationRule はあるが、timeout / retry を担う VirtualService が 1 本も常設されていない
- docs で要求されている tier default と order-server 個別設定が欠落している
- service mesh の重要設計項目である retry policy を検収できない

是正要求:
- baseline の VirtualService を追加する
  - `default-system`
  - `default-business`
  - `default-service`
  - `order-server` 個別 override
- 必要なら scenario 用 VS と競合しないよう apply/remove 戦略を分ける

再検収条件:
- `kubectl get virtualservice -A` で baseline VS が常時確認できる
- manifest 内に docs 記載の timeout / retry 値が反映されている

### 6. 中: React Demo UI のシナリオ実行は Windows で不安定

実装根拠:
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L162)
- [server.ts](C:\work\github\k1s0\infra\demo\kiali\ui\server.ts#L165)
- [setup.sh](C:\work\github\k1s0\infra\demo\kiali\setup.sh#L197)

再現:
- `cmd /c bash --version`
  - `C:\Windows\System32\bash.exe` が優先され、WSL 未構成だと失敗

問題:
- 推奨手順は `npm run dev` だが、バックエンドは単に `bash` を spawn している
- Git Bash が入っていても PATH 順序次第で WSL 用 `bash.exe` を踏み、シナリオ実行に失敗する

是正要求:
- `bash` 依存をやめて Node から直接 Kubernetes API を叩く
- もしくは Windows 用に明示的な bash 解決戦略を入れる

再検収条件:
- PowerShell から `npm run dev` で起動し、シナリオ実行が成功する

## 総評

セットアップ導線自体は成立し、Kiali / Grafana / Jaeger / Prometheus も起動している。しかし、検収対象の本丸である「シナリオを操作して仕様通りの挙動を見せる」部分は成立していない。

特に以下は受け入れ不可:
- トラフィック生成失敗
- トレース偽装
- Loki 無ログ
- Kafka 案内先不一致
- timeout / retry 未実装

上記を是正しない限り、デモは説明用の見栄えに留まり、設計実装の検収物とは認められない。
