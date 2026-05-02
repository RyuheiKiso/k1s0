---
name: audit-protocol
description: k1s0 で `/audit` コマンドを実行する時、または監査レポート（docs/AUDIT.md）を更新する時に必ず参照する方法論 skill。Claude が PASS を勝手に書かない原則、3 段確認、生証跡保存、不在の証明（走査範囲付記）、4 軸（網羅 / 手抜き / k8s / OSS）の判定ルールを強制する。判定基準の正典は docs/00_format/audit_criteria.md。
---

# 監査プロトコル

本 Skill は監査の**方法論**を固定する。判定基準の正典は [`docs/00_format/audit_criteria.md`](../../../docs/00_format/audit_criteria.md) で、本 Skill は「どう監査を実行するか」の振る舞いを定める。

## 監査の根本原則

### Claude は判定者ではない、証跡集積者である

監査の判定（PASS / PARTIAL / FAIL）は **人間が下す**。Claude は：

1. 判定基準（`audit_criteria.md`）に従って証跡を集める
2. 集めた証跡を `docs/AUDIT.md` のマトリクスに整列する
3. 生証跡を `.claude/audit-evidence/<date>/<axis>.txt` に保存する
4. 「これらの証跡から、判定者は X と判断する材料がある」までで止まる

### Claude は AUDIT.md の判定列に PASS / PARTIAL / FAIL を**いかなる場合も**書かない

過去の AUDIT.md では Claude がサマリ表に `**PASS（コード本体）**` `**PASS（kind 段階）**` `PARTIAL` を新規記入し、protocol 自身の原則違反を起こしていた。これを構造的に防ぐため、以下を厳守する：

- AUDIT.md のサマリ表の判定列は「**判定材料**」列とする。Claude は数値（M / N の分数）と証跡パスのみ書く
- 「PASS」「PARTIAL」「FAIL」「N/A」の 4 値は人間記入欄として残す。Claude は空欄のまま提出する
- 例外なし。Claude が「これは明らかに PASS だから書いていい」と思っても書かない（その判断自体が判定行為）
- 「判定材料」列の例:
  - **OK**: 「true 手抜き 0 件 / 1232 ファイル走査 / 許容残置 2 件 (false-positive)」
  - **OK**: 「local cluster (kind) で 93/93 Running、production-equivalent 検証は未実施」
  - **NG**: 「PASS（コード本体）」← Claude が判定している
  - **NG**: 「PARTIAL（kind 段階）」← 同上

### kind / production の混同禁止

C 軸で「PASS」と書く場合、cluster 種別（kind / minikube / k3d / docker-desktop / managed K8s）を必ず併記する。kind 段階の 100% Running を production 検証済として誇張しない。AUDIT.md は **local 列 / production 列を必ず分離**する（`tools/audit/lib/k8s.sh` の `cluster_class` / `verification_tier` を使う）。

### 短絡的な PASS の禁止

「grep して 0 件だったから PASS」は不十分。以下を必ず添える：

- 走査したファイル数（`find ... | wc -l` の結果）
- 走査したパターン（grep の正規表現の正確な文字列）
- 除外したパス（vendor / node_modules / target / dist / generated 等）
- 走査時刻（証跡ファイルのタイムスタンプ）

これがないと「不在の証明」が成立しない。SHIP_STATUS F10 が「ADR-CNCF-004 等 8 件が cite されているが ADR ファイル無し」を発見できたのは、grep の走査範囲を明示したから。

## 3 段確認の必須化

各 ID / 各項目について、以下 3 段すべてを確認する：

| 段 | 内容 | 失敗時の判定 |
|---|---|---|
| **1. docs 定義** | docs に ID が起票されている / 仕様が記述されている | FAIL or N/A |
| **2. 実装サンプル** | コード上にファイルパスを特定可能（src / infra / deploy / tests / examples） | FAIL |
| **3. 動作証跡** | unit test / integration test / kind 実機 / production 実機のいずれかで挙動確認 | PARTIAL |

3 段すべて満たして初めて PASS。1 段でも欠けたら PARTIAL 以下。

「実装サンプルあり = PASS」と判定するのは典型的な誇張。SHIP_STATUS の「同梱済」も実は in-memory backend での動作止まりで、production backend は別段階という区分けがある。本監査でも同種の区分けを保つ。

## 軸別の実行手順

監査は 4 軸（A 網羅 / B 手抜き / C k8s / D OSS）に分かれる。各軸の実行手順：

### A 軸: 要求網羅（FR / NFR / DS / IMP / ADR）

1. docs の ID を網羅列挙する：
   - `grep -rohE "FR-T1-[A-Z]+-[0-9]+" docs/03_要件定義/ | sort -u`
   - `grep -rohE "NFR-[A-I]-[A-Z]+-[0-9]+" docs/03_要件定義/ | sort -u`
   - `grep -rohE "DS-[A-Z]+-[A-Z]+-[0-9]+" docs/04_概要設計/ | sort -u`
   - `grep -rohE "IMP-[A-Z]+-[A-Z]+-[0-9]+" docs/05_実装/ | sort -u`
   - `ls docs/02_構想設計/adr/ADR-*.md` で ADR 列挙
