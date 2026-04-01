---
タイトル: BFF プロキシ SSRF 防御のホワイトリスト方式への移行
ステータス: 承認済み
作成日: 2026-04-01
---

## 背景

BFF プロキシの SSRF 防御（`ssrfSafeDialContext`）は RFC-1918 全域（10.0.0.0/8、172.16.0.0/12、192.168.0.0/16）をブロックしていた。
Docker Compose 環境や Kubernetes クラスター内では、バックエンドサービス（`auth-rust`、`auth-server` 等）がこれらのアドレス帯に解決されるため、BFF からバックエンドへの全リクエストが `502 Bad Gateway` になっていた。

外部監査 CRIT-001（2026-04-01）で指摘を受けた。

## 決定内容

設定ファイル（`config.yaml` / `config.docker.yaml`）の `upstream.base_url` で定義されたホスト名を**許可リスト（allowedHosts）**として扱い、そのホスト宛の接続は RFC-1918 SSRF チェックをスキップする。

具体的な実装内容：

1. `ReverseProxy` 構造体に `allowedHosts map[string]bool` フィールドを追加する。
2. `ssrfSafeDialContext` をグローバル関数からメソッドに変更し、`p.allowedHosts[host]` で許可判定を行う。
3. `NewReverseProxy` に `allowedHosts map[string]bool` パラメータを追加する。
4. `config.BFFConfig` に `AllowedUpstreamHosts()` メソッドを追加し、`upstream.base_url` のホスト名を自動抽出する。
5. `main.go` で `cfg.AllowedUpstreamHosts()` を取得して `NewProxyHandler` に渡す。
6. `NewProxyHandler` のシグネチャに `allowedHosts map[string]bool` パラメータを追加する。

### クラウドメタデータの扱い

`169.254.0.0/16`（クラウドメタデータサービス）は `allowedHosts` に関係なく**常にブロック**する。
専用の `isCloudMetadataIP` 関数で判定し、許可リストのバイパスより先にチェックする。
これにより、仮に設定ファイルのミスでメタデータアドレスが `base_url` に設定されても漏洩を防ぐ。

## 理由

- **根本原因の解消**: BFF は内部サービスへのプロキシが本来の用途であり、全 RFC-1918 ブロックは設計上の矛盾だった。
- **最小権限の許可リスト**: 全ての内部 IP を許可するのではなく、設定ファイルで明示されたホスト名のみを許可することでリスクを最小化する。
- **動的ターゲットの保護継続**: `allowedHosts` にないホストへの接続は引き続き SSRF チェックが適用される。

## 影響

- Docker Compose 環境・Kubernetes クラスターでの BFF → バックエンド通信が正常化する。
- テストスイートが全通過する（`httptest.NewServer` が使う `127.0.0.1` を `allowedHosts` に追加）。
- 動的ターゲットへの SSRF 攻撃は引き続きブロックされる。
- クラウドメタデータ（`169.254.0.0/16`）は常にブロックが維持される。

## 代替案

### 1. SSRF チェックを完全無効化

最もシンプルだが、ユーザー入力由来の動的ターゲットに対する SSRF 防御が失われるため却下。

### 2. 環境変数でホワイトリストを渡す

設定ファイルとの二重管理になり、設定ドリフトのリスクがあるため却下。

### 3. SSRF チェックをループバックと 169.254.0.0/16 のみに限定

実装は単純だが、外部ユーザーが管理する SSRF ターゲット（RFC-1918 への誘導）を防げないため却下。

## 参考資料

- 外部監査報告書 CRIT-001（2026-04-01）
- ADR-0042: BFF プロキシアップストリーム戦略
- `regions/system/server/go/bff-proxy/internal/upstream/reverse_proxy.go`
- `regions/system/server/go/bff-proxy/internal/config/config.go`
