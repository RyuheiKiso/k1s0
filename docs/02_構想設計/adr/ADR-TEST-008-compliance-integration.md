# ADR-TEST-008: コンプライアンス検証を Sonobuoy / SLSA L3 / OSSF Scorecard / OpenSSF Best Practices Badge / FIPS 140-3 で統合する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / セキュリティ担当 / コンプライアンス担当（採用初期）

## コンテキスト

ADR-TEST-001 で「採用検討者の信頼は release artifact で得る」と決定したが、release artifact が「testing maturity が高い」と評価される根拠は、内部の qualify report だけでなく **外部の標準的な評価軸との対応** が示されていることにある。採用検討者が k1s0 を 10 年保守の前提で評価する際、最低限以下 5 軸の外部基準への適合度を見る:

- **CNCF Conformance**（Sonobuoy で取得）— k1s0 が動くクラスタが vanilla Kubernetes として正しいかの公式認定
- **SLSA**（Supply-chain Levels for Software Artifacts）— build artifact の改ざん検知 / provenance / in-toto attestation
- **OSSF Scorecard**（OpenSSF が運営する OSS セキュリティ自動採点）— 18 項目のチェックで OSS の管理品質を 0〜10 点で採点
- **OpenSSF Best Practices Badge**（旧 CII Best Practices Badge）— Passing / Silver / Gold の 3 段階で OSS のベストプラクティス遵守を認定
- **FIPS 140-3**（暗号モジュール認証）— 採用組織がエンタープライズ部門 / 政府系で k1s0 を採用する場合、自作 crypto に FIPS 認証 crypto module を要求する場合がある

これらは「テストピラミッドの L0–L10 とは orthogonal な横断軸」であり、特定の層に閉じず、複数の層 + リポジトリ全体（CI / docs / governance / build process）に跨る検証である。例えば SLSA L3 は build provenance を要求するため `make qualify-release` の build step に in-toto attestation 生成を埋め込む必要があり、これは L0–L10 のどの層にも属さない。OSSF Scorecard は「Branch-Protection」「Code-Review」「Dependency-Update-Tool」のような repository governance 項目を採点するため、テストではなく `.github/CODEOWNERS` / `Renovate` / `branch protection rules` の整備で点が決まる。

CNCF Sandbox 申請段階で testing maturity を評価される際、これら 5 軸の対応状況を申請書で求められる（CNCF Project Maturity Levels）。リリース時点（Phase 0）から少なくとも以下のレベルを満たす必要がある:

- Sonobuoy CNCF Conformance: **PASS**（vanilla K8s + 標準 CNI / CSI 上で）
- SLSA: **L3 相当**（build provenance + in-toto attestation + reproducible build）
- OSSF Scorecard: **8.0/10 以上**（CI-Tests を除く 17 項目で）
- OpenSSF Best Practices Badge: **Silver 取得**（Passing は最低限、Gold は Phase 3 目標）
- FIPS 140-3: **対応宣言**（採用組織の選択で FIPS provider を有効化できる構造）

これらをバラバラに実装すると、各検証の所要時間 / gate 配置 / artifact 生成経路が分散し、運用工数が破綻する。リリース時点で **統合された compliance gate** として ADR で正典化する必要がある。

選定では以下を満たす必要がある:

- **5 軸を統合**: 別々の qualify gate ではなく `make qualify-compliance` で一括実行可能
- **release artifact への同梱**: 各検証の出力（Sonobuoy report / SLSA provenance / OSSF Scorecard JSON / Badge URL / FIPS 宣言）を `tests/qualify-report/<version>/compliance/` に統合
- **Phase 移行への対応性**: Phase 0 では Silver / Phase 3 で Gold のような段階的 maturity 上昇が ADR で表現される
- **CI 移植容易性**: ADR-TEST-001 portable 制約と整合、Phase 1 で CI に移行する際の YAML 化が機械的

## 決定

**コンプライアンス検証は 5 軸（CNCF Conformance / SLSA / OSSF Scorecard / OpenSSF Best Practices Badge / FIPS 140-3）を `make qualify-compliance` target で統合実行し、release artifact の `tests/qualify-report/<version>/compliance/` に出力を一括同梱する。**

### 1. CNCF Conformance — Sonobuoy

