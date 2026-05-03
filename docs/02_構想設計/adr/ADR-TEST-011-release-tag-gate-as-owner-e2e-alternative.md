# ADR-TEST-011: owner full e2e の CI 不可を release tag ゲート（tools/release/cut.sh）で代替保証する

- ステータス: Proposed
- 起票日: 2026-05-03
- 決定日: -
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE（採用初期）

## コンテキスト

ADR-TEST-008 で owner full e2e（multipass + kubeadm + Cilium + Longhorn + MetalLB + フルスタック）を localhost 専用とし、CI では走らせないことを決定した。GitHub Actions runner は nested virtualization が利用できず multipass を起動不可、また 48GB host が要件のため Actions runner（14GB）でも実行不能である。owner full は不定期実走（ADR-TEST-008 §6）とし、CI 機械検証の射程外とする方針を確定したが、**「CI で機械検証されない代わりに何で本番再現性を担保するか」** が未確定のまま残っている。

代替保証なしで放置すると以下の事故が発生し得る:

- owner が release tag を切る前に owner full を実走しないまま release してしまう
- owner full が「数ヶ月前の実走で PASS したまま」放置され、その間の変更で本番再現スタックが壊れていても気づかない
- 採用検討者が「k1s0 は OSS 完成度を主張しているが、最後に本番再現で検証したのは 6 ヶ月前」と判定し、testing maturity 評価が低下

代替保証の候補は質的に 3 つある（事前の設計検討で確認済）。

第一に **release tag ゲート**（X 案、本 ADR で採用）。`tools/release/cut.sh` が release tag を切る時に owner full の PASS 証跡を必須化、未 PASS なら exit 1。release tag という決定的瞬間に owner full の鮮度を物理的に紐付ける。

第二に **audit lint（鮮度監視）**（Y 案）。`tools/audit/run.sh` が `owner-e2e-results.md` の最終 PASS 日付を読み、N 日超で warn / 90 日超で fail。release tag を切らなくても鮮度が監視される。

第三に **GPG sign + 結果検証**（Z 案）。owner が full 実走後に結果 tar.gz を GPG sign + push、CI が署名 + timestamp + cluster manifest を検査。「実走したフリ」を構造的に防げる。

設計上の制約と前提:

- ADR-TEST-008 で owner full は CI 不可、不定期実走と確定
- `tools/release/cut.sh` が既に存在し、release tag 切る時の qualify-release 強制 wrapper として機能（ADR-TEST-001 で確定）
- 既存 cut.sh は qualify-release report の sha256 を tag メッセージに埋め込む設計があり、本 ADR の owner full 証跡埋め込みと整合
- owner-e2e-results.md は ADR-TEST-008 で新設される live document、月次 / 不定期実走の結果を時系列で記録
- 起案者 1 人の運用工数で代替保証の機械化を維持できる射程
- 起案者は GPG 鍵管理経験が浅く、Z 案の即時導入は運用負担が大きい

選定では以下を満たす必要がある:

- **release tag = owner full PASS が物理的に紐付く**: release を切る決定的瞬間に owner full の鮮度が要求される
- **個人 OSS の運用工数で持続可能**: 機械化の実装 / 維持工数が起案者 1 人で吸収できる
- **採用検討者の透明性**: 代替保証の仕組みが ADR / Runbook で公開され、採用検討者が「k1s0 が release ごとに owner full を走らせている」を読める
- **誤魔化し耐性**: owner が「実走したフリ」をしても CI / 監査で検出される構造を将来拡張できる退路を残す

## 決定

**`tools/release/cut.sh` で release tag を切る時、`docs/40_運用ライフサイクル/owner-e2e-results.md` の最新 PASS entry の sha256 を必須検証する。** release tag メッセージに owner full PASS の sha256 を埋め込み、検証不通で exit 1。CI で owner full を機械実行しない代わりに、release tag という決定的瞬間に owner full の鮮度を物理的に紐付ける構造で代替保証を成立させる。

audit lint（Y 案）と GPG sign（Z 案）は本 ADR では採用しないが、Y 案は採用初期で / Z 案は採用後の運用拡大時で SRE 増員後に追加採用する余地を残す（影響セクション §移行・対応事項に記載）。

