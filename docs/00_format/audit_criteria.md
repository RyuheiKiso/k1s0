# 監査判定基準（audit_criteria）

本ファイルは k1s0 の自己監査（audit）における判定基準の正典。`/audit` コマンド・`tools/audit/run.sh` shell script・`audit-protocol` skill はすべて本ファイルの判定ルールに従う。`docs/SHIP_STATUS.md`（人間が書く散文・状況説明）と `docs/AUDIT.md`（機械生成の ID 網羅マトリクス）の両方が本ファイルを参照する。

## 監査の目的

OSS 採用検討者が「docs に書かれた要求のうち、何 % が実装済か / どこに手抜きが残るか / 本当に動くか / OSS として採点に耐えるか」を**第三者視点で判定**できる材料を、機械的に生成・更新できる仕組みを提供する。

監査は「Claude が PASS と書く」行為ではない。**証跡を集めて並べ、人間が判定する**ための前処理である。判定権は人間に残り、Claude は集計と提示に徹する。

## 完璧の定義（不在）

本プロジェクトは「OSS として完璧」という到達不能な目標を採らない。代わりに **「採用検討者が信頼できる水準」** を相対基準で示す。具体的には：

- **OSSF Scorecard**: 7/10 以上
- **CNCF Sandbox 採用基準**: governance / license / CoC / contributors の最低要件 PASS
- **OpenSSF Best Practices Badge**: Passing 達成、Silver を視野
- **k1s0 自己基準**: 本ファイルで定義する 4 軸 audit が全て stub 解消済

これらを満たすことを「採用検討者が信頼できる水準」と呼び、AUDIT.md にはこの相対充足度を記録する。

## 4 つの監査軸

| 軸 | 名称 | 検証対象 | 主な手段 |
|---|---|---|---|
| **A** | 要求網羅 | docs の ID（FR-T1-* / NFR-* / DS-* / IMP-* / ADR-*）が全部実装に反映されているか | docs ↔ src/infra/deploy の双方向照合 |
| **B** | 手抜き検出 | "それっぽく見えて中身が空" な箇所がないか | 静的検査（Unimplemented / TODO / .gitkeep のみ / mock 残存） |
| **C** | k8s 実機動作 | 書けただけで本当に動くか | kind / production cluster で E2E 再現 |
| **D** | OSS 完成度 | 採用検討者から見て採点に耐えるか | OSSF Scorecard / CNCF / Best Practices Badge |

各軸で PASS / PARTIAL / FAIL / N/A の 4 値判定を下す。基準は次節。

## 判定基準（PASS / PARTIAL / FAIL / N/A）

### 共通定義

- **PASS**: 機械的検証 + 実機証跡 + docs 整合の **3 段すべて**を満たす
- **PARTIAL**: 3 段のうち 1-2 段のみ満たす（雛形あり / 部分検証 / 機械検証のみ等）
- **FAIL**: 3 段のいずれも満たさない（設計のみ / 実装欠落 / 動作未確認）
- **N/A**: 該当なし（規約のみの ADR / 採用後の運用拡大時で意図的に未着手）

「3 段」とは：

1. **docs に ID が定義されている**（要件 / 設計 / ADR が起票されている）
2. **実装サンプルがコード上に存在する**（src / infra / deploy / tests / examples 内のファイルパス特定可能）
3. **動作証跡がある**（unit test / integration test / kind 実機 / production 実機のいずれかで挙動確認）

### 軸別の細則

#### A 軸（要求網羅）の判定基準

各 ID について：

| 条件 | 判定 |
|---|---|
| docs 定義あり / 実装サンプルあり / test or 実機証跡あり | PASS |
| docs 定義あり / 実装サンプルあり / 証跡なし | PARTIAL |
| docs 定義あり / 実装サンプルなし | FAIL |
| docs 定義なし（コードに ID 引用があるが ADR 未起票等） | FAIL（orphan） |
| ADR が「規約のみ」/「採用後の運用拡大時」明示 | N/A |

orphan 検出は SHIP_STATUS F10 と同種の手作業を機械化する（「ADR-CNCF-004 / ADR-DAPR-001 / ADR-DEVEX-002 等 8 件がコードから引用されているが ADR 未起票」型 drift）。

#### B 軸（手抜き検出）の判定基準

以下のパターンを全言語横断で検出。**残存件数を数値化**し、目標値（リリース水準）と比較する：

| パターン | 検出方法 | リリース水準目標 |
|---|---|---|
| Go `codes.Unimplemented` 残存 | grep | 0 件（公開 API） |
| Rust `unimplemented!()` / `todo!()` 残存 | grep | 0 件（公開 API） |
| .NET `NotImplementedException` 残存 | grep | 0 件（公開 API） |
| TS `throw new Error("not impl")` | grep | 0 件 |
| Python `NotImplementedError` | grep | 0 件 |
| `TODO` / `FIXME` / `XXX` | grep | ADR / Issue に昇格済のもののみ許容、それ以外 0 件 |
| 「とりあえず」「暫定」「仮置き」「あとで」 | grep | 0 件（コード本体）、docs 内は説明用途のみ許容 |
| `for now` / `temporary` / `quick fix` / `hack` / `workaround` | grep | 0 件（コード本体） |
| 空 catch / silent error suppress | regex | 0 件 |
| `.gitkeep` のみのディレクトリ | find | 「設計のみ」と明示済のみ許容 |
| values.yaml が `# TODO` のみ | grep | 0 件 |
| mock / fake backend 残存（production 経路で in-memory） | grep + 設定値検査 | 採用初期で外部 backend 結線済 |