- **配置**: ADR-TEST-003 の L5 conformance 内で実行（multipass kubeadm cluster 上）
- **実行方法**: `sonobuoy run --mode certified-conformance` を `tools/qualify/compliance/sonobuoy.sh` で wrap
- **所要時間**: 約 1〜2 時間（K8s e2e の certified-conformance スイート全体）
- **PASS 条件**: Sonobuoy が returns `Status: passed` を返し、failed test 0 件
- **artifact**: `compliance/sonobuoy-<k8s-version>.tar.gz` として保存、`compliance/sonobuoy-summary.md` で human readable summary

### 2. SLSA L3 — build provenance + in-toto attestation

- **配置**: `make qualify-release` の build step（tier1 Rust binary / Go binary / container image build 全件）
- **実装**: SLSA GitHub Generator のローカル等価実装として `tools/qualify/compliance/slsa-provenance.sh` を新設
  - 各 build artifact について SHA256 hash を取得
  - in-toto attestation（v1.0 spec）を JSON で生成（builder / source URI / git commit / build command を記録）
  - cosign で attestation に署名（Sigstore / Rekor 公開、Phase 0 ではローカル keypair で署名、Phase 3 で keyless OIDC 署名へ移行）
- **所要時間**: 約 5〜15 分（artifact 数に依存）
- **PASS 条件**: 全 build artifact に対し attestation が生成され、cosign verify で署名検証成功
- **artifact**: `compliance/slsa-attestation/<artifact-name>.intoto.jsonl` を artifact 数分生成、`compliance/slsa-summary.md` で集計

### 3. OSSF Scorecard

- **配置**: `make qualify-scorecard` を `make qualify-compliance` から呼ぶ
- **実装**: scorecard CLI（OpenSSF 公式 binary）をローカル実行 → JSON 出力
  - 18 項目のうち `CI-Tests` は ADR-TEST-001 の Phase 0 / Phase 1 移行待ちで満点取得不可と明示、他 17 項目で 8.0/10 以上を Phase 0 目標
- **所要時間**: 1〜3 分（GitHub API への問い合わせ）
- **PASS 条件**: `Aggregate Score >= 8.0`（17 項目平均、CI-Tests 除外）
- **artifact**: `compliance/scorecard.json` + `compliance/scorecard-summary.md`

### 4. OpenSSF Best Practices Badge

- **配置**: `bestpractices.coreinfrastructure.org` でプロジェクト登録、Silver 要件への自己評価結果を `compliance/openssf-badge-self-assessment.md` で版管理
- **Phase 0 目標**: Silver 取得（Passing は最低限）
- **Phase 3 目標**: Gold 取得
- **実装**: 自己評価項目（約 70 項目）の充足度を YAML で版管理（`docs/governance/openssf-badge-criteria.yaml`）し、`tools/qualify/compliance/openssf-badge.sh` で web 上の登録状態と乖離を検知
- **所要時間**: 1〜2 分（web スクレイピングまたは API）
- **PASS 条件**: 登録済 + Silver 取得済（自己評価で Silver 要件を全充足）
- **artifact**: `compliance/openssf-badge-status.json`

### 5. FIPS 140-3

- **配置**: tier1 自作 crypto（ADR-SEC-002 OpenBao / 自作 ZEN Engine 暗号機能）の crypto module 選択
- **実装**:
  - Rust crypto は `rustls` + `aws-lc-rs`（FIPS 認証済 BoringCrypto fork）の組み合わせを Phase 0 で採用
  - Go crypto は標準 `crypto/*` + Microsoft Go Crypto fork（FIPS 140-3 認証）を Phase 0 で対応宣言、Phase 1 で自動切替実装
  - 「FIPS provider を採用組織が選択可能」という構造を ADR で宣言、強制ではなく選択肢として提供
- **所要時間**: build / test 内で吸収（追加時間ゼロ）
- **PASS 条件**: `make qualify-release` 内で FIPS provider を有効化した build が成功し、tier1 unit test が PASS
- **artifact**: `compliance/fips-declaration.md`（FIPS 140-3 認証 crypto module の使用宣言 + 認証 ID リスト）

### 統合実行

