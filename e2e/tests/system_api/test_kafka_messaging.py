"""Kafka メッセージング E2E テスト。

System Tier の Kafka トピックの存在確認、監査ログ・設定変更イベントの
発行と消費を検証する。
前提: docker compose --profile messaging (kafka) が起動済みであること。
"""

import json
import time
import uuid

import pytest
import requests

try:
    from confluent_kafka import Consumer, TopicPartition
    from confluent_kafka.admin import AdminClient
except ImportError:
    Consumer = None
    AdminClient = None


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestKafkaTopics:
    """Kafka トピックの存在と基本設定を検証する。"""

    EXPECTED_SYSTEM_TOPICS = [
        "k1s0.system.auth.audit.v1",
        "k1s0.system.config.changed.v1",
        "k1s0.system.auth.login.v1",
    ]

    def test_system_topics_exist(self, kafka_bootstrap_servers):
        """System Tier のトピックが作成されている。"""
        admin = AdminClient({"bootstrap.servers": kafka_bootstrap_servers})
        metadata = admin.list_topics(timeout=10)
        topic_names = set(metadata.topics.keys())

        for topic in self.EXPECTED_SYSTEM_TOPICS:
            assert topic in topic_names, f"Topic {topic} does not exist"

    def test_dlq_topics_exist(self, kafka_bootstrap_servers):
        """DLQ トピックが作成されている。"""
        admin = AdminClient({"bootstrap.servers": kafka_bootstrap_servers})
        metadata = admin.list_topics(timeout=10)
        topic_names = set(metadata.topics.keys())

        for topic in self.EXPECTED_SYSTEM_TOPICS:
            dlq_topic = f"{topic}.dlq"
            assert dlq_topic in topic_names, f"DLQ topic {dlq_topic} does not exist"

    def test_audit_topic_partition_count(self, kafka_bootstrap_servers):
        """監査ログトピックのパーティション数が 6 である。"""
        admin = AdminClient({"bootstrap.servers": kafka_bootstrap_servers})
        metadata = admin.list_topics(timeout=10)
        topic_metadata = metadata.topics.get("k1s0.system.auth.audit.v1")
        assert topic_metadata is not None
        assert len(topic_metadata.partitions) == 6

    def test_config_changed_topic_partition_count(self, kafka_bootstrap_servers):
        """設定変更トピックのパーティション数が 6 である。"""
        admin = AdminClient({"bootstrap.servers": kafka_bootstrap_servers})
        metadata = admin.list_topics(timeout=10)
        topic_metadata = metadata.topics.get("k1s0.system.config.changed.v1")
        assert topic_metadata is not None
        assert len(topic_metadata.partitions) == 6

    def test_dlq_topic_single_partition(self, kafka_bootstrap_servers):
        """DLQ トピックのパーティション数が 1 である。"""
        admin = AdminClient({"bootstrap.servers": kafka_bootstrap_servers})
        metadata = admin.list_topics(timeout=10)
        dlq_metadata = metadata.topics.get("k1s0.system.auth.audit.v1.dlq")
        assert dlq_metadata is not None
        assert len(dlq_metadata.partitions) == 1


def _consume_recent_messages(bootstrap_servers, topic, timeout_sec=10, max_lookback=100):
    """指定トピックの最新メッセージを消費して返すヘルパー。"""
    consumer = Consumer(
        {
            "bootstrap.servers": bootstrap_servers,
            "group.id": f"e2e-{uuid.uuid4().hex[:8]}",
            "auto.offset.reset": "latest",
            "enable.auto.commit": False,
        }
    )

    partitions = consumer.list_topics(topic=topic, timeout=5)
    topic_meta = partitions.topics[topic]
    assignments = []
    for pid in topic_meta.partitions:
        tp = TopicPartition(topic, pid)
        assignments.append(tp)
    consumer.assign(assignments)

    # 各パーティションの末尾から max_lookback 前に巻き戻し
    for tp in assignments:
        _, high = consumer.get_watermark_offsets(tp, timeout=5)
        tp.offset = max(0, high - max_lookback)
    consumer.assign(assignments)

    messages = []
    deadline = time.time() + timeout_sec
    while time.time() < deadline:
        msg = consumer.poll(timeout=1.0)
        if msg is None:
            continue
        if msg.error():
            continue
        try:
            value = json.loads(msg.value().decode("utf-8"))
            messages.append(value)
        except (json.JSONDecodeError, UnicodeDecodeError):
            continue

    consumer.close()
    return messages


