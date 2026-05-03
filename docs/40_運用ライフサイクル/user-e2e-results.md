# user e2e 実走結果

本書は ADR-TEST-008 / 010 で確定した user e2e（kind + minimum stack、PR + nightly CI で機械検証）の実走結果を時系列で記録する **live document**。CI で nightly cron が user full を機械実行するため、本書の更新は CI 経由（自動）+ owner の判断（手動）の併用で進む。

## 本書の位置付け

user e2e は ADR-TEST-008 で「16GB host OK / PR + nightly CI 可」と決定された。owner suite と異なり CI で機械検証される（owner は CI 不可で release tag ゲート ADR-TEST-011 で代替保証）。本書は CI 結果の月次サマリと、利用者からのフィードバック（自アプリで遭遇した問題）を集約する hub として機能する。

CI 経路（自動）:
- `pr.yml` の `e2e-user-smoke` job が PR 毎に kind + smoke を 5 分以内で実行
- `nightly.yml` が cron 03:00 JST で `_reusable-e2e-user.yml` を mode=full で呼び出し、smoke + examples を 30〜45 分で実行
- 失敗 artifact は GitHub Actions の `actions/upload-artifact@v4` で 14 日保管

手動経路（利用者 / owner）:
- 利用者が自アプリで遭遇した問題を本書に追記（issue 起票後の集約として）
- owner が定期的に CI 結果トレンドを月次でレビュー

## CI 結果サマリ template

各月のサマリは以下の形式で追記する。

```markdown
### YYYY-MM

- 期間: YYYY-MM-01 〜 YYYY-MM-末日
- nightly 実走数: N 回
- nightly PASS 率: NN.N%（PASS / 総実走）
- PR smoke 実走数: N 回（PR 件数を反映）
- PR smoke PASS 率: NN.N%
- 主要 failure（FAIL 1 件以上時のみ）:
  - YYYY-MM-DD nightly: <root cause + 対応 PR>
  - YYYY-MM-DD PR #NNN smoke: <root cause + 対応>
- 利用者からのフィードバック（issue 起票件数）: N 件
```

## 月次サマリ

（リリース時点では CI 機械検証経路の整備が完了したのみで、実走 entry なし。
nightly cron が初回起動した翌月から月次サマリを追記する。）

## 利用者フィードバックの集約

利用者が自アプリ開発で遭遇した「k1s0 SDK 経路の問題」は、GitHub issue として起票したうえで本書に集約 entry を追記する。これにより「user e2e で機械検証されている範囲」と「利用者が実際に遭遇する問題」のギャップが時系列で見える。

```markdown
#### YYYY-MM-DD: <issue 概要>

- issue: #NNN
- 利用者報告: <概要>
- root cause: <分析結果>
- 対応 PR: #NNN
- 影響範囲（user e2e 既存 test で検出されたか）: 検出済 / 検出漏れ
- 検出漏れの場合、追加 test 配置: tests/e2e/user/{smoke,examples}/<新 test>.go
```

## 関連

- [ADR-TEST-008](../02_構想設計/adr/ADR-TEST-008-e2e-owner-user-bisection.md) — owner / user 二分構造
- [ADR-TEST-010](../02_構想設計/adr/ADR-TEST-010-test-fixtures-sdk-bundled.md) — test-fixtures 4 言語 SDK 同梱
- [ops/runbooks/RB-TEST-USER-SMOKE.md](../../ops/runbooks/RB-TEST-USER-SMOKE.md) — 利用者向け smoke 実走 Runbook
- [.github/workflows/_reusable-e2e-user.yml](../../.github/workflows/_reusable-e2e-user.yml) — CI 経路本体
- [docs/05_実装/30_CI_CD設計/35_e2e_test_design/](../05_実装/30_CI_CD設計/35_e2e_test_design/) — 実装規約一式
