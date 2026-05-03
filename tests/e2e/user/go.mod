// tests/e2e/user module — user suite (16GB host OK、ADR-TEST-008)
//
// 本 module は owner suite (tests/e2e/owner) と物理分離されており、
// kind 起動 + minimum stack + tier1 facade / SDK round-trip を検証する。
// 利用者は test-fixtures (src/sdk/<lang>/test-fixtures、ADR-TEST-010) 経由で
// 同等の経路を自アプリ repo で再現できる。

module github.com/k1s0/k1s0/tests/e2e/user

go 1.23
