# ADR-POL-002: ローカル kind クラスタの構成 Single Source of Truth を tools/local-stack/up.sh に統一する

- ステータス: Accepted
- 起票日: 2026-04-30
- 決定日: 2026-04-30
- 起票者: kiso ryuhei
- 関係者: Platform/Build / SRE / DX チーム / 全 dev role 利用者 / Security

## コンテキスト

k1s0 はモノレポ上で tier1/tier2/tier3 と SDK・infra・deploy・ops を統合運用する採用側組織向け PaaS であり、10 年保守を前提とする。ローカル開発環境は IMP-DEV-POL-006（ローカルは kind/k3d + Dapr Local で本番再現）を満たすため `tools/local-stack/up.sh` を「kind cluster + 17 レイヤ（cert-manager / metallb / istio / kyverno / spire / dapr / flagd / argocd / cnpg / kafka / minio / valkey / openbao / backstage / observability / keycloak / cni）」の自動構築スクリプトとして整備し、role 別配備・layer 別 skip を制御できるよう設計してある。

しかしリリース直前の運用実態（2026-04-30 監査時点）では、ローカル kind cluster `k1s0-local` 上に **31 個の helm release** と **16 個の永続ボリューム**が存在する一方、これらの大半は up.sh の apply\_\* 関数経由で配備されたものではなく、開発者が個別に `helm install` / `kubectl apply` を実行した結果として堆積していた。具体的には次の 6 カテゴリ・15 種類の構造的 drift が検出された。

- **カテゴリ A: up.sh 想定だが現状不在**: cert-manager・metallb の 2 必須レイヤが namespace ごと欠落。TLS 証明書発行不可・LoadBalancer タイプの Service が PENDING 化する致命的状態で稼働していた。
- **カテゴリ B: up.sh に存在せず後付けで増えた helm release**: argo-rollouts（ADR-CICD-002 で正規採用済み）・envoy-gateway・temporal の 3 系。ADR で採用済みの技術が up.sh に組み込まれない時点で「ADR の死文化」が進行している兆候。
- **カテゴリ C: helm でなく手動 manifest apply**: gitea（argocd の sync 元 git！）・registry（image push/pull 基盤）・keycloak（auth 基盤）・envoy-grpcweb・spiffe-helper の 5 系。再構築のたびに data と設定が消失する状態。argocd の依存先である gitea が手動 apply で動いているのは、GitOps 基盤の自己矛盾。
- **カテゴリ D: 設計外の手動 apply 検証物**: rollout-canary-demo（default ns）・spiffe-helper（spire-test ns）等のテスト用残置物が production 想定にない ns で常駐。
- **カテゴリ E: アプリ層の GitOps 不徹底**: tier1-rust（k1s0-tier1 ns）・tier2-\* 5 件（k1s0-tier2 ns）・tier3-\* 4 件（k1s0-tier3 ns）が argocd Application 化されず helm install のみで稼働。さらに tier1（tier1-facade chart）は argocd Application `tier1-facade-{dev,staging,prod}` と helm release `tier1` の**二重管理**になっており、どちらが master か不明な状態。
- **カテゴリ F: stateful drift**: up.sh が `cnpg-system/k1s0-postgres` を作る設計に対し、実環境では `k1s0-tier1/pg-state` という別名・別 ns の CNPG cluster が手動で立っている。Backstage chart が期待する正規 DB が不在のため、Backstage を chart 経由で立ち上げると DB 接続失敗する状態。

これらの drift は単発事故ではない。発生メカニズムを掘ると **「up.sh で立てる → 開発者が手早く試したいので helm install / kubectl apply で直接デプロイ → up.sh に戻す経路が無いまま堆積 → drift」** という再現性のあるサイクルである。今回の発覚は「Backstage を chart 経由で立てたい」という単純な試運用要望から始まり、依存先 DB 不一致の調査で芋づる式に判明したが、Backstage を後回しにしても drift の堆積自体は止まらない。

