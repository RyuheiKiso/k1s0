// k1s0 tier2 Go モジュール: テナントコンテキスト（4 GUC 注入）の Go 実装
module github.com/k1s0/tier2

// Go バージョン: 1.23 を使用する
go 1.23.0

// ツールチェーンバージョンを固定する
toolchain go1.23.4

// 直接依存関係
// github.com/google/uuid: UUID 生成（tenant_id / actor_id に使用する）
require github.com/google/uuid v1.6.0
