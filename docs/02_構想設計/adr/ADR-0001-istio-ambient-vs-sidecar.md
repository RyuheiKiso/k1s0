# ADR-0001: サービスメッシュに Istio Ambient Mesh を採用する

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / インフラチーム / 運用チーム

## コンテキスト

k1s0 は tier1 の building blocks 実装として Dapr を採用済みであり、各アプリ Pod に Dapr サイドカー (`daprd`) が注入される構成が確定している。一方で Phase 2 以降では、tier 間の mTLS・L7 認可・トラフィック分割を担うサービスメッシュとして Istio の導入を計画している。

ここで問題になるのが **Istio サイドカー (Envoy) と Dapr サイドカーの Pod 内二重注入**である。従来の Istio Sidecar モードを採用すると、1 つの Pod に `app container + Envoy sidecar + Dapr sidecar` の 3 コンテナが同居し、以下の構造的衝突が発生する。

- **mTLS 二重掛け**: Dapr-mTLS で暗号化されたトラフィックを Istio Envoy が再度暗号化する。性能オーバーヘッドと証明書期限の二重管理が発生する
- **ポートキャプチャ競合**: Envoy の `iptables REDIRECT` が Dapr の HTTP/gRPC ポート (3500 / 50001 / 50002) を取り込み、Dapr 間通信が破綻する恐れがある
- **AuthorizationPolicy の不整合**: Dapr sidecar → 相手 Dapr sidecar の通信を Istio が「未知の呼び出し」として拒否する
- **起動順序の未規定**: `holdApplicationUntilProxyStarts` と Dapr sidecar-injector の順序制御が仕様化されておらず、アプリ起動時に Dapr が未起動の状態が発生する
- **分散トレースの断絶**: Envoy が挿入する `x-b3-*` ヘッダと Dapr が挿入する W3C Trace Context が分断される

k1s0 の前提制約は次のとおり。

- **2 名で運用し続けられる規模**を維持すること (JTC 情シスの定員制約)
- **tier1 API p99 < 500 ms** (業務ロジック + インフラ合計) の予算を守ること
- **オンプレ / 閉域ネットワーク**で完結すること (クラウドマネージドメッシュは選択肢外)
- **Dapr を tier1 公開 API の内部実装として完全隠蔽**する設計を維持すること

本 ADR は Phase 2 着手前に採択し、Phase 2 の構築計画・工数試算・Runbook 整備の起点とする。

## 決定

**サービスメッシュとして Istio Ambient Mesh モードを採用する。**

- L4 (mTLS / 基本的な通信) は Node 単位に配置される ztunnel が HBONE で担当
- L7 (AuthorizationPolicy / VirtualService / トラフィック分割) は必要な namespace・サービス単位に配置される waypoint proxy が担当
- アプリ Pod 内には Istio 由来のサイドカーを注入しない (Dapr サイドカーとの衝突を構造的に回避)
- Phase 2 着手直後に 1 週間の POC を実施し、Dapr-mTLS と ztunnel HBONE の共存・レイテンシ予算・分散トレースの継承を実測で検証する
- POC が不成立の場合のフォールバックとして、**Istio を採用せず Envoy Gateway + Dapr-mTLS + Argo Rollouts (Gateway API Traffic Splitting) で代替する案**を撤退経路として保持する

## 検討した選択肢

### 選択肢 A: Istio Ambient Mesh モード

- 概要: Istio 1.22 (2024-05 GA) で導入されたサイドカーレスアーキテクチャ。L4 は ztunnel (DaemonSet)、L7 は waypoint proxy (namespace / service 単位で任意配置) で構成される。Pod 内にサイドカーを注入しない
- メリット:
  - Pod 内に Envoy が存在しないため、Dapr サイドカーとの二重注入問題が構造的に消滅する
  - ztunnel の HBONE は Pod IP 単位の Node 間トンネルであり、Pod 内部のポートキャプチャを行わないため Dapr ポートとの競合が発生しない
  - L7 機能が不要な namespace (tier1 / tier2 内部) では waypoint proxy を配置せず L4 のみで回せるため、p99 レイテンシ予算への影響を最小化できる
  - Google / IBM (Red Hat OpenShift Service Mesh 3) / Solo.io / Microsoft AKS / Alibaba Cloud ASM などトップ企業が本番採用している
  - Sidecar モードからの段階移行パスが公式に整備されており、将来方針変更時の可逆性が高い
- デメリット:
  - GA から 2 年 (2026 年時点) であり、Sidecar モードに比べて障害事例・Runbook・外部ノウハウが少ない
  - waypoint proxy の運用設計 (namespace 単位 / service 単位のどちらで配置するか) が新しい設計判断として追加で必要になる
  - Phase 3 以降のマルチクラスタ構成では east-west gateway の扱いが Sidecar モードと異なるため再検証が必要