drift が止まらない**根本原因は「up.sh が local cluster の構成 Single Source of Truth (SoT) として運用されていない」**ことに帰着する。SoT が宣言されていなければ、どの helm release が正規でどれが drift かを機械的に判定する術がなく、Kyverno admission policy も書けず、CI による drift 検知も書けない。Paved Road（ADR-DEV-001）の思想は「正しい道が最短経路」だが、Paved Road の入口（Scaffold CLI）の先で開発者が降り立つクラスタ自体に正しい道が定義されていなければ、入口が舗装されていてもすぐ砂利道に分岐する。

並行して、k1s0 リリース時点での技術判断として次の前提が確立済みである。

- `tools/local-stack/up.sh` は role 別 layer 配備 / `--skip` 引数 / chart バージョン固定（`readonly *_VERSION`）まで設計済みで、SoT としての拡張余地が残されている（README.md Phase 3 残置項目に Renovate 連動・GHCR digest 固定が予定されている）。
- argocd（ADR-CICD-001）は既に local-stack 内に正規配備されており、tier1-facade の 3 環境（dev/staging/prod）が GitOps 経由で sync 済み。argocd の sync 元として gitea が同 cluster 内に立っているため、外部 git ホスティングへの依存なしに GitOps を成立させられる構成が既にある。
- Kyverno（ADR-CICD-003）は admission policy の強制実行基盤として配備済みで、ADR-POL-001（dual-ownership）で「Platform 側 vs Tenant 側の所有境界を policy で表現する」運用パターンが確立されている。drift 防止 policy は同パターンの延長で実装可能。
- リリース時点で SoT を制定しないまま小規模運用に入ると、運用拡大期に「個々の流儀」が定着し、それが組織を超えて引き継がれる。ADR-DEV-001 が思想レベルで指摘したのと同じ構造的リスクが、クラスタ構成レイヤでも進行する。

これらを踏まえると、SoT 制定は技術的判断としても組織運用としてもリリース時点で確定させるべき決定であり、後回しにする合理的根拠は無い。

## 決定

**ローカル kind クラスタ `k1s0-local` の構成 Single Source of Truth を `tools/local-stack/up.sh` に統一する。**

確定事項は以下のとおり。

- **構成定義の単一化**: kind cluster 上の **インフラレイヤ（カテゴリ A・B・C・F）** の helm release / manifest は **up.sh の apply\_\* 関数経由でのみ** 配備可能とする。手動 `helm install` / `kubectl apply` での恒久的な配備は禁止する。例外は次項の ephemeral 用途のみ。
- **アプリレイヤの GitOps 必須化**: **アプリ層（カテゴリ E: tier1 / tier2 / tier3 / 各 BFF・Web）** の helm release は **argocd Application 経由でのみ** 配備可能とする。手動 `helm install -n k1s0-tier*` は禁止する。argocd の sync 元 git は同一 cluster 内 gitea（ローカル運用）または production 用の外部 git（運用フェーズ）を使う。
- **ephemeral 探索の許可と境界**: 開発者個人の探索的 `helm install` / `kubectl apply` は次の条件下でのみ許可する。
  - namespace 名が `tmp-*` または `dev-<username>-*` で始まる ephemeral 専用 namespace に限定する。
  - kind cluster の `down.sh` 実行で消失することを前提とし、**永続化を期待しない**。
  - PR コミット前に `kubectl delete ns <name>` で完全削除し、CI が `tmp-*` / `dev-*` namespace の存在を fail とする。
- **drift 防止の三層防御**: SoT 違反を以下の三層で阻止する。
  - **層 1（runtime 強制）**: Kyverno policy `block-non-canonical-helm-releases.yaml` を `infra/security/kyverno/` に追加し、up.sh が管理する known release 名集合（`apply_*` 関数から生成）以外の helm release secret 作成を deny する。`--mode strict` 下で `enforce`、`--mode dev` 下で `audit` のみとする。
  - **層 2（PR 検出）**: `.github/workflows/drift-check.yml` を追加し、PR で `tools/local-stack/up.sh` の expected release set と最新の cluster 状態を diff、追加された未承認 release を検出して PR を fail させる。CI runner が kind cluster を立てて up.sh を `--mode strict` で実行し、現状コミットが SoT に整合しているかを毎 PR で機械検証する。
  - **層 3（mode 切替）**: `up.sh --mode {dev,strict}` 引数で運用モードを切替える。`docs-writer` 等の軽量 role と個人開発の `dev` モードでは Kyverno を audit のみとし、開発者の探索を阻害しない。`infra-ops` / `full` / production-mirror role では `strict` 必須とし、PR レビュー前に SoT 整合を強制する。
