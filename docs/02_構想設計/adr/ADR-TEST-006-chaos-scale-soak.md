# ADR-TEST-006: L7 chaos を Chaos Mesh、L8 scale を KWOK 1000 node + k6、soak を 24h 月次で構造化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 採用初期の協力者

## コンテキスト

ADR-TEST-003 のテストピラミッドで L7 chaos / L8 scale + soak を独立層として切り出した。これらは本番リリース前に拾うべきバグが他層と質的に異なるため、層として独立させた以上、**何のツールでどう構造化するか** を ADR で正典化しなければ実装が始まらない。

L7 chaos が拾うバグの典型例は、pod が突然 kill された時の再起動順序、network partition 時のリーダー選出、disk full 時のリトライ戦略、clock skew 時の TLS 証明書検証のようなもので、unit test や smoke E2E では原理的に再現できない「故意に壊す」検証である。L8 scale は KWOK / kubemark のような virtual node 技術で 1000 ノード規模の API server / scheduler / etcd の挙動を検証し、soak は 24 時間連続負荷でメモリリーク / fd leak / SLO 維持を検証する。これらは Kubernetes 本体や CNCF Graduated 級 OSS（Cilium / Istio / ArgoCD）が必ず持っている層で、k1s0 が CNCF Sandbox 申請段階で testing maturity の評価軸として直接見られる。

ツール選定の選択肢は実用上以下のとおり:

- **L7 chaos**: ① Chaos Mesh（CNCF Incubating、PingCAP 主導）② Litmus（CNCF Incubating、Harness 主導）③ Toxiproxy（軽量、Shopify）④ 自前 chaos 注入
- **L8 scale**: ① KWOK（SIG Scalability、kubelet 不要の virtual node）② kubemark（Kubernetes 公式の hollow-node、メンテ縮小）③ 実 hardware 大規模 cluster（コスト不可能）④ scale 検証放棄
- **soak (負荷生成)**: ① k6（Grafana Labs、JS シナリオ + Prometheus 統合）② Locust（Python、分散負荷）③ Vegeta（HTTP 単純）④ JMeter（古典的だが GUI 重視）

加えて、これらの実行頻度（chaos: nightly / scale: 月次 / soak: 月次）と、ローカルマシン占有時間（chaos 30 分〜2 時間 / scale 数時間 / soak 24 時間）が異なるため、**いつ・どの gate で・どれだけマシンを占有するか** の決定が ADR-TEST-001 の「nightly 毎晩実行」「個人マシン占有許容」と整合する形で正典化される必要がある。

選定では以下を満たす必要がある:

- **CNCF 公式系列の選択** が ADR-CNCF-001（vanilla K8s + CNCF Conformance）の哲学と整合する（Toxiproxy のような非 CNCF も許容するが、可能なら CNCF 採択ツールを優先）
- **kind cluster 上で動く** 軽量さ（multipass kubeadm まで上げると nightly 30 分〜2 時間が破綻）
- **シナリオの宣言性** が CRD / YAML で記述でき、コードレビュー対象になる
- **ローカルマシン占有時間** が ADR-TEST-002 のハードウェア要件で吸収可能（RAM 32GB / NVMe 1TB の上限内）
- **負荷生成ツール** が SLO assertion（p99 latency / error rate）を Prometheus メトリクスで判定できる

## 決定

**L7 chaos は Chaos Mesh、L8 scale は KWOK + k6、soak は k6 で 24 時間連続負荷を月次実行する。**

### L7 chaos: Chaos Mesh on kind

