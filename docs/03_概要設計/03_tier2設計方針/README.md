---
id: arch.tier2.tier2_index
axis: tier2
phase: architecture
kind: index
status: draft
depends_on:
  - req.overview.provided_scope
  - req.overview.non_scope
  - arch.tier1.tier1_index
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_property_axiom_proof
---

# tier2 設計方針 index

## 一文方針
- tier2 は tier3 にドメイン業務の共通化処理を提供する単一 facade。業界 pack（v1.0.0 = 製造業のみ）+ 業界横断層 + テナント別差分の 4 抽象化レベルを bundle として宣言し、tier3 が tier2 を迂回して tier1 Library を直接組合わせる経路を 8 層強制機構で物理拒否する。

## 位置づけ
- tier3 から見える業務の語彙（製造業であれば「設備」「ロット」「検査結果」等）を tier2 に集約し、業界 / ドメイン横断で再利用される業務資産（Domain Event / Workflow 定義 / 決定表 / 業務マスタ / DB schema 等）を所有
- tier3 は tier2 の提供する API / Service 経由でのみ業務資産にアクセス。tier2 を迂回して tier1 Library を直接組み合わせ業務処理を独自に再実装することは禁止
- tier1 の OSS / ミドルウェアへの直接アクセスは禁止。tier2 内部の infra 接続はすべて tier1 Library 経由

## 1.0.0 出荷スコープ
- 1.0.0 で出荷する業界 pack: 製造業のみ
- 本ディレクトリの全文書で扱う「業界横断」「業界 pack」「業界拡張」は tier2 の構造的前提
- 製造業以外の pack は後続バージョンで [業界拡張モデル](02_業界拡張モデル.md) に従って追加
- 製造業固有概念を業界横断層に置くことは禁止（強制機構: [業界中立性の機械的担保](02_業界拡張モデル.md)）

## 4 抽象化レベル
- **Lv 業界横断（Cross-industry）**: 業界に依存しない業務語彙（通知 / 監査 / 承認 workflow / 帳票 / マスタ管理 等）。第二業界 stub から同型 API で素直に消費可能
- **Lv 業界共通（Industry-common）**: 業界 pack 内で複数ドメインに共通する業務語彙（製造業: 「設備」「ロット」「品目」「拠点」「BOM」等）
- **Lv 業界固有（Industry-specific）**: 業界 pack 内ドメイン固有の業務語彙（FA / 調達 / 検査 各ドメイン）
- **Lv テナント固有（Tenant-specific）**: テナントごとの差分（マスタ / ルール / 権限）。差分は override / 設定で吸収

詳細は [抽象化レベル](03_抽象化レベル.md) を参照。

## 設計原則
- **業界 pack 並立**: 単一業界専用の構造を取らない。業界横断層 → 業界 pack の単方向依存を CI で物理 enforce
- **第二業界 stub conformance**: 1.0.0 では出荷業界が製造業のみのため、サービス業 stub pack を CI 専用で保持し、業界横断層 API が stub からも消費できることを merge 条件
- **テナント識別子強制注入**: tier2 リポジトリ抽象が `tenant_id` 述語を強制注入。業務コードから省略不可
- **生 SQL 経路の Library 境界での完全封鎖**: tier2 アプリ層から SQL 文字列を渡せる経路を一切提供しない
- **State change / Outbox / Audit は同一 DB トランザクションで書く**（atomic 三表書込、subscribe 失敗で audit が欠落する事故を構造的に不可能化）
- **defense-in-depth は 8 層**: tier3 リポジトリ側 / tier2 公開 API 表面 / tier2 内部依存方向 / 第二業界 stub conformance / リポジトリ抽象強制注入 / 内部レジストリ / Library Service 等価性 / 監査スキーマ整合