### 1. release tag ゲートの mechanism

`tools/release/cut.sh` の既存ロジック（qualify-release 強制 + tag メッセージへの sha256 埋め込み、ADR-TEST-001）を拡張し、owner full PASS 検証を追加する。

```text
tools/release/cut.sh の処理順（本 ADR で追加する step は ★ 印）:

1. git status クリーン確認
2. version SemVer 形式確認
3. tag 既存確認
★ 4. owner full PASS 鮮度検証（本 ADR）
★ 5. owner full sha256 抽出
6. make qualify-release 強制実行（既存）
7. qualify report tar.zst 化（既存）
★ 8. tag メッセージに owner full sha256 + qualify report sha256 を埋め込む
9. git tag -a <version> -m "<sha256s 含む message>"
```

#### Step 4: owner full PASS 鮮度検証

`docs/40_運用ライフサイクル/owner-e2e-results.md` を読み、以下を検証:

- 最新の `### YYYY-MM-DD` entry が存在
- 当該 entry の `判定: PASS` が記載
- 当該 entry の日付が **release tag を切る当日から N 日以内**（N の初期値は 30 日、`OWNER_E2E_FRESHNESS_DAYS` env で override 可）
- 全 5 検証（観測性 5 検証 = ADR-TEST-009）+ 全 owner suite 部位（platform / observability / security / ha-dr / upgrade / sdk-roundtrip / tier3-web / perf）で PASS が記載

検証不通の条件:

- entry が存在しない → exit 1（"owner-e2e-results.md に PASS entry がない"）
- 最新 entry の日付が N 日超 → exit 1（"owner full PASS が古い、再実走必須"）
- 判定 FAIL / 部分 PASS → exit 1（"owner full が完全 PASS でない"）

#### Step 5: owner full sha256 抽出

owner-e2e-results.md の最新 PASS entry に **artifact sha256** を必須記載する規約を追加する（ADR-TEST-008 で新設される owner-e2e-results.md に template 拡張）。

```markdown
### 2026-05-15

- 判定: PASS
- 全部位: platform / observability / security / ha-dr / upgrade / sdk-roundtrip / tier3-web / perf 全件 PASS
- artifact sha256: 7a3f9c5e1b2d8a6f...
- artifact path: tests/.owner-e2e/2026-05-15/full-result.tar.zst
- 実走 host: WSL2 48GB / kubeadm v1.30.x / Cilium v1.15.x / ...
```

cut.sh は最新 entry から `artifact sha256: <HEX>` を正規表現で抽出し、当該 sha256 が `tests/.owner-e2e/<YYYY-MM-DD>/full-result.tar.zst` の sha256sum と一致することを検証。改ざん防止と「結果ファイル実在」の同時確認。

#### Step 8: tag メッセージへの埋め込み

```text
release v1.2.3

qualify-report-sha256: e1c4...
owner-e2e-result-sha256: 7a3f9c...
owner-e2e-result-date: 2026-05-15
qualify-mode: full
```

これにより release tag を取得した側（採用検討者 / 採用組織）が `git tag -v <version>` または `git show <version>` で **release 時点の owner full 実走日と sha256** を一意に確認できる。

### 2. dry-run 経路

cut.sh の既存 `--dry-run` flag は本 ADR の owner full PASS 検証も skip できる。release tag を切らずに「現状で release 可能か」を確認する用途で、開発中 / hotfix 検証で使う。

```bash
tools/release/cut.sh --dry-run v1.2.3
# → owner full PASS 検証 skip + qualify-release skip + tag 作成 skip
```

dry-run 時は cut.sh が「owner full PASS 検証を skip した」を STDOUT に明示する（"WARN: --dry-run skips owner-e2e PASS verification"）。

### 3. owner-e2e-results.md template の拡張

ADR-TEST-008 で新設される `docs/40_運用ライフサイクル/owner-e2e-results.md` の月次 / 不定期 entry template に以下フィールドを追加する。本 ADR で必須化する。