- **配置**: kind cluster（ADR-TEST-004 の L7 環境）に Chaos Mesh helm chart を install
- **シナリオ**: `tests/e2e/L7_chaos/experiments/` 配下に Chaos Mesh CRD（PodChaos / NetworkChaos / IOChaos / TimeChaos / StressChaos / DNSChaos）の YAML を配置
- **実行頻度**: `make qualify-nightly` で毎晩実行（30 分〜2 時間）。release tag 直前は `make qualify-release` でも実行
- **シナリオ最低セット**:
  - `pod-kill-tier1.yaml`: tier1 サービス pod を 30 秒間隔で kill、SLO 維持を assert
  - `network-partition-etcd.yaml`: etcd ノード間の通信を 60 秒切断、リーダー選出が完了することを assert
  - `disk-full-pvc.yaml`: PVC を 100% 埋め、PVC 拡張が成功することを assert
  - `clock-skew-control-plane.yaml`: control-plane の時刻を ±10 分ずらし、TLS 証明書検証が継続することを assert
  - `dns-failure-coredns.yaml`: CoreDNS 応答を 50% drop、retry / timeout 設定が機能することを assert
  - `cpu-stress-worker.yaml`: worker ノードに 100% CPU stress、scheduler が他ノードへ pod を退避することを assert
- **失敗判定**: SLO（p99 latency / error rate / availability）が事前定義値を下回ったら fail。assertion は `tests/e2e/L7_chaos/assertions/<scenario>.go` で記述

### L8 scale: KWOK + k6

- **配置**: kind cluster + KWOK（kubelet の振る舞いをシミュレートする virtual kubelet）
- **シナリオ**: `tests/e2e/L8_scale/scenarios/` 配下に KWOK で立てる node 数 / pod 数 / namespace 数を YAML で宣言
- **実行頻度**: `make qualify-soak` で月次実行（24 時間）。手動 cron で起動、結果を desktop notify で通知
- **規模**: Phase 0 は KWOK で 1000 node + 50000 pod + 1000 namespace、Phase 3 で 5000 node に拡張（sponsor cluster 確保時）
- **負荷生成**: k6（Go 製、JS シナリオ）で API server に対し 1000 RPS の API 呼び出しを 24 時間連続実行
- **assertion**:
  - p99 API 呼び出し latency < 1 秒
  - scheduler の pod 配置 latency p99 < 5 秒
  - etcd request latency p99 < 100 ms
  - メモリリーク: 24 時間で 10% 以下の RSS 増加
  - fd leak: ファイルディスクリプタ数の単調増加が無いこと
  - SLI 維持: 24 時間 availability ≥ 99%

### soak

soak は L8 scale と同じ k6 シナリオを 24 時間連続実行し、L7 chaos の `pod-kill-tier1.yaml` を 1 時間ごとに 5 分間注入する複合シナリオも `tests/e2e/L8_scale/scenarios/soak-with-chaos.yaml` として実装する（Phase 0 では soak 単独 + chaos 別、Phase 1 で複合シナリオ追加）。

### マシン占有スケジュール

| ジョブ | gate | 実行時間 | 占有頻度 | 影響 |
|--------|------|---------|---------|------|
| L7 chaos | `make qualify-nightly` | 30 分〜2 時間 | 毎晩 | 開発機を 23:00–01:00 占有 |
| L7 chaos | `make qualify-release` | 30 分〜2 時間 | release tag 時 | release 切る作業の連続 4 時間に含まれる |
| L8 scale + soak | `make qualify-soak` | 24 時間 | 月次 | 開発機を週末 1 日占有 |

開発機の長時間占有は ADR-TEST-002 のハードウェア要件で物理的に支え、`docs/governance/QUALIFY-SCHEDULE.md` に nightly / monthly のスケジュールを公開する。`tools/qualify/notify.sh` で実行開始 / 失敗 / 完了を `notify-send`（Linux）/ macOS notification center に通知する。

## 検討した選択肢

### 選択肢 A: Chaos Mesh + KWOK + k6（採用）

- 概要: L7 chaos に Chaos Mesh、L8 scale の virtual node に KWOK、負荷生成に k6 を使う組み合わせ
- メリット:
  - Chaos Mesh は CNCF Incubating で PingCAP の長期コミットがあり、メンテナンス継続性が高い
  - Chaos Mesh の CRD（PodChaos / NetworkChaos 等）が宣言的で、シナリオを YAML で版管理できる
  - KWOK は SIG Scalability の公式プロジェクトで、kubelet 不要の virtual node により 1 ノードで 1000 ノード規模を再現できる（kubemark より軽量）
  - k6 は Go 製で起動が高速、JS シナリオで複雑な業務ロジックを記述でき、Prometheus / Grafana 統合が標準で備わる
  - 3 ツールとも CNCF / Kubernetes SIG 採択の標準系列、採用組織のスキル流用性が高い
