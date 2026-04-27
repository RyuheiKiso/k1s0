---
name: iteration-and-scope-discipline
description: 同じ修正を 2 回以上失敗している、または PR が当初スコープを大きく超えて膨張している時に必ず参照する。「深追い禁止（iteration trap 回避）」と「巻き込み禁止（scope creep 回避）」の原則を強制する。過去に PR #840 で commitlint footer 解析を 4 回 force push、PR が docs 設計書追加から repo 全体 cleanup に膨張した経緯から作成。
---

# Iteration / Scope Discipline 原則

本 Skill は実装・CI 緑化・PR 対応など作業全般に適用する**作業姿勢の原則**。「いつ止めるか」「いつ分割するか」を判断するための内的ガードレール。書式や手順の skill とは性質が異なり、個別判断ロジックを補正するメタ原則として扱う。

## 原則 1: 同じエラーで 2 回失敗したら別アプローチ（深追い禁止）

ある修正を試して失敗、調整して再試行で同じカテゴリのエラー — **3 回目は試さない**。同種の修正を繰り返すと iteration trap に陥る。

- 失敗を 2 回観測した時点で**仮説を 90 度ずらす**。「メッセージの書き方を変える」ではなく「メッセージそのものを削る」「別 commit に分離する」「別 PR に切り出す」など、**根本構造を変える対処**を選ぶ。
- 「もう 1 回だけ試したら通る気がする」は iteration trap の典型的な内部声。気づいた時点で必ず止まる。
- 過去事例（2026-04-27 / PR #840）: commitlint の `footer-leading-blank` ルールに対し commit message を 4 回書き換えた。エラー遷移は `footer-leading-blank` → `footer-max-line-length` → `footer-leading-blank` → `footer-leading-blank`。3 回目以降は明らかに無駄で、最終的に「commit 自体を PR から分離する」根本変更で解消した。3 回目の段階で切り替えていれば force push 1 回で済んだ。

## 原則 2: PR が pre-existing 問題を巻き込み始めたら即 split 判断

PR の CI が failed → 確認すると repo 全体の既存違反が大量検出 → 「ついでに直そう」と PR に追加する誘惑が湧く。**この誘惑に乗ってはいけない**。

- pre-existing 問題への対処は**当初 PR スコープではない**。原則として別 PR に切り出し、tracking issue で残追跡する。
- 即 split 判断のシグナル:
    - PR の diff stat が当初想定の 3 倍以上に膨張
    - 1 PR で複数 type prefix が混在（feat / fix / docs / ci / style など 4 種以上）
    - 失敗ジョブの理由が 4 種類以上（commitlint / markdownlint / openapi / lint-tier* …）
    - commit 数が当初予定の 2 倍以上
- 過去事例（2026-04-27 / PR #840）: docs 単体の PR が markdownlint の repo 全体違反 1.3 万件 / broken link 17 件 / openapi stale / commitlint policy 等を巻き込み、最終的に 250+ ファイル / 12 commits / scope 4 種以上に膨張した。最初に「これは別 PR」と切る判断ができていれば本来の docs PR が小さく review 可能で済んだ。

## 原則 3: Force push の回数上限を内的に持つ

1 つの PR ブランチへの force push は **3 回まで**を目安。それ以上は branch 設計を見直すサイン。

- force push の真のコストは「やり直しに使う時間」と「reviewer の混乱」。3 回目に達した時点で、commit 構造の見直し / commit 切り離し / branch 再設計のいずれを取るか自問する。
- 4 回目以降は「なぜ繰り返しているか」を分析せずに繰り返している可能性が高い。原則 1 と組み合わせてストップ。

## 原則 4: 検出シグナル早見表

| シグナル | 即取るべきアクション |
|----------|---------------------|
| 同じカテゴリのエラーで 2 回失敗 | 仮説を 90 度ずらす（根本構造を変える） |
| PR の diff が初期想定の 3 倍 | PR 分割を提案、責任範囲外は切り出す |
| ジョブ失敗が 4 種類以上 | 「私の責任 / 私が表面化させた pre-existing / 他人 commit 由来」に分類 |
| force push 4 回目の誘惑 | branch / commit 設計を疑う、別アプローチに切替 |
| 「ついでに直そう」が内部に湧く | 一旦保留、別 PR + issue に切り出すのが筋か自問する |
| 「もう 1 回だけ試したら通る気がする」 | iteration trap の確定的サイン。即止まる |

## 原則 5: 切り離し判断の優先度

「PR を merge するために CI を緑化する」のは目的の取り違え。本来の目的は「**価値あるデリバリを review 可能な単位で確実に届ける**」こと。CI 緑化が**他の問題を巻き込まないと達成できない**なら、PR を分割する方が ROI 高い。

切り離しの優先順位:

1. **私の責任で起こした問題** → 当該 PR 内で解決
2. **私の作業で表面化した pre-existing 問題** → 別 PR + tracking issue で追跡
3. **他人の commit 由来の問題** → 担当者に委譲、可能なら PR から切り離す
4. **私の commit だが iteration trap に陥った問題** → PR から切り離して別 PR で再起票（部分的に放棄する勇気）

## 実際の運用

作業中に以下の内部シグナルが立った時、必ず本 Skill に立ち戻る:

1. 「もう 1 回だけ試したら通る気がする」 → 原則 1（深追い禁止）
2. 「ついでにこれも直しておこう」 → 原則 2（巻き込み禁止）
3. 「force push 何回目だっけ」 → 原則 3（force push 上限）
4. 「自分の責任で全部直すべきだ」 → 原則 5（責任範囲の正しい切り分け）

これらは外部からは見えない内的シグナルであり、**自分で気づいて止める**ことが本 Skill の核心。納品物の品質は「やったこと」だけでなく「やめたこと」でも測られる。

## 関連 Skill / 記憶

- `docs-delivery-principles` — 「量を言い訳にした段階対応は不可」と通底（やるべきは即やる、やるべきでないは即切る）
- 過去事例: PR #840（2026-04-27）/ tracking issue #841