- **up.sh への drift 取り込み**: 監査で検出されたカテゴリ A・B・C・F の項目を up.sh の正規 layer に取り込む。具体的には次の改修を行う。
  - 新設 apply 関数: `apply_argo_rollouts` / `apply_envoy_gateway` / `apply_temporal` / `apply_gitea` / `apply_registry`。
  - 既存 apply 関数の再活性化: `apply_keycloak`（chart 経路へ統一、手動 deployment は破棄）。
  - F1 修正: `apply_cnpg` で `manifests/60-cnpg/k1s0-cluster.yaml` の正規 cluster `cnpg-system/k1s0-postgres` を必ず apply するよう導線を確認・修復し、`pg-state` は tier1 専用 cluster `k1s0-tier1/tier1-state` にリネーム（または argocd 配下のアプリ chart に移管）する。Backstage 等の dev tooling 用 DB は `k1s0-postgres` を共有する。
- **アプリレイヤの Application 化**: 監査で検出されたカテゴリ E の 9 件（tier1-rust / tier2 系 5 件 / tier3 系 4 件）を argocd Application 定義に変換し、gitea repo に push する。tier1（helm release）は argocd Application `tier1-facade-*` に集約し、二重管理状態を解消する。envoy-grpcweb は tier1 chart に統合する。
- **削除対象**: spiffe-helper（spire-test ns）と rollout-canary-demo（default ns）は常駐対象から外し、検証時のみ on-demand で起動する runbook（`docs/40_運用ライフサイクル/local-stack-rebuild.md` 内）に切り出す。
- **運用ガードレール**: `up.sh --mode strict` の運用は `infra-ops` / `full` / `production-mirror` role で必須化する。役割別の mode 既定値は up.sh の `ROLE_LAYERS` と並列に `ROLE_MODE` として定義する。

### スコープ

本 ADR は **ローカル kind cluster `k1s0-local`** の構成 SoT に限る。次は対象外。

- production / staging クラスタの構成 SoT（別途 ADR-INFRA 系列で扱う想定）
- Dev Container 内 toolchain の構成（[ADR-DEV-001](./ADR-DEV-001-paved-road.md) と IMP-DEV-DC-\* 系列）
- ホスト OS / Docker ランタイム選定（[ADR-DEV-002](./ADR-DEV-002-windows-wsl2-docker-runtime.md)）
- Backstage 自体の採用判断（[ADR-BS-001](./ADR-BS-001-backstage.md) で確定済み）

### Paved Road / Kyverno dual-ownership との関係

ADR-DEV-001（Paved Road）は「正しい道が最短経路」を Scaffold / Golden Path レベルで定義したが、本 ADR はその下層の「クラスタ構成レイヤでも同様の SoT 一本化を行う」ものであり、思想として垂直方向に整合する。Paved Road を外れる自由（自己責任）の枠組みも、本 ADR の「ephemeral namespace に限定した探索許可」として継承される。

ADR-POL-001（Kyverno dual-ownership）は Platform 所有 namespace と Tenant 所有 namespace の境界を policy で表現する運用パターンを定義した。本 ADR の drift 防止 Kyverno policy は同パターンの延長として実装され、Platform 所有レイヤ（up.sh apply\_\* 経由）と Tenant 所有レイヤ（argocd Application 経由 / ephemeral）の境界を release 名集合で表現する。

## 検討した選択肢

### 選択肢 A: SoT 制定 + 三層防御（runtime + CI + mode 切替、採用）

