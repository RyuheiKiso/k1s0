import 'package:test/test.dart';
import 'package:k1s0_featureflag/k1s0_featureflag.dart';

void main() {
  late InMemoryFeatureFlagClient client;
  const context = EvaluationContext(userId: 'user-1', tenantId: 'tenant-1');

  setUp(() {
    client = InMemoryFeatureFlagClient();
  });

  group('InMemoryFeatureFlagClient.evaluate', () {
    test('有効なフラグの評価がenabledを返すこと', () async {
      client.setFlag(const FeatureFlag(
        id: 'flag-1',
        flagKey: 'new-feature',
        description: 'A new feature',
        enabled: true,
      ));

      final result = await client.evaluate('new-feature', context);
      expect(result.enabled, isTrue);
      expect(result.reason, equals('ENABLED'));
    });

    test('無効なフラグの評価がdisabledを返すこと', () async {
      client.setFlag(const FeatureFlag(
        id: 'flag-2',
        flagKey: 'old-feature',
        description: 'An old feature',
        enabled: false,
      ));

      final result = await client.evaluate('old-feature', context);
      expect(result.enabled, isFalse);
      expect(result.reason, equals('DISABLED'));
    });

    test('存在しないフラグの評価がFLAG_NOT_FOUNDを返すこと', () async {
      final result = await client.evaluate('unknown-flag', context);
      expect(result.enabled, isFalse);
      expect(result.reason, equals('FLAG_NOT_FOUND'));
    });

    test('バリアント付きフラグの評価がバリアント値を返すこと', () async {
      client.setFlag(const FeatureFlag(
        id: 'flag-3',
        flagKey: 'ab-test',
        description: 'A/B test feature',
        enabled: true,
        variants: [
          FlagVariant(name: 'treatment', value: 'v2', weight: 0.5),
          FlagVariant(name: 'control', value: 'v1', weight: 0.5),
        ],
      ));

      final result = await client.evaluate('ab-test', context);
      expect(result.enabled, isTrue);
      expect(result.variant, equals('v2'));
    });
  });

  group('InMemoryFeatureFlagClient.isEnabled', () {
    test('isEnabledが有効なフラグに対してtrueを返すこと', () async {
      client.setFlag(const FeatureFlag(
        id: 'flag-1',
        flagKey: 'enabled-flag',
        description: 'Enabled',
        enabled: true,
      ));

      final result = await client.isEnabled('enabled-flag', context);
      expect(result, isTrue);
    });

    test('isEnabledが無効なフラグに対してfalseを返すこと', () async {
      client.setFlag(const FeatureFlag(
        id: 'flag-2',
        flagKey: 'disabled-flag',
        description: 'Disabled',
        enabled: false,
      ));

      final result = await client.isEnabled('disabled-flag', context);
      expect(result, isFalse);
    });
  });

  group('InMemoryFeatureFlagClient.getFlag', () {
    test('getFlagが存在するフラグを返すこと', () async {
      const flag = FeatureFlag(
        id: 'flag-1',
        flagKey: 'my-flag',
        description: 'My flag',
        enabled: true,
      );
      client.setFlag(flag);

      final result = await client.getFlag('my-flag');
      expect(result, isNotNull);
      expect(result!.flagKey, equals('my-flag'));
      expect(result.enabled, isTrue);
    });

    test('getFlagがフラグが存在しない場合にnullを返すこと', () async {
      final result = await client.getFlag('nonexistent');
      expect(result, isNull);
    });
  });

  group('FeatureFlagNotFoundException', () {
    test('正しいメッセージを持つこと', () {
      const err = FeatureFlagNotFoundException('my-flag');
      expect(err.flagKey, equals('my-flag'));
      expect(err.toString(), contains('my-flag'));
    });
  });
}
