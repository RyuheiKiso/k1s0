import { describe, it, expect } from 'vitest';
import {
  KafkaConfig,
  bootstrapServersString,
  usesTLS,
  validateKafkaConfig,
  DEFAULT_CONNECTION_TIMEOUT_MS,
  DEFAULT_REQUEST_TIMEOUT_MS,
  DEFAULT_MAX_MESSAGE_BYTES,
  TopicConfig,
  validateTopicName,
  topicTier,
  defaultPartitionsForTier,
  topicDefaultPartitions,
  NoOpKafkaHealthChecker,
  KafkaHealthStatus,
  KafkaError,
} from '../src/index.js';

describe('bootstrapServersString', () => {
  it('joins multiple brokers with comma', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092', 'broker2:9092'], consumerGroup: 'test-group' };
    expect(bootstrapServersString(config)).toBe('broker1:9092,broker2:9092');
  });

  it('returns single broker as-is', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092'], consumerGroup: 'test-group' };
    expect(bootstrapServersString(config)).toBe('broker1:9092');
  });
});

describe('usesTLS', () => {
  it('returns true for SSL', () => {
    const config: KafkaConfig = { bootstrapServers: ['b:9092'], securityProtocol: 'SSL', consumerGroup: 'test-group' };
    expect(usesTLS(config)).toBe(true);
  });

  it('returns true for SASL_SSL', () => {
    const config: KafkaConfig = { bootstrapServers: ['b:9092'], securityProtocol: 'SASL_SSL', consumerGroup: 'test-group' };
    expect(usesTLS(config)).toBe(true);
  });

  it('returns false for PLAINTEXT', () => {
    const config: KafkaConfig = { bootstrapServers: ['b:9092'], securityProtocol: 'PLAINTEXT', consumerGroup: 'test-group' };
    expect(usesTLS(config)).toBe(false);
  });

  it('returns false for SASL_PLAINTEXT', () => {
    const config: KafkaConfig = { bootstrapServers: ['b:9092'], securityProtocol: 'SASL_PLAINTEXT', consumerGroup: 'test-group' };
    expect(usesTLS(config)).toBe(false);
  });
});

describe('validateKafkaConfig', () => {
  it('passes with valid config', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092'], securityProtocol: 'PLAINTEXT', consumerGroup: 'test-group' };
    expect(() => validateKafkaConfig(config)).not.toThrow();
  });

  it('throws on empty bootstrapServers', () => {
    const config: KafkaConfig = { bootstrapServers: [], consumerGroup: 'test-group' };
    expect(() => validateKafkaConfig(config)).toThrow('bootstrap servers must not be empty');
  });

  it('throws on invalid protocol', () => {
    const config = { bootstrapServers: ['broker1:9092'], securityProtocol: 'INVALID', consumerGroup: 'test-group' } as unknown as KafkaConfig;
    expect(() => validateKafkaConfig(config)).toThrow('invalid security protocol');
  });

  it('throws on empty consumerGroup', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092'], consumerGroup: '' };
    expect(() => validateKafkaConfig(config)).toThrow('consumer group must not be empty');
  });

  it('throws on non-positive connectionTimeoutMs', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092'], consumerGroup: 'test-group', connectionTimeoutMs: 0 };
    expect(() => validateKafkaConfig(config)).toThrow('connection timeout must be positive');
  });

  it('throws on non-positive requestTimeoutMs', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092'], consumerGroup: 'test-group', requestTimeoutMs: -1 };
    expect(() => validateKafkaConfig(config)).toThrow('request timeout must be positive');
  });

  it('throws on non-positive maxMessageBytes', () => {
    const config: KafkaConfig = { bootstrapServers: ['broker1:9092'], consumerGroup: 'test-group', maxMessageBytes: 0 };
    expect(() => validateKafkaConfig(config)).toThrow('max message bytes must be positive');
  });

  it('passes with all optional timeout fields set', () => {
    const config: KafkaConfig = {
      bootstrapServers: ['broker1:9092'],
      consumerGroup: 'test-group',
      connectionTimeoutMs: 10000,
      requestTimeoutMs: 60000,
      maxMessageBytes: 2_000_000,
    };
    expect(() => validateKafkaConfig(config)).not.toThrow();
  });
});

