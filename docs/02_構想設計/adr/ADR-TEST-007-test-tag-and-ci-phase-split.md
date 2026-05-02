# ADR-TEST-007: テスト属性タグ（@slow / @flaky / @security / @nightly）と CI 実行フェーズ分離を正典化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / QA リード / 開発者体験チーム

## コンテキスト

ADR-TEST-001〜006 で Test Pyramid 各層と orthogonal 軸（Conformance / Chaos / Upgrade / DR / 観測性）の実装方針を確定したが、**個別テストケースを「いつ・どこで・どの頻度で」実行するか** の制御軸が未定義のままである。具体的には:

- **長時間テスト**（5 秒超の UT、3 分超の結合）が PR 時に実行されると ADR-TEST-001 の CI 時間予算（PR 5 分）を破壊する
- **Flaky テスト**（再実行で結果が変わる）が PR ゲートを偶発的に塞ぎ、開発者体験を破壊する。quarantine 機構なしでは「またこのテストが落ちた」が PR レビュー疲労を生む
- **セキュリティテスト**（OWASP ZAP / DAST / penetration）は実行時間が長く、誤検知も多いため PR ゲートでは扱えない
- **夜間専用テスト**（L4 standard E2E / 観測性 E2E）が PR ゲートで誤起動するとリソース消費が爆発

既存 IMP-CI-PF-031（path-filter）は **ファイル変更パスベース** の実行制御で、tier 別 / 言語別の起動を担う。しかし「path は同じだが、テストケース個別の属性で実行可否が決まる」軸は path-filter で扱えない。例えば tier1 の Go test の中で「short test は PR 実行 / long test は nightly 実行」のような分離は、テストケース自体に属性タグを付与する仕組みが要る。

選定では以下を満たす必要がある:

- **4 言語（Rust / Go / .NET / TypeScript）すべてで実装可能**（言語ネイティブ test framework の機能で属性付与）
- **既存 IMP-CI-* との整合**（IMP-CI-PF-031 / IMP-CI-RWF-010 / IMP-CI-QG-060 と orthogonal で並立）
- **採用組織の標準スキル流用性**（独自 DSL / アノテーション体系を作らない）
- **flaky 検出の自動化**（手作業 quarantine ではなく CI 統計で自動判定）
- **実行フェーズの一意性**（PR / nightly / weekly / release tag のどれで起動するかが属性タグから自動決定）

## 決定

**テスト属性タグを 4 種類（`@slow` / `@flaky` / `@security` / `@nightly`）に正典化し、CI 実行フェーズを 4 段（PR / nightly / weekly / release）に分離する。属性タグは言語ネイティブ test framework の機能で付与し、独自 DSL は導入しない。**

### 1. 4 タグの定義

| タグ | 定義 | 実行フェーズ | 想定 |
|------|------|------------|------|
| `@slow` | 単一テストの所要時間が **5 秒超**（UT）または **3 分超**（結合） | nightly | 重い計算 / 大量データ生成 / 並列度高 |
| `@flaky` | 直近 20 PR で **失敗率 ≥ 5%** と CI 統計が判定したテスト | quarantine（PR ゲートからは除外、別 workflow で報告） | 本来 fail させるべきだが原因調査中 |
| `@security` | OWASP ZAP / DAST / penetration / SAST フルスキャン等の重いセキュリティ検証 | weekly | 採用後の運用拡大時 |
| `@nightly` | 夜間専用テスト（L4 standard E2E / 観測性 E2E / 統合テスト 5 分超） | nightly | リソース消費が大きく PR 時不適 |

タグは AND 結合可能（例: `@slow @security` は週次のみ実行）。タグなしのテストは **PR ゲートで必ず実行** される（デフォルト）。

### 2. 4 段実行フェーズ

| フェーズ | 起動 trigger | 実行対象 | 所要時間予算 |
|---------|-------------|---------|------------|
| PR | `pr.yml` | タグなしテスト全件 | 5 分以内（ADR-TEST-001） |
| nightly | `nightly.yml`（cron 03:00 JST、ADR-TEST-002） | タグなし + `@slow` + `@nightly` | 30〜45 分 |
| weekly | `weekly.yml`（新設、cron 月曜 03:00 JST） | nightly + `@security` | 1〜2 時間 |
| release tag | `tools/release/cut.sh`（採用初期で新設） | 全件（タグなし + 4 タグ全） | 2〜4 時間（release qualify） |