- デメリット:
  - Chaos Mesh は helm chart で controller / dashboard / etc の複数 component を入れる必要があり、kind 起動時間が +30 秒
  - KWOK の virtual node は実 kubelet と挙動が完全一致しないため、kubelet 固有のバグ（CRI 互換性等）は KWOK で検出できない
  - k6 の JS シナリオは TypeScript 化できないため、tier1 の proto schema と型整合が取れない（型崩れリスク）

### 選択肢 B: Litmus + kubemark + Locust

- 概要: Chaos Engineering に Litmus、scale に kubemark、負荷生成に Locust を使う組み合わせ
- メリット:
  - Litmus は CNCF Incubating で、ChaosHub という公開シナリオライブラリが充実
  - kubemark は Kubernetes 本体の test/e2e で長年使われた実績
  - Locust は Python シナリオで、tier1 / tier3 の業務ロジックを Python 側に持つ場合に親和性が高い
- デメリット:
  - **kubemark のメンテ縮小**: 2024 年以降 SIG Scalability が KWOK を後継として推奨しており、kubemark は事実上 deprecated 路線。10 年保守の前提で kubemark に依存すると将来移行が必要になる
  - Litmus は実装言語が Go + TypeScript で、起案者の Go メインスキルと整合するが、Litmus のバージョン互換性が頻繁に壊れる時期がある
  - Locust は分散負荷生成に強いが、k1s0 のリリース時点では分散不要（ローカル 1 台）でオーバースペック
  - 3 ツールの組み合わせとして CNCF 採択 + 公式継続の整合が、選択肢 A より弱い

### 選択肢 C: Toxiproxy + virtual node + ab

- 概要: ネットワーク chaos に Toxiproxy、virtual node を自前実装、HTTP 負荷に ab（Apache Benchmark）を使う軽量組み合わせ
- メリット:
  - Toxiproxy は Shopify 製で軽量、設定が JSON で単純
  - 自前 virtual node は外部依存ゼロ
  - ab は古典的で起動が極めて速い
- デメリット:
  - **Pod chaos / IO chaos / TimeChaos が再現できない**: Toxiproxy はネットワーク chaos に特化しており、pod kill や clock skew は別ツール
  - 自前 virtual node は実装工数が大きく、KWOK の機能を再発明することになる
  - ab は HTTP 単純で、複雑なビジネスシナリオ（gRPC / WebSocket / 認証フロー）を記述できない
  - SLO assertion の Prometheus 統合が無く、自前実装が必要

### 選択肢 D: Chaos / scale / soak を放棄

- 概要: L7 / L8 を実装せず、リリース時点では unit / integration / smoke / standard だけで qualify を完結させる
- メリット:
  - 実装工数ゼロ
  - 開発機の長時間占有が無い
- デメリット:
  - **本番リリース後に chaos / scale 起因のバグが顕在化する**: pod kill 時の再起動順序、メモリリーク、scheduler の大規模時挙動が検証されないまま production に乗る
  - 採用検討者が CNCF Sandbox 採用基準の testing maturity 評価で「chaos / scale を持たない」と判定し、CNCF Sandbox 申請が通らない
  - ADR-TEST-003 で 11 層に分けた決定と矛盾し、L7 / L8 が「未来への先送り」になる

## 決定理由

選択肢 A（Chaos Mesh + KWOK + k6）を採用する根拠は以下。