- 概要: 上記決定の通り。up.sh を SoT 化、Kyverno + CI + mode 切替で drift を防止。アプリ層は argocd 必須、個人探索は ephemeral namespace 限定。
- メリット:
  - drift の発生・堆積・検知漏れの 3 経路を独立に塞ぐ三層防御で、単層の破綻が即 drift に繋がらない。
  - mode 切替により「strict が開発体験を殺す」問題を回避し、軽量 role では audit のみで済む。
  - SoT が単一ファイル（up.sh）に集約されるため、新規参加者が「正規構成は何か」を 1 ファイルの読了で把握できる。
  - GitOps 必須化により、production と local の deploy 経路が同一になる（IMP-DEV-POL-006「ローカルは本番再現」の本懐達成）。
  - ADR-POL-001 の dual-ownership パターンを踏襲できるため、既存運用との認知負荷の差分が小さい。
- デメリット:
  - 実装コストが大きい（up.sh への 6 関数追加 / Application 9 件作成 / Kyverno policy / CI workflow / runbook / バックアップ復元自動化）。リリース時点での集中投資が必要。
  - mode 切替の運用ルール（どの role でどちらを既定とするか）を継続的に PMC レビューする必要がある。
  - `tmp-*` / `dev-*` namespace の事前削除を CI が強制するため、開発者の手元クリーニング作業が増える（緩和策として `tools/local-stack/cleanup-ephemeral.sh` を提供する）。

### 選択肢 B: SoT を制定せず現状継続

- 概要: drift を許容し、必要に応じて手作業で再構築する。
- メリット: 短期的な実装コストがゼロ。
- デメリット:
  - 今回の事象（Backstage 試運用で芋づる式に drift 発覚 → 1.5〜2.5 時間の再構築）が再発確実。10 年保守期間で複数回発生すれば、累積コストが選択肢 A の初期投資を上回る。
  - production / staging cluster への drift 伝播リスク（ローカルで動く構成を本番に持ち込む際、ローカル側の drift が混入する経路）が残る。
  - ADR-DEV-001 の Paved Road 思想と矛盾。「正しい道一本化」を Scaffold レイヤで宣言しながらクラスタ構成レイヤで放置するのは、思想の言行不一致。
  - 採用側組織が運用拡大期に入った時、新規参加者が「どれが正規構成か」を学習するコストが線形増加する。

### 選択肢 C: SoT 強制のみ（Kyverno のみ、CI なし）

- 概要: Kyverno で runtime 強制するが、PR 段階の drift 検出 workflow は導入しない。
- メリット: CI 構築コストが浮く。
- デメリット:
  - Kyverno が破られた時（policy 自身のバグ、admission webhook の停止、override で強制 apply された等）に検出できない。
  - PR レビュアーが「この PR で up.sh に追加された apply 関数は実際に動くか」を手で確認する必要がある。10 年保守でレビュー品質が劣化する構造的欠陥。
  - production cluster でも SoT 整合を機械検証する仕組みが流用できなくなる（CI を作っておけば production 側にも横展開可能）。

### 選択肢 D: CI 検出のみ（Kyverno なし）

- 概要: PR で drift 検出するが、runtime での admission 拒否は行わない。
- メリット: 開発者の手元で `kubectl apply` が即時 fail せず、探索の自由度が保てる。
- デメリット:
  - drift が PR 段階で検出されても、既に local cluster で稼働した状態で開発が進んでいる時、「PR を出すまで気付かない」運用になる。手戻りコストが大きい。
  - admission レベルの強制が無いため、drift が cluster 内に発生してから検出までのタイムラグが大きい（数時間〜数日）。その間、drift 構成に依存した PR や ADR が並行で書かれるリスク。
  - production / staging cluster で同じ設計を採用する場合、admission policy が無いと「ローカルで通った→本番でも通るはず」という暗黙の期待が裏切られる経路が残る。

### 選択肢 E: 完全閉鎖モデル（全 helm install 禁止、argocd 一本化）

- 概要: インフラレイヤも含め全ての deploy を argocd Application に統一し、`up.sh` を argocd と root Application を立てるだけのスクリプトに削減する。
- メリット:
  - 「全てが argocd」で経路が一本化され、認知負荷が最小。
  - production と完全に同じ deploy 経路になる。