### 選択肢 B: Istio Sidecar モード + Dapr mTLS 無効化

- 概要: Istio は従来通り Sidecar モードで導入し、Dapr の `configuration.spec.mtls.enabled: false` で Dapr の mTLS 機能を停止する。Pod 内トラフィックは平文、Pod 間は Envoy Sidecar の mTLS に寄せる
- メリット:
  - mTLS の管理主体が Istio に一本化され、監査上「mTLS は Istio 管轄」と単純化できる
  - Microsoft AKS Istio add-on + Dapr の組み合わせとして公式ドキュメントで推奨されるパターンの 1 つ
- デメリット:
  - Pod 内の 3 コンテナ同居は変わらず、起動順序問題・ポートキャプチャ競合・トレース断絶は未解決のまま残る
  - Dapr control plane (placement / operator) との通信も平文化するため、Istio mTLS の STRICT モードを全 namespace に強制する必要があり、運用上の例外を作りにくくなる
  - Dapr を将来撤退して別 Actor モデル実装 (Orleans / Akka 等) に移行する際、「Istio mTLS 依存で組み上げた tier1」を引き剥がすコストが大きくなる
  - Dapr のコードパスに mTLS 無効分岐が存在し続け、「使わない機能を抱える」コード負債となる

### 選択肢 C: Istio Sidecar モード + Dapr ポートをキャプチャ除外

- 概要: Istio は Sidecar モードのまま、Pod アノテーション (`traffic.sidecar.istio.io/excludeInboundPorts` / `excludeOutboundPorts`) で Dapr の 3500 / 50001 / 50002 ポートを Envoy のキャプチャ対象から除外する
- メリット:
  - 既存の Istio Sidecar 運用ノウハウを流用できる
  - Dapr 間通信は Dapr-mTLS のみに寄り、mTLS の二重掛けは部分的に回避される
- デメリット:
  - Dapr 間トラフィックが Istio の Telemetry / AuthorizationPolicy の対象外となり、「Istio を入れたのに監視対象外の穴が空く」運用的矛盾が生じる
  - 公式ドキュメントで紹介された採用例なし。本番運用事例が確認できない
  - Dapr のポート番号が将来変更された場合 (過去にも発生) に追従メンテナンスが必要
  - Zen of Python の「特殊ケースはルールを破るほど特別ではない」原則に反するワークアラウンド的な設計となり、技術的負債化しやすい

### 選択肢 D: Istio を採用しない (Envoy Gateway + Dapr-mTLS のみ)

- 概要: サービスメッシュを導入せず、North-South は Envoy Gateway (Gateway API)、East-West は Dapr-mTLS / Dapr service invocation で完結させる。Canary / Blue-Green は Argo Rollouts + Gateway API Traffic Splitting で実現する
- メリット:
  - 運用 OSS 数を 1 つ削減でき、2 名運用の負荷を直接的に下げられる
  - Dapr サイドカーとの衝突問題そのものが発生しない
  - Rocket Mortgage / ZEISS / Ignition など Dapr 単独運用の本番事例が存在する
  - Envoy Gateway 1.0 以降で Gateway API Traffic Splitting が完全実装され、Canary 機能の代替性が確立している
- デメリット:
  - JWT 検証・レート制限・高度なリクエストシェイピングなど Istio 固有の L7 機能が必要になった場合、Dapr middleware + Envoy Gateway のみで対応範囲が限定される
  - サービス数が 30+ を超え、L7 ポリシーの複雑度が増した時点で後からサービスメッシュを入れる移行コストが発生する
  - 「業界標準のサービスメッシュを導入していない」ことへの稟議レビュアーの心理的抵抗

## 決定理由

本決定は 4 つの比較軸で選択肢を評価した結果による。

1. **Dapr サイドカーとの衝突回避の確実性**: 案 A は構造的に衝突が発生しない。案 B / C は部分的にしか解消されない。案 D は衝突自体が存在しない。この軸では A と D が同格
2. **2 名運用との適合性**: 案 D が最も運用負荷が低いが、将来サービスメッシュが必要になった際の後付け移行コストが大きい。案 A は Ambient の運用ノウハウが少ない点で案 D より重いが、waypoint proxy を段階配置できるため初期負荷は抑えられる
3. **将来の拡張性**: 案 A は Istio エコシステムの将来機能 (Ambient と Sidecar の双方向移行、マルチクラスタ強化) を享受できる。案 D はサービスメッシュ相当の機能が必要になった時点で案 A への追加採用を検討することになり、二度手間になる
4. **技術的美学と業界の方向性**: 案 A は Google / IBM / Microsoft / Alibaba が推進する次世代標準であり、「Pod 内にネットワーク関心事を入れない」という Unix 哲学に合致する。案 D は YAGNI 原則として美しいが、Phase 5 の全社ロールアウト後のサービス数を考えると「結局必要になる」可能性が高い

