# ADR-TEST-001: Test Pyramid（UT 70% / 結合 20% / E2E 10%）+ testcontainers でテスト戦略を正典化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / QA リード / 開発者体験チーム

## コンテキスト

k1s0 は tier1（Rust + Go）/ tier2（Go + .NET）/ tier3（TypeScript + .NET MAUI）の 4 言語並走の採用側 PaaS であり、tier1 の不具合は全 tier2 / tier3 に波及する構造を持つ。テストが不足するとプラットフォーム障害の検出を採用組織の運用現場に委ねることになり、MTTR と Change Failure Rate（DX-MET-005、5% 目標）の双方が劣化する。一方で過剰なテスト投資は CI 時間を長期化させ、Lead Time 1h（DX-MET-003）と PR 5 分の CI 時間予算を破綻させる。

このバランスを「個別の判断者」の主観に委ねると 4 言語ごとに散逸し、採用組織の世代交代でテスト戦略が崩壊する。tier1 の品質が全プラットフォーム品質を決める構造のため、リリース時点でテスト戦略の **層構成 / 比率 / ツール選定 / CI 時間予算** を ADR で正典化する必要がある。さらに、本 ADR は概要設計 `docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md` の 4 箇所で「前提」として cite されている docs-orphan であり、本体起票による参照整合性確保も同時の責務である。

加えて、`docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md:209` で **ADR-DEVEX-003（テスト戦略標準化）** が別 cite として参照されており、これも本体未起票の docs-orphan である。両 ADR は「k1s0 のテスト戦略を標準化する」という同一射程を扱うため、本 ADR で **ADR-DEVEX-003 を吸収** し、cite 元の参照を ADR-TEST-001 に書き換えることで docs-orphan を 2 件解消する。

テスト戦略の階層モデルとして実用上は以下が候補となる:

- **Test Pyramid**（Mike Cohn 1990s）— UT を最大層、結合・E2E を上層に積む古典モデル
- **Test Trophy**（Kent C. Dodds 2018）— integration を最大層に、UT を圧縮
- **Test Diamond** — integration を最大、UT を最小化
- **テスト戦略を定型化しない**（個別判断）

結合テストの実装手段として、mock/stub 中心 vs testcontainers の選択も併存する。前者は実装が軽いが本番乖離が生まれる（モックが本物と乖離した瞬間に検出が漏れる）。後者は Docker daemon 依存が生まれるが、本番と同じバイナリ・同じ設定で検証できる。

CI 時間予算（PR 5 分 / main 10 分 / 夜間バッチ 30 分）は IMP-CI-RWF-010 で reusable workflow 4 本構成として既に物理化されており、本 ADR で決定する Test Pyramid 比率はこの予算と整合する必要がある。

選定では以下を満たす必要がある:

- **CI 時間予算（PR 5 分 / main 10 分）** に層別実行時間が収まる
- **本番乖離の構造的回避**（mock の本番乖離を許容しない設計）
- **4 言語（Rust / Go / .NET / TypeScript）すべて**で成熟ライブラリが揃う
- **採用組織の世代交代**後も保守可能な業界標準に乗る
- **既存 ADR との整合**（ADR-TIER1-002 Protobuf gRPC / ADR-CICD-* / IMP-CI-RWF-010 / IMP-CI-QG-065）

## 決定

**k1s0 のテスト戦略は Test Pyramid（UT 70% / 契約 5% / 結合 20% / E2E 5%）+ testcontainers ベースの結合テストで構成する。** 詳細層の実装は概要設計 `05_テスト戦略方式.md`（DS-DEVX-TEST-001〜011）で展開され、本 ADR はその構想設計レベルの基本方針を確定する。

層構成と比率（数量目安は 1 サービスあたり）:

| 層 | 比率 | 数量目安 | 実行時間 | ツール |
|----|------|---------|---------|--------|
| UT（単体） | 70% | 300〜500 テスト | < 30 秒 | cargo test / go test / xUnit / Jest |
| 契約テスト | 5% | 20〜30 契約 | < 30 秒 | pact-rust / pact-go |
| 結合 | 20% | 80〜100 テスト | < 3 分 | testcontainers（Rust / Go / .NET） |
| E2E | 5% | 20〜30 シナリオ | < 10 分 | Playwright（UI） / k6（API 負荷） |

横断軸（Test Pyramid と orthogonal）:

