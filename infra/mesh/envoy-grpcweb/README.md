# infra/mesh/envoy-grpcweb — gRPC-Web 翻訳プロキシ

tier3 web (TypeScript / browser) は **gRPC-Web protocol** で tier1 facade を呼び出す
（直接 gRPC HTTP/2 trailer は browser fetch では扱えないため）。本ディレクトリの envoy
は tier1 facade-state の gRPC エンドポイント (50001) を gRPC-Web (8080) に translate する。

## 設計位置づけ

- **ADR-MESH-001 (Istio Ambient)** の中で、tier3 web からの gRPC-Web ↔ gRPC は ingress
  gateway / sidecar が担う想定だが、リリース時点 ambient mesh では grpc-web filter
  は組み込まれない。tier3 web 経路では明示的な envoy（または ambient mesh の
  gateway envoy）を front に置く必要がある。
- 本マニフェストはローカル kind 環境向けの最小構成。production では Istio Ambient
  Gateway の grpc-web filter 設定に置き換える。

## 検証

- 2026-04-30 実 K8s（kind）にて TypeScript SDK (`@k1s0/sdk-rpc`) から
  `K1s0Client.state.save / get` を呼び出し、envoy → tier1-facade-state の経路で
  Postgres backed state にラウンドトリップを確認済（`hello-from-ts-sdk` の round-trip）。
