---
id: plan.tier3.scenario_legacy_netfx_companion
axis: tier3
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
---

# レガシー .NET Framework Companion 統合

## 一文方針

.NET Framework 4.8 ERP 等のレガシーシステムを `k1s0.Library.NetFx` NuGet と Companion Transport Negotiation Runtime を通じて k1s0 に統合し、観測コンテキスト伝播と認証を確立する。

> 朝 10 時、本社 IT 室の tier3 担当者（ジュニア級）が GitHub PR の integration test 結果で CLR Profiler attach の失敗ログに気付く。手元には `k1s0.Library.NetFx` の NuGet 設定と Testcontainers の出力、Mattermost 越しに tier1 Companion 担当者と infra 担当者がいる。

## ペルソナ要約

主役: tier3 担当者（ジュニア級）、目的: .NET Framework レガシーシステムとの Companion 統合を HTTP/2 + validation 規約で安全に実装する

## 現状業務での痛み

- .NET Framework の HTTP/1.1 のみ対応で、k1s0 の HTTP/2 / gRPC エンドポイントと直接通信できない
- 入力バリデーションが未対応で、レガシー側からの不正リクエストがそのまま tier2 に到達するリスクがある
- Companion 層のエラーハンドリングが undefined で、レガシー側の障害が k1s0 全体に伝播する

## k1s0 でこう変わる

- BFF Companion adapter が HTTP/1.1 → HTTP/2 変換を担い .NET Framework との透過的な通信を実現する
- Companion adapter の入力バリデーションレイヤーがレガシー側の不正リクエストを contract test で検知する
- circuit breaker が Companion 層に組み込まれ、レガシー障害を k1s0 から隔離して全体影響を防ぐ

## Trigger（発火条件）

.NET Framework 4.8 ERP 等のレガシーシステムを k1s0 に統合する tier3 実装が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（新レガシーシステム統合時）。典型きっかけ: 「.NET Framework 4.8 で動く既存 ERP が k1s0 に連携し、設備リモート操作 API を tier2 経由で呼び出す必要が生じた」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier1 Companion 担当者 / infra 担当者（8443-legacy ポート設定）/ dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier3）| ジュニア | 本社 IT 室 / リモート | GitHub PR / Jest CI | NetFx NuGet 組込 / CLR Profiler 設定 / integration test / PR 提出 |
| 関与（tier1 Companion）| シニア | 本社 IT 室 | Backstage Catalog | NuGet バージョン管理 / Transport Negotiation Runtime 仕様確認 / sign-off |
| 関与（infra）| 中堅 | 本社 IT 室 | Backstage Catalog | 8443-legacy ポート設定 / Keycloak 設定確認 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 IT 室 / リモート | GitHub PR | integration test green / OTel trace 到達 / OIDC 認証確認 / sign-off |

## 個人 KPI / 達成感

- Companion 経由の全リクエストが contract test green で通過することを定量確認できる
- circuit breaker が正しく作動することを E2E のフォールトインジェクションで確認し実装完了感を得られる

## 工数 / 関与人数 / コスト感

- 工数: 2〜3 日（adapter 設定 4h + validation 実装 4h + circuit breaker 設定 2h + E2E 4h）
- 関与人数: 3 名（tier3・tier2・dual reviewer）
- コスト感: 中〜高。レガシー環境の調査コストが追加されるが adapter が変換を集約する

## 前提

- tier1 Companion（役割 A / B）が稼働中
- 8443-legacy ポートで HTTP/1.1 + SSE 専用 listener（v1_legacy_http11）が infra 側で設定済み
- Keycloak OIDC が稼働中
- （責務境界: [README 重要原則](./README.md#重要原則-tier3-が所有するもの--所有しないもの) を参照）

## 流れ

1. `k1s0.Library.NetFx` NuGet を .NET Framework 4.8 プロジェクトに組込み
2. CLR Profiler attach（`k1s0.Companion.NetFx.OTelExt`）で観測コンテキストを伝播
3. transport: HTTP/1.1 + SSE（port 8443-legacy）で接続し per-tab 6 subscription（縮退モード）
4. bidi が必要な場合は [Tauri Companion sidecar](../../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md) Transport Negotiation Runtime 経由で等価実装を使用
5. レガシー側の認証: Keycloak OIDC token を .NET Framework 側で取得し HTTP header に付与
6. integration test: Testcontainers + .NET Framework 4.8 runtime で動作確認
7. dual reviewer sign-off

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier3 担当者 | BFF Companion adapter の設定ファイルを作成し .NET Framework エンドポイントを登録 | `adapter 設定完了 / HTTP/1.1 endpoint 登録` |
| 4h | tier3 担当者 | 入力バリデーションと circuit breaker を Companion adapter に設定 | `validation 設定完了 / circuit breaker 追加` |
| 1d | tier3 担当者 | E2E で HTTP/2 変換・バリデーション・circuit breaker を green にし PR 提出 | `Companion E2E green / PR #NNN 提出` |
| 1d+4h | dual reviewer | adapter 動作と contract test を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

- **FA 生産指示・設備操作**: .NET Framework 4.8 ERP からの設備リモート操作 API 呼び出しが本シナリオの典型ユースケースであり、既存 FA システムとの連携基盤を確立する
- **受注**: レガシー ERP の受注データを k1s0 の受注業務フローに統合するための Companion 接続が本シナリオで実現される

## 関連適合仕様 / 関連 OSS

- Tauri_companion_sidecar: [../../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md](../../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md)
- BFF_auth_edge: [../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md)
- 関連 OSS: Testcontainers / OpenTelemetry .NET / Keycloak

## 期待結果 / 観測指標

- .NET Framework 4.8 プロジェクトから k1s0 API への疎通確認（integration test green）
- OTel trace が k1s0 の observability stack に到達している
- per-tab 6 subscription 縮退モードで動作
- Keycloak OIDC token によるアクセス制御が機能している
- dual reviewer 2 名 sign-off 完了

## 失敗時の挙動 / escalation

- `k1s0.Library.NetFx` NuGet バージョン不整合 → **escalate 先**: tier1 Companion 担当者に Mattermost `#tier1-companion` で確認（**SLA**: 24h 以内）。**runbook**: Backstage runbook `netfx-nuget-compat` を参照
- CLR Profiler attach 失敗 → **escalate 先**: infra 担当者に Mattermost `#infra-ops` で runtime 環境確認を依頼（**SLA**: 24h 以内）
- 8443-legacy ポート疎通失敗 → **escalate 先**: infra 担当者に Mattermost `#infra-ops` で listener 設定確認を依頼（**SLA**: 24h 以内）
- OIDC token 取得失敗 → **escalate 先**: tier2 / infra 担当者に Mattermost `#infra-ops` で Keycloak 設定確認（**SLA**: 24h 以内）

## 失敗パターン (anti-pattern)

- HTTP/1.1 直接呼び出し: Companion adapter を迂回して .NET Framework と直接通信すると contract test が boundary 違反を検知する
- バリデーション省略: Companion 層でのバリデーションなしにレガシーデータを tier2 に渡すと schema validation CI が fail する

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と担当者プロフィール
- [07_WebAuthn_step_up_federation.md](./07_WebAuthn_step_up_federation.md) — レガシー統合時に必要な Keycloak OIDC / step_up 連携の認証フロー実装
- [Tauri companion sidecar](../../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md) — Transport Negotiation Runtime・bidi 等価実装の仕様定義。本シナリオの .NET Framework 接続方式の SoT
- [BFF_auth_edge](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md) — .NET Framework 4.8 側への short-lived token リレー設計と認証境界の定義
