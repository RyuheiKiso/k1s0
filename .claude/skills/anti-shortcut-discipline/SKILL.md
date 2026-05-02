---
name: anti-shortcut-discipline
description: 長時間作業 / 大量編集中に「とりあえずここだけ修正」の短絡的局所修正に陥らないための強制規律。bug 修正の 4 段必須プロトコル（根本原因 / 横展開検査 / regression test / 波及範囲確認）と禁止コメント語彙を強制する。`iteration-and-scope-discipline`（scope 広がりすぎ防止）と対称の、scope 狭まりすぎ防止 skill。
---

# 短絡修正禁止規律

本 Skill は scope を**狭めすぎる**癖の対策。`iteration-and-scope-discipline` が「広げすぎ」を止めるなら、本 Skill は「狭めすぎ」を止める。両者は対称で、どちらも欠けると bug が残るか PR が膨れる。

## なぜ必要か（k1s0 における実証）

過去に短絡修正で済ませていたら危なかった事例が SHIP_STATUS.md に記録されている：

- **G3 → H1（2026-04-30）**: Go HTTP gateway 1 箇所で cross-tenant CRITICAL bug を発見。「その 1 RPC だけ修正」で止めていたら、Rust audit gRPC 4 RPC + Go workflow 6 RPC + Go secret 4 RPC の**同パターン 14 RPC が残った**。横展開検査をサボらなかったから 14 RPC 全部潰せた。
- **F10**: 「ADR-MESH-001 を新規導入」で済ませていたら ADR 採番体系が崩壊。ADR-0001 への統合が正解だった。短絡的な「とりあえず新規 ADR」が罠。
- **H4**: 「Backstage を試す」だけのつもりで `kubectl apply` / `helm install` を直接呼び続けた結果、local-stack SoT が崩壊し **31 helm release / 9 種類 / 6 カテゴリの drift** に発展。"とりあえず手で apply" の累積が構造破壊を生んだ。

3 件とも「目の前の症状を局所修正で済ませる」誘惑に勝った結果、根本治療に到達している。逆に言えば、勝ち損ねていれば**現状の品質はなかった**。

## 短絡修正発生のメカニズム

長時間 / 大量編集で発生しやすい背景：

1. **コンテキスト疲労**: tool 呼び出しが連続すると、目の前の差分しか見えなくなる
2. **報酬ハッキング**: 「とりあえず動いた」を完了と錯覚するバイアス
3. **横展開の盲点**: 1 箇所修正の達成感で、同パターン検査をスキップする
4. **時間圧の自己生成**: 「今すぐ進めたい」気持ちで根本修正を避ける
5. **scope creep 回避の過剰反応**: 広がりすぎ対策が効きすぎて、必要な横展開まで止める

特に Claude セッションでは、長時間 / 連続 Edit / 連続エラーで顕在化しやすい。

## bug 修正の 4 段必須プロトコル

bug を発見した時、以下 4 段を**省略不可**で踏む。1 段でもサボったら短絡修正と判定する。

### 1. 根本原因（5 Whys）

なぜその bug が起きたかを 5 段掘る。表層の "fix" ではなく、**構造的にその bug が許される理由**まで到達する。

例（G3 cross-tenant bug の 5 Whys）:
1. なぜ tenant-A JWT で tenant-B のリソースが取れるか → cross-tenant 検査が skip された
2. なぜ skip されたか → HTTP gateway が AuthInterceptor を呼ぶ際 `req=nil` を渡した
3. なぜ `req=nil` か → gRPC 経路と HTTP 経路で auth 検査の入力契約が違っていた
4. なぜ契約が違うか → handler 段に auth 検査がなく、interceptor 段の単一点に集約されていた
5. なぜ単一点だったか → 「気をつける」設計（規律依存）で、構造的強制になっていなかった

→ 根本原因 = 「規律依存設計」。修正は handler 段に強制関数を置くことで「構造的強制」へ変える。

