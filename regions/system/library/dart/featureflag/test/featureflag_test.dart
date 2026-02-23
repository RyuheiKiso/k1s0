import 'package:test/test.dart';
import 'package:k1s0_featureflag/k1s0_featureflag.dart';

void main() {
  late InMemoryFeatureFlagClient client;
  const context = EvaluationContext(userId: 'user-1', tenantId: 'tenant-1');

  setUp(() {
    client = InMemoryFeatureFlagClient();
  });

  group('InMemoryFeatureFlagClient.evaluate', () {
    test('evaluate enabled flag returns enabled', () async {
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

    test('evaluate disabled flag returns disabled', () async {
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

    test('evaluate nonexistent flag returns FLAG_NOT_FOUND reason', () async {
      final result = await client.evaluate('unknown-flag', context);
      expect(result.enabled, isFalse);
      expect(result.reason, equals('FLAG_NOT_FOUND'));
    });

    test('evaluate flag with variants returns variant value', () async {
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
    test('isEnabled returns true for enabled flag', () async {
      client.setFlag(const FeatureFlag(
        id: 'flag-1',
        flagKey: 'enabled-flag',
        description: 'Enabled',
        enabled: true,
      ));

      final result = await client.isEnabled('enabled-flag', context);
      expect(result, isTrue);
    });

    test('isEnabled returns false for disabled flag', () async {
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
    test('getFlag returns flag when it exists', () async {
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

    test('getFlag returns null when flag does not exist', () async {
      final result = await client.getFlag('nonexistent');
      expect(result, isNull);
    });
  });

  group('FeatureFlagNotFoundException', () {
    test('has correct message', () {
      const err = FeatureFlagNotFoundException('my-flag');
      expect(err.flagKey, equals('my-flag'));
      expect(err.toString(), contains('my-flag'));
    });
  });
}