`make qualify-compliance` で 5 軸を順次実行し、結果を `tests/qualify-report/<version>/compliance/` に統合する。`make qualify-release`（ADR-TEST-001）から呼ばれ、release tag 時に必須実行される。所要時間は合計約 1.5〜2.5 時間（Sonobuoy が支配的）。

## 検討した選択肢

### 選択肢 A: 5 軸統合（採用）

- 概要: Sonobuoy + SLSA L3 + OSSF Scorecard + OpenSSF Best Practices Badge + FIPS 140-3 を `make qualify-compliance` で一括実行、release artifact に統合同梱
- メリット:
  - 採用検討者が必要とする 5 軸すべてを 1 ヶ所（`compliance/`）で確認できる
  - CNCF Sandbox 申請時の testing maturity 説明が「`compliance/` ディレクトリの内容を見てください」で済む
  - Phase 移行（Silver → Gold、Phase 0 → Phase 3）が ADR で段階的に表現できる
  - artifact 生成経路が統合されるため、release tag 時の所要時間が予測可能（1.5〜2.5 時間）
- デメリット:
  - 5 軸すべてをリリース時点で実装する工数が大きい（10〜20 人日規模）
  - Sonobuoy の 1〜2 時間が release qualify 全体の time budget を圧迫
  - SLSA L3 の reproducible build 要件が Rust / Go の build 設定に追加制約を課す（`SOURCE_DATE_EPOCH` 固定 / build path 正規化 等）

### 選択肢 B: Sonobuoy のみ

- 概要: CNCF Conformance だけを取り、他軸（SLSA / OSSF / OpenSSF / FIPS）は Phase 1 以降に先送り
- メリット:
  - リリース時点の実装工数が最小（Sonobuoy のみ、3〜5 人日）
  - `compliance/` ディレクトリが小さく、採用検討者の認知負荷が低い
- デメリット:
  - **採用検討者の評価軸が不足**: SLSA / OSSF / OpenSSF Badge は CNCF Sandbox 採用基準で具体的に問われる項目で、無いと testing maturity が「basic」評価になる
  - **supply chain attack 耐性が示せない**: SLSA L3 を持たないと build artifact の改ざん検知が成立しない、採用検討者が「k1s0 の binary を信用していいか」を判定できない
  - **エンタープライズ採用で FIPS 要求が出た瞬間に対応不可**: FIPS 140-3 は宣言だけで実装は標準 crypto + 認証 fork の選択で済むため、リリース時点で対応宣言だけは出すべき

### 選択肢 C: CNCF 公式チェックのみ（Sonobuoy + Scorecard）

- 概要: CNCF / OpenSSF が直接運営する 2 軸（Sonobuoy + OSSF Scorecard）に絞る
- メリット:
  - 公式系列のみで採用検討者の説得力が高い
  - 実装工数が中程度（5〜10 人日）
- デメリット:
  - **SLSA は CNCF / OpenSSF どちらも公式系列だが**、軸として独立で扱う必要がある（supply chain 専用評価）
  - OpenSSF Best Practices Badge は OpenSSF 公式の認定で、Silver / Gold は Scorecard と独立した評価軸を持つ。Scorecard だけでは Badge は取得できない
  - FIPS は政府系 / 金融系採用の必須条件で、宣言レベルでも持たないと対応する選択肢が消える

### 選択肢 D: コンプライアンス検証を全部放棄

- 概要: リリース時点では CNCF Conformance / SLSA / OSSF / OpenSSF / FIPS のいずれも取らず、qualify report 内部の自前検証だけで release artifact を出す
- メリット:
  - 実装工数ゼロ
  - release qualify 時間が短縮
- デメリット:
  - **採用検討者の評価軸との対応が完全にゼロ**: 採用検討者が外部基準で k1s0 を評価した瞬間に「該当なし」となり、評価不能
  - CNCF Sandbox 申請が通らない（testing maturity 必須項目を欠く）
  - エンタープライズ採用 / 政府系採用の選択肢が消える
  - 個人 OSS の信頼性が「内部主張のみ」となり、release artifact 中心モデル（ADR-TEST-001）の根拠が崩れる

## 決定理由

選択肢 A（5 軸統合）を採用する根拠は以下。

