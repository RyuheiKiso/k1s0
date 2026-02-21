/** Kafka 接続設定。 */
export interface KafkaConfig {
  bootstrapServers: string[];
  securityProtocol?: 'PLAINTEXT' | 'SSL' | 'SASL_PLAINTEXT' | 'SASL_SSL';
  saslMechanism?: string;
  saslUsername?: string;
  saslPassword?: string;
}

/** BootstrapServers をカンマ区切りの文字列に変換する。 */
export function bootstrapServersString(config: KafkaConfig): string {
  return config.bootstrapServers.join(',');
}

/** TLS 接続を使用するかどうかを返す。 */
export function usesTLS(config: KafkaConfig): boolean {
  return config.securityProtocol === 'SSL' || config.securityProtocol === 'SASL_SSL';
}

const VALID_PROTOCOLS = new Set(['PLAINTEXT', 'SSL', 'SASL_PLAINTEXT', 'SASL_SSL']);

/** 設定を検証する。 */
export function validateKafkaConfig(config: KafkaConfig): void {
  if (config.bootstrapServers.length === 0) {
    throw new KafkaError('bootstrap servers must not be empty');
  }
  if (config.securityProtocol !== undefined && !VALID_PROTOCOLS.has(config.securityProtocol)) {
    throw new KafkaError(`invalid security protocol: ${config.securityProtocol}`);
  }
}

/** トピック名のバリデーション正規表現 */
const TOPIC_NAME_REGEX = /^k1s0\.(system|business|service)\.[a-z0-9-]+\.[a-z0-9-]+\.v[0-9]+$/;

/** Kafka トピック設定。 */
export interface TopicConfig {
  name: string;
  partitions?: number;
  replicationFactor?: number;
  retentionMs?: number;
}

/** トピック名を検証する。 */
export function validateTopicName(topic: TopicConfig): void {
  if (topic.name === '') {
    throw new KafkaError('topic name must not be empty');
  }
  if (!TOPIC_NAME_REGEX.test(topic.name)) {
    throw new KafkaError(
      `invalid topic name: ${topic.name} (expected format: k1s0.(system|business|service).<domain>.<event>.v<version>)`,
    );
  }
}

/** トピック名からティアを返す。 */
export function topicTier(topic: TopicConfig): string {
  const match = TOPIC_NAME_REGEX.exec(topic.name);
  if (!match || match.length < 2) {
    return '';
  }
  return match[1];
}

/** Kafka ヘルスチェックの結果。 */
export interface KafkaHealthStatus {
  healthy: boolean;
  message: string;
  brokerCount: number;
}

/** Kafka の疎通確認インターフェース。 */
export interface KafkaHealthChecker {
  healthCheck(): Promise<KafkaHealthStatus>;
}

/** テスト用の no-op KafkaHealthChecker 実装。 */
export class NoOpKafkaHealthChecker implements KafkaHealthChecker {
  constructor(
    private readonly status: KafkaHealthStatus,
    private readonly error?: Error,
  ) {}

  async healthCheck(): Promise<KafkaHealthStatus> {
    if (this.error) {
      throw this.error;
    }
    return this.status;
  }
}

/** Kafka エラー。 */
export class KafkaError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'KafkaError';
  }
}
