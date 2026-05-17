---
id: arch.tier1.tier1_index
axis: tier1
phase: architecture
kind: index
status: draft
depends_on:
  - req.overview.provided_scope
  - req.overview.non_scope
  - req.overview.oss_catalog
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier1 設計方針 index

## 一文方針
- tier1 は infra 層（Kubernetes / OSS / ミドルウェア）を tier2 / tier3 から完全に隠蔽する単一 facade。Server 系（動かす成果物）と Library（配る成果物）の二系統で構成し、`.proto` を単一の真として全 transport / 全言語に同型に投影する。

## 位置づけ
- tier1 は本企画の**契約境界**。tier2 / tier3 は tier1 facade のみを通じて infra にアクセスし、OSS のクライアント library を直接 import する経路を物理 enforce で塞ぐ
- OSS の選定 / 追従責務は tier1 に集約。tier2 / tier3 のエンジニアは OSS 動向 / ライセンスを意識する必要がない
- OSS ライフサイクルイベント（ライセンス変更 / 改廃 / サポート終了）発生時の影響範囲を tier1 に局所化することで、tier2 / tier3 の改修を最小化する

## 設計原則
- **infra 隠蔽**: OSS の固有型 / 概念は tier1 公開 API 表面に出さない（L1+ 単一深耕カテゴリの露出概念は事前宣言済の allowlist のみ許容）
- **`.proto` を単一の真**: 業務 API は External / Internal の 2 層 proto + transport adapter layer を介して全言語 / 全 transport に投影
- **3 抽象化レベル**: L3（OSS 中立）/ L2\*（同族 OSS 保証）/ L1+（単一深耕 + 移行コミットメント）をカテゴリ別に割付
- **L1+ 移行コミットメント**: 単一 OSS への深耕を選ぶ代わりに、ライフサイクルイベント時の移行 toolchain（schema diff / dual-write / observability 連続性 / 業務コード移行ガイド）を tier1 が成果物として継続維持
- **L2\* 同族保証**: 同族 OSS 2 実装で Testcontainers conformance test green を merge 条件
- **defense-in-depth は 5 層**: 依存導入 lint / 公開 API snapshot / supply chain mirror / Runtime enforcement / 物理 enforcement

## 主要コンポーネント

### Server 系（動かす成果物、ランタイム）
- Gateway（Library 非対応言語の呼び口、Envoy Gateway + Rust ハンドラ）
- Sidecar / Agent（Outbox relay / Secret 同期 / Trace buffer、Rust 常駐 process）
- Backend-for-Library（Token 検証 / 認可ポリシー一括評価、Rust gRPC server）
- Control Plane（動的設定配信、Rust + flagd 連携）
- Operator / Controller（K8s リソース宣言的管理、Go + controller-runtime / kubebuilder）

### Library（配る成果物、tier2 / tier3 向けパッケージ）
- core / frontend / backend の 3 パッケージ構成
- 対象言語: Rust / C# (.NET 8+) / Go / TypeScript（.NET Framework は Companion で代替）
- 17 機能カテゴリ（Observability / Auth / Secret / FeatureFlag / KV / ObjectStore / RPC / Messaging / SchemaReg / Relational / Vector / Workflow / Rule など）

### Companion（Library 非対応言語向け）
- 役割 A: Observability / 認証コンテキスト伝播（CLR Profiler attach + k1s0.Companion.NetFx.OTelExt）
- 役割 B: Transport Negotiation Runtime（bidi 含む RPC の transport 選択と等価実装）

## 5 方針構成
- [01_Server 系](01_Server系.md) — Gateway / Sidecar / Backend-for-Library / Control Plane / Operator / Proto 二層化 / Transport Adapter Layer
- [02_Library](02_Library.md) — 3 抽象レベル / frontend-backend 分離 / 強制機構 3 層 / L1+ 移行コミットメント / Companion
- [03_言語スタック](03_言語スタック.md) — Server 系 = Rust（Operator のみ Go）、Library = Rust / C# / Go / TypeScript
- [04_提供機能カテゴリ](04_提供機能カテゴリ.md) — 17 カテゴリ × Lv 割付 + 露出概念 + L2\* 同族 + L1+ 本命 OSS

## 詳細設計への参照
- [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)
- [移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md)
- [観測適合仕様](../../04_詳細設計/01_適合仕様/03_観測適合仕様.md)
- [認証適合仕様](../../04_詳細設計/01_適合仕様/04_認証適合仕様.md)
- [鍵管理適合仕様](../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
- [スキーマ進化適合仕様](../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- [SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [テナント容量適合仕様](../../04_詳細設計/01_適合仕様/09_テナント容量適合仕様.md)
- [tier1 強制機構](../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- [HTTP/2 enforcement](../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- [KEK Shamir 分散](../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md)
- [apicurio GitOps SoT](../../04_詳細設計/03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md)
- [protoc-gen-go FSM](../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)
- [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md)
- [BFF auth edge](../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md)
- [Tauri Companion sidecar](../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md)

## 採用しない選択肢
- 業務固有領域（tier3 業務ロジック / UI / 画面遷移制御）の Library 抽象
- Operator / Controller の Library 配布（Go で別途実装）
- 可視化 UI（Perses / Superset / Jaeger）の Library 抽象
- Service Mesh（Istio）機能の Library 再露出
- ブロックストレージ（Longhorn / Ceph RBD）の Library 抽象
- 証明書 / PKI（cert-manager）の Library 抽象
- 開発体験 / 運用ツール（Backstage / Testcontainers / Tilt / Helm / Kustomize / sqlx-cli）の Library 配布
- L3 / L2\* で OSS 概念の API 表面露出（L1+ 露出概念 allowlist 外）
- day-1 multi-backend 並走（L1+ は OSS ライフサイクルイベント時のみ移行）
- 自製 WebSocket bidi（gRPC 公式仕様外、Connect-RPC L1+ 単一深耕）

## 至高路線における立ち位置
- 「複数 OSS 同時サポート」を責任放棄しない代わりに、L1+ で単一 OSS に深耕しつつ移行 toolchain を年次 dry-run green で物理証明
- 「External Proto の都合で Internal Proto を曲げる」逆流を構造的に拒否（External / Internal 二層、CI 差分検査）
- transport 中立性は「単一 transport 深耕」ではなく「bidi semantics 深耕 + transport 移行コミットメント」で実現

## 関連参照
- [Server 系](01_Server系.md) / [Library](02_Library.md) / [言語スタック](03_言語スタック.md) / [提供機能カテゴリ](04_提供機能カテゴリ.md)
- [tier2 設計方針](../03_tier2設計方針/README.md) / [tier3 設計方針](../04_tier3設計方針/README.md)
- [client 設計方針](../09_client設計方針/README.md)
