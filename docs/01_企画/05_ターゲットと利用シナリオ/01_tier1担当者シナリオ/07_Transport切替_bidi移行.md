---
id: plan.tier1.scenario_transport_migration
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

# Transport 切替 / bidi 移行

## 一文方針

HTTP/2 → HTTP/3 / WebTransport の standard 化追従または bidi semantics の transport 変更において、Transport Adapter Layer への新 handler 並走追加と Testcontainers conformance test による bidi 等価性検証を経て、年次 dry-run 実績を lock ファイルに記録してから dual reviewer sign-off を取得する。

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が buf CI ダッシュボードを確認し、「IETF が WebTransport RFC を確定し tier1 が追従タイミングを評価する必要がある」という設計ディスカッションが上がっているのに気付く。手元には `transport_migration.lock.yaml` と Testcontainers CI ログ、Mattermost 越しに dual reviewer 2 名・infra 軸担当者・ops 軸担当者がいる。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: HTTP/2→HTTP/3/WebTransport 移行で全 client SDK を無改修にしながら bidi semantics の等価性を年次 dry-run で物理証明する

## 現状業務での痛み

- HTTP/2 から HTTP/3 移行時に全 client SDK を手動改修する必要があり、4 言語分の工数が膨大になる
- WebTransport 対応が都度の特殊実装になり、transport 切替のたびに設計が積み重なって複雑化する
- 年次 dry-run 未実施のまま移行コミットメントの実現可能性が確認されず、EOL 時に「動かない」が判明する
- bidi semantics（全 4 種）の等価性確認が手動になり、一部パターンで不等価が本番稼働後に発覚する

## k1s0 でこう変わる

- Transport Adapter Layer への新 handler 並走追加で、既存 client SDK への変更なしに新 transport を段階的に導入する
- bidi 二層化（Application semantic / Transport adapter）で transport 非依存の client SDK を実現する
- Testcontainers conformance test が全 4 semantics（unary/server streaming/client streaming/bidi）の等価性を物理証明する
- 年次 dry-run を `transport_migration.lock.yaml` に記録することで、移行コミットメントの実現可能性を毎年客観的に担保する

## Trigger（発火条件）

HTTP/2 → HTTP/3 / WebTransport の standard 化追従または bidi semantics の transport 変更が必要になった時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 年次〜不定期（標準化に追従）
- 典型きっかけ: 「IETF が WebTransport RFC を確定し tier1 が追従タイミングを評価する必要が生じた」「HTTP/3 QUIC が主要 CDN で標準化され Envoy Gateway 対応が要求された」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ infra 軸担当者（network policy 変更連携）/ ops 軸担当者（SLO への影響評価）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | buf CI / Testcontainers CI | 移行計画確認・bidi 等価性検証・新 handler 追加・dry-run 実施 |
| 関与（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | Transport Adapter Layer 変更レビュー・sign-off |
| 関与（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | Transport Adapter Layer 変更レビュー・sign-off |
| 関与（infra 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | network policy 変更・ALPN 設定レビュー |
| 関与（ops 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | SLO 影響評価・Grafana レイテンシ監視 |

## 個人 KPI / 達成感

- Testcontainers conformance test 全 4 semantics green（新 transport）
- 年次 dry-run green が `transport_migration.lock.yaml` に記録済み
- HTTP2 enforcement 適合仕様との整合確認完了（HTTP/2 継続維持）
- SLO 劣化なし（レイテンシ・エラー率の regression 0）

## 工数 / 関与人数 / コスト感

- 初回（新 transport handler 追加）: 3〜5 日（bidi 等価性検証・handler 追加・dry-run）、関与 5 名（主役 + dual reviewer 2 名 + infra 担当者 + ops 担当者）
- 平常（年次 dry-run のみ）: 0.5〜1 日、関与 3 名（主役 + dual reviewer 2 名）
- 失敗時（bidi 等価性 fail・SLO 劣化）: +1〜2 日、関与 5〜6 名

## 前提

- transport 中立性は「bidi semantics 深耕 + transport 移行コミットメント」で実現する方針（[tier1 担当者シナリオ index](README.md) 参照）
- 現行 transport は Connect-RPC（L1+ 単一深耕）で HTTP/2 を基盤とする
- [Transport Adapter Layer](../../../03_概要設計/02_tier1設計方針/01_Server系.md) が既存の transport handler を抽象化している
- `transport_migration.lock.yaml` が移行計画と dry-run 実績の管理ファイルとして存在する
- [HTTP2 enforcement 適合仕様](../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md) が有効

## 流れ

1. **現行移行コミットメント計画の確認**: `transport_migration.lock.yaml` に記録されている Connect-RPC（HTTP/2 L1+）の移行コミットメント計画を確認する。新 transport 追加が既存計画と整合しているか検証する。HTTP/3 / WebTransport 対応は 1.0.0 範囲内でのスコープ追加か、次世代バージョンの先行実装かを明確にする。

2. **bidi semantics の等価性検証**: 新 transport が既存の bidi semantics を完全に再現できることを Testcontainers conformance test で検証する。
   - unary RPC: リクエスト 1 件 → レスポンス 1 件
   - server streaming: リクエスト 1 件 → レスポンスストリーム
   - client streaming: リクエストストリーム → レスポンス 1 件
   - bidi streaming: リクエストストリーム → レスポンスストリーム（全二重）
   - transport 固有の制限（例: HTTP/1.1 では bidi streaming 不可、SSE では server-only streaming のみ）を明示

