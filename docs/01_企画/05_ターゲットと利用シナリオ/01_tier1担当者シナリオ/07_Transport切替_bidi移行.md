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
  defense_in_depth_layers: []
  proof_classes: []
---

# Transport 切替 / bidi 移行

## 一文方針

HTTP/2 → HTTP/3 / WebTransport の standard 化追従または bidi semantics の transport 変更において、Transport Adapter Layer への新 handler 並走追加と Testcontainers conformance test による bidi 等価性検証を経て、年次 dry-run 実績を lock ファイルに記録してから dual reviewer sign-off を取得する。

## Trigger（発火条件）

HTTP/2 → HTTP/3 / WebTransport の standard 化追従または bidi semantics の transport 変更が必要になった時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 年次〜不定期（標準化に追従）
- 典型きっかけ: 「IETF が WebTransport RFC を確定し tier1 が追従タイミングを評価する必要が生じた」「HTTP/3 QUIC が主要 CDN で標準化され Envoy Gateway 対応が要求された」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ infra 軸担当者（network policy 変更連携）/ ops 軸担当者（SLO への影響評価）

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

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [Companion 役割 AB 拡張](06_Companion役割AB拡張.md)
