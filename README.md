# k1s0

> **業務不変条件を、文章運用ではなく物理機構で守りきる業務プラットフォーム。**
> 5 階層論 × 19 軸同型構造 × 6 層 defense-in-depth × 5 proof_class を、製造業 1.0.0 完璧主義で ship する。

---

## このプラットフォームが面白い理由

エンジニアにとってのフックは「**書いてあるから守られる**」を一切信じないこと。
SLO / 監査 / 認可 / テナント分離 / OSS 移行 / 形式検証 — そのすべてを、人間の規律ではなく **CI / lint / Kyverno / HSM / 外部公証 + formal proof** の多重防御で *物理* に enforce する。

- **L1+ 単一深耕 + 移行コミットメント**
  「複数 OSS を同時にサポート」を諦める代わりに、単一 OSS の全機能 + 移行 toolchain を持つ。`dry_run.lock.yaml` の last_green_at が 365 日以内で release_gate を通過。
- **19 軸同型構造**
  tier1 / tier2 / tier3 / infra / data / security / ops / client / test / formal + meta-axis、それに 13 件の cross-cutting 適合仕様。全軸が同じ形（class bundle / dimension override 禁止 / dead spec 殺し / build artifact 化 / 5 層 defense-in-depth）。
- **dimension override 禁止 + dead spec 殺し**
  軸の class bundle が他 dimension を一意に導出する。参照消失で CI fail。`lock.yaml` は build script の生成物で、手書き drift を物理拒否。
- **defense-in-depth 6 層**
  A: compile（codegen / 型）/ B: lint（規約 / 公開 API snapshot）/ C: integration & contract test / D: runtime（RLS FORCE / pgaudit / Envoy jwt_authn / atomic 三表書込）/ E: 物理（Cosign / Kyverno / HSM zeroize / 外部公証 / WebCrypto non-extractable）/ F: 数学的（TLA+ / Apalache / Stainless / Dafny / Lean 4 / Kani / CBMC）。
- **5 proof_class × 95 cell coverage**
  temporal_safety / liveness / refinement / program_correctness / runtime_modelcheck。1.0.0 で verified or accepted_with_assumption が AND-gate。
- **業界 pack 並立 day-1**
  1.0.0 は製造業のみ ship。だが業界横断層が業界固有概念で汚染されない構造を、命名禁則 + 依存方向 + 第二業界 stub conformance の 3 種で機械的に担保。

> 哲学: **「至高を目指す判断」。** 運用コスト度外視、1.0.0 完璧主義、段階的 release 禁止、機能削減なし。詳しくは [CLAUDE.md](CLAUDE.md)。

---

## アーキテクチャ概観

### 5 階層 — 隠蔽境界を物理 enforcement で固定

```
tier3  業務 UI（Web SPA / デスクトップ exe / .NET Framework 4.6.2+）
       4 layer client state（Server Truth / Optimistic Local / Pending Queue / Draft）
       ↓ tier2 経由でのみ業務資産にアクセス
tier2  ドメイン業務共通化 + 業界 pack 並立（業界横断 / 業界共通 / 業界固有 / テナント固有）
       atomic 三表書込（State change + Outbox + Audit）
       BusinessConflict subtype 4 種（stale_write / lost_update / supersede / concurrent_edit）
       ↓ tier1 Library 経由でのみ infra にアクセス
tier1  Server 系（Rust 主体 / Operator のみ Go）+ 4 言語 Library + Companion
       17 機能カテゴリ × 3 抽象化（L3 / L2* / L1+）+ 9 適合仕様
       ↓
data   PostgreSQL(CloudNativePG) / Kafka(Strimzi) / ClickHouse / Valkey / Apicurio / Rook+Ceph / Longhorn
       5 preservation_class × 4 種 restore_drill × expand-contract migration
infra  Kubernetes / 5 topology_class / 5 clock_integrity_class / HLC runtime / 25+ Kyverno policy
```

逆方向の依存（tier1 → tier3 等）は CI で物理拒否。隠蔽境界の bypass は強制機構で物理拒否。

### 19 軸 matrix

