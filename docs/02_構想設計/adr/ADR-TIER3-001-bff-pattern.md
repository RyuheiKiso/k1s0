# ADR-TIER3-001: tier3 client ごとに専用 BFF を配置する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: tier3 開発チーム / tier1 開発チーム / 採用検討組織

## コンテキスト

k1s0 の tier3 は Web SPA（portal / admin / docs-site）/ Native（MAUI Hub / Admin）/ Legacy wrap（.NET Framework サイドカー）の 3 系統で構成され、これらの client が tier1（公開 12 API gRPC ファサード）/ tier2（ドメインサービス）にアクセスして業務機能を実現する。client から backend への呼び出し経路を定める際、以下の対立軸が顕在化する。

- **client が tier1 / tier2 を直接呼ぶ**（薄い構造、レイテンシ最小、ただし client 側で集約・認証・権限判定の責務が膨らむ）
- **client と backend の間に集約層を置く**（responsabilité 分離、ただし 1 段増えるレイテンシ）
- **集約層を全 client 共有**（API Gateway パターン）か、**client ごと専用**（BFF パターン）か

各 client の特性は次のとおり：

- Web SPA: ブラウザの CORS / セッション cookie / OIDC リダイレクト / GraphQL 親和性
- Native MAUI: モバイル / デスクトップで HTTPS REST + Bearer token、低帯域・断続接続
- Legacy wrap: .NET Framework / Web API、内部 backend 呼出のみ

これらが**異なる集約戦略**を要求するため、API Gateway パターン（共通集約）では「Web SPA 用に GraphQL を出すと Native でも GraphQL を使うことになる」「Web のセッション cookie 戦略を Native に押し付ける」といった非対称が発生する。

加えて k1s0 は tier1 公開 12 API（state / pubsub / serviceinvoke / secrets / binding / workflow / log / telemetry / decision / audit / feature / pii）を gRPC で提供するが、ブラウザは素の gRPC HTTP/2 trailer に到達不能（SHIP_STATUS F2）。何らかの **gRPC ↔ HTTP / GraphQL 変換層**が必要である。

集約層のパターン選択は **two-way door** だが、tier3 各 client の依存先 API 設計を全部書き直す移行コストは大きい。リリース時点で確定する。

## 決定

**tier3 の client（Web SPA / Native / Legacy wrap）ごとに専用 BFF（Backend for Frontend）を配置する。**

- Web SPA: **Go BFF**（GraphQL + REST、Keycloak OIDC 統合、`src/tier3/bff/cmd/{portal-bff,admin-bff}/`）
- Native MAUI: **HTTPS REST 直接**（tier1 facade に Bearer JWT で直接アクセス、`src/sdk/dotnet/` 経由）
- Legacy wrap: **.NET Web API サイドカー**（`src/tier3/legacy-wrap/sidecars/K1s0.Legacy.Sidecar/`、SDK アダプター経由で tier1 を呼ぶ）

BFF の責務:
- tier1 / tier2 の集約とフロントエンド向けスキーマへの変換
- 認証（Keycloak OIDC、Bearer JWT 検証）と per-tenant tenant_id 注入（SHIP_STATUS H1 cross-tenant boundary）
- gRPC ↔ HTTP / GraphQL の translator（Web は GraphQL / Native は REST）
- セッション cookie（Web のみ）
- BFF パターンに反する責務（業務ロジック / 永続化）は tier1 / tier2 に押し戻す

`src/tier3/bff/cmd/{portal-bff,admin-bff}/` の Go BFF が portal / admin の Web SPA に対応、Native は SDK + tier1 facade に直接、Legacy wrap は .NET サイドカー BFF として動作。

## 検討した選択肢

### 選択肢 A: client ごと専用 BFF（採用）

