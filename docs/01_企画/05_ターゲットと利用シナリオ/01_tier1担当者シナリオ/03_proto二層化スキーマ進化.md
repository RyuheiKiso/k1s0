---
id: plan.tier1.scenario_proto_schema_evolution
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# proto 二層化スキーマ進化

## 一文方針

External / Internal 2 層 proto 構成を維持しつつ、[Apicurio Schema Registry](../../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md) の compatibility check と 4 言語コード生成の全 green を merge 条件として、backward / forward 互換を tier1 facade 内部で完結させる。

## Trigger（発火条件）

業務 API 変更で External Proto か Internal Proto の backward / forward 互換を確保する必要が生じた時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 週次〜月次（External Proto の breaking change は四半期）
- 典型きっかけ: 「受注 Domain Event に配送先住所フィールドを追加する要求が tier2 から来た」「検査結果スキーマで数値精度型変更が必要になった」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ tier2 担当者（下流影響の確認のみ）

## 前提

- External / Internal 2 層 proto + Transport Adapter Layer 構成が確立済み（[tier1 設計方針 01_Server系](../../../03_概要設計/02_tier1設計方針/01_Server系.md) 参照）
- Apicurio Schema Registry が稼働しており `apicurio_gitops_sot.lock.yaml` が管理されている
- Buf によるプロトコルバッファ lint / breaking change 検出が CI に組み込まれている
- 4 言語（Rust / C# / Go / TypeScript）のコード生成パイプラインが CI に組み込まれている

## 流れ

1. **External Proto 変更案の draft**: 変更種別を明示した draft を作成する。
   - field 追加（backward 互換）: field number を新規採番し既存 field には影響なし
   - field 削除（backward 非互換）: 削除前に reserved 宣言を挟む移行ステップが必要
   - 型変更（backward 非互換）: 原則として型変更は禁止。新 field を追加し旧 field を deprecated マークする形で対応する
   - oneof / map の変更: 追加は互換、削除・型変更は非互換として扱う

2. **External / Internal 差分検査**: CI の diff 検査で「External Proto の変更が Internal Proto に意図せず漏洩していないか」を確認する。Transport Adapter Layer は External と Internal を明示的に分離する責務を持つため、layer 間の proto フィールド対応が崩れていないことを検証する。

3. **互換方針の判定**: backward / forward 互換性の有無を判定し対応方針を決定する。
   - 互換あり: External Proto のみ更新し Internal Proto は変更不要の場合は Transport Adapter Layer のみ修正
   - 非互換で External/Internal 二層吸収: Internal Proto に互換ラッパーを追加し External は新旧両 API surface を提供
   - 非互換で SemVer major bump 必要: External Proto の major バージョンを上げ旧 endpoint を deprecation 期間後に廃止

4. **Apicurio Schema Registry への push と compatibility check**: 候補スキーマを Apicurio Schema Registry に push し compatibility check を実行する。
   - FULL_TRANSITIVE: 過去全バージョンに対して互換性を検証（推奨）
   - BACKWARD: 直前バージョンのみ検証（移行期間中の暫定設定時のみ）
   - compatibility check fail は merge 阻止の gate となる

5. **4 言語コード生成の検証**: Buf の `buf generate` を 4 言語（Rust / C# / Go / TypeScript）に対して実行し、全て成功することを確認する。生成コードの型エラーも含めてコンパイルが通ることを検証する。

6. **dual reviewer sign-off と lock ファイル更新**: 互換 CI green を確認後、dual reviewer（tier1 2 名）が sign-off する。`apicurio_gitops_sot.lock.yaml` に採用スキーマバージョンを記録する。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [スキーマ進化適合仕様](../../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- [apicurio_gitops_sot](../../../04_詳細設計/03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md)

**関連 OSS**:
- Buf: proto lint / breaking change 検出 / コード生成
- Apicurio Schema Registry: スキーマ互換性検証（FULL_TRANSITIVE / BACKWARD）
- gRPC / Connect-RPC: Transport Adapter Layer の実装基盤

## 期待結果 / 観測指標

- Apicurio Schema Registry の compatibility check が green（FULL_TRANSITIVE）
- Buf breaking change 検出が意図した変更のみを検出している（unexpected 0 件）
- 4 言語（Rust / C# / Go / TypeScript）のコード生成が全て成功
- `apicurio_gitops_sot.lock.yaml` にスキーマバージョンが記録されている
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- tier2 / tier3 の既存 import が破壊されていない（Testcontainers 結合テスト green）

## 失敗時の挙動 / escalation

- **Apicurio compatibility check fail**: merge 阻止。変更を revert するか、互換ラッパー追加・非互換バージョン bump のいずれかの方針を選択して再 draft する。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に方針決定）。Backstage runbook `schema-compat-failure` を参照。
- **4 言語コード生成 fail**: merge 阻止。生成エラーのある言語の proto 定義を修正する。特定言語の proto プラグインバージョン不整合の場合は Buf lockfile を更新する。escalate 先: tier1 担当者 dual reviewer（SLA: 8 時間以内に修正）。
- **tier2 / tier3 の結合テスト fail**: 変更が [Transport Adapter Layer](../../../03_概要設計/02_tier1設計方針/01_Server系.md) で吸収できていない可能性がある。adapter 実装を修正して再確認する。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に adapter 修正）。
- **Buf breaking change 検出（意図外）**: PR の変更内容を精査し意図しない breaking change を revert する。escalate 先: tier1 担当者 dual reviewer（SLA: 4 時間以内に revert）。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [新規 OSS 採用評価](01_新規OSS採用評価.md)