| 軸 | 採用ツール | 確定段階 |
|----|----------|---------|
| カバレッジ硬性基準 | 行 80% / ブランチ 70%（cargo tarpaulin / go test -cover / coverlet / jest --coverage） | リリース時点 |
| SAST | Semgrep / gosec / clippy | リリース時点 |
| SCA | Trivy + Grype 並列実行 | リリース時点 |
| DAST | OWASP ZAP | 採用後の運用拡大時 |
| Chaos | LitmusChaos（ADR-TEST-004 で確定、CNCF Incubating + Apache 2.0、`infra/chaos/` 配下に CRD 配置、週次 CronChaosEngine） | 採用後の運用拡大時 |

CI 時間予算（IMP-CI-RWF-010 / IMP-CI-QG-065 と整合、4 段フェーズ分離は ADR-TEST-007 で確定）:

- PR 時（タグなし）: UT + 契約 + 結合 + SAST/SCA = 合計 5 分以内
- main 時: PR 構成 + Build = 合計 10 分以内
- nightly（@slow + @nightly タグ）: E2E + 観測性 E2E = 合計 30〜45 分（朝会前完了）
- weekly（@security タグ）: nightly + DAST = 合計 1〜2 時間
- release tag（全タグ）: nightly + Chaos + Upgrade drill + DR drill + Conformance = 合計 2〜4 時間（release qualify）

**結合テストの実装手段は testcontainers を標準とし、mock/stub のみによる結合層は許容しない。** Docker daemon 依存は CI runner では Docker-in-Docker（DinD）または `/var/run/docker.sock` の bind mount で吸収する（既存 reusable workflow で対応済）。ローカル開発では devcontainer の `docker-outside-of-docker` Feature 経由で host docker daemon に接続する（ADR-DEV-002）。

**ADR-DEVEX-003（テスト戦略標準化）は本 ADR が吸収する。** cite 元 `docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md:209` の参照を `ADR-TEST-001` に書き換え、`docs/AUDIT.md` の docs-orphan リストから 2 件を除外する。

## 検討した選択肢

### 選択肢 A: Test Pyramid + testcontainers（採用）

- 概要: Mike Cohn の Test Pyramid（UT 70% / 結合 20% / E2E 10%）を採用、結合層は testcontainers で本番依存と組み合わせて検証
- メリット:
  - 業界標準で採用組織の世代交代後も保守可能
  - UT 70% で速度担保、E2E 10% で CI 時間予算（PR 5 分）に収まる
  - testcontainers が 4 言語（Rust / Go / .NET / TypeScript）すべてに成熟ライブラリ
  - mock/stub の本番乖離を構造的に回避
  - ADR-TIER1-002（Protobuf gRPC）の契約検証と pact 連携が成立
- デメリット:
  - testcontainers の Docker daemon 依存、CI runner で DinD or socket mount 設定が必要
  - 結合テスト 3 分制約が tight、コンテナ起動時間の最適化（distroless / Ryuk 共有セッション）が要る

### 選択肢 B: Test Trophy（integration 重視）

- 概要: Kent C. Dodds が提唱する階層、integration を最大層、UT を圧縮し E2E は最小化
- メリット:
  - integration が最大化され、本物の依存と組み合わせた検証量が増える
  - フロントエンド React 系プロジェクトで人気
- デメリット:
  - **UT が圧縮されるため CI 時間予算（PR 5 分）に収まらない**: integration 主体だと 3 分以上かかり、PR ゲートで開発者体験が悪化
  - 4 言語並走の k1s0 では tier1（Rust / Go の高頻度 unit lint）が成立しない
  - tier1 の Protobuf gRPC レイヤは UT で網羅すべきロジック（バリデーション / シリアライズ / 暗号）が大量にあり、integration に逃すと検証粒度が粗くなる

### 選択肢 C: Test Diamond（integration を最大、UT 最小）

- 概要: integration を最大、UT を最小化（モックを廃止、すべて本物で検証）
- メリット:
  - 本番乖離が原理的にゼロに近づく
  - mock 維持の運用工数がゼロ
- デメル:
  - **CI 時間が爆発**: 全テストが integration 経路で 30 分以上、Lead Time 1h を破壊
  - UT 圧縮で「純粋ロジックの単体検証」が薄くなり、複雑な業務ルール（ZEN Engine の決定モデル等）の網羅検証が成立しない
  - 4 言語の言語ネイティブ test framework の使用機会が減り、開発者体験が劣化

