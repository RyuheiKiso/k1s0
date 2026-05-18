// scenarios モジュール定義
// 4 つの migration pair シナリオを個別モジュールとして公開する

// PostgreSQL CNPG → StackGres の移行シナリオモジュール
pub mod relational_pg;
// Kafka Strimzi → RedPanda の移行シナリオモジュール
pub mod messaging_kafka;
// Workflow Temporal → 自製エンジンの移行シナリオモジュール
pub mod workflow;
// Rule Engine zen_rule → internal_rule の移行シナリオモジュール
pub mod rule_engine;