2. 各 ID について実装サンプルを grep：`grep -rln "<ID>" src/ infra/ deploy/ tests/ examples/ tools/`
3. 動作証跡の有無を確認：
   - `*_test.go` / `*_test.rs` / `*.test.ts` 等の test ファイル参照
   - SHIP_STATUS.md の「実 K8s 検証実績」表との突合せ
4. 結果を `docs/AUDIT.md` の A 軸マトリクスに記入。判定欄は人間記入用に空欄、Claude は証跡列のみ埋める

#### A 軸補助: 双方向トレース（NFR / DS / IMP）

NFR / DS / IMP は業界標準の設計慣行で「コードに ID を直接埋め込まない」性質がある。grep ベースの coverage で 92-98% が「impl 不在」と分類されるのを「grep 限界」で逃げず、`tools/audit/lib/trace.sh` で**間接トレース**を採る：

1. **direct**: `src/infra/deploy/...` での直接 grep ヒット（coverage と同じ）
2. **verify**: `infra/security/kyverno/`, `infra/observability/`, `deploy/rollouts/`, `ops/sli-slo/`, `tests/{contract,e2e,fuzz}/` 内の ID 引用
3. **via-fr / via-adr**: ID 言及 docs に同居する FR-T1-* / ADR-* のうち、impl_refs>0 のもの（trace 経由の reach）

trace_status が `unreached` の ID のみ「真の impl 不在候補」として AUDIT.md に上げる。`reach(...)` が付いたものは間接 reach 済として扱う。

呼び方: `tools/audit/run.sh trace-nfr` / `trace-ds` / `trace-imp`（coverage の後に呼ぶこと、impl 集合を使うため）

#### orphan 検出（コードから引用されているが ADR 未起票）

1. コード内 ADR 参照: `grep -rohE "ADR-[A-Z0-9]+-[A-Z0-9]+(-[0-9]+)?" src/ infra/ deploy/ tools/`
2. ADR ファイル存在: `ls docs/02_構想設計/adr/ADR-*.md`
3. (1) − (2) で orphan を抽出
4. orphan を `audit-evidence/<date>/orphans.txt` に列挙、AUDIT.md に件数記入

### B 軸: 手抜き検出

`tools/audit/run.sh slack` を呼ぶ。検出ロジックは `tools/audit/lib/slack.sh` に集約。本 skill では呼び方と結果の扱いだけ定める：

1. `tools/audit/run.sh slack > .claude/audit-evidence/<date>/slack.txt` で生出力保存
2. パターンごとの件数を集計（`grep -c` で各パターン）
3. リリース水準目標（`audit_criteria.md` B 軸の表）と比較し PASS / PARTIAL / FAIL 判定の**材料**を AUDIT.md に並べる
4. 残存件数が 0 でない場合、ファイルパス + 行番号を `audit-evidence/<date>/slack-locations.txt` に記録

### C 軸: k8s 実機動作

実機検証は外部依存があるため、Claude が自動で全部はやらない。**実行ログを集める**側に徹する：

1. `tools/local-stack/up.sh` の最終実行ログ（`tools/local-stack/.up.log` 等）を確認
2. 直近の `kubectl get pods --all-namespaces` の状態を取得（人間が起動済の場合）
3. SHIP_STATUS.md の「実 K8s 検証実績」セクションを引用
4. AUDIT.md には「直近検証日 / 検証 commit hash / 起動 namespace 数 / Running Pod 数 / **cluster_class** / **verification_tier**」を記録

`tools/audit/lib/k8s.sh` は context 名から cluster 種別を判定し `cluster_class: kind|production-equivalent (GKE/EKS/AKS)|...` と `verification_tier: local-only|production-equivalent` を出力する。AUDIT.md の C 軸表は **必ず local 列 / production 列の 2 列**を持つ。kind での 100% Running を production verification として誇張しない。

cluster が起動していない場合：「最終実機検証は SHIP_STATUS のコミット `XXXX` 時点」と明記し、現時点での実機検証は **保留** とする（嘘 PASS を書かない）。

### D 軸: OSS 完成度

外部基準（OSSF Scorecard / CNCF Sandbox / OpenSSF Best Practices Badge）を参照。

1. ルートの存在確認: `LICENSE` / `CODE_OF_CONDUCT.md` / `CONTRIBUTING.md` / `GOVERNANCE.md` / `SECURITY.md`（既に全部存在）
2. OSSF Scorecard 項目を手動チェック（scorecard-cli の自動実行は別途）
3. OpenSSF Best Practices の 17/Passing 項目を 1 つずつチェック
4. 各項目の Met / Unmet / N/A を AUDIT.md に記入

OSS 完成度は単発の grep では判定不能で、項目ごとの検証パスがある。本軸は時間がかかるため、Phase 3 で `oss-completeness-criteria` skill が補完する。