| 軸 | 種別 | 主要適合仕様 |
|---|---|---|
| tier1 | 階層 | Bidi / 移行 Pair / 観測 / 認証 / 鍵管理 / スキーマ進化 / SLO / OSS lifecycle / テナント容量 |
| tier2 | 階層 | テナント分離 |
| tier3 | 階層 | クライアント状態 |
| infra | 階層 | クラスタ位相 / 時刻整合 |
| data | 階層 | データ保全 |
| security | 横断 | 脅威モデル / build_provenance |
| ops | 横断 | 運用ループ |
| client | 横断 | クライアント SDK 配布 |
| test | 横断 | 検証規律 |
| formal | meta-meta | 形式検証 |
| meta-axis | 軸登録 | `00_軸登録適合仕様` が軸の追加削除を物理 enforce（cap v1 = 20） |

cross-cutting 適合仕様 13 件 — HTTP/2 enforcement / KEK Shamir 分散 / Apicurio GitOps SoT / protoc-gen-go FSM / SLO protection layers / BFF auth-edge / Tauri Companion sidecar / PII 専用クラスタ / audit ingest gap monitor / ops-edge cluster / Companion OTel 拡張 / UA-aware adapter / .NET 8 Connect-RPC 自製。

詳細: [03_概要設計/01_アーキテクチャ概観/](docs/03_概要設計/01_アーキテクチャ概観/README.md)

---

## 技術スタック

### 言語

| 用途 | 言語 |
|---|---|
| tier1 Server | **Rust** stable（主言語） |
| Operator / Controller | **Go 1.22** + controller-runtime / kubebuilder |
| tier1 Library / tier2 / tier3 | **Rust / C# (.NET 8+) / Go / TypeScript** の 4 言語等価強度 |
| Web SPA | TypeScript + React |
| デスクトップ | C# (.NET 8+) または Rust + **Tauri** |
| レガシー | **.NET Framework 4.6.2+** + WinForms / WPF + Companion |
| SDK 配布 | 9 言語 lockstep（.NET 8 / .NET Framework / Java 21 / Node.js 20 / Browser TS / Tauri / Rust / Python 3.12 / Ruby 3.3 / Go 1.22） |

### データ / インフラ

PostgreSQL（CloudNativePG, RLS FORCE + pgaudit） / Kafka（Strimzi） / ClickHouse / Valkey / Apicurio Registry / Rook+Ceph + Longhorn / Kubernetes + Kyverno admission policy 25+。

### 形式検証

TLA+ + Apalache（temporal safety / liveness） / Stainless / Dafny（program correctness） / Lean 4 + mathlib（KEK Shamir threshold algebra） / Kani / CBMC（Rust / C runtime modelcheck）。

### セキュリティ / build provenance

Cosign signed + SBOM（Syft） + SLSA L3+ + Witness multi-attestation chain / WebCrypto non-extractable CryptoKey / HSM PKCS#11 zeroize / KEK Shamir M-of-N / RFC 3161 trusted timestamp + Sigstore transparency log / WORM Object Lock / 8 secret class lifecycle。

### 認証

Keycloak + Envoy `jwt_authn` filter + token introspection + DPoP / WebAuthn step_up / federated exchange / break-glass emergency step_up + 強制 audit emit。

---

## docs/ ナビゲーション

機械可読な単一の真がすべて `docs/` に固定されている。各 markdown は frontmatter（`id` / `axis` / `phase` / `depends_on` / `covered_by`）を持ち、軸間整合は build script で検証される。

| Phase | ディレクトリ | 役割 |
|---|---|---|
| 0 | [docs/00_format/](docs/00_format/) | テンプレート / 規約 / frontmatter schema / lint 規約 |
| 1 | [docs/01_企画/](docs/01_企画/README.md) | 背景 / 価値 / 競合 / 法務 / ターゲット / OSS 公開 / 業界 pack 戦略 / 開発体制 / 用語集 |
| 2 | [docs/02_要件定義/](docs/02_要件定義/README.md) | スコープ / 機能要件 / 非機能要件 / 技術選定 / 開発体制 / 制約と前提 |
| 3 | [docs/03_概要設計/](docs/03_概要設計/README.md) | 5 視点アーキテクチャ概観 + 10 軸別設計方針 + クロスカッティング設計 8 機構 |
| 4 | [docs/04_詳細設計/](docs/04_詳細設計/README.md) | 20 適合仕様 + 10 強制機構 + 13 cross-cutting + 8 運用 UI + 5 lock.yaml 体系 |
| - | [docs/90_knowledge/](docs/90_knowledge/) | 技術学習用 reference |