- 概要: Sam Newman / ThoughtWorks 提唱の Backend for Frontend パターン。各 client の特性に最適化された集約層を per-client で配置
- メリット:
  - **client の特性に最適化**（Web は GraphQL / Native は REST / Legacy はサイドカー）
  - 認証経路が client ごとに分離（Web セッション cookie が Native に漏れない）
  - tier1 / tier2 から見ると「BFF からの呼び出し」のみで、client 多様性を意識しなくて済む
  - cross-tenant boundary（NFR-E-AC-003）の per-request enforcement を BFF 段で実装しやすい
- デメリット:
  - BFF 数が増える（portal-bff / admin-bff / 将来追加）
  - 各 BFF を独立に保守する人的リソース必要
  - 共通ロジック（auth / tenant 抽出 / observability）の重複を防ぐため shared library が必要

### 選択肢 B: 共通 API Gateway

- 概要: Kong / Envoy Gateway 等の API Gateway で全 client 集約
- メリット:
  - 集約点が 1 つ、運用 component 削減
  - Rate Limit / Auth / observability が単一点で完結
- デメリット:
  - **client 別最適化が困難**（GraphQL / REST / Native 固有要件を 1 つの Gateway で扱うと巨大化）
  - tier1 公開 12 API の gRPC を Gateway で HTTP / GraphQL に変換すると、Gateway 設定が複雑化
  - cross-tenant 強制が「Gateway 段で全 RPC を解析」となり、業務ドメイン知識を Gateway に押し込むことになる
- 注: Envoy Gateway は **南北 ingress**（外部 → クラスタ）として ADR-MIG-002 で採用済だが、これは BFF とは責務が異なる（Envoy Gateway は L4/L7 ingress、BFF はアプリ層集約）

### 選択肢 C: GraphQL Federation（Apollo Federation 等）

- 概要: 複数 backend を GraphQL Federation で統合し、すべての client が単一 GraphQL 経路でアクセス
- メリット:
  - GraphQL の型システムで API 整合性が強制される
  - Web には親和性が高い
- デメリット:
  - **Native / Legacy wrap には不適合**（モバイルの帯域・GraphQL クライアント未成熟、.NET Framework の GraphQL クライアント弱い）
  - tier1 の 12 API すべてに GraphQL schema を被せる作業が膨大
  - tier1 が gRPC（ADR-TIER1-002）であるため、Federation の subgraph として gRPC を扱う変換層が別途必要
  - 業界での Federation 経験者が少なく、採用組織の人材流動性が下がる

### 選択肢 D: client が tier1 / tier2 を直接呼ぶ（集約層なし）

- 概要: BFF を置かず client から tier1 / tier2 に直接アクセス
- メリット:
  - 構造が薄い、レイテンシ最小
  - BFF 運用コストなし
- デメリット:
  - **gRPC HTTP/2 trailer がブラウザから到達不能**（SHIP_STATUS F2）、Web SPA で実質不可能
  - 認証 / cross-tenant 強制 / 集約を全部 client 側で実装することになる
  - tier1 を変更すると全 client が壊れる、tier1 のリファクタリング自由度が下がる
  - 採用組織の Web 開発者と backend 開発者の責務分界が曖昧化

### 選択肢 E: Server-Side Rendering（Next.js / Remix）の API Routes

- 概要: SSR フレームワーク内蔵の API Routes を BFF 相当として使う
- メリット: SSR / API が一体運用、Web 開発体験が良い
- デメリット:
  - **Native / Legacy には適用できない**（Web SPA 専用）
  - Web SPA 採用方針（ADR-TIER3-002）と整合しない場合は不採用
  - Next.js 採用判断は ADR-TIER3-002 で別途扱う

## 決定理由

選択肢 A（client ごと専用 BFF）を採用する根拠は以下。