## 主要方針構成（28 方針）
- [01_責務](01_責務.md) — ドメイン業務共通化、tier3 への提供、抽象化レベル別影響範囲
- [02_業界拡張モデル](02_業界拡張モデル.md) — 業界 pack 並立、業界中立性の 3 種機械的担保
- [03_抽象化レベル](03_抽象化レベル.md) — 4 Lv の定義 / 昇格降格 / override 拡張点
- [04_ドメイン分割方針](04_ドメイン分割方針.md) — bounded context、Domain Event 経由の cross-domain 参照
- [05_マルチテナント方針](05_マルチテナント方針.md) — テナント識別子伝播、共有 / 専用 Service の昇格条件
- [06_業務資産所有権](06_業務資産所有権.md) — 既定実装と override 拡張点、contract test
- [07_Service 運用方針](07_Service運用方針.md) — Library / Service 配布形態、共有 Pod / DB
- [08_業務エラー監査コンプライアンス](08_業務エラー監査コンプライアンス.md) — Domain Event emit 必須化、audit hash chain、PII 取扱
- [09_権限モデル](09_権限モデル.md) — Keycloak ABAC、Delegation、Emergency Override
- [10_表現手段](10_表現手段.md) — コード / Schema 駆動 / DSL の 3 表現手段、codegen
- [11_互換性ポリシー](11_互換性ポリシー.md) — SemVer、並行バージョン提供、移行コミットメント
- [12_API 設計規約](12_API設計規約.md) — wire 仕様、エラー / 冪等性 / Pagination / Streaming / LRO
- [13_状態遷移パターン](13_状態遷移パターン.md) — 4 言語等価強度、Domain Event 命名標準、Saga
- [14_管理境界](14_管理境界.md) — admin API 分離、Backstage プラグイン、emergency 経路
- [15_読み取りモデル方針](15_読み取りモデル方針.md) — CQRS、Read model projector、検索 / 集計 / cache
- [16_外部システム統合](16_外部システム統合.md) — External Proto / Companion / Webhook、token exchange
- [17_データ保持](17_データ保持.md) — カテゴリ別 retention、hot / warm / cold、crypto-shred
- [18_信頼性運用準備](18_信頼性運用準備.md) — DR / SLO / Workflow 運用 / インシデント response
- [19_セキュリティアーキテクチャ](19_セキュリティアーキテクチャ.md) — 脅威モデル / Secret rotation / 暗号アジリティ
- [20_データライフサイクル](20_データライフサイクル.md) — onboarding / export / offboarding / bulk / 品質
- [21_位置づけ](21_位置づけ.md) — 5 階層論における位置、隣接層との接合
- [22_配布形態](22_配布形態.md) — Library / Service の 2 配布形態、共通 conformance
- [23_言語スタック](23_言語スタック.md) — Rust / C# / Go / TypeScript の 4 言語
- [24_テスト方針](24_テスト方針.md) — property test / Pact / Testcontainers / Litmus / atomic 三表書込 invariant
- [25_スケジューラ・バッチ](25_スケジューラ・バッチ.md) — Argo CronWorkflow + Temporal Workflow + KEDA
- [26_国際化方針](26_国際化方針.md) — i18n key + locale-aware フォーマット + UTF-8 統一
- [27_開発者体験](27_開発者体験.md) — Tilt + Companion local mock + Backstage Software Template
- [28_業務添付帳票資産](28_業務添付帳票資産.md) — Object Storage 経由 + envelope 暗号化 + retention

全 28 方針を完備。個別非提供スコープ（17_非提供スコープ）は要件定義 [02_非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md) に統合済。

## 詳細設計への参照
- [テナント分離適合仕様](../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [tier2 強制機構](../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- [protoc-gen-go FSM](../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)
- [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md)

## 採用しない選択肢
- 業界横断層に業界固有概念を置く
- 業界横断層 → 業界 pack の依存
- 業界 pack 間の相互依存
- tier2 を迂回した tier1 Library / OSS 直接利用
- 業務コードから `tenant_id` を省略する API
- 任意の `tenant_id` を引数で受け付ける成りすまし可能 API
- 生 SQL 経路を tier2 アプリ層に露出
- aggregate 状態変更で Domain Event emit を抑制
- audit / Outbox を別トランザクションに分割
- AGPL / SSPL 系 OSS の業務資産配信

## 至高路線における立ち位置
- 「1 業界しか出荷しないから業界横断層は曖昧でよい」を採らない（出荷物が 1 業界の時こそ業界固有概念漏洩の圧力が最大）
- 「テナント分離は文章で運用」を採らない（5 層 defense-in-depth で物理 enforce）
- 「OSS 直接利用を一部許容」を採らない（4 言語 8 層強制で物理拒否）

## 関連参照
- [tier1 設計方針 index](../02_tier1設計方針/README.md)
- [tier3 設計方針 index](../04_tier3設計方針/README.md)
- [client 設計方針 index](../09_client設計方針/README.md)
