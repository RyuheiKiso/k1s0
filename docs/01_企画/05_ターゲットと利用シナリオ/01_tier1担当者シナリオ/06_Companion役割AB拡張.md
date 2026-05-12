---
id: plan.tier1.scenario_companion_extension
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

# Companion 役割 A/B 拡張

## 一文方針

.NET Framework 4.8 ERP / レガシーシステムの Companion 追加要件を役割 A（Observability / 認証コンテキスト伝播）か役割 B（Transport Negotiation Runtime）に正確に分類し、レガシー環境テストで動作確認して Harbor mirror へ発行する。

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が GitHub PR list を確認し、「既存の .NET Framework 4.8 ERP が新しい観測 endpoint を要求した」という issue が上がっているのに気付く。手元には Harbor NuGet proxy ダッシュボードと Backstage Catalog、Mattermost 越しに dual reviewer 2 名と .NET Framework 環境保有の tier2 担当者がいる。

## Trigger（発火条件）

.NET Framework 4.8 ERP / レガシーシステムの Companion 経由観測 / transport 要件が追加された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 年次〜不定期
- 典型きっかけ: 「既存の .NET Framework 4.8 ERP が新しい観測 endpoint を要求した」「レガシー工場システムが bidi streaming を必要とする制御シナリオを追加した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ .NET Framework 環境保有の tier2 担当者（動作確認協力）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | GitHub PR list | 役割分類・実装・.NET FW 4.8 テスト・NuGet push・PR 提出 |
| 関与（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | 動作確認結果レビュー・sign-off |
| 関与（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | 動作確認結果レビュー・sign-off |
| 関与（tier2 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | .NET Framework 4.8 テスト環境での動作確認協力 |

## 前提

- Companion は .NET Framework 4.8 で動作する NuGet パッケージとして提供されている
- `k1s0.Companion.NetFx.OTelExt`（役割 A）と `k1s0.Library.NetFx`（役割 B 含む統合パッケージ）が Harbor mirror で管理されている
- Testcontainers または VM による .NET Framework 4.8 テスト環境が用意されている
- HTTP/2 enforcement 仕様と transport 縮退ルールが定義済み

## 流れ

1. **役割の分類**: 追加要件が役割 A か役割 B かを判定する。
   - 役割 A（Observability / 認証コンテキスト伝播 / CLR Profiler attach）: .NET Framework 4.8 アプリからの OpenTelemetry トレース / メトリクス / ログ送信、または JWT / OIDC 認証コンテキストの伝播が要件の場合
   - 役割 B（Transport Negotiation Runtime）: .NET Framework 4.8 アプリが bidi を含む RPC を利用する際の [Transport Adapter Layer](../../../03_概要設計/02_tier1設計方針/01_Server系.md) を経由した transport 自動選択・縮退が要件の場合
   - 両役割にまたがる要件の場合は、影響の大きい役割を主とし補完的に他役割の実装も行う

2. **役割 A の実装手順**:
   - CLR Profiler attach 手順を決定する（ApplicationInsights / OpenTelemetry の CLR Profiler API を使用）
   - `k1s0.Companion.NetFx.OTelExt` NuGet の更新バージョン（SemVer）を決定する
   - OpenTelemetry Collector への export endpoint を Companion 設定で指定できるように実装
   - 認証コンテキスト伝播: Baggage API を使用し、JWT の claim を downstream サービスに伝播
   - W3C TraceContext と Baggage の双方をサポートすることを確認

3. **役割 B の実装手順**:
   - bidi 含む RPC の transport 選択ロジックを実装する。優先順位: HTTP/2 (gRPC) > HTTP/1.1 + SSE（縮退）> long polling（最終縮退）
   - HTTP/1.1 + SSE 縮退時の per-tab 6 subscription 上限（ブラウザ HTTP/1.1 concurrent connection 制限由来）を考慮した実装を確認
   - transport ネゴシエーション失敗時の retry / fallback ロジックをテストする
   - .NET Framework 4.8 の HttpClient 制限（HTTP/2 非対応）を Companion 内部で吸収する実装を確認

4. **レガシー .NET Framework 4.8 テスト環境での動作確認**: Testcontainers（Windows コンテナ）または VM で .NET Framework 4.8 テスト環境を起動し以下を確認する。
   - CLR Profiler attach が正常に機能すること（役割 A）
   - OTel span が Collector に到達すること（役割 A）
   - bidi RPC が期待する transport で通信できること（役割 B）
   - transport 縮退が正常に動作すること（役割 B）

5. **NuGet パッケージの発行と Harbor mirror への push**:
   - `k1s0.Library.NetFx` NuGet パッケージとしてビルドし内部 Harbor mirror（NuGet proxy）に push する
   - cosign 署名と SBOM 生成を実施する
   - 旧バージョンは deprecation マークを付け一定期間後に削除する

6. **dual reviewer sign-off**: 動作確認結果と NuGet パッケージ発行の完了を dual reviewer（tier1 2 名）が確認し sign-off する。

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **FA 生産指示・設備操作**: .NET Framework 4.8 ERP と連携する設備操作システムへの Companion 提供により、FA 生産指示フローの observability とトレーサビリティが向上する。
- **図面 review**: レガシー ERP システム（.NET Framework 4.8）から tier1 facade を通じた図面データ連携が可能となり、図面 review フローの自動化・統合が進む。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [HTTP2 enforcement 適合仕様](../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- [Bidi 適合仕様](../../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)

**関連 OSS**:
- OpenTelemetry .NET: OTel SDK for .NET Framework 4.8（制限あり。バージョン固定管理が必要）
- CLR Profiler API: .NET Framework のプロセスレベル計装 API
- Testcontainers / Windows containers: .NET Framework 4.8 テスト環境
- Harbor: NuGet proxy として動作する内部 registry

## 期待結果 / 観測指標

- .NET Framework 4.8 テスト環境で動作確認が完了している
- 役割 A: OTel span が Collector に到達し、認証コンテキストが downstream に伝播している
- 役割 B: bidi RPC と transport 縮退が期待通りに動作している
- `k1s0.Library.NetFx` NuGet が Harbor mirror に push され cosign 署名が存在する
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている

## 失敗時の挙動 / escalation

- **CLR Profiler attach 失敗**: .NET Framework 4.8 の CLR バージョンと Profiler の互換性を確認する。Windows コンテナのベースイメージバージョンを調整する。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に修正案）。Backstage runbook `companion-clr-profiler-failure` を参照。
- **HTTP/2 非対応環境での transport ネゴシエーション失敗**: HTTP/1.1 + SSE への縮退パスが正常に動作することを確認する。縮退パスも失敗する場合は long polling を最終縮退として有効化する。escalate 先: tier1 担当者 dual reviewer（SLA: 8 時間以内に縮退パス確認）。
- **per-tab 6 subscription 上限超過**: subscription を multiplexing する実装に変更し、単一の SSE 接続で複数 subscription を処理する設計に修正する。escalate 先: tier1 担当者 dual reviewer（SLA: 48 時間以内に設計修正）。
- **NuGet push 失敗（Harbor mirror）**: Harbor の NuGet proxy 設定を確認し、認証トークンが有効であることを検証する。escalate 先: ops 担当者へ Mattermost `#tier1-incident` で連絡（SLA: 4 時間以内に復旧）。Backstage runbook `harbor-nuget-push-failure` を参照。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [Transport 切替 bidi 移行](07_Transport切替_bidi移行.md)
