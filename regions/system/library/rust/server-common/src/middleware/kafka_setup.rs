use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub group_id: Option<String>,
    pub client_id: Option<String>,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: vec!["localhost:9092".to_string()],
            group_id: None,
            client_id: None,
        }
    }
}

pub struct KafkaSetup {
    config: KafkaConfig,
}

impl KafkaSetup {
    pub fn new(config: KafkaConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &KafkaConfig {
        &self.config
    }
}
