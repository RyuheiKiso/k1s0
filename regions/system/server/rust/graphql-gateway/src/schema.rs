// schema.rs は以前プレースホルダーの QueryRoot/MutationRoot/build_schema() を含んでいたが、
// 実際の GraphQL スキーマは adapter::graphql_handler で構築されているため削除。
// このファイルは後方互換性のために残されている。
//
// 実際のスキーマ定義: adapter::graphql_handler::AppSchema
