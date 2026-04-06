# ADR-0114: gRPC ポートバインド戦略（Windows Hyper-V 動的排除範囲への対応）

## ステータス

承認済み

## コンテキスト

Windows 上で Docker Desktop を使用する開発環境では、Hyper-V がポート範囲を動的に排除することがある（`netsh int ipv4 show excludedportrange protocol=tcp` で確認可能）。

外部監査（ARCH-001）において、以下の問題が指摘された:

1. `.env.dev` でハードコードされた gRPC ホストポートが Hyper-V の動的排除範囲（50174-50273, 50279-50378 等）と衝突し、6 サービスが起動不能になった（CRIT-002）
2. gRPC サービスをホストポートにバインドすること自体がセキュリティ上の懸念を生む（開発者ローカル環境でも gRPC エンドポイントが 0.0.0.0 に公開される）
3. ポートの長期的安定性を保証できない

## 決定

以下の 2 段階の対策を採用する:

### 短期対策（実施済み）

Hyper-V が動的排除する可能性が低い 50400 帯（50400-50405）に gRPC ホストポートを移動する。

```bash
# .env.dev の現行設定（CRIT-002 対応済み）
EVENT_MONITOR_GRPC_HOST_PORT=50400
MASTER_MAINTENANCE_GRPC_HOST_PORT=50401
NAVIGATION_GRPC_HOST_PORT=50402
POLICY_GRPC_HOST_PORT=50403
RULE_ENGINE_GRPC_HOST_PORT=50404
SESSION_GRPC_HOST_PORT=50405
```

### 長期対策（将来実装）

gRPC サービスのホストポートバインドを廃止し、コンテナ間通信のみで gRPC を使用する。外部からのデバッグアクセスは `kubectl port-forward` または `docker compose exec` を経由する。

```yaml
# 将来の docker-compose.yaml（gRPC ポートをホストに公開しない）
services:
  event-monitor-rust:
    # ports から gRPC のバインドを削除
    # - "${EVENT_MONITOR_GRPC_HOST_PORT:-50400}:50051"  ← 廃止
    expose:
      - "50051"  # コンテナ間通信のみ
```

## 理由

1. **50400 帯の安定性**: Windows の Hyper-V は 50174-50273, 50279-50378 を動的予約するが、50400 帯は現時点で安定して利用可能であることを確認済み（`netsh` コマンドで検証）
2. **長期解決はホストバインド廃止**: ポートを移動しても Hyper-V の排除範囲が将来変わるリスクがある。根本解決はホストへの公開を廃止すること
3. **移行コスト**: 長期対策はローカル開発フローの変更（`kubectl port-forward` の習慣化）が必要であり、段階的に移行する

## 影響

**ポジティブな影響**:
- CRIT-002 が解消し、Windows 開発環境で 6 サービスが起動可能になった
- 長期的にはポート競合リスクが根本解決される

**ネガティブな影響・トレードオフ**:
- 50400 帯も将来的に Hyper-V 排除範囲に含まれる可能性がある（要定期確認）
- 長期対策実施後は gRPC デバッグに `kubectl port-forward` が必要になる

**確認方法**:
```bash
# Hyper-V の現在の排除範囲を確認する
netsh int ipv4 show excludedportrange protocol=tcp
```

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 5 フィールドポート固定 | 排除されにくいポート帯（例: 10000 帯）に移動 | Hyper-V は動的に変更するため根本解決にならない |
| gRPC ホストバインド廃止（即時） | 全 gRPC ポートを今すぐホストから除去する | 既存の開発ワークフロー（Postman/grpcurl でのデバッグ）が壊れる。段階的移行が必要 |
| WSL2 専用ネットワーク使用 | WSL2 で Docker を動かし Hyper-V の影響を回避 | Windows Docker Desktop の設定変更が必要で開発者全員への影響が大きい |

## 参考

- [`docs/infrastructure/devenv/troubleshooting.md`](../../../infrastructure/devenv/troubleshooting.md) — Windows Hyper-V ポート排除確認手順
- [`docs/infrastructure/docker/ポート割り当て.md`](../../../infrastructure/docker/ポート割り当て.md) — gRPC ポート帯の割り当て規則
- ADR-0040 — gRPC ポート帯の初期設定（50200 帯）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-06 | 初版作成（ARCH-001 外部監査対応） | kiso ryuhei |
