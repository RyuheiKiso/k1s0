---
id: plan.tier1.scenario_library_new_lang_category
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# Library 新言語 / 新カテゴリ追加

## 一文方針

新言語 / 新カテゴリの Library 追加は tier1 言語スタック登録を先行条件とし、17 カテゴリ表への Lv 割付・3 パッケージ構成配置・全言語 Testcontainers conformance test green を揃えて dual reviewer sign-off を取得する。

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が GitHub PR list を確認し、「Swift が tier1/tier2 言語スタックに登録され Library サポートの要求が来た」という issue が上がっていることに気付く。手元には `04_提供機能カテゴリ.md` と Backstage Catalog、Mattermost 越しに dual reviewer 2 名と要求提起元の tier2 担当者がいる。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: 17 機能カテゴリ × 対応言語の facade を codegen で統一し、手作業による整合確認コストをゼロにする

## 現状業務での痛み

- 新言語追加のたびに 17 カテゴリ全てとの整合を手動で確認し、見落としが後のコンパイルエラーで発覚する
- codegen が未整備で言語ごとに手書き実装が増殖し、カテゴリ間の挙動差異が蓄積する
- 公開 API snapshot の管理が曖昧で、新言語追加時に既存 API surface が変化していても気付かない
- 言語スタック登録前に Library 要求が来て、先行実装が半端な状態で混在し始める

## k1s0 でこう変わる

- 17 カテゴリ × 4 言語 facade を codegen で統一し、手書き実装の増殖を構造的に防止する
- `public_api_snapshot.lock.yaml` の snapshot 差分 CI が意図外 API 変更を merge 前に検出する
- 言語スタック登録を先行条件として CI で強制し、未登録言語の要求を早期に保留する
- Testcontainers conformance test が全言語 / 全 L2* 実装で green を merge 条件とし、整合確認を自動化する

## Trigger（発火条件）

新言語（例: Swift / Kotlin）または新機能カテゴリの Library 追加要求が上がった時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 年次〜不定期
- 典型きっかけ: 「Swift が tier1/tier2 言語スタックに登録され Library サポートの要求が来た」「AI 推論カテゴリが新設される際に 17 カテゴリへの追加が必要になった」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ 要求提起者（tier2 担当者 or コミュニティ）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | GitHub PR list | 言語スタック登録確認・17 カテゴリ追加・conformance test 作成・snapshot 更新 |
| 関与（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | 追加内容レビュー・sign-off |
| 関与（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | 追加内容レビュー・sign-off |
| 関与（tier2 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | 新言語 / 新カテゴリの要求提起・動作確認協力 |

## 個人 KPI / 達成感

- 既存 4 言語の Library コード生成 all green（新言語追加後も既存が壊れない）
- 公開 API snapshot 差分 CI 意図外変更 0 件
- Testcontainers conformance test 全言語 green 率
- dual reviewer 応答時間 ≤ 24h

## 工数 / 関与人数 / コスト感

- 初回（新カテゴリ L3 追加）: 1 日、関与 3〜4 名（主役 + dual reviewer 2 名 + 要求提起者 1 名）
- 平常（新言語追加・L2* ペア選定）: 2〜3 日、関与 3〜4 名
- 失敗時（conformance fail・snapshot 意図外変更）: +1〜2 日、関与 4〜5 名

## 前提

- v1.0.0 サポート言語は Rust / C# / Go / TypeScript の 4 言語のみ
- 17 機能カテゴリが `04_提供機能カテゴリ.md` に定義済み
- core / frontend / backend の 3 パッケージ構成が確立済み
- 公開 API snapshot が `public_api_snapshot.lock.yaml` で管理されている

## 流れ

1. **言語スタック登録の先行条件確認**: 新言語追加の場合は tier1 / tier2 言語スタックへの登録が先行条件であることを確認する。v1.0.0 スコープ外の言語（例: Swift / Kotlin / Python）は原則として要求を保留し、ロードマップ議題として記録する。
   - 登録済み言語のみ: 新カテゴリ追加フローへ進む
   - 未登録言語: 言語スタック登録 PR を別途作成し承認を取得してから本シナリオを再開する

2. **新言語 vs 新カテゴリの分岐**: 追加要求の性質に応じて対応フローを分岐する。
   - 新言語の場合: Server 系コンポーネントで利用するか Library としての提供かを選択。Companion 代替として使用する場合は [シナリオ 06](06_Companion役割AB拡張.md) へ。
   - 新カテゴリの場合: 手順 3 以降で 17 カテゴリ表への追加手続きを行う。

3. **17 カテゴリ表への追加と Lv 割付**: `04_提供機能カテゴリ.md` の 17 カテゴリ表に新カテゴリを追加し [3 抽象レベル](../../../03_概要設計/02_tier1設計方針/02_Library.md)（L1+ / L2* / L3）の Lv 割付を決定する。
   - L3 OSS 中立: 複数 OSS への薄い抽象層。OSS 差し替えを保証するが深い機能は提供しない。
   - L2* 同族保証: 同族 OSS 2 実装を選定し conformance test で等価性を保証する。
   - L1+ 単一深耕: 特定 OSS の深い機能を 100% 露出。移行 toolchain 計画が必須。

