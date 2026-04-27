// k1s0 tier 横断 E2E テスト独立 module。
// 本 module は src/tier1/go や src/sdk/go と物理的に独立しており、kind cluster
// 経由でのみ tier1 公開 API を呼ぶ。
module github.com/k1s0/k1s0/tests/e2e

go 1.22