PASS = 残存 0 件 / PARTIAL = リリース水準内（許容例外あり）/ FAIL = 残存超過。

#### C 軸（k8s 実機動作）の判定基準

| 条件 | 判定 |
|---|---|
| `tools/local-stack/up.sh` で kind 起動 + 該当機能 E2E 通過 | PASS |
| 該当機能の unit test / integration test のみ通過、kind 未確認 | PARTIAL |
| docs / Helm chart のみ、test なし、実機なし | FAIL |
| kind では本質的に検証不可（production 必須）と SHIP_STATUS で明示 | N/A（production carry-over） |

#### D 軸（OSS 完成度）の判定基準

外部基準を参照し、各項目の充足度を出す。

**OSSF Scorecard（10 項目スコア）**:
- Code-Review / Maintained / CII-Best-Practices / License / Signed-Releases / Branch-Protection / Token-Permissions / Pinned-Dependencies / Vulnerabilities / Binary-Artifacts 等
- 自動採点ツール `scorecard-cli` の出力をそのまま記録

**CNCF Sandbox 最低要件**:
- LICENSE（OSI 承認）
- CODE_OF_CONDUCT.md
- CONTRIBUTING.md
- GOVERNANCE.md
- SECURITY.md
- 外部 contributor の存在
- 公開 issue tracker

**OpenSSF Best Practices Badge（Passing 17 + Silver 追加 + Gold 追加）**:
- Basics / Change Control / Reporting / Quality / Security / Analysis の 6 領域
- 各項目を Met / Unmet / N/A で記録

PASS = 全項目 Met / PARTIAL = 1 領域以上 Unmet / FAIL = 半数以上 Unmet。

## 監査の運用ルール

### Claude が守るべき原則

1. **PASS を勝手に書かない**: Claude は証跡を集めて並べる。最終判定（PASS / PARTIAL / FAIL）は人間が `docs/AUDIT.md` を編集して下す。Claude が `PASS` を新規記入する場合、必ず証跡パス（実機ログ / test 出力 / commit hash）を 1 対 1 で添える
2. **生証跡の保存**: grep / find / kubectl / curl の生 stdout を `.claude/audit-evidence/<YYYY-MM-DD>/<axis>.txt` に保存し、AUDIT.md からリンクする。レビュアーが嘘を後追いできる状態にする
3. **不在の証明**: 「該当 0 件」を主張する場合、走査範囲（パターン / 除外パス / 走査ファイル数）を必ず併記する。「grep を打ったら 0 件だった」ではなく「N ファイルに対し M パターンで grep、該当 0 件」
4. **誇張しない**: PARTIAL を PASS に丸めない。手抜き検出で 1 件残存 = PARTIAL。0 件まで PASS にしない
5. **数値で語る**: 「ほぼ完了」「大部分」を禁じ、必ず分数（M / N）で示す

### 人間が守るべき原則

1. **定期実行**: main commit 後 / PR 提出時 / リリース前に `/audit all` を走らせる（CI 自動化はしない、明示実行が運用ルール）
2. **AUDIT.md の commit**: 監査スナップショットは `docs/AUDIT.md` に commit し、history に残す。退行検知は git diff で行う
3. **乖離の昇格**: AUDIT.md と SHIP_STATUS.md が乖離したら、SHIP_STATUS.md（散文）を更新して両者を整合させる。SHIP_STATUS は人間判断の正本

## SHIP_STATUS.md と AUDIT.md の役割分担

| 項目 | SHIP_STATUS.md | AUDIT.md |
|---|---|---|
| 形式 | 人間散文 + 表 | 機械生成マトリクス |
| 主役 | 採用検討者向けの「現状の物語」 | ID 単位の網羅チェック |
| 更新 | 検証セッション後に手動 | `/audit` コマンドで再生成 |
| 判定 | 「同梱済 / 雛形あり / 設計のみ」の 3 段 | PASS / PARTIAL / FAIL / N/A の 4 値 |
| 役割 | 文脈・経緯・判断の根拠を残す | 網羅性・退行検知・第三者証跡 |

両者は補完関係で、片方では完結しない。SHIP_STATUS は AUDIT を引用し、AUDIT は SHIP_STATUS の特定セクションへリンクする。

## 監査の限界（本基準でカバーしない範囲）

正直に明記する：

1. **`/audit` で「動作する」とは保証できない**: kind 実機検証は外部依存（Docker / kubectl / Helm）の状態に依存する。CI 化しないため、Claude / 人間が実行しない時点で証跡は古くなる
2. **OSSF Scorecard は public repo 前提**: 本リポジトリが private のうちは scorecard-cli の一部項目が N/A になる
3. **「採用検討者が信頼できる水準」も主観**: 本基準で PASS でも、特定組織の評価基準では FAIL になる可能性がある。本基準は最低限の合意点

これらの限界を AUDIT.md 冒頭にも明記し、採用検討者の誤認を防ぐ。

## 関連

- `docs/AUDIT.md` — 監査結果のスナップショット
- `docs/SHIP_STATUS.md` — 実装現状の散文記述
- `tools/audit/run.sh` — 監査ロジックの正本（shell）
- `.claude/commands/audit.md` — `/audit` コマンド
- `.claude/skills/audit-protocol/SKILL.md` — 監査の方法論 skill
- `.claude/skills/anti-shortcut-discipline/SKILL.md` — 監査で見つかった手抜きを潰す時の規律
- `.claude/skills/principal-architect-mindset/SKILL.md` — 監査結果から優先順位を判断する思考軸