- デメリット:
  - ローカルで argocd 自体が壊れた時の bootstrap 経路が成立しない（chicken-and-egg 問題）。argocd を立てる前段で誰かが何かを apply する必要があり、その「誰かが何か」を up.sh が担う以上、SoT の一階層は up.sh に残る。
  - cert-manager / cni / istio 等の cluster-scoped CRD・Webhook を argocd で管理すると、CRD 同期と admission webhook の起動順序問題（rolling install 中の admission 失敗）が発生しやすい。
  - 個人の探索的試行（`tmp-*` namespace でのワンショット動作確認）が argocd 経由になると、Application 定義作成 → git push → sync 待ちの遅延が発生し、開発体験を殺す。

### 選択肢 F: 構成 SoT を docs に置く（コードでなく文書で表現）

- 概要: 「local cluster 構成は `docs/40_運用ライフサイクル/local-stack-baseline.md` に記載した状態を正規とする」と文書で宣言し、機械的強制を行わない。
- メリット: 実装コストがほぼゼロ。
- デメリット:
  - 文書と実装の乖離が時間とともに必ず発生する。ADR-DEV-001 が「docs を一次ソースとせず examples を一次ソースとする」と決めたのと同じ理由で、SoT を文書に置く設計は構造的に劣る。
  - 機械的強制が無いため、drift の発生確率は選択肢 B（現状維持）と差が無い。

## 決定理由

選択肢 A（SoT 制定 + 三層防御）を採用する判断は、以下の比較軸で他の選択肢を退けた結果である。

- **drift 再発確率の最小化（B / F を退けた最大の理由）**: 今回の事象は単発事故ではなく、「SoT 不在」という構造的欠陥の症状である。SoT を制定しない（B）または機械強制無しで宣言だけする（F）と、再発を構造的に防げない。10 年保守を前提とする以上、再発確率の構造的低減は短期実装コストを正当化する。

- **多層防御の必要性（C / D を退けた理由）**: drift 防止は単層では破綻する。Kyverno のみ（C）では admission 経路を経ない設定変更（kubectl edit、cluster API 直接呼び出し、policy 自身のバグ）に対応できない。CI のみ（D）では検出までのタイムラグが許容できない。両層を組み合わせれば「runtime での即時拒否（Kyverno）」と「PR 段階での網羅的検証（CI）」が独立に機能し、片方が破綻してももう片方が後ろ盾になる。

- **bootstrap 整合（E を退けた理由）**: 完全 argocd 一本化は理論的に美しいが、「argocd 自身を立てる経路」が必ず argocd の外側に必要となる構造的限界がある。up.sh はその「argocd の外側の最小経路」として既に機能しており、これを SoT として固定するのが現実解。production / staging の bootstrap も同じ構造（argocd の外に最小スクリプトが要る）であるため、本決定は本番構成にも横展開可能なパターンを提示する。

- **Paved Road 思想との整合（B / F を退けた理由）**: ADR-DEV-001 は「正しい道一本化」を上位思想として確定済み。クラスタ構成レイヤでこれを骨抜きにすると、Paved Road の入口で舗装した道がクラスタ降臨地点で砂利道に変わる。思想の垂直整合は 10 年保守における新規参加者の認知負荷低減に直結する。

- **既存資産の活用（A 採用の積極理由）**: up.sh は既に role 別 layer / `--skip` / version 固定の設計を持ち、argocd と Kyverno は既に local-stack 内に正規配備されている。SoT 化に必要な基盤は既に存在し、本 ADR は「既にある部品を SoT として明示宣言する」+「不足分（mode 切替・追加 apply 関数・Kyverno policy・CI workflow）を補完する」で完結する。新規技術導入を伴わないため、追加 ADR を要する設計判断が無い。

- **dual-ownership パターンの再利用（A 採用の追加理由）**: ADR-POL-001 で確立した「Platform 所有 vs Tenant 所有」の policy 表現パターンが、本 ADR の「up.sh 所有 vs argocd 所有 vs ephemeral 個人所有」の三層所有モデルにそのまま適用できる。Kyverno policy の記法と運用フローが既存運用と一貫し、SRE / Platform チームの認知負荷増加が最小。

