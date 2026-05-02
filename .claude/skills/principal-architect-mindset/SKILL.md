---
name: principal-architect-mindset
description: k1s0 で設計判断・実装判断・運用判断を下す時に参照する積極的思考フレーム。プリンシパル級エンジニア / アーキテクトの判断軸を 3 レイヤで具体化する。over-engineering を避けつつ長期視点・トレードオフ・連鎖反応を読む。ADR 起票時 / 設計書執筆時 / コード Edit 時 / インフラ変更時 / トラブル対応時に参照する。
---

# プリンシパル級アーキテクト思考

本 Skill は「どう書くか」の規約ではなく「どう判断するか」の思考軸を定める。既存 skill 群（`docs-adr-authoring` / `docs-design-spec` / `iteration-and-scope-discipline` / `anti-shortcut-discipline`）はそれぞれ書式や反パターン回避を担うが、それらの**判断の前段**に位置する。

## 大前提: プリンシパル級 ≠ over-engineering

CLAUDE.md にある「Don't add features, refactor, or introduce abstractions beyond what the task requires / Three similar lines is better than a premature abstraction」と矛盾しないこと。

**「読み」は厚く、「書き」は薄く**。

長期視点・トレードオフ・連鎖反応を**読む**のは厚く、実際に**書く**抽象化やコード・YAML は必要十分に薄く保つ。判断時の思考量と成果物の量は別軸。プリンシパル級とは：

1. 長期視点・トレードオフ・連鎖反応を**読む**（書く前に）
2. 必要十分な構造を選ぶ（過剰でも過少でもない）
3. 不可逆な判断ほど慎重に（reversibility 軸）

「先回りして拡張ポイントを切る」「将来のために generics / interface / trait を切る」は**禁じ手**。具体例 3 つが揃うまで抽象化しない（Rule of Three）。

## 3 つの判断レイヤ

判断の粒度別に、Edit 前に通すべきチェック軸を分離する。各レイヤは独立だが、上位レイヤの決定が下位レイヤを縛る関係にある。

### Layer 1: 設計判断（ADR / 概要設計レベル）

ADR 起票・技術選択・アーキテクチャ判断時に通すフレーム。

1. **時間軸**: 5〜10 年後にこの決定で困らないか。採用組織が世代交代しても保守できるか
2. **one-way door 判定**: 後戻り可能か。two-way door なら早く決めて進む、one-way door なら 2-3 倍慎重に（Bezos 由来）
3. **業界標準との整合**: de facto standards / 業界ベストプラクティスに逆らっていないか。逆らうなら k1s0 固有の制約で正当化できるか（標準逸脱は技術的債務として記録）
4. **逆 Conway の法則**: 技術選択が組織分業を縛る関係 / 組織構造が技術構造を縛る関係を意識したか
5. **物理制約**: CAP / FLP / 光速 / メモリ階層 / ネットワーク帯域 / ストレージ階層に反していないか
6. **TCO**: 採用コスト + 教育コスト + 引き継ぎコスト + 退職リスク + ランタイムコストの総和
7. **検討肢の最弱点**: 3 案以上書き出した上で、各案の**最大の弱点**を必ず言語化する（ADR 規約に既存）

### Layer 2: 実装判断（コード Edit レベル）

コード変更・バグ修正・refactor 時に通すフレーム。

1. **症状 / 根本判定**: 症状治療か根本治療か。症状治療単独では commit しない（→ `anti-shortcut-discipline`）
2. **横展開検査**: 同パターンの問題が別箇所にないか grep を**必ず 1 回サボらない**
3. **規律 vs 構造**: 再発防止を「気をつける」ではなく「型 / 共通ライブラリ / hook / コンパイラ強制」で担保できないか
4. **Chesterton's Fence**: 消そうとしているコード / 設定 / コメントの存在理由を理解したか。理解せず消すのは事故
5. **Hyrum's Law**: 観測可能な API / 挙動の変更が、暗黙に依存している呼び出し元を壊さないか（公開 API / 公開挙動はすべて誰かに依存される）
6. **Postel's law**: 入力には寛容、出力は厳格（外部 API 境界）
7. **抽象 vs ハードコード**: いま抽象化する必要があるか。Rule of Three 未達ならハードコード継続が正解

### Layer 3: 運用判断（インフラ / デプロイ / トラブル対応レベル）

