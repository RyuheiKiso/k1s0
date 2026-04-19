<div align="center">

# k1s0

**JTC 情シスのための、OSS 積み上げ型マイクロサービス基盤プラットフォーム**

*レガシーと共存し、オンプレで完結し、ベンダーに縛られない*

![status](https://img.shields.io/badge/status-Phase_0_%E7%A8%9F%E8%AD%B0%E6%89%BF%E8%AA%8D%E5%BE%85%E3%81%A1-orange)
![version](https://img.shields.io/badge/version-draft_v0.1-blue)
![license](https://img.shields.io/badge/license-TBD_(OSI_%E6%89%BF%E8%AA%8D%E4%BA%88%E5%AE%9A)-lightgrey)
![stack](https://img.shields.io/badge/stack-Kubernetes_%2B_Dapr_%2B_Istio_Ambient-0db7ed)
![lang](https://img.shields.io/badge/lang-Go_%2B_Rust_%2B_C%23%2FGo%2FReact-2b7489)

[企画書 (Marp)](./docs/01_企画/企画書.md) &nbsp;·&nbsp; [全体構成図](./docs/01_企画/全体構成図.md) &nbsp;·&nbsp; [構想設計](./docs/02_構想設計/) &nbsp;·&nbsp; [要件定義](./docs/03_要件定義/) &nbsp;·&nbsp; [プロジェクト規約](./CLAUDE.md)

</div>

---

## 概要

> **「古い単一技術のモノリス集合体が崩れかけのジェンガ状態」** になった JTC 情シスに対し、OSI 承認 OSS のみを積み上げて内製する **オンプレ完結のマイクロサービス基盤** を提供する。クラウド依存を排し、.NET Framework 資産を切り捨てず共存させ、tier2 / tier3 の開発者を横断的関心事から解放する。

<table>
<tr>
<td width="33%" align="center"><b>無償で始める</b><br/><sub>OSI 承認 OSS のみで構成<br/>稟議ハードルゼロ</sub></td>
<td width="33%" align="center"><b>レガシー共存</b><br/><sub>.NET Framework 資産を<br/>tier3 拡張として抱え込み</sub></td>
<td width="33%" align="center"><b>オンプレ完結</b><br/><sub>クラウド依存なし<br/>閉域ネットワーク対応</sub></td>
</tr>
<tr>
<td width="33%" align="center"><b>言語の自由</b><br/><sub>C# / Go / MAUI / React<br/>tier2 / tier3 で自由選択</sub></td>
<td width="33%" align="center"><b>Dapr 隠蔽</b><br/><sub>tier1 ファサードで<br/>OSS ロックインを回避</sub></td>
<td width="33%" align="center"><b>TCO 17&ndash;31% 安</b><br/><sub>中規模 3,000 名<br/>5 年累計 商用 IDP 比</sub></td>
</tr>
</table>

---

## なぜ作るか

JTC 情シスは次の **4 つの痛み** を同時に抱えており、単一製品の選定ではどれか 1 つしか解けない。

| 痛み | 影響 |
|---|---|
| レガシー .NET Framework 資産の固着 | 捨てるに捨てられず、新規開発の足を引っ張り続ける |
| 横断的関心事のコピペ実装 | 認証 / ログ / 監視を各サービスが独自に抱え、業務ロジックに集中できない |
| 端末への手動アプリインストール | PC リプレース時に情シスが数人月消耗、退職者権限剥奪も遅延 |
| 商用基盤の高額ライセンスとベンダーロックイン | 稟議が通らない / 契約後の撤退コストが膨大 |

商用 IDP (OpenShift / Tanzu / Humanitec / Mia-Platform) は Web 系テック企業や大規模 SaaS を主対象とし、**クラウドマネージド前提で組まれているため、JTC 情シスが直面する 4 条件 (レガシー共存・オンプレ完結・既存言語資産の尊重・高額ライセンス回避) を同時には満たせない**。k1s0 はこの空白地帯を埋める位置取りで、中規模 (3,000 名) 5 年累計で **商用 IDP 比 3.68 億 vs 4.43〜5.32 億 (17〜31% 安)**、撤退コスト **実質 0 円** (OSS 資産を社内で保持継続) を定量根拠として提示する。

---

## アーキテクチャ

![k1s0 レイヤ間相互作用](./docs/01_企画/img/レイヤ間相互作用.svg)

k1s0 は責務レイヤ (**アプリ層 / ネットワーク層 / インフラ層 / データ層**) を水平スイムレーンとして配置し、その中に tier1〜tier3 を入れ子で積み上げる構造をとる。`tier3 → tier2 → tier1 → infra` の **1 方向依存** を厳守し、infra への直接依存は **tier1 のみ** に許可する。「tier1 は tier2 を楽にするために、tier2 は tier3 を楽にするために存在する」を一貫した設計信念とする。

### tier 構成と責務

| tier | 担当 | 役割 | 言語 |
|---|---|---|---|
| **infra** | インフラ | Kubernetes / サービスメッシュ / 観測基盤 / メッセージング | OSS をそのまま採用 (自作なし) |
| **tier1** | システム基盤 | 11 種の公開 API を統一提供 (Service Invoke / State / PubSub / Secrets / Binding / Workflow / Log / Telemetry / Decision / Audit-PII / Feature) | Go (Dapr ファサード) + Rust (自作領域) + 各言語クライアントライブラリ |
| **tier2** | ドメイン開発 | 業務ドメインロジック。tier1 公開 API のみで基盤機能にアクセス | C# / Go 自由 |
| **tier3** | アプリ開発 | UI / API / 配信ポータル。Dapr / OSS を一切意識しない | C# / Go / MAUI / React 自由 |
| **operation** | 運用 | 監視 / オンコール / リリース管理 / 運用手順 | (横断) |

下位 tier ほど複雑性を引き受け、上位 tier ほど参入コストが下がるのは設計上の意図である。完全な俯瞰図は [`docs/01_企画/全体構成図.md`](./docs/01_企画/全体構成図.md) を参照。

### tier1 の核心 — Dapr ファサードによる隠蔽

tier1 は Dapr を内部で利用するが、tier2 / tier3 には Dapr の存在を一切露出させない。**Dapr のバージョン更新・破壊的変更・将来差し替えの影響を tier1 内に閉じ込め**、ベンダーロックインならぬ「OSS ロックイン」も構造的に回避する。

```csharp
// tier2/tier3 のコード — Dapr を一切意識しない
await k1s0.Log.Info("注文を受領", new { orderId });
await k1s0.State.SaveAsync("orders", orderId, order);
await k1s0.PubSub.PublishAsync("order-events", "created", order);
await k1s0.Audit.RecordAsync("ORDER_CREATED", userId, orderId);
```

tier2 / tier3 が隠蔽を破って Dapr SDK を直接呼ぶのを防ぐため、**雛形生成 CLI / Opinionated API / CI ガード / リファレンス実装 / PR チェックリスト / 内製 analyzer** の 6 段多層防御を敷く。機械で遮断できる逸脱 (静的言語の禁止 import) は CI で、残余 (動的言語のリフレクション経由呼び出しや設計思想逸脱) は人のレビューで捕捉し、頻出パターンを内製 analyzer に吸い上げる。

> **サービスメッシュは Istio Ambient Mesh を採用 (ADR-0001)**
> 従来の Sidecar モードは Dapr サイドカーと Pod 内で二重注入の構造的衝突を起こすが、Ambient Mesh はサイドカー不注入で根本回避する。L4 は ztunnel DaemonSet が HBONE mTLS を、L7 は waypoint proxy が AuthorizationPolicy / リトライ / 負荷分散を担う。

---

## 主要技術スタック

k1s0 は **「ベンダー中立・CNCF 中心・OSI 承認ライセンス限定」** という選定方針のもと、中核 OSS を組み合わせて構築する。

<details>
<summary><b>主要コンポーネント一覧 (クリックで展開)</b></summary>

<br/>

| 領域 | 採用 | 選定理由 |
|---|---|---|
| コンテナオーケストレーション | **Kubernetes** | 業界標準。OSS で代替なし |
| サービスメッシュ | **Istio Ambient Mesh** | Dapr サイドカーとの二重注入を構造的に回避 |
| API Gateway | **Envoy Gateway** | k8s Gateway API 準拠。Istio と Envoy 統一 |
| メッセージング | **Apache Kafka** (Strimzi) | イベントソーシング前提 |
| 観測性 | **OpenTelemetry + LGTMP** (Tempo / Pyroscope / Loki / Prometheus / Grafana) | ベンダー中立。統合スタック |
| Building Blocks | **Dapr** (CNCF Graduated) | tier1 内部実装に組込、tier2 / tier3 から不可視 |
| tier1 (Dapr ファサード) | **Go** | Dapr Go SDK が stable、k8s エコシステムと整合 |
| tier1 (自作領域) | **Rust** | メモリ安全 / 長期保守性 / ZEN Engine 統合 |
| ルールエンジン | **ZEN Engine** (Rust / MIT) | 決定表で稟議 / 権限ポリシー / 業務ルールを宣言化 |
| ワークフロー (短期) | **Dapr Workflow** | 秒〜数分 / 10 ステップ以下の Saga |
| ワークフロー (長期) | **Temporal** | 数時間〜数週間 / 人的承認 / SLA タイマー |
| RDBMS | **CloudNativePG + PostgreSQL 17** | k8s ネイティブ HA |
| シークレット管理 | **OpenBao** (MPL 2.0、Vault LF fork) | 動的シークレット / Transit 暗号化 / PKI |
| オブジェクトストレージ | **MinIO** (AGPL-3.0) | S3 互換 (infra 内部限定で AGPL 義務発動なし) |
| 認証 | **Keycloak** | OIDC で全コンポーネント SSO 統一 |
| 開発者ポータル | **Backstage** (CNCF Incubating) | サービスカタログ / TechDocs / Software Templates |
| IaC | **OpenTofu** (CNCF Sandbox / MPL 2.0) | VM / ネットワーク / k8s 構築を宣言化 |
| Feature Flag | **OpenFeature + flagd** (CNCF Incubating) | 段階的ロールアウト / レガシー共存の制御弁 |
| Chaos Engineering | **Litmus** (CNCF Incubating) | 縮退動作を自動検証、設計を仕様に変える |
| Policy as Code | **Kyverno** | Pod セキュリティ / image pull / Admission 強制 |

</details>

ワークフロー基盤を **2 つ併用** (Dapr Workflow + Temporal) するのは、短期・簡易フローと長期・複雑フローの特性が大きく異なるためである。tier2 / tier3 からは `k1s0.Workflow.StartAsync(workflowType, input)` の同一 API のみを叩き、tier1 Go ファサードが workflowType の属性 (YAML 設定) を見て自動振り分けする。2 基盤併用固有工数は中央値 **0.5 人月/年 (上振れ 1.0)**、全運用工数 20.4 人月への感度は +2.5〜5% に収まり、統合案のカスタマイズ工数より安い。

完全な一覧と選定根拠は [`docs/02_構想設計/03_技術選定/`](./docs/02_構想設計/03_技術選定/) を参照。

---

## ロードマップ

**20+ の OSS コンポーネントを 1 人で構築・運用するリスク** を避けるため、MVP を 2 段階に分割する。MVP-0 で「動くもの」を 2 週間で実演して協力者を獲得し、MVP-1 は 2 名体制で進めることで **バス係数 1 のリスクを構造的に解消** する。

| フェーズ | 位置付け | 主な成果物 |
|---|---|---|
| **Phase 0** | 企画承認待ち (**現在地**) | 企画書 / 技術選定 / 競合分析 / 要件定義ひな形 |
| **Phase 1a (MVP-0)** | デモ構成、起案者単独、VM 1 台 (4 vCPU / 8 GB / 100 GB) | kubeadm + Dapr + Keycloak SSO + 配信ポータル実演 |
| **Phase 1b (MVP-1)** | パイロット運用、2 名体制、VM 3 台 (16 vCPU / 32 GB / 500 GB) | infra フルスタック + Backstage + Argo CD + OpenTofu |
| **Phase 2** | 機能拡張、2〜3 名協力体制 | tier1 拡張 + tier2 サンプル + 端末台帳 + Backstage プラグイン |
| **Phase 3** | エンドユーザー体験の拡充 | tier3 サンプル + ネイティブ配信 + 端末設定コピー |
| **Phase 4** | 業務運用 | 申請ワークフロー / レガシー .NET Framework 共存 |
| **Phase 5** | 全社ロールアウト | 本番展開 / マルチクラスタ / マルチリージョン |

Phase 1a (単一 VM) → Phase 2 (2 ラック分散) → Phase 3 (同一リージョン 2 クラスタ) → Phase 5 (マルチリージョン) と、段階的に耐障害性を拡張する。詳細は [`docs/01_企画/03_ロードマップと体制/`](./docs/01_企画/03_ロードマップと体制/)。

---

## ドキュメント構成

本リポジトリは稟議通過前のドキュメント整備フェーズにあり、現在は `docs/` 以下の資料群が中心となる。ソースコードは稟議通過と MVP-0 承認後に `src/` 配下へ追加する。

| 層 | パス | 内容 | 読むタイミング |
|---|---|---|---|
| 企画 | [`docs/01_企画/`](./docs/01_企画/) | 稟議向け薄い提案書 (背景 / 競合 / ロードマップ / 定量試算 / 法務) | 稟議判断の**最初** |
| 構想設計 | [`docs/02_構想設計/`](./docs/02_構想設計/) | 技術深掘り (アーキテクチャ / tier1 設計 / 技術選定 / CI/CD / 法務) | 稟議通過**後**の設計フェーズ |
| 要件定義 | [`docs/03_要件定義/`](./docs/03_要件定義/) | IPA 共通フレーム 2013 / 非機能要求グレード 2018 準拠 | 設計と並行 |
| 概要設計 | [`docs/04_概要設計/`](./docs/04_概要設計/) | 要件を実装単位に落とす層 (後続、現在はひな形) | 要件定義**後** |
| 規約 | [`docs/00_format/`](./docs/00_format/) | フォーマット / drawio 記法規約 (ADR-0002) | 執筆時の参照 |
| 学習 | [`docs/90_knowledge/`](./docs/90_knowledge/) | 技術学習用ドキュメント | 任意 |
| 壁打ち | [`docs/99_壁打ち/`](./docs/99_壁打ち/) | 企画段階のブレスト・検討メモ | 任意 (経緯追跡) |

### 初読の推奨順 — 稟議判断モード

1. [`docs/01_企画/企画書.md`](./docs/01_企画/企画書.md) — 10 分で読み切れる Marp 形式のエグゼクティブサマリ
2. [`docs/01_企画/全体構成図.md`](./docs/01_企画/全体構成図.md) — 30 秒で全体像を掴む 1 枚俯瞰図
3. [`docs/01_企画/01_背景と目的/`](./docs/01_企画/01_背景と目的/) — 何を解決したいか / 誰のためか / 撤退条件
4. [`docs/01_企画/02_競合と差別化/`](./docs/01_企画/02_競合と差別化/) — 商用 IDP 比較と k1s0 の勝ち筋
5. [`docs/01_企画/03_ロードマップと体制/`](./docs/01_企画/03_ロードマップと体制/) — フェーズ計画 / MVP スコープ / KPI
6. [`docs/01_企画/04_定量試算/`](./docs/01_企画/04_定量試算/) — 5 年 TCO / 開発工数 / 運用工数
7. [`docs/01_企画/05_法務サマリ/`](./docs/01_企画/05_法務サマリ/) — ライセンス / 法令 / 知財 / 監査の即答

---

## 法務コンプライアンス即答ブロック

稟議で必ず刺される **4 論点** について、本リポジトリは以下の即答根拠を提供する。

<table>
<tr><th>論点</th><th>即答</th></tr>
<tr>
<td><b>AGPL OSS の社内利用</b></td>
<td>Phase 1a では AGPL OSS <b>ゼロ</b>。Phase 1b 以降で採用する MinIO / Grafana 派生等 6 本は <code>infra</code> 内部ネットワークのみで動作し、FSF / Grafana Labs 公式 FAQ により義務発動なし (ADR-0003 で隔離アーキテクチャ採択済)</td>
</tr>
<tr>
<td><b>個人情報保護法 / マイナンバー法 / 電帳法</b></td>
<td>k1s0 基盤は業務データを保持しない。必要な技術要件 (監査ログ / 暗号化 / RBAC / ハッシュチェーン改ざん防止) は tier1 公開 API で基盤提供</td>
</tr>
<tr>
<td><b>知財帰属</b></td>
<td>職務著作 (著作権法 15 条) により JTC 法人に帰属。個人契約の保有権構造は不採用</td>
</tr>
<tr>
<td><b>5 年後の監査応答</b></td>
<td>SBOM / CVE / ライセンス判定 / 四半期レポートを CI で自動生成、<code>infra/compliance-reports</code> に 5 年保管</td>
</tr>
</table>

詳細は [`docs/01_企画/05_法務サマリ/`](./docs/01_企画/05_法務サマリ/) および [`docs/02_構想設計/05_法務とコンプライアンス/`](./docs/02_構想設計/05_法務とコンプライアンス/)。

---

## 開発・ドキュメント規約

本リポジトリにコントリビュートする際は [`CLAUDE.md`](./CLAUDE.md) のプロジェクト規約に準拠する。稟議通過前の品質維持のため、以下は特に厳守。

<details>
<summary><b>コーディング規約</b></summary>

<br/>

- 各行の 1 行上に日本語コメント必須、ファイル先頭に日本語ファイル説明コメント必須
- 1 ファイル 300 行以内、超過時は分割 (docs 内は例外)
- tier1 内部サービス間通信は Protobuf gRPC 必須
- tier2 / tier3 から内部言語は不可視 (クライアントライブラリと gRPC エンドポイントのみ公開)

</details>

<details>
<summary><b>ドキュメント規約</b></summary>

<br/>

- アスキー図は禁止。drawio で作成し `img/` に `.drawio` + `.svg` を格納して md に埋め込む
- 表セルに要件本体を流し込むのは禁止。散文で「現状 → 達成後 → 崩れた時」を展開し、表は章末サマリに限定
- ラベル羅列のテーブルは禁止。各セルに根拠・数値・出典を入れ、1 行読んで納得できる密度にする
- 各章冒頭に導入段落、表の前後に解説、関係性は drawio 図で可視化

</details>

<details>
<summary><b>drawio 規約</b></summary>

<br/>

- レイヤ記法規約 ([`docs/00_format/drawio_layer_convention.md`](./docs/00_format/drawio_layer_convention.md)) に準拠
  - アプリ層 = 暖色 / ネットワーク層 = 寒色 / インフラ層 = 中性灰 / データ層 = 薄紫
- XML 生成後、SVG エクスポート前に全矢印と全ボックス・テキストの交差判定を必ず実施
- GitHub ダークテーマ対応のため、`<root>` 直下の最初の要素として白矩形を配置 (背景透明は禁止)
- 白矢印 / 矢印とボックスの重なり / `orthogonalEdgeStyle` 自動ルーティング依存は禁止

</details>

---

## 現在のステータス

<table>
<tr><td width="180"><b>フェーズ</b></td><td>Phase 0 — 稟議承認待ち</td></tr>
<tr><td><b>基準日</b></td><td>2026-04-12</td></tr>
<tr><td><b>版</b></td><td>ドラフト v0.1</td></tr>
<tr><td><b>次のマイルストーン</b></td><td>稟議承認 → MVP-0 (デモ用 VM 1 台確保 → 2 週間以内に SSO ログイン / 配信ポータル / サンプルアプリ起動の実演)</td></tr>
<tr><td><b>直近の決定</b></td><td>ADR-0001 Istio Ambient Mesh / ADR-0002 drawio 図解レイヤ記法 / ADR-0003 AGPL 隔離アーキテクチャ</td></tr>
</table>

---

## ライセンスと知財

- **k1s0 自体**: 職務著作 (著作権法 15 条) により所属法人に帰属予定。稟議通過時に OSI 承認ライセンスで社内公開し、将来的な外部公開可能性も確保する
- **依存 OSS**: OSI 承認ライセンス限定。AGPL OSS は `infra` 内部ネットワークのみで動作する前提で、採用可否を個別 ADR で判定する
- **SBOM**: CI で自動生成、`infra/compliance-reports` に 5 年保管

---

<div align="center">

[企画書](./docs/01_企画/企画書.md) &nbsp;·&nbsp; [全体構成図](./docs/01_企画/全体構成図.md) &nbsp;·&nbsp; [構想設計](./docs/02_構想設計/) &nbsp;·&nbsp; [要件定義](./docs/03_要件定義/) &nbsp;·&nbsp; [プロジェクト規約](./CLAUDE.md)

<sub>本リポジトリは「稟議通過前の Proposed 状態の企画資料」であり、確定した技術決定は ADR として個別に記録する。</sub>

</div>