- **client 多様性への最適化**: Web SPA / Native / Legacy wrap という異質な client に共通 API（B / C）を被せると最大公約数的な API になり、各 client での開発体験が悪化する。BFF パターンは「各 client に最適化された API」を提供しつつ、tier1 / tier2 の安定性を守る業界標準パターン
- **gRPC ↔ HTTP / GraphQL 変換の局所化**: tier1 が gRPC である以上、Web からは何らかの変換層が必要。BFF は「変換 + 集約 + 認証」を一体で受けられる適切な責務単位
- **認証経路の分離**: Web のセッション cookie 戦略と Native の Bearer JWT を BFF 単位で分離できる。共通 Gateway（B）では cookie / Bearer の両対応で複雑化する
- **cross-tenant boundary の強制点**: SHIP_STATUS H1 で導入した `EnforceTenantBoundary` を BFF 段でも適用する経路が確立しており（`src/tier3/bff/internal/auth/`）、tier1 と二段防御が成立
- **API Gateway との責務直交**: Envoy Gateway（ADR-MIG-002）は南北 L4/L7 ingress / TLS 終端 / HTTPRoute の責務、BFF はアプリ層集約 / 認証 / 集約変換の責務。両者は責務が直交し、層として併存させることで責任分界が明瞭になる
- **Conway の法則整合**: 採用組織で「Web 担当 / モバイル担当 / Legacy 統合担当」のチーム分割が現実的であり、BFF パターンはチーム分割と技術分割が一致する

## 帰結

### ポジティブな帰結

- 各 client（Web / Native / Legacy）の最適 API が独立進化可能
- tier1 / tier2 の安定性が tier3 client 多様性から守られる
- 認証経路 / cross-tenant 強制が BFF 段で per-request 実施
- 採用組織のチーム分割（Web / モバイル / Legacy 統合）と技術分割が一致
- BFF を `tier3-bff` Helm chart で標準化（`deploy/charts/tier3-bff/`）

### ネガティブな帰結 / リスク

- BFF 数が client / 業務ドメインで増える運用負担
- BFF の認証 / observability / cross-tenant 共通ロジックを shared library で重複排除する規律が必要（`src/tier3/bff/internal/` の `auth/` `shared/` で実装）
- BFF 段でのバグ（特に cross-tenant skip）が tier1 の二段防御で発見されないと CRITICAL bug 化（SHIP_STATUS G3 で実例）→ regression test と anti-shortcut-discipline §横展開検査 で防止

### 移行・対応事項

- `src/tier3/bff/cmd/{portal-bff,admin-bff}/` で Go BFF を確定（既存実装あり、SHIP_STATUS § tier3）
- `src/tier3/bff/internal/auth/` で OIDC + Bearer JWT 共通検証 + `EnforceTenantBoundary` 適用を集約（SHIP_STATUS H1 と整合）
- `deploy/charts/tier3-bff/` で BFF 標準 Helm chart を提供
- BFF を増やす場合の Golden Path（`/scaffold` テンプレート）整備
- BFF 段の cross-tenant boundary regression test を CI で固定（SHIP_STATUS H2 同パターン）

## 関連

- ADR-TIER3-002（SPA + BFF）— Web 側の SPA 構成
- ADR-TIER3-003（.NET MAUI Native）— Native 側の構成（BFF 不要、SDK 直接）
- ADR-MIG-002（API Gateway / Envoy Gateway）— 南北 ingress、BFF と責務直交
- ADR-TIER1-002（Protobuf gRPC）— tier1 公開 API の通信規約、BFF が変換受け
- ADR-SEC-001（Keycloak）— OIDC / JWT の発行元
- ADR-DEV-001（Paved Road）— BFF Golden Path 整備
- DS-SW-COMP-* — tier3 BFF 設計
- IMP-DIR-INFRA-* — `src/tier3/bff/` 配置

## 参考文献

- Sam Newman, "Backend For Frontend Pattern": samnewman.io/patterns/architectural/bff/
- ThoughtWorks Technology Radar: BFF
- SHIP_STATUS.md §H1（cross-tenant boundary regression test）