- **CNCF / SIG 公式系列の整合性**: Chaos Mesh は CNCF Incubating、KWOK は SIG Scalability 公式、k6 は Grafana Labs（k1s0 が ADR-OBS-002 で採用済の Grafana LGTM スタックの本家）製で、3 ツールとも上流のメンテナンス継続性が高い。選択肢 B の kubemark は SIG Scalability が KWOK を後継推奨しており、10 年保守の前提で deprecated 路線に乗るのは合理的でない
- **宣言性とコードレビュー対象化**: Chaos Mesh の CRD（PodChaos / NetworkChaos 等）は YAML で記述され、`tests/e2e/L7_chaos/experiments/` に配置すれば PR レビュー対象になる。シナリオが procedure（shell script）ではなく declaration（YAML）として版管理されるため、ADR-OPS-001 の Runbook 8 セクション形式と同じ思想で chaos シナリオが管理できる。選択肢 C の Toxiproxy + 自前 virtual node はこの宣言性が崩れる
- **ローカルマシンでの 1000 ノード再現性**: KWOK は kubelet 不要の virtual node 実装で、1 物理ノードの上で 1000 virtual node を 30GB RAM 程度で立てられる。kubemark は hollow-kubelet（実 kubelet を mock 化）で重く、1000 ノードに 80GB RAM 必要なので ADR-TEST-002 のハードウェア要件（32GB）に収まらない。選択肢 C の自前 virtual node は実装工数で破綻
- **Grafana LGTM 整合**: k6 は Prometheus exporter / Grafana dashboard を標準で持ち、ADR-OBS-002 で採用済の Grafana LGTM スタックに直接統合できる。ADR-TEST-009（観測性 E2E）で SLO assertion を Prometheus クエリで判定する経路と自然に一致する。選択肢 B の Locust は Prometheus 統合に追加プラグインが要り、選択肢 C の ab は統合不可
- **k1s0 とのスキル整合**: k6 の JS シナリオは tier3 SPA（ADR-TIER3-002）の TypeScript と概念的に近く、起案者の保守スキル流用性が高い。選択肢 B の Locust（Python）は k1s0 のメイン言語（Rust + Go）と乖離する
- **個人 OSS の運用工数最小化**: Chaos Mesh / KWOK / k6 はそれぞれ単独で広く使われており、helm chart / 公式 docs / コミュニティ Q&A が充実。選択肢 C の自前実装は運用工数が線形以上で増え、bus 係数 2 と矛盾

## 影響

### ポジティブな影響

- L7 chaos / L8 scale / soak が CNCF / SIG 公式ツールで構造化され、採用検討者が CNCF Sandbox 採用基準の testing maturity 評価で加点を得られる
- Chaos Mesh の CRD で chaos シナリオが宣言的に版管理され、シナリオ変更が PR レビュー対象になる
- KWOK の軽量 virtual node により、ADR-TEST-002 の HW 要件（32GB RAM）の上限内で 1000 ノード規模が再現できる
- k6 の Prometheus 統合により、SLO assertion が Grafana LGTM スタックと自然に統合される
- 24 時間 soak の月次実行により、メモリリーク / fd leak / SLO 維持の継続的検証が確立する
- nightly chaos / 月次 scale + soak のスケジュールが `docs/governance/QUALIFY-SCHEDULE.md` で採用検討者向けに公開される

### ネガティブな影響 / リスク

- 開発機が毎晩 23:00–01:00 で chaos qualify に占有され、夜間使用に制約が出る。`tools/qualify/notify.sh` で開始通知を出すが、緊急の手元作業時はスケジュール衝突リスクが残る
- 月次 24 時間 soak は週末を 1 日占有するため、起案者の私生活との両立が継続課題。Phase 2 で contributor が増えた段階で実行担当のローテーション化を `docs/governance/RELEASE-PROCESS.md` で規定する必要
- KWOK の virtual node は実 kubelet と挙動が完全一致しないため、kubelet 固有のバグ（CRI 互換性 / cgroup 操作 / image pull retry 等）は L8 では検出できない。これらは L5 conformance（multipass kubeadm）で別途検証する必要があり、層別の責務分界を `docs/governance/QUALIFY-POLICY.md` で明文化する
- Chaos Mesh の helm chart で controller / dashboard / DNS server などの複数 component が入るため、kind cluster の起動時間が +30 秒。L7 を毎晩 1 回起動するため、年間 +3 時間程度の累積待ち時間
- k6 の JS シナリオは proto schema と型整合が取れず、tier1 API の breaking change が k6 シナリオの runtime error として遅延検出される。型整合は `tools/qualify/k6-schema-check.sh` で proto から JSON Schema を生成し起動前 lint で吸収する
- Phase 1 以降で Chaos Mesh / KWOK / k6 のいずれかが上流で破綻した場合、3 ツールの差し替えが必要になる。代替候補（Litmus / kubemark / Locust）の動作確認を Phase 2 で年次にやる規律が要る

