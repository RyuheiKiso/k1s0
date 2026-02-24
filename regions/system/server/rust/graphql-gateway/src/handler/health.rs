// handler::health は以前プレースホルダーの healthz/readyz を含んでいたが、
// 実際のヘルスチェックは adapter::graphql_handler 内で実装されているため削除。
//
// 実際のヘルスチェック: adapter::graphql_handler::healthz, readyz