`@flaky` タグはどのフェーズでも **メイン実行から除外**し、別 workflow（`flaky-report.yml`、新設）で結果のみ収集して採用組織の SRE に通知。連続 4 週 fail 率 ≥ 5% の場合は ADR-TEST-007 の「採用後の運用拡大時」にて quarantine 解除（修正必須）の運用ルールを Runbook 化（`ops/runbooks/RB-TEST-001-flaky-quarantine.md`）。

### 3. 言語別実装

| 言語 | 属性タグ実装手段 |
|------|----------------|
| Rust | `#[ignore = "slow"]` + `cargo nextest run --filter 'not test(slow)'`（denylist） |
| Go | `// +build slow` build tag + `go test -tags=slow`（または `t.Skip` + `testing.Short()`） |
| .NET | xUnit `[Trait("Category", "slow")]` + `dotnet test --filter Category!=slow` |
| TypeScript | Vitest `test.skip` + custom `defineConfig` で fileFilter による include/exclude |

各言語で「単一の build tag / trait / filter で属性タグを表現する」標準手法のみを採用。独自 DSL / カスタムアノテーション / マクロは導入しない。

### 4. flaky 自動検出

`tools/qualify/flaky-detector.py`（採用初期で新設）が以下を実行:

1. 直近 20 PR の CI 結果（GitHub Actions API）を取得
2. テストケース別 fail 率を計算
3. 5% 超のケースを `tests/.flaky-quarantine.yaml` に自動追加（PR で起案者レビュー）
4. quarantine 入りしたテストは以降の PR で `@flaky` 扱いとなり PR ゲート除外

採用初期で `flaky-detector.py` を整備し、採用後の運用拡大時で運用フローに組み込む。

### 5. IMP-CI-TAG-* 系列

本 ADR で IMP-CI-TAG-001〜005 を確定する（実装段階の正典記述は別 commit で `90_対応IMP-CI索引/01_対応IMP-CI索引.md` へ展開）:

- IMP-CI-TAG-001: 4 タグ（`@slow` / `@flaky` / `@security` / `@nightly`）の正典化
- IMP-CI-TAG-002: 4 段実行フェーズ（PR / nightly / weekly / release tag）の起動 trigger 一意化
- IMP-CI-TAG-003: 言語別属性タグ実装（Rust ignore / Go build tag / .NET Trait / Vitest filter）
- IMP-CI-TAG-004: flaky 自動検出（直近 20 PR で fail 率 ≥ 5% を quarantine 自動追加）
- IMP-CI-TAG-005: `tests/.flaky-quarantine.yaml` の PR レビュー必須化（quarantine 解除も同様）

## 検討した選択肢

### 選択肢 A: 属性タグ + フェーズ分離（採用）

- 概要: 4 タグ + 4 段フェーズで CI 実行を制御、言語ネイティブ test framework の機能で属性付与
- メリット:
  - **既存 IMP-CI-PF-031（path-filter）と orthogonal で並立**: ファイル変更ベース + 属性ベースの 2 軸制御
  - 4 言語すべてで言語ネイティブの標準手法（build tag / Trait / ignore）で実装可能、独自 DSL なし
  - flaky 自動検出で開発者体験が破壊されない
  - 4 段フェーズが ADR-TEST-001 の CI 時間予算（PR 5 分 / main 10 分 / 夜間 30 分）と整合
  - release tag 時に全件実行することで release qualify が成立
- デメリット:
  - 4 タグの維持メンテで「属性タグの増殖」を防ぐ規律が要る（新タグ追加は本 ADR 改訂必須）
  - 言語別 implementation が 4 系統並走するため、新言語追加時の作業がある（ADR-TIER1-001 と同等の判断）
  - flaky 自動検出の閾値（5%）の調整が継続コスト

### 選択肢 B: 属性タグなし、全テストを PR で実行

- 概要: テスト属性タグを導入せず、全テストを PR ゲートで毎回実行
- メリット:
  - 実装工数ゼロ
  - 「PR が緑なら全テスト緑」が単純
