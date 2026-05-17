---
id: detail.cross_http2.http2_enforcement
axis: cross_http2
phase: cross_cutting
kind: cross_cut_spec
status: draft
depends_on:
  - arch.tier1.server_systems
  - detail.tier1.bidi_conformance
  - detail.tier1.tenant_capacity_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, D, E]
  proof_classes:
    - v1_program_correctness_proof
related_axes:
  - tier1
  - tier3
  - client
  - infra
---

# tier1 ingress HTTP/2 強制（v1）

## 位置づけ
- tier3/16_リアルタイム更新 UX / tier3/04_アプリケーション形態 / tier3/36_レガシー資産統合 / 01_Server 系 と双方向 lock
- per-tab 同時 subscription 上限 16 を物理的に達成するために、tier1 ingress を HTTP/2 強制（HTTP/1.1 ingress を v1 から廃止）し、Companion は ALPN h2 必須を NuGet で強制する

## 不可避性
- HTTP/1.1 は同一 origin 同時 6 接続が RFC / 主要ブラウザ実装の固定上限。HTTP/1.1 + SSE で 16 stream を per-tab に張ることは物理的に不可能
- HTTP/2 multiplexing（`max_concurrent_streams`、Envoy 既定 100）で 16 streams を single connection に乗せれば per-tab 16 subscription が成立
- 結果、tier1 ingress を HTTP/2 強制することが per-tab 16 subscription の唯一の物理的解

## 役割

### 役割 H2-A: 業務 ingress HTTP/2 強制
- Envoy Gateway の **業務 listener（port 443 / HTTP/3 UDP 443）** は HTTP/2 + HTTP/3 のみ accept、HTTP/1.1 は廃止する
- HTTP/2 prior-knowledge は accept しない（ALPN ネゴシエーション h2 必須、TLS ALPN なしの client は cleartext h2c も accept しない）
- 結果、業務 listener に到達する全 client は TLS + ALPN h2 が前提

### 役割 H2-A': v1_legacy_http11 専用 listener
- 業務 listener とは **物理的に別ポート（例: TLS port 8443-legacy）** に v1_legacy_http11 専用 listener を分離して残す。当該 listener は TLS + HTTP/1.1 のみ accept、ALPN は h2 を offer しない
- L7 firewall（Envoy access control + Network Policy）で当該ポートに到達できるのは、`capability_matrix.lock.yaml` で `v1_legacy_http11` cell に entry を持つ tenant の Companion / 旧 proxy 経由経路のみ
- 業務 SLO（`v1_request_latency_p99_ms` / per-tab 16 subscription 等）は当該 listener に対しては別系統で管理し、`v1_legacy_http11_request_latency_p99_ms` を独立 SLO として 17_運用ループ適合仕様 に登録
- 当該 listener は v1 では仕様内、v2 で廃止候補（業界 OS の WinHTTP h2 普及率 trigger）として 08_OSS ライフサイクル適合仕様 の `deprecation_clock` に乗せる

### 役割 H2-B: Companion ALPN h2 強制
- .NET Framework Companion（`k1s0.Companion.NetFx`）は WinHttpHandler を必須依存として NuGet 配布、WinHttpHandler の H2 サポートを利用して ALPN h2 をネゴシエート
- WinHttpHandler の HTTP/2 ALPN は「.NET Framework 4.6.2 以上」だけでは決まらず、ホスト OS 側の WinHTTP API が h2 を実装している版数（**Windows 10 version 1607 以降 / Windows Server 2016 以降**）が合わさって初めて成立
- Companion 配布時の prerequisite は「**.NET Framework 4.6.2+ かつ Windows 10 1607+ / Server 2016+ かつ WinHttpHandler 必須**」を 3 条件の合成として明示
- prerequisite check は Companion 起動時に `Environment.OSVersion`（10.0.14393 = Windows 10 1607 の build number 14393）を probe し、不足端末は `v1_legacy_http11` へ起動時 fall back（H2-C）。Backstage onboarding workflow の prerequisite step として OS 版数 check を H2 reachability check と並列必須化