### 2. 横展開検査

同パターンの問題が**他の RPC / 他の言語 / 他のレイヤ**に存在しないか、grep を**必ず 1 回**サボらず実行する。

- **言語横断**: Go / Rust / .NET / TypeScript すべての該当ファイルを grep
- **レイヤ横断**: tier1 / tier2 / tier3 / SDK / infra / deploy で同パターンを grep
- **API 横断**: 12 公開 API + Admin 2 + Health 2 = 16 service すべての RPC をリスト化

検査結果は **「N 箇所スキャン、M 箇所該当、修正対象 K 箇所」** を必ず数値化する。「該当なし」の場合も、走査範囲（パターン / 除外）を明記する（不在の証明）。

### 3. regression test 追加

bug 修正だけの commit を作らない。同 commit に**最低 1 件の regression test** を含める。test の役割は：

- 修正が正しいことの検証
- 将来の退行（regression）の自動検出
- 「気をつける」を「テストで強制」に変換

H2 のように **16 ケースの regression test を CI 化**するのが理想（go test ./... と cargo test --release --lib で全 PASS）。最低でも修正コードの 5 行に対して test 1 ケースは追加する。

### 4. 波及範囲確認

修正が他レイヤに伝播するか確認。

- **SDK**: 公開 API の挙動が変わるなら 4 言語 SDK すべての E2E を確認
- **docs**: ADR / 概要設計 / 実装設計の記述と矛盾しないか
- **runbook**: 既存 runbook の手順が無効化されていないか
- **下流サービス**: tier2 / tier3 / examples が同 API を呼んでいる経路を破壊していないか
- **monitoring**: 既存 alert / metric の意味が変わっていないか

波及範囲 0 を確認した場合も「波及範囲なし、確認した観点 N 件」と明記する。

## 大量編集中の自己点検

長時間セッション / 大量 Edit 中は**連続 10 Edit ごとに**以下を自答する：

1. **意図逸脱**: 当初のユーザー指示から外れた作業に滑っていないか
2. **横展開未消化**: 直近で発見した bug パターンが、まだ未修正の箇所に残っていないか
3. **報酬ハッキング**: 「とりあえず動いた」を完了と判定していないか。テスト / ビルド / E2E まで通したか
4. **構造的負債の蓄積**: 個別修正の積み重ねで、共通化すべき構造が散在していないか

5 分以上同じファイルを編集している時は、**一旦離れて全体構造を見直す**。視野狭窄の典型。

## 禁止コメント語彙

以下の語彙を**新規追加禁止**。短絡修正の自白に他ならない：

- `TODO` / `FIXME` / `XXX`
- `とりあえず` / `暫定` / `仮置き` / `あとで` / `後で`
- `今は〜のまま` / `仮に〜` / `暫定的に〜`
- 英語: `for now` / `temporary` / `quick fix` / `hack` / `workaround` (without ticket reference)

残す必要がある場合は **ADR / Issue / Plan に昇格**する。コード内に留めない。コードコメントで未来を約束しない（CLAUDE.md「No half-finished implementations either」）。

既存のこれら語彙は `/audit slack` で炙り出して順次潰す。

## 症状治療 vs 根本治療の判定

Edit を実行する**前**に必ず自問する：

- **この修正は症状治療か、根本治療か**

症状治療の例（短期で動かすため）:
- 例外を握りつぶす（catch して log するだけ）
- magic value をハードコードして回避
- 1 箇所だけパッチして同パターンを残す
- 失敗時の retry を入れて根本原因を隠す

根本治療の例:
- 例外を起こす設計上の前提を直す
- 共通関数 / 型 / interceptor で構造的に強制する
- 横展開検査で同パターンを全部潰す
- 失敗を発生させない invariant を確立する