3. **Transport Adapter Layer への新 handler 追加**: 既存 handler と並走する形で新 transport handler を Transport Adapter Layer に追加する。
   - 既存 HTTP/2 handler は継続稼働（並走）
   - 新 transport（HTTP/3 / WebTransport 等）の handler を追加
   - handler 選択ロジック: ALPN（TCP+TLS 上のプロトコルネゴシエーション）で HTTP/2 / HTTP/3 を識別し、Alt-Svc（HTTP ヘッダで代替エンドポイントを通知する仕組み、ALPN とは別レイヤ）で HTTP/3 対応 endpoint をクライアントに広告して自動選択を促す
   - feature flag で新 transport を段階的に有効化できるようにする

4. **HTTP2 enforcement spec との整合確認**: [HTTP2 enforcement 適合仕様](../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md) との整合を確認する。HTTP/3 対応後も 1.0.0 範囲内では HTTP/2 enforcement を維持し、HTTP/3 はオプショナルな追加 transport として扱う。HTTP/1.1 fallback は Companion 役割 B の範囲のみに限定する。

5. **年次 dry-run の実施と記録**: 新 transport での bidi streaming full round-trip を dry-run 環境で実施し、`transport_migration.lock.yaml` に green として記録する。dry-run には以下を含める。
   - 全 4 semantics（unary / server streaming / client streaming / bidi streaming）の疎通確認
   - 旧 transport から新 transport への切替（feature flag ON/OFF）の往復確認
   - SLO への影響（レイテンシ・エラー率の変化）の計測

6. **dual reviewer sign-off と CI green**: dual reviewer（tier1 2 名）の sign-off を取得する。全言語の CI が green であることを確認する。

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | 移行コミットメント計画確認・新 transport スコープ確定 | `WebTransport RFC 確定 / 1.0.0 スコープ外: 先行評価として追加` |
| 1〜2 日 | tier1 担当者 | bidi 等価性検証（Testcontainers 4 semantics）| `全 4 semantics conformance test: green` |
| 2〜3 日 | tier1 担当者 | Transport Adapter Layer 新 handler 追加・HTTP2 enforcement 整合確認 | `新 handler 追加完了 / HTTP/2 継続維持確認` |
| 4 日 | tier1 担当者 / ops 担当者 | 年次 dry-run 実施・SLO 影響評価 | `dry-run green / SLO: レイテンシ+3% 以内` |
| 5 日 | dual reviewer A/B | sign-off・CI green 確認 | `dual sign-off 完了 / lock.yaml 更新済` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA テレメトリ**: bidi streaming transport の変更は SCADA 設備からのリアルタイムテレメトリ収集チャンネルに直結し、transport 移行中の全二重通信断絶が設備監視の空白時間を生む可能性がある。
- **警報配信**: 警報は全二重（bidi）の低レイテンシ配信を必要とするため、新 transport の bidi semantics 等価性が不完全な場合に警報の到達遅延・欠落が発生するリスクがある。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [HTTP2 enforcement 適合仕様](../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- [Bidi 適合仕様](../../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)

**関連 OSS**:
- Connect-RPC: 現行 L1+ 単一深耕 transport（HTTP/2 基盤）
- tonic（Rust）: gRPC / HTTP/2 実装
- quinn（Rust）: QUIC / HTTP/3 実装候補
- Testcontainers: bidi semantics conformance test の実行基盤

## 期待結果 / 観測指標

- Testcontainers conformance test が全 4 semantics で新 transport に対して green
- Transport Adapter Layer に新 handler が追加され既存 handler と並走稼働している
- HTTP2 enforcement 適合仕様との整合が確認されている（HTTP/2 継続維持）
- `transport_migration.lock.yaml` に年次 dry-run green が記録されている
- feature flag による段階的有効化が機能している
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている

## 失敗時の挙動 / escalation

- **bidi semantics 等価性 fail（新 transport）**: 新 transport の bidi 実装を修正するか、対応 semantics の制限を明示した上でサポート範囲を縮小する。縮小の場合は [シナリオ 06](06_Companion役割AB拡張.md) の Companion 役割 B で補完する設計を検討する。escalate 先: tier1 担当者 dual reviewer（SLA: 48 時間以内に方針決定）。Backstage runbook `transport-bidi-equivalence-failure` を参照。
- **HTTP2 enforcement 違反検出**: HTTP/3 専用の enforcement が HTTP/2 enforcement を上書きしていないか確認する。両者は並立する。escalate 先: tier1 担当者 dual reviewer（SLA: 4 時間以内に修正）。
- **年次 dry-run fail**: dry-run 失敗原因（ネットワーク設定 / TLS / ALPN ネゴシエーション）を特定し修正する。infra 軸と連携して network policy を確認する。escalate 先: infra 担当者へ Mattermost `#tier1-incident` で連絡（SLA: 72 時間以内に dry-run 再実施）。Backstage runbook `transport-dryrun-failure` を参照。
- **SLO 劣化（新 transport）**: 新 transport の有効化を feature flag で停止し、レイテンシ・エラー率の regression 原因を特定する。escalate 先: ops 担当者（SLA: 1 時間以内に feature flag 停止）。

## 失敗パターン (anti-pattern)

- **bidi semantics の確認を手動 + 目視**: 全 4 種のうち client streaming が確認漏れになり、本番で初めて動かないことが判明する。Testcontainers conformance test による全 4 semantics の自動確認を merge 必須条件とする。
- **HTTP/3 専用の enforcement を追加して HTTP/2 enforcement を上書き**: 1.0.0 の HTTP/2 継続要件が破れ、HTTP/2 しか対応していない環境での接続が拒否され始める。両 enforcement が並立することを CI で検証する。
- **dry-run なしで本番移行計画を立案**: 年次 dry-run をスキップしたまま EOL を迎え、実際に移行を始めると toolchain が動かないことが判明する。dry-run 未実施を 1.0.0 ship blocker と同等の重大度として扱い、Backstage でアラートを発火させる。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [Companion 役割 AB 拡張](06_Companion役割AB拡張.md)
