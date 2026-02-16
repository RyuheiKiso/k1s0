"""メッセージング設計.md の仕様準拠テスト。

Kafka クラスタ、トピック、Schema Registry の設定が
設計ドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
MSG = ROOT / "infra" / "messaging"


class TestKafkaCluster:
    """メッセージング設計.md: Kafka クラスタ設定の検証。"""

    def setup_method(self) -> None:
        path = MSG / "kafka" / "kafka-cluster.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.docs = list(yaml.safe_load_all(self.content))
        self.kafka = self.docs[0]

    def test_kafka_cluster_file_exists(self) -> None:
        assert (MSG / "kafka" / "kafka-cluster.yaml").exists()

    def test_strimzi_api_version(self) -> None:
        """メッセージング設計.md: Strimzi Operator を使用。"""
        assert self.kafka["apiVersion"] == "kafka.strimzi.io/v1beta2"

    def test_kafka_kind(self) -> None:
        assert self.kafka["kind"] == "Kafka"

    def test_kafka_name(self) -> None:
        assert self.kafka["metadata"]["name"] == "k1s0-kafka"

    def test_kafka_namespace(self) -> None:
        assert self.kafka["metadata"]["namespace"] == "messaging"

    def test_kafka_version(self) -> None:
        """メッセージング設計.md: Kafka 3.6.1。"""
        assert self.kafka["spec"]["kafka"]["version"] == "3.6.1"

    def test_kafka_replicas(self) -> None:
        """メッセージング設計.md: Kafka ブローカー 3 台。"""
        assert self.kafka["spec"]["kafka"]["replicas"] == 3

    def test_kafka_listeners(self) -> None:
        """メッセージング設計.md: plain(9092) と tls(9093) リスナー。"""
        listeners = self.kafka["spec"]["kafka"]["listeners"]
        names = [l["name"] for l in listeners]
        assert "plain" in names
        assert "tls" in names

    def test_kafka_replication_factor(self) -> None:
        config = self.kafka["spec"]["kafka"]["config"]
        assert config["default.replication.factor"] == 3

    def test_kafka_min_insync_replicas(self) -> None:
        config = self.kafka["spec"]["kafka"]["config"]
        assert config["min.insync.replicas"] == 2

    def test_kafka_storage(self) -> None:
        """メッセージング設計.md: persistent-claim ストレージ。"""
        storage = self.kafka["spec"]["kafka"]["storage"]
        assert storage["type"] == "persistent-claim"
        assert storage["size"] == "100Gi"
        assert storage["class"] == "ceph-block-fast"

    def test_kafka_metrics(self) -> None:
        """メッセージング設計.md: JMX Prometheus Exporter。"""
        metrics = self.kafka["spec"]["kafka"]["metricsConfig"]
        assert metrics["type"] == "jmxPrometheusExporter"

    def test_zookeeper_replicas(self) -> None:
        assert self.kafka["spec"]["zookeeper"]["replicas"] == 3

    def test_entity_operator(self) -> None:
        assert "topicOperator" in self.kafka["spec"]["entityOperator"]
        assert "userOperator" in self.kafka["spec"]["entityOperator"]


class TestKafkaTopics:
    """メッセージング設計.md: KafkaTopic 定義の検証。"""

    def setup_method(self) -> None:
        path = MSG / "kafka" / "topics.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(self.content) if d]

    def test_topics_file_exists(self) -> None:
        assert (MSG / "kafka" / "topics.yaml").exists()

    @pytest.mark.parametrize(
        "topic_name",
        [
            "k1s0.system.auth.login.v1",
            "k1s0.service.order.created.v1",
            "k1s0.service.order.updated.v1",
            "k1s0.service.inventory.reserved.v1",
            "k1s0.business.accounting.entry.v1",
        ],
    )
    def test_normal_topic_defined(self, topic_name: str) -> None:
        """メッセージング設計.md: 通常トピックが定義されている。"""
        names = [d["metadata"]["name"] for d in self.docs]
        assert topic_name in names, f"Topic '{topic_name}' が定義されていません"

    @pytest.mark.parametrize(
        "dlq_name",
        [
            "k1s0.service.order.created.v1.dlq",
            "k1s0.service.order.updated.v1.dlq",
            "k1s0.service.inventory.reserved.v1.dlq",
            "k1s0.business.accounting.entry.v1.dlq",
            "k1s0.system.auth.login.v1.dlq",
        ],
    )
    def test_dlq_topic_defined(self, dlq_name: str) -> None:
        """メッセージング設計.md: DLQ トピックが定義されている。"""
        names = [d["metadata"]["name"] for d in self.docs]
        assert dlq_name in names, f"DLQ Topic '{dlq_name}' が定義されていません"

    def test_audit_topic_defined(self) -> None:
        """メッセージング設計.md: 監査トピックが定義されている。"""
        names = [d["metadata"]["name"] for d in self.docs]
        assert "k1s0.system.audit.events.v1" in names

    def test_all_topics_use_strimzi(self) -> None:
        for doc in self.docs:
            assert doc["apiVersion"] == "kafka.strimzi.io/v1beta2"
            assert doc["kind"] == "KafkaTopic"

    def test_dlq_retention_30_days(self) -> None:
        """メッセージング設計.md: DLQ の保持期間は 30 日。"""
        for doc in self.docs:
            if doc["metadata"]["name"].endswith(".dlq"):
                assert doc["spec"]["config"]["retention.ms"] == 2592000000

    def test_audit_retention_90_days(self) -> None:
        """メッセージング設計.md: 監査トピックの保持期間は 90 日。"""
        for doc in self.docs:
            if doc["metadata"]["name"] == "k1s0.system.audit.events.v1":
                assert doc["spec"]["config"]["retention.ms"] == 7776000000

    def test_normal_topic_retention_7_days(self) -> None:
        """メッセージング設計.md: 通常トピックの保持期間は 7 日。"""
        for doc in self.docs:
            name = doc["metadata"]["name"]
            if not name.endswith(".dlq") and "audit" not in name:
                assert doc["spec"]["config"]["retention.ms"] == 604800000

    def test_topics_replicas(self) -> None:
        """メッセージング設計.md: 全トピック replicas=3。"""
        for doc in self.docs:
            assert doc["spec"]["replicas"] == 3

    def test_tier_labels(self) -> None:
        """メッセージング設計.md: Tier ラベルが設定されている。"""
        for doc in self.docs:
            assert "tier" in doc["metadata"]["labels"]


class TestSchemaRegistry:
    """メッセージング設計.md: Schema Registry 設定の検証。"""

    def setup_method(self) -> None:
        path = MSG / "schema-registry" / "schema-registry-config.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.docs = list(yaml.safe_load_all(self.content))

    def test_schema_registry_file_exists(self) -> None:
        assert (MSG / "schema-registry" / "schema-registry-config.yaml").exists()

    def test_deployment_exists(self) -> None:
        kinds = [d["kind"] for d in self.docs if d]
        assert "Deployment" in kinds

    def test_service_exists(self) -> None:
        kinds = [d["kind"] for d in self.docs if d]
        assert "Service" in kinds

    def test_schema_registry_image(self) -> None:
        """メッセージング設計.md: Confluent Schema Registry を使用。"""
        deploy = [d for d in self.docs if d and d["kind"] == "Deployment"][0]
        containers = deploy["spec"]["template"]["spec"]["containers"]
        assert "confluentinc/cp-schema-registry" in containers[0]["image"]

    def test_schema_registry_replicas(self) -> None:
        deploy = [d for d in self.docs if d and d["kind"] == "Deployment"][0]
        assert deploy["spec"]["replicas"] == 2

    def test_compatibility_level(self) -> None:
        """メッセージング設計.md: BACKWARD 互換性レベル。"""
        assert "BACKWARD" in self.content

    def test_protobuf_provider(self) -> None:
        """メッセージング設計.md: Protobuf スキーマプロバイダー。"""
        assert "protobuf" in self.content

    def test_part_of_label(self) -> None:
        assert "app.kubernetes.io/part-of" in self.content


class TestTierPartitionCounts:
    """メッセージング設計.md: Tier 別パーティション数の検証。"""

    def setup_method(self) -> None:
        path = MSG / "kafka" / "topics.yaml"
        content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(content) if d]

    def test_system_tier_partitions_6(self) -> None:
        """メッセージング設計.md: system Tier パーティション数=6。"""
        system_topics = [
            d for d in self.docs
            if d["metadata"]["labels"].get("tier") == "system"
            and not d["metadata"]["name"].endswith(".dlq")
        ]
        for topic in system_topics:
            assert topic["spec"]["partitions"] == 6, (
                f"Topic '{topic['metadata']['name']}' のパーティション数が 6 ではありません"
            )

    def test_business_tier_partitions_3(self) -> None:
        """メッセージング設計.md: business Tier パーティション数=3。"""
        biz_topics = [
            d for d in self.docs
            if d["metadata"]["labels"].get("tier") == "business"
            and not d["metadata"]["name"].endswith(".dlq")
        ]
        for topic in biz_topics:
            assert topic["spec"]["partitions"] == 3, (
                f"Topic '{topic['metadata']['name']}' のパーティション数が 3 ではありません"
            )

    def test_service_tier_partitions_3(self) -> None:
        """メッセージング設計.md: service Tier パーティション数=3。"""
        svc_topics = [
            d for d in self.docs
            if d["metadata"]["labels"].get("tier") == "service"
            and not d["metadata"]["name"].endswith(".dlq")
        ]
        for topic in svc_topics:
            assert topic["spec"]["partitions"] == 3, (
                f"Topic '{topic['metadata']['name']}' のパーティション数が 3 ではありません"
            )


class TestSagaWorkflowDefinition:
    """メッセージング設計.md: Saga ワークフロー定義ファイルの検証。"""

    SAGA = ROOT / "infra" / "messaging" / "saga" / "workflows"

    def test_order_fulfillment_yaml_exists(self) -> None:
        """メッセージング設計.md: order-fulfillment.yaml が存在。"""
        assert (self.SAGA / "order-fulfillment.yaml").exists()

    def test_order_fulfillment_name(self) -> None:
        """メッセージング設計.md: ワークフロー名が order-fulfillment。"""
        path = self.SAGA / "order-fulfillment.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        assert config["name"] == "order-fulfillment"

    def test_order_fulfillment_has_steps(self) -> None:
        """メッセージング設計.md: steps が定義されている。"""
        path = self.SAGA / "order-fulfillment.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        assert "steps" in config
        assert len(config["steps"]) >= 3

    def test_order_fulfillment_reserve_inventory(self) -> None:
        """メッセージング設計.md: reserve-inventory ステップが存在。"""
        path = self.SAGA / "order-fulfillment.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        step_names = [s["name"] for s in config["steps"]]
        assert "reserve-inventory" in step_names

    def test_order_fulfillment_process_payment(self) -> None:
        """メッセージング設計.md: process-payment ステップが存在。"""
        path = self.SAGA / "order-fulfillment.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        step_names = [s["name"] for s in config["steps"]]
        assert "process-payment" in step_names

    def test_order_fulfillment_confirm_order(self) -> None:
        """メッセージング設計.md: confirm-order ステップが存在。"""
        path = self.SAGA / "order-fulfillment.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        step_names = [s["name"] for s in config["steps"]]
        assert "confirm-order" in step_names

    def test_steps_have_compensate(self) -> None:
        """メッセージング設計.md: 各ステップに compensate が定義されている。"""
        path = self.SAGA / "order-fulfillment.yaml"
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
        for step in config["steps"]:
            assert "compensate" in step, (
                f"ステップ '{step['name']}' に compensate が定義されていません"
            )


class TestEnvironmentClusterConfig:
    """メッセージング設計.md: 環境別クラスタ構成の検証。"""

    def test_kafka_cluster_has_prod_replicas_3(self) -> None:
        """メッセージング設計.md: prod 環境は Kafka ブローカー 3 台。"""
        path = MSG / "kafka" / "kafka-cluster.yaml"
        content = path.read_text(encoding="utf-8")
        docs = list(yaml.safe_load_all(content))
        kafka = docs[0]
        assert kafka["spec"]["kafka"]["replicas"] == 3

    def test_kafka_cluster_has_zookeeper_replicas_3(self) -> None:
        """メッセージング設計.md: prod 環境は ZooKeeper 3 ノード。"""
        path = MSG / "kafka" / "kafka-cluster.yaml"
        content = path.read_text(encoding="utf-8")
        docs = list(yaml.safe_load_all(content))
        kafka = docs[0]
        assert kafka["spec"]["zookeeper"]["replicas"] == 3