- デメリット:
  - **PR 5 分制約を破壊**: L4 standard E2E（30 分）/ 観測性 E2E（30〜45 分）/ DAST（60 分）が PR で走ると Lead Time 1h を破壊
  - flaky 検出機構なし、開発者が「またあのテストが落ちた」を毎回手動で再実行する疲労累積
  - セキュリティテスト（OWASP ZAP）の実行頻度が PR ベースだと過剰、リソース浪費

### 選択肢 C: ファイル名規約で分離

- 概要: テストファイル名で `_slow_test.go` `_security_test.go` 等の suffix 規約で分離
- メリット:
  - shell script で grep するだけで実行制御可能、追加 framework 機能不要
  - 視覚的に分かりやすい
- デメリット:
  - **テストケース粒度の制御が不可**: 1 ファイル内に slow と fast が混在する場合に分離できない
  - ファイル名規約は機械的だが、属性タグ（複数タグ AND 結合）が表現できない
  - 採用組織のスキル（言語ネイティブ test framework の trait / build tag）から外れた独自規約

### 選択肢 D: 別 module で分離

- 概要: slow / security 等を `tests/slow/` `tests/security/` の別ディレクトリ + 別 Go module で物理分離
- メリット:
  - ディレクトリで完全に隔離、import 関係も切れる
  - 既存 `tests/e2e/` のディレクトリ慣習と整合
- デメリット:
  - **個別テストケース粒度で属性が変わる場合に対応不可**: 同じテストファイル内で「fast 部分は PR / slow 部分は nightly」が表現できない
  - module 数が爆発（slow / flaky / security / nightly × 言語 4）、保守負担増
  - flaky の自動検出で「quarantine 入り」時に module 移動が必要、機械化困難

## 決定理由

選択肢 A（属性タグ + フェーズ分離）を採用する根拠は以下。

- **既存 IMP-CI-PF-031 との orthogonal 並立**: path-filter（ファイル変更ベース）と属性タグ（テストケースベース）が独立した制御軸として並立し、reusable workflow（IMP-CI-RWF-010）から両軸を組み合わせて使える。選択肢 B（属性なし）はこの粒度制御を欠き、選択肢 C / D は属性タグ表現力で劣る
- **4 言語ネイティブ手法での実装**: Rust ignore / Go build tag / xUnit Trait / Vitest filter は採用組織の開発エンジニアが標準的に学ぶ機能で、独自 DSL の学習コストゼロ。選択肢 C（ファイル名規約）は独自規約で採用組織のスキル流用性が低下
- **CI 時間予算との整合**: 4 段フェーズが ADR-TEST-001 の予算（PR 5 分 / main 10 分 / 夜間 30 分 / release 2-4 時間）と機械的に対応する。選択肢 B は予算を破壊、選択肢 D は粒度不足で予算境界が表現できない
- **flaky 自動検出の独自価値**: `tools/qualify/flaky-detector.py` で直近 20 PR の fail 率を算出し 5% 超を quarantine 自動追加する仕組みは、本 ADR の選択肢 A でしか実装できない。選択肢 C / D は属性タグの動的付与（quarantine 入り）を機械化できない
- **release tag 時の全件実行による release qualify 成立**: 選択肢 A では release tag 時に `@slow / @flaky / @security / @nightly` を含む全件を実行することで release qualify が成立する。選択肢 B は釈然たる release qualify 概念がない、選択肢 D は module 跨ぎ実行のオーケストレーションが複雑
- **4 タグの最小セット**: 4 タグは「時間 / 信頼性 / 性質 / 実行枠」の 4 軸を最小限で表現し、これ以上分割しても採用組織の運用工数で維持できない上限。新タグ追加は本 ADR の改訂必須として「タグ増殖」を構造的に抑制

## 影響

### ポジティブな影響

- 既存 IMP-CI-PF-031（path-filter）と orthogonal な属性タグ軸が確立し、reusable workflow から 2 軸組み合わせで実行制御が可能になる
- 4 言語すべてで言語ネイティブ test framework の標準手法を使うため、採用組織の開発エンジニアが追加学習なしで属性タグを付与できる
- flaky 自動検出により「またあのテストが落ちた」の PR レビュー疲労が解消される
- 4 段フェーズが ADR-TEST-001 / 002 の CI 時間予算と機械的に対応し、release tag 時の release qualify 概念が成立する
- IMP-CI-TAG-001〜005 が確定し、`docs/05_実装/30_CI_CD設計/` 配下への展開経路が確立
- 採用検討組織が「k1s0 はテスト属性で実行制御している」と理解でき、testing maturity 評価が補強される