## 生証跡の保存先と命名

すべての監査実行は `.claude/audit-evidence/<YYYY-MM-DD>/` 以下に証跡を残す：

```
.claude/audit-evidence/2026-05-02/
├── meta.txt          # 実行時刻 / commit hash / 走査範囲
├── ids-fr.txt        # FR ID 一覧
├── ids-nfr.txt
├── ids-ds.txt
├── ids-imp.txt
├── ids-adr.txt
├── orphans.txt       # ADR orphan
├── slack.txt         # B 軸生出力
├── slack-locations.txt
├── k8s-pods.txt      # kubectl 出力
└── oss-checklist.txt
```

`.claude/audit-evidence/` は `.gitignore` 対象（証跡が肥大化するため）。ただし AUDIT.md には**証跡ファイル名と相対パス**を引用するので、再生成すれば同じ証跡が再現できる。

## bug 修正との連携（anti-shortcut-discipline 統合）

監査で発見した手抜き / 不足を修正する時は **`anti-shortcut-discipline` の 4 段プロトコル**を必ず通す：

1. **根本原因（5 Whys）**: なぜ手抜きが発生したか / なぜ実装漏れたか
2. **横展開検査**: 同種の手抜きが他にないか
3. **regression test**: 手抜き再発を防ぐ test
4. **波及範囲確認**: SDK / docs / runbook / 下流への伝播

監査で「N 件の手抜きを発見、1 件だけ修正してマージ」は **短絡修正の典型**。発見した時点で全件の対応方針を決める（即修正 / ADR 起票 / Issue 化）。

## 監査結果の優先順位（principal-architect-mindset 統合）

監査で複数の不足が見つかった場合、潰す優先順位は `principal-architect-mindset` Layer 1-3 で判断：

| 優先 | 性質 | 例 |
|---|---|---|
| 最優先 | one-way door / blast radius 大 / 採用判断のブロッカー | LICENSE 不在 / cross-tenant boundary bug / 公開 API の Unimplemented |
| 高 | 採用初期で必須 / 構造的な手抜き | 主要 backend 結線未完 / regression test 不足 |
| 中 | 個別 ID の orphan / mock 残存 | ADR orphan / dev only path に残る TODO |
| 低 | 採用後の運用拡大時で対応予定 | OpenTofu / 高度 observability |

優先度判断の根拠を AUDIT.md に**必ず**記載する（何を最優先と判断したか / その根拠は何か）。

## 自己点検チェックリスト

`/audit` 実行終了時に以下を確認：

1. [ ] `docs/AUDIT.md` に「PASS」「PARTIAL」「FAIL」「N/A」のどれかを Claude が新規記入していないか（**ゼロ件であるべき**、判定列は人間記入用に空欄のままにする）
2. [ ] サマリ表の列名は「状態」ではなく「判定材料」になっているか（数値と証跡パスのみ）
3. [ ] 「該当 0 件」を主張するすべての項目で、走査範囲（ファイル数 / パターン / 除外）を併記したか
4. [ ] 残存件数 / orphan 件数を **数値（M / N の分数）** で示したか
5. [ ] C 軸で `cluster_class` / `verification_tier` を明記し、kind / production を 2 列に分離したか
6. [ ] gitkeep-only ディレクトリの SHIP_STATUS 整合検査結果（documented / undocumented）を反映したか
7. [ ] NFR / DS / IMP の trace 軸（reach 経路）を記録したか（unreached のみが要 inspect 候補）
8. [ ] D 軸で Unknown を Met / Unmet に振り替えられる項目（CII Best Practices ローカル判定可能分 / Dangerous-Workflow）を採点したか
9. [ ] kind 実機検証が古い場合、「最終検証は commit XXXX」と明記したか
10. [ ] SHIP_STATUS.md と AUDIT.md の判定材料が乖離する場合、人間判断を仰ぐコメントを残したか
11. [ ] 監査で見つかった不足の優先順位を Layer 1-3 で判断し、根拠を記載したか
12. [ ] 生証跡 `.claude/audit-evidence/<date>/` を git add したか（`.gitignore` 対象は `.*.tmp` と `*-filelist.txt` のみ）

## 適用範囲外（本 skill を呼ばないケース）

- 個別の bug fix（`anti-shortcut-discipline` を使う）
- ADR 起票（`docs-adr-authoring` を使う）
- 新規実装（`principal-architect-mindset` を使う）

`/audit` コマンド実行時、または `docs/AUDIT.md` を編集する時のみ本 skill を呼ぶ。

## 関連

- 判定基準: [`docs/00_format/audit_criteria.md`](../../../docs/00_format/audit_criteria.md)
- 監査スナップショット: [`docs/AUDIT.md`](../../../docs/AUDIT.md)
- 監査ロジック: `tools/audit/run.sh`
- コマンド: `.claude/commands/audit.md`
- 連携 skill: `anti-shortcut-discipline` / `principal-architect-mindset` / `iteration-and-scope-discipline`