**症状治療単独で commit しない**。症状治療しか時間がない場合：
1. 症状治療を施す
2. 根本治療を別 task / ADR / Issue に**必ず**切り出す
3. コードに残すコメントは「TODO」ではなく **`See ADR-XXX-NNN` / `See #issue` の参照のみ**

## 規律 vs 構造（再発防止の質）

bug の再発防止には 3 段階ある。**下に行くほど強い**：

| 段階 | 例 | 強度 |
|---|---|---|
| ドキュメント | 「気をつける」とコメント / runbook に書く | 弱 |
| 規律 | code review / lint で人間が check | 中 |
| 構造 | 型 / 共通関数 / コンパイラ / hook で物理強制 | 強 |

**可能な限り「構造」まで降ろす**。

例（k1s0 の実例）:
- G3 → H1: cross-tenant 検査を 共通 `EnforceTenantBoundary` 関数に集約 → 全 RPC が必ず通る構造
- G8: Audit WORM をアプリ層で気をつける → Postgres trigger（`BEFORE UPDATE OR DELETE`）で物理拒否
- H4: SoT 維持を運用規律に依存 → Kyverno admission deny + CI drift-check で物理強制

## 自己点検チェックリスト

Edit / commit 前に以下を通す：

### bug 修正時（必須 4 段）

1. [ ] 根本原因を 5 Whys で言語化したか（コメントまたは PR description に記載）
2. [ ] 横展開 grep を打ち、対象 N 箇所 / 該当 M 箇所 / 修正 K 箇所を数値化したか
3. [ ] regression test を最低 1 件追加し、修正と同 commit に含めたか
4. [ ] 波及範囲を SDK / docs / runbook / 下流 / monitoring の 5 軸で確認したか

### 大量編集時（10 Edit ごと）

1. [ ] 当初のユーザー指示から外れていないか
2. [ ] 直近の bug パターンの未修正箇所はないか
3. [ ] 「とりあえず動いた」で止まっていないか
4. [ ] 共通化すべき構造が散在していないか

### コメント追加時

1. [ ] 禁止語彙（TODO / FIXME / とりあえず / 暫定 等）を追加していないか
2. [ ] 残す必要があるなら ADR / Issue に昇格したか

### 修正の質（規律 vs 構造）

1. [ ] この修正は症状治療か根本治療か即答できるか
2. [ ] 再発防止が「気をつける」で終わっていないか
3. [ ] 構造（型 / 共通関数 / hook）で物理強制できる経路を検討したか

## 関連 skill / hook / memory

- `iteration-and-scope-discipline`: 対称（scope 広がりすぎ防止）。両方読むこと
- `principal-architect-mindset` Layer 2: 「規律 vs 構造」「横展開検査」「Chesterton's Fence」を本 skill と直接連携
- `audit-protocol`: 短絡修正の事後検出（slack 軸 audit）
- `.claude/hooks/anti-shortcut.py`: 本 skill の物理強制（warn のみ、block しない）
- memory `feedback_anti_shortcut.md`: 本 skill を呼ぶべき状況の記録

## hook との関係

`.claude/hooks/anti-shortcut.py` は本 skill の対応 hook。Edit / Write 時に：

- 禁止語彙の新規追加
- 同一ファイル連続 Edit
- bug fix らしき Edit で test ファイル touch なし
- Unimplemented 新規導入
- 空 catch / silent error suppress

を warn する（block しない、誤検知時の足止めを避ける）。warn が出たら本 skill を読み返す合図。`K1S0_ANTI_SHORTCUT_DISABLE=1` で無効化可能（commit しない運用）。

## 適用範囲外（本 skill を呼ばないケース）

- 純粋な新規実装（既存の bug を修正していない）
- typo 修正 / コメント誤字訂正
- 機械的な rename
- 自動生成コードの再生成

「直前に bug を発見した」「Edit が長時間 / 大量に及んでいる」「同じエラーが繰り返し出ている」のいずれかなら本 skill を呼ぶ。
