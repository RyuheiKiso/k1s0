import 'package:test/test.dart';
import 'package:k1s0_kafka/kafka.dart';

void main() {
  group('KafkaConfig', () {
    group('bootstrapServersString', () {
      test('joins multiple brokers with comma', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker1:9092', 'broker2:9092', 'broker3:9092'],
        );
        expect(config.bootstrapServersString(),
            equals('broker1:9092,broker2:9092,broker3:9092'));
      });

      test('returns single broker as-is', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker1:9092'],
        );
        expect(config.bootstrapServersString(), equals('broker1:9092'));
      });
    });

    group('usesTLS', () {
      test('returns true for SSL', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SSL',
        );
        expect(config.usesTLS(), isTrue);
      });

      test('returns true for SASL_SSL', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_SSL',
        );
        expect(config.usesTLS(), isTrue);
      });

      test('returns false for PLAINTEXT', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'PLAINTEXT',
        );
        expect(config.usesTLS(), isFalse);
      });

      test('returns false for SASL_PLAINTEXT', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_PLAINTEXT',
        );
        expect(config.usesTLS(), isFalse);
      });

      test('returns false when securityProtocol is null', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
        );
        expect(config.usesTLS(), isFalse);
      });
    });

    group('validate', () {
      test('throws on empty bootstrapServers', () {
        final config = KafkaConfig(bootstrapServers: []);
        expect(
          () => config.validate(),
          throwsA(isA<KafkaError>().having(
              (e) => e.message, 'message', contains('bootstrap servers'))),
        );
      });

      test('throws on invalid security protocol', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'INVALID',
        );
        expect(
          () => config.validate(),
          throwsA(isA<KafkaError>().having(
              (e) => e.message, 'message', contains('invalid security'))),
        );
      });

      test('passes with valid PLAINTEXT config', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'PLAINTEXT',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('passes with valid SSL config', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SSL',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('passes with valid SASL_PLAINTEXT config', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_PLAINTEXT',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('passes with valid SASL_SSL config', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_SSL',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('passes without securityProtocol', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
        );
        expect(() => config.validate(), returnsNormally);
      });
    });
  });

  group('TopicConfig', () {
    group('validateName', () {
      test('accepts valid system topic', () {
        final topic =
            TopicConfig(name: 'k1s0.system.user.created.v1');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('accepts valid business topic', () {
        final topic =
            TopicConfig(name: 'k1s0.business.order.placed.v1');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('accepts valid service topic', () {
        final topic =
            TopicConfig(name: 'k1s0.service.payment.completed.v2');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('accepts topic with hyphens', () {
        final topic = TopicConfig(
            name: 'k1s0.system.user-auth.session-created.v1');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('accepts topic with numbers', () {
        final topic =
            TopicConfig(name: 'k1s0.system.user123.created456.v10');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('throws on empty name', () {
        final topic = TopicConfig(name: '');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()
              .having((e) => e.message, 'message', contains('empty'))),
        );
      });

      test('throws on missing prefix', () {
        final topic = TopicConfig(name: 'invalid.system.user.created.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()
              .having((e) => e.message, 'message', contains('invalid topic'))),
        );
      });

      test('throws on invalid tier', () {
        final topic = TopicConfig(name: 'k1s0.invalid.user.created.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });

      test('throws on missing version', () {
        final topic = TopicConfig(name: 'k1s0.system.user.created');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });

      test('throws on uppercase characters', () {
        final topic = TopicConfig(name: 'k1s0.system.User.Created.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });

      test('throws on missing segments', () {
        final topic = TopicConfig(name: 'k1s0.system.user.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });
    });

    group('tier', () {
      test('returns system tier', () {
        final topic =
            TopicConfig(name: 'k1s0.system.user.created.v1');
        expect(topic.tier(), equals('system'));
      });

      test('returns business tier', () {
        final topic =
            TopicConfig(name: 'k1s0.business.order.placed.v1');
        expect(topic.tier(), equals('business'));
      });

      test('returns service tier', () {
        final topic =
            TopicConfig(name: 'k1s0.service.payment.completed.v1');
        expect(topic.tier(), equals('service'));
      });

      test('returns empty string for invalid topic', () {
        final topic = TopicConfig(name: 'invalid');
        expect(topic.tier(), equals(''));
      });
    });
  });

  group('KafkaHealthStatus', () {
    test('can be created with required fields', () {
      const status = KafkaHealthStatus(
        healthy: true,
        message: 'OK',
        brokerCount: 3,
      );
      expect(status.healthy, isTrue);
      expect(status.message, equals('OK'));
      expect(status.brokerCount, equals(3));
    });
  });

  group('NoOpKafkaHealthChecker', () {
    test('returns configured status', () async {
      const status = KafkaHealthStatus(
        healthy: true,
        message: 'All brokers connected',
        brokerCount: 3,
      );
      final checker = NoOpKafkaHealthChecker(status: status);
      final result = await checker.healthCheck();
      expect(result.healthy, isTrue);
      expect(result.message, equals('All brokers connected'));
      expect(result.brokerCount, equals(3));
    });

    test('throws configured error', () async {
      const status = KafkaHealthStatus(
        healthy: false,
        message: 'error',
        brokerCount: 0,
      );
      final checker = NoOpKafkaHealthChecker(
        status: status,
        error: Exception('connection failed'),
      );
      expect(
        () => checker.healthCheck(),
        throwsA(isA<Exception>()),
      );
    });
  });

  group('KafkaError', () {
    test('toString includes message', () {
      const err = KafkaError('test error');
      expect(err.toString(), contains('test error'));
    });
  });
}