### 移行・対応事項

- `tests/e2e/L7_chaos/experiments/` を新設し、Chaos Mesh CRD で 6 つの最低シナリオ（pod-kill / network-partition / disk-full / clock-skew / dns-failure / cpu-stress）を YAML で記述
- `tests/e2e/L7_chaos/assertions/` を新設し、SLO assertion を Go で記述（Prometheus クエリで p99 latency / error rate を判定）
- `tests/e2e/L8_scale/scenarios/` を新設し、KWOK で 1000 node + 50000 pod + 1000 namespace を立てる YAML、k6 で 1000 RPS の負荷シナリオ JS を配置
- `tests/e2e/L8_scale/scenarios/soak-with-chaos.yaml` を Phase 1 で新設し、L7 chaos と L8 soak の複合シナリオを記述
- `tools/qualify/cluster/chaos-mesh-install.sh` を新設し、kind cluster に Chaos Mesh helm chart を冪等 install する
- `tools/qualify/cluster/kwok-install.sh` を新設し、kind cluster に KWOK controller を install する
- `tools/qualify/notify.sh` を新設し、qualify 開始 / 失敗 / 完了を desktop notify で通知（Linux notify-send / macOS osascript の両対応）
- `Makefile` に `qualify-nightly`（L7 chaos）/ `qualify-soak`（L8 scale + soak）target を追加（ADR-TEST-003 と整合）
- `docs/governance/QUALIFY-SCHEDULE.md` を新設し、nightly 23:00–01:00 / 月次 24 時間 soak のスケジュールを採用検討者向けに公開
- `docs/governance/QUALIFY-POLICY.md` に「KWOK は kubelet 固有バグを検出しない、L5 conformance で吸収」の層別責務分界を散文で記述
- `tools/qualify/k6-schema-check.sh` を新設し、proto schema から k6 用の JSON Schema を生成 + 起動前 lint で型整合を吸収
- `ops/runbooks/RB-OPS-003-qualify-nightly-failure.md` を新設し、nightly chaos qualify 失敗時のトリアージ手順を 8 セクション形式（ADR-OPS-001 準拠）で記述
- `infra/observability/grafana/dashboards/qualify-chaos-soak.json` を新設し、chaos / soak 結果を Grafana で可視化（ADR-OBS-002 と整合）

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— nightly 毎晩実行 / マシン占有許容の前提
- ADR-TEST-002（devcontainer + HW 要件）— L7 / L8 のリソース上限の根拠
- ADR-TEST-003（テストピラミッド L0–L10）— L7 / L8 の責任分界
- ADR-TEST-004（kind + multipass 二層 E2E）— L7 chaos が kind 上で動く根拠
- ADR-TEST-005（環境マトリクス）— 層別 matrix 重み付け（L7 は CNI 重視、L8 は arch 重視）
- ADR-OBS-002（Grafana LGTM）— k6 / Chaos Mesh の Prometheus 統合の前提
- ADR-OPS-001（Runbook 標準化）— RB-OPS-003 の形式根拠
- ADR-CNCF-001（CNCF Conformance）— Chaos Mesh / KWOK の CNCF / SIG 公式系列の整合
- NFR-A-CONT-001（HA / RTO 4 時間）— L7 chaos の復旧可能性 assertion
- NFR-B-PERF-001〜007（性能要件）— L8 scale / soak の SLO assertion
- NFR-B-WL-001〜002（バースト負荷）— L8 の 1000 RPS 負荷シナリオの根拠
- Chaos Mesh: chaos-mesh.org
- KWOK: kwok.sigs.k8s.io
- k6: k6.io
- 関連 ADR（採用検討中）: ADR-TEST-007（upgrade / DR）/ ADR-TEST-008（コンプライアンス）/ ADR-TEST-009（観測性 E2E）
