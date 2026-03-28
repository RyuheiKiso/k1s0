pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod proto;
pub mod usecase;

/// テスト用インメモリリポジトリとヘルパー（統合テストから利用）
/// リリースバイナリへの混入を防ぐため test/test-utils フィーチャー時のみコンパイルする（L-07 監査対応）
/// tests/ 配下の統合テストから利用する場合は test-utils フィーチャーを有効化すること
#[cfg(any(test, feature = "test-utils"))]
pub mod test_support;