これらの軸を総合した時、A 以外を選ぶ積極的理由は実装コスト回避（B / F）か単層強制の完結性（C / D）か理論的純粋性（E）のみであり、いずれも 10 年保守期間の累積コスト・思想整合性・bootstrap 現実解の前で優位性を失う。

## 影響

### ポジティブな影響

- ローカル kind cluster の構成 SoT が単一ファイル（`tools/local-stack/up.sh`）に集約され、新規参加者が正規構成を 1 ファイル読了で把握できる。Paved Road 思想がクラスタ構成レイヤまで貫徹される。
- drift の発生・堆積・検知漏れが Kyverno（runtime）+ CI（PR 段階）+ mode 切替（運用境界）の三層で独立に阻止され、単層破綻が即事故に直結しない構造になる。
- アプリレイヤが argocd Application 必須化されることで、ローカルと production の deploy 経路が同一となり、IMP-DEV-POL-006「ローカルは本番再現」の本懐が達成される。「ローカルで動いたが本番で動かない」経路が原理的に消失する。
- 採用側組織が拡大段階に入った時、新規参加者は「up.sh を読んで `--role <自分のロール>` で起動する」という単一手順で本番再現環境に到達でき、time-to-first-commit（ADR-DEV-001 の DX SLI）が悪化しない。
- 今回の事象（Backstage 試運用で 1.5〜2.5 時間の再構築発覚）が再発しない。仮に同種の drift が発生しても CI で PR 段階で検出され、再構築コストが発生する前に修正される。
- `up.sh --mode strict` の存在により、production-mirror role で「本番と同じ強制度のローカル検証」が成立する。

### ネガティブな影響 / リスク

- リリース時点での実装投資が大きい（up.sh への 6 関数追加 / argocd Application 9 件作成 / Kyverno policy / CI workflow / runbook / バックアップ復元自動化）。リリーススケジュールへの圧迫リスクが残る。
- 三層防御の運用が継続的にメンテ対象となる。Kyverno policy の known release set 更新を up.sh の改修と同期させるルール（PR テンプレートで強制）を設けないと、policy 側が陳腐化する恐れがある。
- ephemeral namespace（`tmp-*` / `dev-*`）の自動削除を CI が強制するため、開発者が「明日続きをやろう」と残した namespace が消える事故が起きうる。緩和策として `tools/local-stack/cleanup-ephemeral.sh` の dry-run モードと、削除前の `kubectl get -n <ns> -o yaml > /tmp/<ns>.snapshot.yaml` 自動実行を提供する。
- mode 切替の既定値選定で開発体験と統制のバランスを誤ると、`dev` モードが緩すぎて drift が滲み出す / `strict` モードが厳しすぎて開発が止まる、のいずれかに転ぶ。リリース後 3 ヶ月で PMC レビューを必須化する。
- 既存の手動運用に慣れた開発者が一時的に生産性低下を体験する。「これまで helm install で済んだ作業が argocd Application 起票になる」変化への移行コスト。緩和策として `tools/local-stack/scaffold-application.sh` で Application 定義雛形を生成する CLI を提供する。

### 移行・対応事項

- `tools/local-stack/up.sh` を改修し、以下の apply 関数を追加・改修する。
  - 新設: `apply_argo_rollouts`（ADR-CICD-002 連動）/ `apply_envoy_gateway` / `apply_temporal` / `apply_gitea` / `apply_registry`。
  - 改修: `apply_keycloak`（chart 経路統一、手動 deployment 破棄手順を runbook 化）/ `apply_cnpg`（k1s0-postgres の確実 apply、pg-state の扱い決定）。
  - 引数追加: `--mode {dev,strict}`、`ROLE_MODE` テーブルで role 別既定値を定義。