### ネガティブな影響 / リスク

- 4 タグの維持メンテで「属性タグの増殖」を防ぐ規律が要る。新タグ追加は本 ADR の改訂必須とし、PR レビューで「なぜこのタグが必要か」を毎回問う運用
- 言語別 implementation が 4 系統並走するため、新言語追加時（例: Python SDK）に属性タグ実装を追加する作業が発生（ADR-TIER1-001 言語政策と同等）
- flaky 自動検出の閾値（5%）は経験則で設定、採用組織の運用実績で適切値を継続調整する必要
- `@security` タグの weekly フェーズは採用後の運用拡大時に整備、リリース時点では `@nightly` までで運用
- xUnit Trait / Vitest filter / Rust ignore / Go build tag の差で開発者がタグ付与を忘れる事故が発生し得る。lint 規約（例: 5 秒超の test に `@slow` 必須）を `tools/lint/` で機械検証する経路を採用初期で整備
- `tests/.flaky-quarantine.yaml` の PR レビューが起案者に集中、採用後の運用拡大時で SRE / QA リードへの分散が要る

### 移行・対応事項

- リリース時点で本 ADR + IMP-CI-TAG-001〜005 の確定のみ、実装は採用初期から段階導入
- 採用初期で 4 言語の test framework に 4 タグの implementation 例を `tests/<lang>/examples/` で整備
- 採用初期で `.github/workflows/weekly.yml` を新設、cron 月曜 03:00 JST + workflow_dispatch で `@security` タグ実行
- 採用初期で `.github/workflows/flaky-report.yml` を新設、`@flaky` 結果収集と SRE 通知
- 採用初期で `tools/qualify/flaky-detector.py` を新設、直近 20 PR の fail 率算出と quarantine 自動追加
- 採用初期で `tools/lint/test-tag-lint.sh` を新設、5 秒超の test に `@slow` 必須等の機械検証
- 採用初期で `tests/.flaky-quarantine.yaml` の PR レビュー必須化を `CODEOWNERS` で整備
- 採用初期で `ops/runbooks/RB-TEST-001-flaky-quarantine.md` を新設、quarantine 入り / 解除のフローを 8 セクション形式（ADR-OPS-001 準拠）で記述
- 採用後の運用拡大時で `@security` タグ（OWASP ZAP / DAST）を weekly 実行に組み込み（ADR-TEST-001 で「採用後の運用拡大時」と決定済の DAST を本 ADR で実行枠化）
- 採用後の運用拡大時で `tools/release/cut.sh` を新設、release tag 時に 4 タグ含む全件実行を強制
- ADR-TEST-001 の「決定」表内 CI 時間予算行に「PR / nightly / weekly / release の 4 段は ADR-TEST-007 で確定」を追記する relate-back
- IMP-CI-PF-031 の relate-back に「属性タグ軸（ADR-TEST-007）と orthogonal で並立」を追記
- ADR-TIER1-001 の「帰結」に「言語追加時は本 ADR の 4 タグ implementation 追加が必要」を追記する relate-back

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— CI 時間予算（PR 5 分 / main 10 分 / 夜間 30 分）の前提
- ADR-TEST-002（E2E 自動化）— `nightly.yml` の前例（本 ADR が `weekly.yml` / `flaky-report.yml` を追加）
- ADR-TEST-006（観測性 E2E）— `@nightly` タグで実行される observability E2E の例
- ADR-TIER1-001（Go + Rust ハイブリッド）— 4 言語 implementation の前提
- ADR-CICD-001（Argo CD）— release tag 連動の前提
- ADR-OPS-001（Runbook 標準化）— RB-TEST-001 の形式根拠
- IMP-CI-PF-031（path-filter 単一真実源）— 本 ADR の orthogonal 軸
- IMP-CI-RWF-010（reusable workflow 4 本）— 5 / 6 本目（_reusable-e2e / _reusable-conformance）に加えて weekly / flaky-report の起動経路
- IMP-CI-TAG-001〜005（本 ADR で確定）
- DX-MET-003（Lead Time 1h）/ DX-MET-005（Change Failure Rate 5%）
- 関連 ADR（採用検討中）: なし（テスト戦略系列 ADR-TEST-001〜007 の最終 ADR）
