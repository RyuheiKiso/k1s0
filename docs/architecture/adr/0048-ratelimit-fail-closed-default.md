# ADR-0048: ratelimit fail_open デフォルト false 化（fail-closed）

## ステータス
承認済み

## コンテキスト
`regions/system/server/rust/ratelimit/src/usecase/check_rate_limit.rs` の `CheckRateLimitUseCase::new()` で `fail_open: true` がデフォルトとして設定されていた。Redis 障害発生時に全リクエストがレートリミットなしで通過する（fail-open）ため、Redis 障害が DDoS 攻撃のバッファになるリスクがある。

Kong 側は既に `fault_tolerant: false`（fail-closed）で統一済みであり、ratelimit サービスのデフォルトも合わせる必要がある。

## 決定
`CheckRateLimitUseCase::new()` のデフォルトを `fail_open: false` に変更する。Redis 障害時は明示的に `with_fallback_policy(fail_open: true)` を使用するケースのみ fail-open とする。

## 理由
- セキュリティのデフォルト設定は「安全側」（fail-closed）であるべき
- Kong の `fault_tolerant: false` との一貫性
- Redis 障害時のリクエスト通過はセキュリティリスク

## 影響

**ポジティブな影響**:
- Redis 障害時にレートリミットが機能しなくなるリスクが解消される
- Kong の設定との一貫性が保たれる

**ネガティブな影響・トレードオフ**:
- Redis 障害時にレートリミット超過エラー（429）が増加する可能性がある
- アプリケーション層でフォールバックが必要な場合は `with_fallback_policy` で明示的に指定が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| fail-open のまま | 現状維持 | Redis 障害時のセキュリティリスクが残存する |
| 設定ファイルで制御 | `config.yaml` のデフォルト値を変更 | コードレベルでの安全なデフォルトが優先される |

## 参考
- [ADR-0041](./0041-ratelimit-api-path-alignment.md) - ratelimit API パス整合
- `infra/kong/kong.yaml` - Kong fault_tolerant: false 設定

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（外部監査 HIGH-11 対応） | k1s0-team |