```markdown
### YYYY-MM-DD

- 実走者: <owner 名>
- 判定: PASS / FAIL（部位ごとの内訳）
- 各部位 PASS 数: platform N/N / observability N/5 / security N/N / ha-dr N/N / upgrade N/N / sdk-roundtrip N/48 / tier3-web N/N / perf N/N
- artifact sha256: <HEX>
- artifact path: tests/.owner-e2e/<YYYY-MM-DD>/full-result.tar.zst
- 実走環境: <host RAM / multipass version / kubeadm version / Cilium version / Longhorn version / ...>
- 所要時間: <N 時間 N 分>
- 失敗詳細: <FAIL の場合のみ、root cause / 修正対応 / 関連 issue / 次回再実走予定>
```

`artifact sha256` フィールドは cut.sh が必須参照する。記載漏れは cut.sh で exit 1。

### 4. owner full の artifact 保管

owner full 実走時、`tests/.owner-e2e/<YYYY-MM-DD>/` 配下に以下を保管する。これは ADR-TEST-008 で新設する Runbook（RB-TEST-OWNER-E2E-FULL）の手順に組み込む。

- `full-result.tar.zst`: 全部位の test 結果（go test JSON / chromedp screenshot / k6 summary / kubectl logs / cluster info）
- `cluster-info.txt`: kubectl version / nodes / get all -A / Cilium / Longhorn / MetalLB の status
- `dmesg.txt`: 実走中の OOM / kernel error 監視ログ（48GB ピーク管理）

`full-result.tar.zst` の sha256 を計算し、owner-e2e-results.md の `artifact sha256` フィールドに記載。

過去 12 ヶ月分は git LFS で版管理する（ADR-TEST-003 の conformance results 12 ヶ月版管理と同パタン）。古い artifact は採用初期で release asset への昇格 + cold storage 移行を整備（採用後の運用拡大時の Runbook で扱う）。

### 5. CI から見える経路

CI（PR / nightly）からは owner full は走らないが、**release tag 検出時に CI が tag メッセージを検査** する補助 workflow を追加する余地を残す（本 ADR の射程外、ADR-TEST-009 / 010 / 011 起票 commit と独立で必要に応じて追加）。

採用初期で `_reusable-release-verify.yml`（新設候補）を作り、release tag が push された時に tag メッセージから `owner-e2e-result-sha256` を抽出 → `tests/.owner-e2e/<日付>/full-result.tar.zst` の存在確認 → sha256 整合性確認、を機械実行する経路を追加できる。本 ADR 採用時点では cut.sh のローカル検証のみ。

## 検討した選択肢

### 選択肢 A: release tag ゲートのみ（本 ADR 採用、X 案）

- 概要: cut.sh で release tag 切る時に owner full PASS 証跡（sha256 + 鮮度 N 日）を必須化、release tag メッセージに sha256 埋め込み
- メリット:
  - **release tag = owner full PASS が物理的に紐付く**: release を切る決定的瞬間に owner full の鮮度が要求される構造
  - 既存 cut.sh の qualify-release 強制経路（ADR-TEST-001）の延長で実装でき、追加複雑性が小さい
  - tag メッセージに sha256 が埋まるため、採用検討者 / 採用組織が release 時点の owner full 実走日を一意に確認できる
  - owner が release を切らない時期は強制されない（不定期方針 ADR-TEST-008 §6 と整合）
  - 個人 OSS の運用工数で実装 / 維持できる
- デメリット:
  - **release tag を切らない期間は検証ゼロのまま放置可能**: long-running development 中 / 採用初期で release を出さない時期は鮮度監視なし（mitigation: 採用初期で audit lint = Y 案 を追加採用する余地を残す）
  - owner が「結果記録だけ更新して artifact を捏造」する誤魔化しは検出不能（mitigation: artifact sha256 一致確認で「ファイル実在」は担保されるが、「実際に full e2e を走らせた」は信頼ベース、運用拡大時で Z 案 = GPG sign 追加余地）

### 選択肢 B: audit lint のみ（Y 案）