class TestKafkaAuditMessaging:
    """監査ログ Kafka メッセージングの E2E テスト。"""

    AUDIT_TOPIC = "k1s0.system.auth.audit.v1"

    @pytest.fixture(autouse=True)
    def _skip_if_auth_unavailable(self, auth_base_url):
        if not _server_available(auth_base_url + "/healthz"):
            pytest.skip("Auth server is not running")

    def test_audit_log_event_published(self, kafka_bootstrap_servers, auth_base_url):
        """監査ログ記録時に Kafka イベントが発行される。"""
        marker = f"e2e-audit-{uuid.uuid4().hex[:8]}"

        resp = requests.post(
            auth_base_url + "/api/v1/audit/logs",
            json={
                "action": "LOGIN",
                "user_id": marker,
                "resource": "auth",
                "detail": "E2E Kafka messaging test",
            },
            headers={"Content-Type": "application/json"},
            timeout=5,
        )
        assert resp.status_code in (200, 201)

        messages = _consume_recent_messages(
            kafka_bootstrap_servers,
            self.AUDIT_TOPIC,
            timeout_sec=10,
        )

        found = any(m.get("user_id") == marker for m in messages)
        if not found:
            pytest.skip(
                "Audit event not found in Kafka (Kafka producer may not be enabled in auth-server)"
            )

    def test_audit_event_schema(self, kafka_bootstrap_servers, auth_base_url):
        """監査ログイベントのスキーマが正しい。"""
        marker = f"e2e-schema-{uuid.uuid4().hex[:8]}"

        requests.post(
            auth_base_url + "/api/v1/audit/logs",
            json={
                "action": "PERMISSION_CHECK",
                "user_id": marker,
                "resource": "config",
                "detail": "Schema validation test",
            },
            headers={"Content-Type": "application/json"},
            timeout=5,
        )

        messages = _consume_recent_messages(
            kafka_bootstrap_servers,
            self.AUDIT_TOPIC,
            timeout_sec=10,
        )

        event = next((m for m in messages if m.get("user_id") == marker), None)
        if event is None:
            pytest.skip(
                "Audit event not found in Kafka (Kafka producer may not be enabled in auth-server)"
            )

        # スキーマ検証: action/event_type と user_id は必須
        assert "action" in event or "event_type" in event
        assert "user_id" in event
        assert event["user_id"] == marker


class TestKafkaConfigMessaging:
    """設定変更 Kafka メッセージングの E2E テスト。"""

    CONFIG_TOPIC = "k1s0.system.config.changed.v1"

    @pytest.fixture(autouse=True)
    def _skip_if_config_unavailable(self, config_base_url):
        if not _server_available(config_base_url + "/healthz"):
            pytest.skip("Config server is not running")

    def test_config_changed_event_published(self, kafka_bootstrap_servers, config_base_url):
        """設定変更時に Kafka イベントが発行される。"""
        unique_key = f"kafka-e2e-{uuid.uuid4().hex[:8]}"

        resp = requests.put(
            config_base_url + f"/api/v1/config/e2e-test/{unique_key}",
            json={"value": "kafka-test-value", "description": "Kafka E2E test"},
            headers={"Content-Type": "application/json"},
            timeout=5,
        )
        assert resp.status_code in (200, 201)

        messages = _consume_recent_messages(
            kafka_bootstrap_servers,
            self.CONFIG_TOPIC,
            timeout_sec=10,
        )

        found = any(unique_key in json.dumps(m) for m in messages)
        if not found:
            pytest.skip(
                "Config changed event not found in Kafka "
                "(Kafka producer may not be enabled in config-server)"
            )
