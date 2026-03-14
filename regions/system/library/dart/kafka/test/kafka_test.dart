import 'package:test/test.dart';
import 'package:k1s0_kafka/kafka.dart';

void main() {
  group('KafkaConfig', () {
    group('bootstrapServersString', () {
      test('複数のブローカーをカンマで結合すること', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker1:9092', 'broker2:9092', 'broker3:9092'],
        );
        expect(config.bootstrapServersString(),
            equals('broker1:9092,broker2:9092,broker3:9092'));
      });

      test('単一ブローカーをそのまま返すこと', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker1:9092'],
        );
        expect(config.bootstrapServersString(), equals('broker1:9092'));
      });
    });

    group('usesTLS', () {
      test('SSLの場合にtrueを返すこと', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SSL',
        );
        expect(config.usesTLS(), isTrue);
      });

      test('SASL_SSLの場合にtrueを返すこと', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_SSL',
        );
        expect(config.usesTLS(), isTrue);
      });

      test('PLAINTEXTの場合にfalseを返すこと', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'PLAINTEXT',
        );
        expect(config.usesTLS(), isFalse);
      });

      test('SASL_PLAINTEXTの場合にfalseを返すこと', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_PLAINTEXT',
        );
        expect(config.usesTLS(), isFalse);
      });

      test('securityProtocolがnullの場合にfalseを返すこと', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
        );
        expect(config.usesTLS(), isFalse);
      });
    });

    group('validate', () {
      test('bootstrapServersが空の場合にエラーを投げること', () {
        final config = KafkaConfig(bootstrapServers: []);
        expect(
          () => config.validate(),
          throwsA(isA<KafkaError>().having(
              (e) => e.message, 'message', contains('bootstrap servers'))),
        );
      });

      test('不正なセキュリティプロトコルでエラーを投げること', () {
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

      test('有効なPLAINTEXT設定でバリデーションが通ること', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'PLAINTEXT',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('有効なSSL設定でバリデーションが通ること', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SSL',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('有効なSASL_PLAINTEXT設定でバリデーションが通ること', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_PLAINTEXT',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('有効なSASL_SSL設定でバリデーションが通ること', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
          securityProtocol: 'SASL_SSL',
        );
        expect(() => config.validate(), returnsNormally);
      });

      test('securityProtocolなしでバリデーションが通ること', () {
        final config = KafkaConfig(
          bootstrapServers: ['broker:9092'],
        );
        expect(() => config.validate(), returnsNormally);
      });
    });
  });

  group('TopicConfig', () {
    group('validateName', () {
      test('有効なsystemトピック名を受け入れること', () {
        final topic =
            TopicConfig(name: 'k1s0.system.user.created.v1');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('有効なbusinessトピック名を受け入れること', () {
        final topic =
            TopicConfig(name: 'k1s0.business.order.placed.v1');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('有効なserviceトピック名を受け入れること', () {
        final topic =
            TopicConfig(name: 'k1s0.service.payment.completed.v2');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('ハイフンを含むトピック名を受け入れること', () {
        final topic = TopicConfig(
            name: 'k1s0.system.user-auth.session-created.v1');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('数字を含むトピック名を受け入れること', () {
        final topic =
            TopicConfig(name: 'k1s0.system.user123.created456.v10');
        expect(() => topic.validateName(), returnsNormally);
      });

      test('空のトピック名でエラーを投げること', () {
        final topic = TopicConfig(name: '');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()
              .having((e) => e.message, 'message', contains('empty'))),
        );
      });

      test('プレフィックスが欠けている場合にエラーを投げること', () {
        final topic = TopicConfig(name: 'invalid.system.user.created.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()
              .having((e) => e.message, 'message', contains('invalid topic'))),
        );
      });

      test('不正なティアでエラーを投げること', () {
        final topic = TopicConfig(name: 'k1s0.invalid.user.created.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });

      test('バージョンが欠けている場合にエラーを投げること', () {
        final topic = TopicConfig(name: 'k1s0.system.user.created');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });

      test('大文字が含まれる場合にエラーを投げること', () {
        final topic = TopicConfig(name: 'k1s0.system.User.Created.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });

      test('セグメントが不足している場合にエラーを投げること', () {
        final topic = TopicConfig(name: 'k1s0.system.user.v1');
        expect(
          () => topic.validateName(),
          throwsA(isA<KafkaError>()),
        );
      });
    });

    group('tier', () {
      test('systemティアを返すこと', () {
        final topic =
            TopicConfig(name: 'k1s0.system.user.created.v1');
        expect(topic.tier(), equals('system'));
      });

      test('businessティアを返すこと', () {
        final topic =
            TopicConfig(name: 'k1s0.business.order.placed.v1');
        expect(topic.tier(), equals('business'));
      });

      test('serviceティアを返すこと', () {
        final topic =
            TopicConfig(name: 'k1s0.service.payment.completed.v1');
        expect(topic.tier(), equals('service'));
      });

      test('不正なトピック名で空文字列を返すこと', () {
        final topic = TopicConfig(name: 'invalid');
        expect(topic.tier(), equals(''));
      });
    });

    group('partitionsForTier', () {
      test('systemティアで6を返すこと', () {
        expect(TopicConfig.partitionsForTier('system'), equals(6));
      });

      test('businessティアで6を返すこと', () {
        expect(TopicConfig.partitionsForTier('business'), equals(6));
      });

      test('serviceティアで3を返すこと', () {
        expect(TopicConfig.partitionsForTier('service'), equals(3));
      });

      test('不明なティアで3を返すこと', () {
        expect(TopicConfig.partitionsForTier('other'), equals(3));
      });
    });

    group('defaultPartitionsForTier', () {
      test('systemトピックで6を返すこと', () {
        final topic =
            TopicConfig(name: 'k1s0.system.auth.login.v1');
        expect(topic.defaultPartitionsForTier(), equals(6));
      });

      test('businessトピックで6を返すこと', () {
        final topic =
            TopicConfig(name: 'k1s0.business.order.placed.v1');
        expect(topic.defaultPartitionsForTier(), equals(6));
      });

      test('serviceトピックで3を返すこと', () {
        final topic =
            TopicConfig(name: 'k1s0.service.payment.done.v1');
        expect(topic.defaultPartitionsForTier(), equals(3));
      });

      test('不正なトピック名で3を返すこと', () {
        final topic = TopicConfig(name: 'invalid');
        expect(topic.defaultPartitionsForTier(), equals(3));
      });
    });
  });

  group('KafkaHealthStatus', () {
    test('必須フィールドで生成できること', () {
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
    test('設定済みステータスを返すこと', () async {
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

    test('設定済みエラーを投げること', () async {
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
    test('toStringにメッセージが含まれること', () {
      const err = KafkaError('test error');
      expect(err.toString(), contains('test error'));
    });
  });
}