### 選択肢 D: テスト戦略を定型化しない（自由形式）

- 概要: ADR で層構成・比率・ツールを規定せず、各サービス / 各言語の判断に委ねる
- メリット:
  - 規約整備工数ゼロ
  - 開発者の自由度が高い
- デメリット:
  - **属人化が爆発**: サービスごとに Test Pyramid / Trophy / Diamond / 自由形式が混在し、CI 時間予算が機械的に守れない
  - 採用組織が「k1s0 のテスト戦略とは何か」を ADR から答えられず、10 年保守の前提を欠く
  - tier1 の不具合が tier2 / tier3 に波及する構造に対し、層別の責務分界が崩壊
  - DS-DEVX-TEST-001〜011 の概要設計と整合する ADR 上位決定が不在となる

## 決定理由

選択肢 A（Test Pyramid + testcontainers）を採用する根拠は以下。

- **CI 時間予算との唯一の整合性**: PR 5 分 / main 10 分の予算（IMP-CI-RWF-010 / IMP-CI-QG-065 で物理化済）に層別実行時間（UT 30 秒 / 契約 30 秒 / 結合 3 分 / SAST/SCA 1 分 = 合計 5 分）が収まるのは Test Pyramid のみ。選択肢 B / C は integration 主体で 3 分制約に収まらず、Lead Time 1h 目標を破壊する
- **業界標準の採用組織スキル流用性**: Test Pyramid は Mike Cohn 1990s 以降の業界標準で、採用組織の世代交代後も「世間で標準的に学ぶスキル」で保守できる。選択肢 B（Test Trophy）はフロントエンド界隈で局所的に人気だが、4 言語並走の PaaS では浸透していない
- **本番乖離の構造的回避**: testcontainers は本番と同じバイナリで結合検証するため、mock/stub の本番乖離（モックが本物と乖離した瞬間に検出が漏れる）を原理的に排除する。選択肢 D（自由形式）では mock 中心の結合層が選ばれる可能性があり、本番乖離が累積するリスクが残る
- **4 言語すべての成熟ライブラリ**: testcontainers-rs / testcontainers-go / Testcontainers for .NET / @testcontainers/postgresql など、k1s0 の 4 言語すべてに公式 / 準公式ライブラリがあり、言語横断の学習コストが最小
- **既存 ADR / IMP との整合**: ADR-TIER1-002（Protobuf gRPC）の契約検証は pact-rust / pact-go の Consumer-Driven Contract で成立、IMP-CI-RWF-010（reusable workflow 4 本）の test job が本 ADR の各層に対応、IMP-CI-RWF-018（coverage 段階導入）が本 ADR のカバレッジ硬性基準と整合する
- **docs-orphan 解消の同時達成**: 既存 cite 4 箇所（ADR-TEST-001）+ ADR-DEVEX-003 cite 1 箇所の合計 2 件の docs-orphan を本 ADR の起票で同時解消できる。「決定は docs に記述されているが ADR file 形式に未昇格」状態を 2 件減らす（`docs/AUDIT.md` 監査軸との整合）

## 影響

### ポジティブな影響

- CI 時間予算 PR 5 分 / main 10 分が成立し、Lead Time 1h（DX-MET-003）の構造的担保が確立する
- testcontainers ベースの結合テストにより mock/stub の本番乖離が構造的に排除される
- 4 言語並走でテスト戦略が一貫し、tier1 / tier2 / tier3 の品質保証が単一の ADR で正典化される
- DS-DEVX-TEST-001〜011（概要設計）と本 ADR が双方向トレーサビリティを持つ
- ADR-TEST-001 + ADR-DEVEX-003 の docs-orphan 2 件解消（AUDIT.md docs-orphan: 40 → 38 推定）
- 採用検討組織が「k1s0 のテスト戦略とは Test Pyramid + testcontainers」と単一 ADR から答えを得られる

### ネガティブな影響 / リスク