4. **3 パッケージ構成への配置**: 新カテゴリをどのパッケージに配置するか決定する。
   - core: tier1 facade 共通の型定義 / ユーティリティ / バリデーション（全パッケージ共通基盤）
   - frontend: UI コンポーネント / クライアントサイド状態管理 / 認証 UI フロー
   - backend: サーバーサイドビジネスロジック / データアクセス / 外部 API クライアント
   - 配置が曖昧なカテゴリは core に置き、明確化次第で frontend / backend に移動する

5. **Testcontainers conformance test の作成**: 4 言語（+ 新言語の場合はその言語）で conformance test を作成し全 green を merge 条件とする。
   - 各 test は新カテゴリの主要 API を網羅する
   - L2* の場合は 2 実装に対して同一 test suite を実行
   - L1+ の場合は OSS の代表的なシナリオを網羅

6. **公開 API snapshot の更新**: 新 API surface を `public_api_snapshot.lock.yaml` に追加し、snapshot 差分 CI を通じて意図した API のみが追加されていることを確認する。既存 API への影響が 0 件であることを検証する。

7. **dual reviewer sign-off**: 以上の手順が完了したことを dual reviewer（tier1 2 名）が確認し sign-off する。

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | 言語スタック登録確認・新言語 vs 新カテゴリ分岐判定 | `言語スタック登録確認: Swift 未登録 / 要求を保留しロードマップ登録` |
| 0 | tier1 担当者 | （新カテゴリの場合）17 カテゴリ表追加・Lv 割付決定 | `新カテゴリ「AI 推論」L3 割付 / 3 パッケージ配置: core` |
| 1 日 | tier1 担当者 | Testcontainers conformance test 作成・snapshot 更新 | `conformance test 作成中 / 4 言語 green 確認待ち` |
| 2 日 | dual reviewer A/B | sign-off | `dual sign-off 完了 / snapshot 差分 CI: 意図した追加のみ` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **FA 生産指示・設備操作**: 新言語（例: Swift）の Library 追加により、iOS / macOS ベースの設備操作端末から tier1 facade を直接利用できるようになり、FA 生産指示システムの端末多様化に対応できる。
- **在庫**: 新カテゴリ（例: AI 推論）の Library 追加は、在庫予測・需要予測機能の共通基盤整備に直結し、在庫管理精度の向上を可能にする。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [検証規律適合仕様](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md)
- [OSSライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)

**関連 OSS**:
- Testcontainers: conformance test の実行基盤（全言語対応）
- Buf: proto コード生成（新カテゴリが gRPC を利用する場合）
- cargo / dotnet / go / npm: 各言語のパッケージマネージャ（Harbor mirror 経由）

## 期待結果 / 観測指標

- `04_提供機能カテゴリ.md` に新カテゴリが Lv 割付と露出概念 allowlist とともに追記されている
- Testcontainers conformance test が全言語で green
- 公開 API snapshot 差分 CI が意図した追加のみを検出している（意図外変更 0 件）
- `public_api_snapshot.lock.yaml` に新 API surface が記録されている
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- 既存 4 言語の Library コード生成が全て成功している

## 失敗時の挙動 / escalation

- **未登録言語の要求**: 要求を保留しロードマップ議題に記録する。言語スタック登録 PR の作成を要求者に案内する。escalate 先: tier1 担当者 dual reviewer（SLA: 5 営業日以内にロードマップ議題登録）。
- **Testcontainers conformance test fail**: fail している言語 / 実装の OSS 選定を見直す。L2* で 1 実装が fail した場合は同族 OSS の別実装を選定する。escalate 先: tier1 担当者 dual reviewer（SLA: 48 時間以内に代替選定）。Backstage runbook `testcontainers-conformance-failure` を参照。
- **公開 API snapshot 差分 CI fail（意図外変更）**: 意図外の API 変更を revert し、[シナリオ 08](08_公開API_snapshot違反対応.md) の手順で原因を特定する。escalate 先: tier1 担当者 dual reviewer（SLA: 4 時間以内に revert）。
- **3 パッケージ配置の合意不成立**: dual reviewer 間で配置方針の合意が得られない場合は、tier1 全体での方針議論に escalate する。escalate 先: tier1 担当者全員（SLA: 5 営業日以内に方針決定）。

## 失敗パターン (anti-pattern)

- **未登録言語の要求を即時実装開始**: 言語スタック登録 PR が承認される前に Library コードを書き始め、後で承認が下りず中途半端な実装が残る。言語スタック登録の先行条件確認を最初のステップとして明文化する。
- **3 パッケージ配置を後から変える**: core に仮置きした API を frontend / backend に移動する際に SemVer major bump が必要になり、全 consumer への影響が発生する。配置決定を draft 段階で dual reviewer に確認し、後の移動をなくす。
- **L2* の 1 実装しか conformance test を書かない**: 一方の実装が green でも他方が fail していて、実際に切り替えたときに機能しないことが判明する。L2* の conformance test は 2 実装への同一 test suite 実行を merge 条件として強制する。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [公開 API snapshot 違反対応](08_公開API_snapshot違反対応.md)