インフラ変更・デプロイ操作・障害対応時に通すフレーム。

1. **blast radius**: この変更が壊れた時の影響範囲（何ノード / Pod / tenant / region）
2. **reversibility**: rollback の所要時間 / state を変更する操作なら no-rollback を覚悟しているか
3. **observability**: 効果 / 副作用を観測する metric / log / trace は事前に存在するか。存在しないなら計装を先に
4. **canary / gradual rollout**: いきなり全環境に流していないか
5. **SoT 整合**: この操作は Single Source of Truth に反映されているか（H4 ADR-POL-002 の教訓）
6. **playbook 先行**: 失敗時の手順は事前に書かれているか（Runbook タイプ C 5 段構成）

## k1s0 における代表事例（フレームの実証）

各判断軸が機能した実例。抽象論だけでは忘れられるため具体で裏付ける。

### Layer 1（設計判断）の実例

- **ADR-TIER1-001（Go + Rust hybrid）**: 単一言語の方が楽だが、Dapr SDK stable は Go、ZEN Engine は Rust という制約から hybrid 選択。「採用組織の引き継ぎコスト」と「stable backend の有無」を天秤にかけた TCO 判断。
- **ADR-DIR-001（contracts 昇格）**: proto を tier1 に置くと依存方向が逆転するため root 直下の `contracts/` に昇格。「依存方向が組織分業を決める」逆 Conway 判断。
- **ADR-0001（Istio Ambient vs sidecar）**: sidecar の方が成熟しているが、Pod 起動時間 / リソース効率 / 5-10 年の保守の観点で Ambient を選択。長期時間軸 + 物理制約判断。

### Layer 2（実装判断）の実例

- **G3 → H1（cross-tenant 集約）**: 1 RPC の cross-tenant CRITICAL bug を発見した時、共通 `EnforceTenantBoundary` 関数（Go: `internal/common/auth.go` / Rust: `crates/common/src/auth.rs`）に集約することで全 RPC が同関数を必ず通る構造に変更。Go secret 4 RPC + Go workflow 6 RPC + Rust audit 4 RPC の同パターンを横展開で潰した上、regression test 16 ケースを CI 化。「規律 vs 構造」+「横展開検査」の典型。
- **F2（envoy gRPC-Web translator 追加）**: TS SDK が gRPC-Web 専用で素 gRPC HTTP/2 trailer に到達不能 → 各 SDK に修正を入れず、別 layer の translator で吸収。「言語別 SDK の物理制約を別レイヤで吸収」する Hyrum's Law 配慮（既存 SDK 利用者を壊さない）。
- **E2 schemathesis（OpenAPI contract）**: 生成された 5000+ test case で 50 件失敗 → 個別の 405 / 400 / 500 を場当たりに直すのではなく、「未登録 path の 404 / 非 POST の 405 + Allow / 必須欠落の InvalidArgument 統一」という**横断的な HTTP semantics 統一**として解いた。症状治療を根本治療に昇華した例。

### Layer 3（運用判断）の実例

- **H4 ADR-POL-002（SoT 三層防御）**: drift の根本原因が「人間の規律依存」だったため、Kyverno（admission deny）+ CI（drift-check）+ mode 切替（dev/strict）の三層で構造的に阻止。「規律ではなく構造で再発防止」+「失敗前提設計」。
- **G8（Audit WORM Postgres trigger）**: Audit hash chain の WORM 性を「アプリ層で気をつける」ではなく Postgres trigger（`BEFORE UPDATE OR DELETE`）で物理拒否。アプリ層の bug があっても DB 層が止める二段防御。
- **H3a（NetworkPolicy + topology spread）**: 単に NetworkPolicy を書くだけでなく、kindnet では強制されない事実を実機検証で確認し、Calico への切替が production 前提条件であることを裏付けた。「設定の存在 ≠ 強制されている」を疑う observability 軸。

## やってはいけない（プリンシパル級の偽物パターン）

判断軸を間違って適用すると、見かけだけプリンシパル級で実体は害になる。以下は**偽物**：