- 概要: tools/audit/run.sh が owner-e2e-results.md の最終 PASS 日付を読み、30 日超で warn / 90 日超で fail
- メリット:
  - 不定期でも「あまりに古い」状態は警告される
  - release tag を切らなくても鮮度が監視される
  - 実装が cut.sh より独立で、release 経路に介入しない
- デメリット:
  - **release tag という決定的瞬間に owner full が要求されない**: release を切る前日に owner full を走らさず、30 日前の PASS を信じて release できてしまう
  - owner が「PASS ではないが結果記録だけ更新」する形で誤魔化せる（記録の質を信頼する前提、誤魔化し耐性ゼロ）
  - 警告 / 失敗の閾値（30 / 90 日）が任意で、採用組織が「k1s0 はどれだけの鮮度を保証するか」を読み取りにくい

### 選択肢 C: GPG sign + 結果検証（Z 案）

- 概要: owner が full 実走後に結果 tar.gz を GPG sign + push、CI が署名 + timestamp + cluster manifest を検査
- メリット:
  - 「実走したフリ」できなくなる（誤魔化し耐性最強）
  - 結果記録の質を機械検証できる
- デメリット:
  - **GPG 鍵管理コストが起案者 1 人運用で過大**: 鍵生成 / 管理 / rotation / 紛失対応の手順整備、起案者の鍵紛失で release が止まるリスク
  - 実装複雑性が高い（CI 側に GPG 検証 + cluster manifest 解析 + timestamp 検証の 3 経路）
  - リリース時点で導入すると個人 OSS の運用工数を圧迫

### 選択肢 D: 代替保証なし

- 概要: owner full は CI 不可のまま放置、release tag に owner full の紐付けなし
- メリット:
  - 実装工数ゼロ
- デメリット:
  - **本番再現性の保証経路が一切ない**: ADR-TEST-008 で「不定期実走」と決めただけで、release 時に走らせる強制がない
  - owner が release tag を切る前に owner full を走らさずに release できてしまう
  - 採用検討者が「k1s0 は OSS 完成度を主張しているが、release ごとに本番再現検証された保証がない」と判定、testing maturity 評価が低下

## 決定理由

選択肢 A（release tag ゲートのみ、X 案）を採用する根拠は以下。

- **release tag という決定的瞬間に owner full PASS が物理的に紐付く**: 採用組織が pull する release artifact は必ず「直近 30 日以内の owner full PASS で検証された」構造になり、testing maturity の最低ライン担保が ADR レベルで成立する。選択肢 B（audit lint）は警告止まりで強制力がない、選択肢 D（なし）は強制ゼロ
- **既存 cut.sh の延長で実装できる**: ADR-TEST-001 で確定済の qualify-release 強制 wrapper の経路に owner full PASS 検証を追加するだけで、新規 mechanism を立てない。実装複雑性が最小
- **個人 OSS の運用工数整合**: 選択肢 C（GPG sign）は鍵管理コストで運用破綻、選択肢 A は ADR-TEST-008 の不定期実走方針と整合し、release を切らない期間は強制されない（owner の自由）
- **将来の補強経路を残す**: 選択肢 A 単独ではカバーしきれない領域（release を切らない期間の鮮度 / 結果記録の誤魔化し）に対し、Y 案（audit lint）を採用初期で / Z 案（GPG sign）を運用拡大時で追加する退路を残す。リリース時点で全部入れずに段階的に補強する設計
- **採用検討者の透明性**: tag メッセージに sha256 + 実走日が埋まるため、採用検討者が `git show <release-tag>` で release 時点の owner full 実走証跡を一意に確認できる。選択肢 B / C / D ではこの透明性が成立しない

## 影響

### ポジティブな影響

- release tag = owner full PASS（直近 30 日以内）が物理的に紐付き、release artifact の testing maturity 最低ラインが ADR レベルで担保される
- ADR-TEST-008 の owner full CI 不可方針が「実装ゼロの放置」ではなく構造的代替保証を持つ状態に昇格する
- 既存 cut.sh の延長で実装でき、追加複雑性が小さい
- tag メッセージへの sha256 + 実走日埋め込みで、採用検討者が release 時点の owner full 検証証跡を一意に確認できる
- 採用初期 / 運用拡大時で Y 案（audit lint）/ Z 案（GPG sign）を追加する退路が ADR で正典化されている