- **採用検討者の評価軸との完全対応**: CNCF Sandbox 採用基準 / 商用採用検討者の RFP / 政府系採用の必須条件 / エンタープライズ部門のセキュリティレビュー、いずれも「Sonobuoy / SLSA / OSSF / OpenSSF / FIPS の対応状況」で k1s0 を評価する。5 軸すべてに最低限の対応宣言があれば、評価軸の欠落で「対象外」とされるリスクがゼロになる。選択肢 B / C はいずれかの軸が欠落して採用機会を逃す
- **release artifact 中心モデルとの整合**: ADR-TEST-001 で「採用検討者の信頼は release artifact で得る」と決定したが、release artifact が外部評価軸との対応を持たないと「内部主張のみ」になる。5 軸の出力を `compliance/` に統合同梱することで、release artifact が外部評価軸の対応証跡を物理的に保有する
- **CNCF Sandbox 申請の testing maturity 必須項目を網羅**: CNCF Project Maturity Levels で Sandbox / Incubating / Graduated に上がるごとに testing maturity の要求が上がる。Sandbox 申請時点で 5 軸すべてに対応宣言があると、Incubating / Graduated への昇格が滑らかになる。選択肢 D は Sandbox 申請が通らない
- **Phase 移行表現との整合**: 選択肢 A は Phase 0（Silver / SLSA L3）/ Phase 3（Gold / SLSA L4）のような段階的 maturity 上昇を ADR で表現できる。これは ADR-TEST-001 の Phase 表と整合し、「将来構想」が客観条件で起動する構造を保つ
- **統合実行による所要時間予測可能性**: `make qualify-compliance` で 5 軸を順次実行する設計により、release qualify 時の所要時間が 1.5〜2.5 時間と予測可能になる。各軸を別 gate でバラバラに実行すると、release tag 時にどれを必須でどれが任意かの判断が属人化する。選択肢 A は統合 gate により判断が機械化
- **個人 OSS の運用工数最小化**: 5 軸を `compliance/` に統合することで、artifact レビュー / Phase 移行レビュー / CNCF 申請レビューが 1 ヶ所で完結する。選択肢 B / C はリリース時点で軸を絞るが、Phase 1 / Phase 2 で軸を追加する際にディレクトリ構造の再編が必要になり、累積工数で選択肢 A を上回る

## 影響

### ポジティブな影響

- 採用検討者が `compliance/` ディレクトリを見るだけで 5 軸の対応状況を一括確認できる
- CNCF Sandbox 申請時の testing maturity 説明が release artifact の `compliance/` 同梱で済み、申請書記述工数が最小化
- SLSA L3 の build provenance により、k1s0 の binary を採用組織が cosign verify で改ざん検知可能になる
- OSSF Scorecard 8.0 以上の継続維持により、OSS としての管理品質が客観評価される
- OpenSSF Best Practices Badge Silver により、採用組織のセキュリティレビューで「OSS のベストプラクティス遵守を第三者認定済」と説明可能
- FIPS 140-3 対応宣言により、政府系 / 金融系 / エンタープライズ採用の選択肢が消えない
- Phase 0 / Phase 3 の段階的 maturity 上昇（Silver → Gold、SLSA L3 → L4）が ADR で表現され、採用検討者がロードマップを把握できる

### ネガティブな影響 / リスク

- 5 軸すべてのリリース時点実装で 10〜20 人日の初期工数が発生する。特に SLSA L3 の reproducible build 要件は Rust / Go の build 設定に `SOURCE_DATE_EPOCH` 固定 / build path 正規化 / dependency lock の追加制約を課す
- Sonobuoy の 1〜2 時間が release qualify 全体の time budget の支配項になる。multi-cluster で並列化する余地が無いため（multipass cluster 1 つでしか動かない）、release tag 時の連続マシン占有が伸びる
- OSSF Scorecard の 18 項目のうち `CI-Tests` は ADR-TEST-001 の Phase 0 / Phase 1 移行待ちで満点取得不可。`Aggregate Score 8.0/10` 目標を達成しても採用検討者から「CI-Tests が低い」と指摘される説明責務が継続発生する
- OpenSSF Best Practices Badge の Silver 自己評価項目（約 70 項目）の維持メンテが継続コスト。年次で `openssf-badge-criteria.yaml` の項目見直しが必要
- FIPS 140-3 認証 crypto module（aws-lc-rs / Microsoft Go Crypto fork）は本家 BoringCrypto / Go crypto の追従が遅れることがあり、最新 Rust / Go バージョンとの互換性が壊れるリスク。Phase 1 で「FIPS provider を採用組織が選択可能」という構造で吸収するが、デフォルト build とは別 binary を提供する 2 build 体制になる
- Sigstore / Rekor への公開が Phase 0 ではローカル keypair で署名（Phase 3 で keyless OIDC へ移行）と決定したが、ローカル keypair の private key 管理が起案者一人に集中する単一障害点リスク。OpenBao（ADR-SEC-002）に格納するが、起案者不在時に協力者が release tag を切る経路の設計が要る