describe('default constants', () => {
  it('has correct default connection timeout', () => {
    expect(DEFAULT_CONNECTION_TIMEOUT_MS).toBe(5000);
  });

  it('has correct default request timeout', () => {
    expect(DEFAULT_REQUEST_TIMEOUT_MS).toBe(30000);
  });

  it('has correct default max message bytes', () => {
    expect(DEFAULT_MAX_MESSAGE_BYTES).toBe(1_000_000);
  });
});

describe('validateTopicName', () => {
  const validNames = [
    'k1s0.system.user.created.v1',
    'k1s0.business.order.placed.v2',
    'k1s0.service.payment.processed.v1',
    'k1s0.system.user-profile.updated.v10',
    'k1s0.system.auth.token-refreshed.v1',
  ];

  it.each(validNames)('passes for valid topic name: %s', (name) => {
    expect(() => validateTopicName({ name })).not.toThrow();
  });

  const invalidNames = [
    '',
    'invalid',
    'k1s0.invalid.user.created.v1',
    'k1s0.system.USER.created.v1',
    'k1s0.system.user.created',
    'k1s0.system.user.created.v',
  ];

  it.each(invalidNames)('throws for invalid topic name: "%s"', (name) => {
    expect(() => validateTopicName({ name })).toThrow();
  });
});

describe('topicTier', () => {
  it('returns "system" for system topic', () => {
    expect(topicTier({ name: 'k1s0.system.user.created.v1' })).toBe('system');
  });

  it('returns "business" for business topic', () => {
    expect(topicTier({ name: 'k1s0.business.order.placed.v1' })).toBe('business');
  });

  it('returns "service" for service topic', () => {
    expect(topicTier({ name: 'k1s0.service.payment.done.v1' })).toBe('service');
  });

  it('returns empty string for invalid topic name', () => {
    expect(topicTier({ name: 'invalid-name' })).toBe('');
  });
});

describe('defaultPartitionsForTier', () => {
  it('returns 6 for system tier', () => {
    expect(defaultPartitionsForTier('system')).toBe(6);
  });

  it('returns 6 for business tier', () => {
    expect(defaultPartitionsForTier('business')).toBe(6);
  });

  it('returns 3 for service tier', () => {
    expect(defaultPartitionsForTier('service')).toBe(3);
  });

  it('returns 3 for unknown tier', () => {
    expect(defaultPartitionsForTier('other')).toBe(3);
  });
});

describe('topicDefaultPartitions', () => {
  it('returns 6 for system topic', () => {
    expect(topicDefaultPartitions({ name: 'k1s0.system.auth.login.v1' })).toBe(6);
  });

  it('returns 6 for business topic', () => {
    expect(topicDefaultPartitions({ name: 'k1s0.business.order.placed.v1' })).toBe(6);
  });

  it('returns 3 for service topic', () => {
    expect(topicDefaultPartitions({ name: 'k1s0.service.payment.done.v1' })).toBe(3);
  });

  it('returns 3 for invalid topic name', () => {
    expect(topicDefaultPartitions({ name: 'invalid' })).toBe(3);
  });
});

describe('NoOpKafkaHealthChecker', () => {
  it('returns configured healthy status', async () => {
    const status: KafkaHealthStatus = { healthy: true, message: 'OK', brokerCount: 3 };
    const checker = new NoOpKafkaHealthChecker(status);
    const result = await checker.healthCheck();
    expect(result.healthy).toBe(true);
    expect(result.brokerCount).toBe(3);
  });

  it('returns configured unhealthy status', async () => {
    const status: KafkaHealthStatus = { healthy: false, message: 'connection refused', brokerCount: 0 };
    const checker = new NoOpKafkaHealthChecker(status);
    const result = await checker.healthCheck();
    expect(result.healthy).toBe(false);
  });

  it('throws when error is configured', async () => {
    const status: KafkaHealthStatus = { healthy: false, message: '', brokerCount: 0 };
    const err = new Error('connection refused');
    const checker = new NoOpKafkaHealthChecker(status, err);
    await expect(checker.healthCheck()).rejects.toThrow('connection refused');
  });
});

describe('KafkaError', () => {
  it('has correct name and message', () => {
    const err = new KafkaError('test error');
    expect(err.name).toBe('KafkaError');
    expect(err.message).toBe('test error');
    expect(err).toBeInstanceOf(Error);
  });
});