- `tools/local-stack/manifests/` 配下に上記新設レイヤの values.yaml / manifest.yaml を配置（既存パターンに揃える）。
- `infra/security/kyverno/block-non-canonical-helm-releases.yaml` を新規作成し、`infra/security/kyverno/kustomization.yaml` に登録する。known release set は up.sh の apply 関数名から自動生成する仕組み（`tools/local-stack/known-releases.sh`）を整備し、policy ファイル更新を up.sh 改修と同期する。
- `.github/workflows/drift-check.yml` を新規作成し、PR で kind cluster を立てて up.sh `--mode strict` を実行、`helm list -A` の expected 集合との diff を機械検証する。
- argocd Application 定義 9 件（tier1-rust-{dev,staging,prod} / tier2-\* 5 件 / tier3-\* 4 件）を `deploy/argocd/applications/` 配下に追加し、gitea repo に push する。tier1 helm release は削除し、argocd Application `tier1-facade-*` に集約する。envoy-grpcweb は tier1-facade chart に統合する。
- `pg-state`（k1s0-tier1 ns）と `k1s0-postgres`（cnpg-system ns）の責務分離を明文化する。tier1 専用 state は `k1s0-tier1/tier1-state` にリネーム、Backstage 等 dev tooling 用は `cnpg-system/k1s0-postgres` を共有とする。
- `docs/40_運用ライフサイクル/local-stack-rebuild.md` runbook を新規作成し、「down → backup → up → restore → 検証」の手順と、spiffe-helper / rollout-canary-demo の on-demand 起動手順を記載する。
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` と `docs/05_実装/99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md` に ADR-POL-002 セクションを追加する。
- `docs/02_構想設計/adr/README.md` の「ポリシー」節を 1 件 → 2 件に更新する。
- `docs/SHIP_STATUS.md` の H 系列に「H4: ローカル SoT 統一（ADR-POL-002）」エントリを追加し、production carry-over verification matrix に「local cluster drift check 自動化」を加える。
- リリース後 3 ヶ月で PMC レビューを実施し、`--mode dev/strict` の既定 role 配分・ephemeral namespace の運用実績・drift 再発有無をレビューする。再発が観測された場合は本 ADR の policy / CI 設計を見直し、必要なら新 ADR で補強する。
- 開発者向けオンボーディング資料に「ephemeral namespace の使い方」「argocd Application を起票する手順」を必須項目として追加する。

## 参考資料

- [ADR-DEV-001: 開発者体験の根幹思想として Paved Road を採用](./ADR-DEV-001-paved-road.md)
- [ADR-DEV-002: Windows 11 + WSL2 環境の Docker ランタイムに WSL ネイティブ docker-ce を採用](./ADR-DEV-002-windows-wsl2-docker-runtime.md)
- [ADR-CICD-001: Argo CD で GitOps 配信を行う](./ADR-CICD-001-argocd.md)
- [ADR-CICD-002: Progressive Delivery に Argo Rollouts を採用](./ADR-CICD-002-argo-rollouts.md)
- [ADR-CICD-003: ポリシー適用に Kyverno を採用](./ADR-CICD-003-kyverno.md)
- [ADR-POL-001: Kyverno による Platform / Tenant の dual-ownership 表現](./ADR-POL-001-kyverno-dual-ownership.md)
- [ADR-BS-001: 開発者ポータルに Backstage を採用](./ADR-BS-001-backstage.md)
- [tools/local-stack/README.md](../../../tools/local-stack/README.md)
- [tools/local-stack/up.sh](../../../tools/local-stack/up.sh)
- IMP-DEV-POL-006: ローカルは kind/k3d + Dapr Local で本番再現
- IMP-DEV-DC-014: ローカル Kubernetes と Dapr Local の統合
- [ADR-TEST-002: E2E テストを kind cluster + tools/local-stack + reusable workflow で自動化](./ADR-TEST-002-e2e-automation-via-local-stack.md) — `--role e2e` で SoT を E2E に拡張
- [ADR-TEST-003: CNCF Conformance を Sonobuoy + kind multi-node + Calico で月次実行](./ADR-TEST-003-cncf-conformance-sonobuoy.md) — `--role conformance` で SoT を Conformance に拡張
- 監査ログ: 2026-04-30 セッション（drift 31 helm release / 16 PVC / 6 カテゴリ・15 種類）