- testcontainers は Docker daemon 依存があり、CI runner では Docker-in-Docker（DinD）または `/var/run/docker.sock` の bind mount 設定が必要（IMP-CI-RWF-010 で対応済だが、新規 runner image / 別 runtime（Podman 等）への移行時に再対応が要る）
- 結合テスト 3 分制約は tight で、依存コンテナ数 / 起動時間の継続最適化（distroless image / Ryuk 共有セッション）が必要。Phase 1 以降の依存追加で 3 分超過する場合は概要設計 `05_テスト戦略方式.md` の見直しを起動する
- Chaos ツールの選定が概要設計（Chaos Mesh）と構想設計（LitmusChaos、`02_構想設計/04_CICDと配信/03_テスト戦略.md` / `02_構想設計/03_技術選定/03_周辺OSS/02_周辺OSS.md`）で乖離している既存問題は本 ADR で解決しない。別 ADR（ADR-TEST-004 候補）で確定する
- E2E（Playwright / k6）が PR 時に走らないため、本番乖離バグが夜間バッチまで検出されない待ち時間がある（最大 24 時間）。緊急 PR 時は手動で `make verify` 経由の E2E 実行を Runbook 化する必要

### 移行・対応事項

- `docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md:209` の `ADR-DEVEX-003` 参照を `ADR-TEST-001` に書き換える（本 ADR 起票と同 commit で実施）
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` に `TEST` 系列を新設し、本 ADR を「テスト戦略」カテゴリで登録する
- `docs/04_概要設計/90_付録/02_ADR索引.md:344` の ADR-TEST-001 行は既存記載が本 ADR と一致するため変更不要、ただし本体 ADR ファイルが起票されたことを次回 audit 走行で AUDIT.md docs-orphan リストから ADR-TEST-001 / ADR-DEVEX-003 を除外する
- E2E（L4）の自動化経路は別 ADR で再起票する（前 ADR-TEST-002 は撤回済、テスト基盤刷新後に再策定）
- Chaos ツール選定の不整合（概要設計 Chaos Mesh / 構想設計 LitmusChaos）を解消する別 ADR（ADR-TEST-004 候補）を採用後の運用拡大時の前に起票する
- testcontainers の言語別ライブラリバージョン固定を `tools/devcontainer/profiles/<role>/Dockerfile` の依存リストに反映（既存 SHA256 検証と整合）
- カバレッジ閾値の段階導入（IMP-CI-RWF-018: リリース時点 = 計測のみ警告 / 採用初期 = 80% baseline / 運用拡大 = 90% + mutation testing）を本 ADR のカバレッジ硬性基準（行 80% / ブランチ 70%）と整合させ、`05_テスト戦略方式.md` で Phase 別の整合表を整備する

## 参考資料

- `docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md` — 本 ADR を「前提」として cite する概要設計（DS-DEVX-TEST-001〜011 を展開）
- `docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md` — tests レイアウトの正典（IMP-DIR-COMM-112）、ADR-DEVEX-003 を cite していたが本 ADR が吸収
- `docs/04_概要設計/90_付録/02_ADR索引.md:344` — ADR-TEST-001 の概要設計索引
- `docs/AUDIT.md` — docs-orphan / code-orphan の監査軸、本 ADR 起票で 2 件解消
- ADR-TIER1-002（Protobuf gRPC）— 契約テスト pact 連携の前提
- ADR-CICD-001/002/003（Argo CD / Argo Rollouts / Kyverno）— CI / CD パイプラインとテストの統合
- ADR-DEV-001（Paved Road）— Dev Container と testcontainers の統合経路
- ADR-DEV-002（WSL2 + Docker runtime）— testcontainers の Docker daemon 経路
- IMP-CI-RWF-010 / IMP-CI-RWF-018（reusable workflow / coverage 段階導入）
- IMP-CI-QG-060 / IMP-CI-QG-065（quality gate 順序 / coverage 段階）
- IMP-CI-PF-031（path-filter 単一真実源）
- DX-TEST-001〜008（要件定義のテスト戦略要件）
- NFR-E-RSK-002（ペネトレーションテスト）/ NFR-E-AV-001/002（コンテナイメージスキャン）/ NFR-E-WEB-001（OWASP Top 10）
- DX-MET-003（Lead Time 1h）/ DX-MET-005（Change Failure Rate 5%）
- 関連 ADR（採用検討中、本 ADR から派生）: ADR-TEST-003（CNCF Conformance / Sonobuoy）/ ADR-TEST-004（Chaos Engineering ツール選定）/ ADR-TEST-005（Upgrade / DR drill）/ ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）。E2E 自動化経路 / 観測性 E2E はテスト基盤刷新後に新 ADR で再策定