**最終的な判断**: 案 A を採用する。案 D は企画段階の議論では十分魅力的だが、**Phase 4〜5 でサービス数が 30+ を超えた時点で結局サービスメッシュが必要になる**という中長期予測を重視した。その時点で新たに Ambient を入れるより、Phase 2 着手時に Ambient で始めてしまう方が総工数が少ない。

案 B・C は Pod 内 3 コンテナ同居が解消されない時点で、本決定の本質的動機 (サイドカー二重注入の回避) を満たさないため採用しない。

**フォールバック経路**: 案 D を明示的に撤退経路として保持する。Phase 2 POC で Ambient Mesh の成熟度に問題が見つかった場合、案 D に切り替える。この切替判断は Phase 2 着手から 1 ヶ月以内に行う。

## 影響

### ポジティブな影響

- Pod 内サイドカー数が 1 (Dapr のみ) に固定され、Pod のメモリ・CPU オーバーヘッドが Sidecar モード採用時より削減される
- tier1 API p99 予算の内訳から「Envoy サイドカー経由の追加レイテンシ」を除外できるため、500 ms 目標達成の余地が広がる
- 起動順序・ポートキャプチャ・トレース断絶の 3 問題が構造的に消滅するため、Phase 2 の障害切り分け Runbook が簡素化される
- Grafana Tempo との連携で W3C Trace Context が ztunnel 層でも保持されるため、tier1 API 呼び出しから infra まで一貫したトレースが得られる

### ネガティブな影響 / リスク

- Ambient Mesh の GA 後の歴史が 2 年と浅く、障害事例データベースが Sidecar モードより乏しい。特にエッジケース (大量 namespace 同居・高頻度 waypoint proxy 切替) での挙動は未知数
- waypoint proxy の配置戦略 (namespace 単位か service 単位か) が新しい設計判断として追加で必要になり、構想設計フェーズの工数が +0.2 人月程度増加する見込み
- Dapr と Ambient Mesh の相互作用に関する公式ドキュメントは 2026 年時点でも断片的であり、POC での実測が必須となる
- Phase 3 以降のマルチクラスタ構成では east-west gateway の扱いが Sidecar モードと異なるため、Phase 3 着手前に再検証が必要

### 移行・対応事項

- Phase 2 着手直後に 1 週間の Ambient Mesh POC を実施する (tier1 Go サービス × 2 本 + Dapr pub-sub の最小構成で検証)
- POC の検証項目: ztunnel HBONE と Dapr-mTLS の共存 / tier1 API p99 レイテンシへの影響 / W3C Trace Context の継承 / AuthorizationPolicy の動作
- POC 不成立時の切替判断 (案 D へのフォールバック) は Phase 2 着手から 1 ヶ月以内に行う
- `docs/02_構想設計/03_技術選定/02_中核OSS/04_サービスメッシュ.md` を新規作成し、Ambient 採用の前提で waypoint proxy 配置戦略・Dapr 共存構成・観測性統合を詳細化する
- `docs/02_構想設計/01_アーキテクチャ/03_セキュリティ/` 配下のセキュリティモデル資料で「Istio mTLS」表記を「Istio Ambient ztunnel HBONE」に統一する
- `docs/01_企画/04_定量試算/03_運用工数試算.md` の Istio 関連工数を Ambient 前提で再見積もり (Sidecar 運用より簡素化される方向で -0.1〜0.2 人月/年 の見込み)
- `docs/01_企画/img/全体構成図.drawio` の Istio サイドカー表現を Phase 2 着手前に Ambient 構成 (ztunnel DaemonSet + waypoint proxy) に書き直す

## 参考資料

- [Istio Ambient Mesh 公式ドキュメント](https://istio.io/latest/docs/ambient/)
- [Istio 1.22 Ambient GA リリースノート](https://istio.io/latest/news/releases/1.22.x/announcing-1.22/)
- [Dapr と Istio 共存に関する Diagrid の技術ブログ](https://www.diagrid.io/blog)
- `docs/01_企画/企画書.md` — 技術スタック表 (本 ADR 採択後の Ambient 表記に更新済み)
- `docs/01_企画/03_ロードマップと体制/01_MVPスコープ.md` — Phase 2 除外理由 (本 ADR 採択後の Ambient 表記に更新済み)
- `docs/02_構想設計/03_技術選定/02_中核OSS/01_実行基盤中核OSS.md` — 中核 OSS 選定の全体方針