1. **過剰抽象化**: 先回りして generics / interface / trait / abstract class を切る。Rule of Three 違反
2. **銀の弾丸の信奉**: 「これを使えば全部解決」を謳う技術選択。必ず弱点を 3 つ言語化する。弱点が言語化できない技術選択は危険信号
3. **暗黙の前提**: 「みんな分かってる」「業界の常識」で書く。新規参画者が読んで理解できる粒度で書く
4. **コンテキストフリーの best practice 引用**: 「業界標準だから」「FAANG がやっているから」だけで判断する。k1s0 固有の制約に照らした正当化を必ず添える
5. **過去の判断の追認**: 既存 ADR / 既存設計を疑わず継承する。Chesterton's Fence で「なぜそうしたか」を理解した上で継承する。理解せず継承するのは責任放棄
6. **完璧主義による麻痺**: two-way door の判断で 2-3 倍時間をかける。後戻り可能なら早く決めて進む方が長期 TCO は安い
7. **goal displacement**: 測定指標が目標に化ける（Goodhart's law）。「テストカバレッジ 90%」が目標化すると意味のないテストが増える
8. **未来の自分への先送り**: 「あとで refactor する」「v2 で解決」と書く → CLAUDE.md「No half-finished implementations either」違反

## 既存 skill / memory との連携導線

本 skill は単独では機能しない。既存の規約 skill と組み合わせて使う：

| 既存 skill / memory | 本 skill との連携 |
|---|---|
| `docs-adr-authoring` | ADR 起票時に Layer 1 を必ず通す。「検討した選択肢 3 件以上」「決定理由（なぜ他ではないか）」が Layer 1 と直接連携 |
| `docs-design-spec` | 設計書執筆時に Layer 1 + Layer 2 を通す |
| `docs-delivery-principles` | 「部外者が読めない納品は要件未達」を Layer 1 「暗黙の前提」回避と統合 |
| `anti-shortcut-discipline` | bug 修正時に Layer 2（特に「規律 vs 構造」「横展開検査」「症状 / 根本判定」） |
| `iteration-and-scope-discipline` | PR scope 判断に Layer 1（reversibility / blast radius） |
| `avoid-meta-only-work`（memory） | 楽な判断軸（docs を整える方）に逃げず、Layer 2-3 の本質判断に踏み込む |
| `audit-protocol` | audit 結果を読む時に Layer 1-3（不足を潰す優先順位判断、blast radius 順） |

## 自己点検チェックリスト

判断を下す前に以下を通す。10 秒以内に自答できないなら、その判断はまだ熟していない：

### 設計判断（Layer 1）

1. [ ] 5 年後にこの決定で困るシナリオを 1 つ以上具体化したか
2. [ ] one-way door / two-way door のどちらか即答できるか
3. [ ] 検討肢を 3 つ以上、各々の最大の弱点付きで書き出したか
4. [ ] 業界標準に逆らうなら、k1s0 固有制約での正当化を 1 行で書けるか
5. [ ] TCO を「採用 / 教育 / 引き継ぎ / 退職 / ランタイム」の 5 軸で評価したか

### 実装判断（Layer 2）

1. [ ] この修正は症状治療か根本治療か即答できるか
2. [ ] 同パターンの問題が別箇所にないか grep を 1 回打ったか
3. [ ] 再発防止を「規律」ではなく「構造」で担保できないか考えたか
4. [ ] 消す / 変えるコードの存在理由を理解しているか（Chesterton's Fence）
5. [ ] 公開挙動を変える場合、暗黙依存している呼び出し元を確認したか

### 運用判断（Layer 3）

1. [ ] blast radius（影響範囲）を具体的に言語化したか
2. [ ] rollback の所要時間 / 不可逆性を判定したか
3. [ ] 効果 / 副作用を観測する metric / log / trace が事前にあるか
4. [ ] SoT に反映されているか / 反映する手順が明確か
5. [ ] 失敗時の playbook が事前に存在するか

各レイヤ 5 項目、計 15 項目。全部に Yes が出るまで実装に着手しない。

## 適用範囲外（本 skill を呼ばないケース）

以下では本 skill は不要 / 過剰で、呼ぶと判断が遅れて害になる：

- typo 修正
- コメントの誤字訂正
- 機械的な rename（IDE 機能で完結）
- 自動生成コードの再生成
- 依存バージョンの patch level 上げ（minor / major は Layer 1 が要る）

「Edit が 3 行未満で、Layer 1-3 のいずれの懸念も顕在化しない」と即答できるなら本 skill をスキップしてよい。迷ったら通す。
