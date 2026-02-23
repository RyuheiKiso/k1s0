import Testing
@testable import K1s0Kafka

@Suite("Kafka Tests")
struct KafkaTests {
    @Test("bootstrapServers が正しい形式であること")
    func testBootstrapServers() {
        let config = KafkaConfig(brokers: ["broker1:9092", "broker2:9092"])
        #expect(config.bootstrapServers == "broker1:9092,broker2:9092")
    }

    @Test("単一ブローカーの bootstrapServers")
    func testSingleBrokerBootstrapServers() {
        let config = KafkaConfig(brokers: ["broker1:9092"])
        #expect(config.bootstrapServers == "broker1:9092")
    }

    @Test("TLS 判定が正しいこと")
    func testTLSDetection() {
        let tlsConfig = KafkaConfig(brokers: ["b:9093"], securityProtocol: "SSL")
        let plainConfig = KafkaConfig(brokers: ["b:9092"], securityProtocol: "PLAINTEXT")
        #expect(tlsConfig.usesTLS)
        #expect(!plainConfig.usesTLS)
    }

    @Test("空ブローカーはバリデーションエラー")
    func testEmptyBrokersValidation() {
        let config = KafkaConfig(brokers: [])
        #expect(throws: KafkaError.self) {
            try config.validate()
        }
    }

    @Test("トピック命名規則の検証")
    func testTopicNameValidation() {
        let validTopic = TopicConfig(name: "k1s0.service.orders.order-created.v1")
        let invalidTopic = TopicConfig(name: "invalid")
        #expect(validTopic.validateName())
        #expect(!invalidTopic.validateName())
    }

    @Test("KafkaError の説明が含まれること")
    func testKafkaErrorDescription() {
        let error = KafkaError.connectionFailed("timeout")
        #expect(error.description.contains("CONNECTION_FAILED"))
    }

    @Test("ヘルスチェックが設定エラーを検出すること")
    func testHealthCheckerConfigError() async {
        let config = KafkaConfig(brokers: [])
        let checker = KafkaHealthChecker(config: config)
        let status = await checker.check()
        if case .healthy = status {
            Issue.record("空ブローカー設定はunhealthyであるべき")
        }
    }
}