### 役割 H2-C: v1_legacy_http11 capability_class
- h2 prior-knowledge / ALPN h2 が不可な環境向けに `v1_legacy_http11` を別 capability_class として維持
- 該当する物理条件:
  - 企業 FW / DLP / TLS インスペクション appliance が ALPN h2 を遮断
  - Windows 7 / 8.1 / Server 2008R2 / Server 2012 等、ホスト OS の WinHTTP が h2 を持たない版数
  - .NET Framework 4.6.1 以前
- `v1_legacy_http11` の per-tab subscription 上限は 6（HTTP/1.1 の物理上限）と honest に明示、機能は同等だが connection 数縛りで縮退
- 該当環境の tier3 利用者は `capability_matrix.lock.yaml` で `v1_legacy_http11` cell に明示 entry を持ち、SLO が degrade する旨を機械可読化

## HTTP/3（h3）併設
- HTTP/2 強制 = h3 廃止ではない。h3 over QUIC は併設し、UDP 出口を持つ環境では h3 を優先（Envoy Gateway HTTP/3 listener で同一 backend に proxy）
- h3 不可（UDP block）環境は h2 へ自動 fall back（Alt-Svc / HTTPS DNS RR）

## per-tab 16 の機械的成立
- tier3/16_リアルタイム更新 UX の「per-tab 同時 subscription 16」は HTTP/2 single connection 上の stream multiplex 16 までとして明示
- SSE / Connect-RPC server-streaming / `paired_post_sse` はすべて HTTP/2 stream として multiplex され、per-tab 16 までは single TCP connection で完結
- `max_concurrent_streams` は Envoy 設定で 100、保守 buffer を持って 16 まで業務利用、残りは内部 control / health probe 用

## 副作用
- h2 prior-knowledge / ALPN h2 不可な企業 FW 環境は、業務 listener に接続できず、**役割 H2-A' の `v1_legacy_http11` 専用 listener（別ポート 8443-legacy）に明示的に向け直す経路** を経由する以外、業務疎通不能。fall back 先 listener は物理的に分離して常設するため listener 不在で接続不能になる従来の不整合は発生しない
- .NET Framework 4.6.1 以下、または OS 版数が Windows 10 1607 / Server 2016 未満の端末は `v1_legacy_http11` にのみ fall back（4 stack 観測経路は server 側 Collector 補完に依存）
- 既存社内 proxy / DLP / TLS インスペクション appliance の H2 互換性確認、および Companion 利用端末の OS 版数確認が tier3 導入前の prerequisite

## 整合
- 整合 1: tier3/16_リアルタイム更新 UX の per-tab 16 を HTTP/2 multiplex 上での 16 streams に明示
- 整合 2: tier3/04_アプリケーション形態 の HTTP/1.1 + SSE 記述は `v1_legacy_http11` capability_class への参照に置換、primary は HTTP/2 + SSE multiplex
- 整合 3: tier3/36_レガシー資産統合 の Companion 経路は WinHttpHandler 必須化 + ALPN h2 必須化を明示
- 整合 4: 12_client/15_クライアント SDK 配布適合仕様 `capability_matrix.lock.yaml` に `v1_legacy_http11` cell を追加、当該 cell の per-tab 上限 6 を明示
- 整合 5: 11_ops/15_運用ループ適合仕様 SLO に `ingress_http_version_distribution` を追加（h3 / h2 / `v1_legacy_http11` の比率を可視化）。さらに listener 別 SLO として `v1_legacy_http11_request_latency_p99_ms`（別ポート listener 個別計測）を独立登録、業務 listener と混合計測しない
- 整合 6: 12_client/15 `capability_matrix.lock.yaml` の `v1_legacy_http11` cell に「経路: 別ポート listener 8443-legacy、TLS + HTTP/1.1、ALPN h2 offer なし、L7 firewall で tenant entry 必要」を機械可読 entry として明示
- 整合 7: 08_OSS ライフサイクル適合仕様 の `deprecation_clock` に「`v1_legacy_http11` listener は v2 で廃止候補（業界 OS の WinHTTP h2 普及率 trigger）」を登録

## 関連参照
- [Server 系](../../03_概要設計/02_tier1設計方針/01_Server系.md)
- [Bidi 適合仕様](../01_適合仕様/01_Bidi適合仕様.md)
- [テナント容量適合仕様](../01_適合仕様/09_テナント容量適合仕様.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