### 移行・対応事項

- `tools/qualify/compliance/sonobuoy.sh` を新設し、L5 conformance 内で `sonobuoy run --mode certified-conformance` を wrap
- `tools/qualify/compliance/slsa-provenance.sh` を新設し、in-toto attestation 生成 + cosign 署名 + Rekor 公開を統合
- `tools/qualify/compliance/scorecard.sh` を新設し、scorecard CLI でローカル採点を実行
- `tools/qualify/compliance/openssf-badge.sh` を新設し、`bestpractices.coreinfrastructure.org` 上のプロジェクト状態と `openssf-badge-criteria.yaml` の差分を検知
- `tools/qualify/compliance/fips-build.sh` を新設し、aws-lc-rs / Microsoft Go Crypto fork を有効化した build を実行
- `Makefile` に `qualify-compliance` target を追加（5 軸を順次実行し `tests/qualify-report/<version>/compliance/` に集約）
- `docs/governance/openssf-badge-criteria.yaml` を新設し、Silver 自己評価項目の充足状況を版管理
- `docs/governance/COMPLIANCE-MATRIX.md` を新設し、5 軸の現状と Phase 移行ロードマップを採用検討者向けに公開
- `docs/governance/SLSA-PROVENANCE.md` を新設し、SLSA L3 の build process 図と検証手順（採用検討者向けの cosign verify 手順）を散文 + 図で記述
- `docs/governance/FIPS-DECLARATION.md` を新設し、対応 crypto module の認証 ID と build 切替手順を記述
- `.github/CODEOWNERS` / `Renovate` 設定 / branch protection rules / `SECURITY.md` を OSSF Scorecard 17 項目（CI-Tests 除く）の充足に整合させる
- `bestpractices.coreinfrastructure.org` でプロジェクト登録を起案者が手動実行、Silver 自己評価を完了
- ADR-SEC-002（OpenBao）の「帰結」に「SLSA L3 ローカル keypair 管理」を追記する relate-back 作業
- Phase 1 で SLSA L3 の cosign 署名を keyless OIDC（Sigstore Fulcio）に移行する手順を `docs/governance/SLSA-PROVENANCE.md` で予告

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— release artifact 中心モデルと compliance/ 同梱の整合
- ADR-TEST-003（テストピラミッド L0–L10）— Sonobuoy が L5 conformance 内に配置される根拠
- ADR-TEST-004（kind + multipass 二層 E2E）— Sonobuoy が multipass kubeadm 上で動く根拠
- ADR-CNCF-001（CNCF Conformance）— Sonobuoy 採用の前提
- ADR-SEC-002（OpenBao）— SLSA L3 ローカル keypair 管理
- ADR-DEP-001（Renovate）— OSSF Scorecard `Dependency-Update-Tool` 項目の充足
- NFR-E-ENC-001〜003（暗号要件）— FIPS 140-3 対応の根拠
- NFR-F-STD-001（業界標準）— 5 軸統合の整合
- Sonobuoy: sonobuoy.io
- SLSA: slsa.dev
- in-toto: in-toto.io
- Sigstore / Cosign / Rekor: sigstore.dev
- OSSF Scorecard: scorecard.dev
- OpenSSF Best Practices Badge: bestpractices.coreinfrastructure.org
- FIPS 140-3 認証 crypto module: aws-lc-rs（aws.amazon.com/security/）、Microsoft Go Crypto（github.com/microsoft/go）
- 関連 ADR（採用検討中）: ADR-TEST-009（観測性 E2E）
