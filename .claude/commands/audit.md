# /audit — k1s0 自己監査の実行

`docs/00_format/audit_criteria.md` に従って 4 軸（網羅 / 手抜き / k8s / OSS）の自己監査を実行する。証跡は `.claude/audit-evidence/<date>/` に保存し、`docs/AUDIT.md` 更新の diff 案を提示する。

判定（PASS / PARTIAL / FAIL）は **人間が下す**。本コマンドは証跡を集めて並べるまでで止まる。

## 引数

$ARGUMENTS

軸を指定する。省略時は対話的に選ばせる。

| 引数 | 軸 | 内容 |
|---|---|---|
| `slack` | B 軸 | 手抜き検出（Unimplemented / TODO / 禁止語彙 / .gitkeep のみ） |
| `fr` | A 軸 | FR-T1-* 全 ID の網羅監査 |
| `nfr` | A 軸 | NFR-* 全 ID の網羅監査 |
| `ds` | A 軸 | DS-* 全 ID の網羅監査 |
| `imp` | A 軸 | IMP-* 全 ID の網羅監査 |
| `adr` | A 軸 | ADR 全件 + orphan 検出 |
| `k8s` | C 軸 | k8s 実機状態スナップショット（cluster 不在時は保留） |
| `oss` | D 軸 | OSS 完成度（CNCF / OpenSSF Best Practices） |
| `all` | 全軸 | 上記すべて順次実行 |

## 実行手順

### Step 0: 前提 skill 読み込み

1. `.claude/skills/audit-protocol/SKILL.md` を読む（Claude が PASS を勝手に書かない原則 / 3 段確認 / 生証跡保存 / 不在の証明）
2. `.claude/skills/principal-architect-mindset/SKILL.md` を読む（監査結果から優先順位を判断する Layer 1-3 軸）
3. `docs/00_format/audit_criteria.md` を読む（判定基準の正典）

### Step 1: 軸の決定

引数から軸を決定する。引数なし or `all` の場合、ユーザーに軸を選ばせる：

```
監査軸を選んでください:
  1. slack  — 手抜き検出（最速、外部依存なし）
  2. fr/nfr/ds/imp/adr — ID 網羅
  3. k8s    — 実機状態（cluster 起動が前提）
  4. oss    — OSS 完成度
  5. all    — すべて
```

### Step 2: 実行

`tools/audit/run.sh <axis>` を呼ぶ：

```bash
bash tools/audit/run.sh <axis>
```

- 出力は `.claude/audit-evidence/<YYYY-MM-DD>/` に蓄積
- `meta.txt` に実行時刻 / commit hash / 走査範囲を記録
- 各軸の生証跡（grep 出力 / kubectl 出力 / find 結果）を保存

`all` の場合、slack → fr → nfr → ds → imp → adr → k8s → oss の順で実行（依存なしの軽い順）。

### Step 3: 結果の集約

実行後、`.claude/audit-evidence/<date>/` の各ファイルを読み、以下をユーザーに提示する：

#### サマリ表

| 軸 | 検出パターン | 件数 | 走査範囲 | 判定材料 |
|---|---|---|---|---|
| ... | ... | ... | ... | ... |

「**判定材料**」列には「PASS の根拠 / PARTIAL の根拠 / FAIL の根拠」を**Claude の判断ではなく**「N 件のうち M 件で X が確認、K 件で Y が未確認」のような数値ベースで書く。

#### 注目すべき発見

- 想定外の数値（SHIP_STATUS の主張と乖離するもの）
- audit 自身の誤検知の可能性（false positive）
- 優先順位の高い不足（Layer 1: blast radius 大 / one-way door / 採用判断ブロッカー）

### Step 4: AUDIT.md 更新の diff 案

`docs/AUDIT.md` を読み、新しい監査結果との diff 案を提示する：

- 既存項目の数値更新
- 新規発見の追加（orphan / 残存件数の増加など）
- 採用検討者向けの「採点に耐える水準」充足度の更新

**Claude は AUDIT.md を直接書き換えない**。diff 案を提示して、ユーザーがレビューして commit する形にする。

### Step 5: 不足修正の提案

監査で発見した不足について、以下を提示：

1. **修正優先順位**（principal-architect-mindset Layer 1-3）:
   - 最優先: one-way door / blast radius 大 / 採用判断ブロッカー
   - 高: 採用初期で必須 / 構造的な手抜き
   - 中: 個別 ID の orphan / dev only path の TODO
   - 低: 採用後の運用拡大時で対応予定
2. **修正アプローチ**（anti-shortcut-discipline 4 段プロトコル必須）:
   - 即時修正可能なもの → 修正案を提示（ただし short-cut 修正にならないよう、4 段必ず通す）
   - ADR 起票が要るもの → `/adr` コマンド誘導
   - Issue 化が適切なもの → タイトル案を提示

## 記述ルール

- Claude が PASS を新規記入する場合、必ず証跡パスを 1 対 1 で添える
- 「該当 0 件」を主張する時は、走査範囲（ファイル数 / パターン / 除外）を併記
- 数値は分数（M / N）で示す。「ほぼ完了」「大部分」は禁止
- kind 実機検証が古い場合、「最終検証は commit XXXX」と明記し、保留扱いとする
- audit 自身の誤検知を発見したら、`tools/audit/lib/<axis>.sh` の修正を提案する（`/audit` の信頼性を維持するため）

## 関連

- 判定基準: `docs/00_format/audit_criteria.md`
- 監査スナップショット: `docs/AUDIT.md`
- 監査ロジック: `tools/audit/run.sh` + `tools/audit/lib/`
- 方法論: `.claude/skills/audit-protocol/SKILL.md`
- 連携: `.claude/skills/anti-shortcut-discipline/SKILL.md`（不足修正時）/ `.claude/skills/principal-architect-mindset/SKILL.md`（優先順位判断時）