### ネガティブな影響 / リスク

- release tag を切らない長期間（採用初期で release を出さない時期）は鮮度強制がゼロ（mitigation: 採用初期で Y 案を追加採用する余地を本 ADR で正典化）
- owner が「PASS 記録だけ更新して artifact を捏造」する誤魔化しは現状検出不能（mitigation: artifact sha256 一致確認で「ファイル実在」は担保、運用拡大時で Z 案 = GPG sign を追加で誤魔化し耐性を強化）
- N 日（初期値 30 日）の閾値が任意で、起案者の負荷状態によって妥当性が変動（mitigation: `OWNER_E2E_FRESHNESS_DAYS` env で override 可、Runbook で N 値の意図を明文化）
- release tag 切る直前に owner full を実走する負担が起案者に集中（不定期実走方針との整合は、release 切る予定がある時期に合わせて owner full を走らせる運用で吸収）
- artifact 保管（git LFS 12 ヶ月）の累積容量管理が継続コストとして発生（mitigation: ADR-TEST-003 の conformance results 12 ヶ月管理と同 Runbook で吸収）

### 移行・対応事項

- `tools/release/cut.sh` を改訂し、step 4-5（owner full PASS 鮮度検証 + sha256 抽出）と step 8（tag メッセージへの埋め込み）を追加実装
- `OWNER_E2E_FRESHNESS_DAYS` env を cut.sh で受け取り、デフォルト 30 / 1〜90 範囲を許容
- ADR-TEST-008 で新設される `docs/40_運用ライフサイクル/owner-e2e-results.md` の月次 entry template を本 ADR §3 のフィールド構成で拡張
- ADR-TEST-008 で新設される `ops/runbooks/RB-TEST-OWNER-E2E-FULL.md` に artifact 生成 + sha256 計算 + owner-e2e-results.md 更新の手順を組み込む
- `tests/.owner-e2e/<YYYY-MM-DD>/` ディレクトリ構造を git LFS で管理する経路を整備、`.gitattributes` 更新
- 本 ADR の検証ロジック（cut.sh の owner full PASS 鮮度検証）の unit test を `tools/release/cut_test.sh`（新設）として実装
- 採用初期で Y 案（audit lint）を追加採用するか判断、追加する場合は `tools/audit/run.sh` に owner-e2e-results.md の鮮度監視ロジックを追加
- 採用後の運用拡大時で Z 案（GPG sign）を追加採用するか判断、追加する場合は新規 ADR 起票（GPG 鍵管理 + CI 側 verification 経路）
- `_reusable-release-verify.yml`（新設候補）を採用初期で追加し、release tag push 時に tag メッセージから owner-e2e-result-sha256 を抽出 → artifact 存在確認 → sha256 整合確認、を CI 側でも機械実行する経路を整備
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` に本 ADR を追加
- `docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` に本 ADR の対応 IMP-CI を追記

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— 既存 cut.sh の qualify-release 強制経路の起源
- ADR-TEST-008（e2e owner / user 二分構造、別 commit で起票）— owner full CI 不可の根拠
- ADR-TEST-009（観測性 E2E 5 検証 owner only、別 commit で起票予定）— 本 ADR が検証する owner full の中身
- ADR-TEST-010（test-fixtures 4 言語 SDK 同梱、別 commit で起票予定）— SDK release と本 ADR の release tag ゲートの整合
- ADR-TEST-003（CNCF Conformance / Sonobuoy 月次）— 12 ヶ月 artifact 管理の同パタン
- ADR-OPS-001（Runbook 標準化）— RB-TEST-OWNER-E2E-FULL の形式根拠
- `tools/release/cut.sh` — 本 ADR が拡張する既存 wrapper
- 関連 ADR（採用検討中）: 採用初期で audit lint（Y 案）を追加する場合の補強 ADR / 運用拡大時で GPG sign（Z 案）を追加する場合の補強 ADR