### よく読まれる入口

- **設計思想を 1 ファイルで掴む**: [03_概要設計/01_アーキテクチャ概観/](docs/03_概要設計/01_アーキテクチャ概観/README.md)
  └─ [5 階層論](docs/03_概要設計/01_アーキテクチャ概観/01_5階層論.md) / [19 軸論](docs/03_概要設計/01_アーキテクチャ概観/02_19軸論.md) / [defense-in-depth](docs/03_概要設計/01_アーキテクチャ概観/03_defense_in_depth.md) / [5 proof_class 論](docs/03_概要設計/01_アーキテクチャ概観/04_5proof_class論.md) / [軸間依存図](docs/03_概要設計/01_アーキテクチャ概観/05_軸間依存図.md)
- **形式検証どうするか**: [20_形式検証適合仕様](docs/04_詳細設計/01_適合仕様/20_形式検証適合仕様.md) / [11_formal 設計方針](docs/03_概要設計/11_formal設計方針/README.md)
- **OSS をどう選ぶか / どう乗り換えるか**: [04_技術選定](docs/02_要件定義/04_技術選定/) / [08_OSS ライフサイクル適合仕様](docs/04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- **クライアント状態とコンフリクト**: [11_クライアント状態適合仕様](docs/04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)
- **lock.yaml 体系**: [05_lock_yaml 体系](docs/04_詳細設計/05_lock_yaml体系/README.md)
- **用語集**: [01_企画/09_用語集](docs/01_企画/09_用語集/README.md)

---

## 1.0.0 ship blocker

`release_gate.lock.yaml` は AND-gate。次のすべてが green でなければ tag が切れない。

- 全 19 軸の `release_gate.lock.yaml` cell が green
- 4 primary pair の `dry_run.lock.yaml` の `last_green_at` が 365 日以内
- formal proof 95 cell が `verified` or `accepted_with_assumption`
- 製造業 pack 9 stress test 全 green（FA 生産指示 / 進捗実績 / 検査結果 / SCADA / 図面 review / 在庫 / 受注 / 警報配信 / 計量装置）
- `cosign signed tag` が物理 prerequisite

### 1.0.0 で ship するもの

- 業界 pack: 製造業のみ（業界並立構造は day-1 から有効）
- アプリ形態: Web SPA / デスクトップ exe / レガシー .NET Framework 4.6.2+ の 3 形態
- 9 言語 SDK lockstep release（MAJOR.MINOR skew = 0）

### 採用しないもの

- 機能数 / 統合 OSS 数 / 価格 / 即座に release での競争
- 文章運用のみの保証（必ず class bundle + lock artifact + 物理 enforcement）
- LLM 単独 sign-off / 公開前 security review なしの公開
- 「dev mode で runtime check を緩める」経路（production / development 区別なし）

---

## OSS 公開

`v1_inhouse_authoritative` 区画を **Apache 2.0** で公開。配布は Cosign signed + SBOM + SLSA L3+ + in-toto attestation。コミュニティ contribution は DCO + CLA + 4 reviewer dual sign-off。

公開対象例:

- `k1s0.Connect.NetCore` — .NET 8 LTS 向け Connect-RPC 自製実装（Connect Conformance Suite 全 case green）
- `k1s0_apicurio_additive_controller` — Apicurio Operator 補完 controller
- `k1s0_hlc_lib` — HLC（Hybrid Logical Clock）言語別 wrapper（Rust / Go / .NET）
- `protoc-gen-k1s0-go-fsm` — Go の typestate via nominal types codegen plugin
- `k1s0 Companion Mock` / `Tauri Companion sidecar` / `k1s0.Companion.NetFx.OTelExt`
- 自製 DSL backend（ZEN Engine 互換 Rust 実装）

詳細: [01_企画/06_OSS 公開戦略](docs/01_企画/06_OSS公開戦略/README.md)

---

## プロジェクト状態

- 現在は **設計フェーズ**。`docs/` の全 frontmatter は `status: draft`。
- ソース実装（`src/`）は未着手。`tools/` に `docs_lint` / `lock_yaml_generator` の骨格のみ。
- 設計は完成度 first。「文章で書いた」を「物理で守る」に置き換える作業を、コードに先んじてやりきる方針。

---

## License

[Apache License 2.0](LICENSE)。
