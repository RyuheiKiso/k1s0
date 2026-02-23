import 'context.dart';
import 'flag.dart';
import 'result.dart';

abstract class FeatureFlagClient {
  Future<EvaluationResult> evaluate(
    String flagKey,
    EvaluationContext context,
  );

  Future<bool> isEnabled(String flagKey, EvaluationContext context);

  Future<FeatureFlag?> getFlag(String flagKey);
}

class InMemoryFeatureFlagClient implements FeatureFlagClient {
  final Map<String, FeatureFlag> _flags = {};

  void setFlag(FeatureFlag flag) {
    _flags[flag.flagKey] = flag;
  }

  @override
  Future<EvaluationResult> evaluate(
    String flagKey,
    EvaluationContext context,
  ) async {
    final flag = _flags[flagKey];
    if (flag == null) {
      return const EvaluationResult(
        enabled: false,
        reason: 'FLAG_NOT_FOUND',
      );
    }

    if (!flag.enabled) {
      return const EvaluationResult(
        enabled: false,
        reason: 'DISABLED',
      );
    }

    String? variantValue;
    if (flag.variants.isNotEmpty) {
      variantValue = flag.variants.first.value;
    }

    return EvaluationResult(
      enabled: true,
      variant: variantValue,
      reason: 'ENABLED',
    );
  }

  @override
  Future<bool> isEnabled(String flagKey, EvaluationContext context) async {
    final result = await evaluate(flagKey, context);
    return result.enabled;
  }

  @override
  Future<FeatureFlag?> getFlag(String flagKey) async {
    return _flags[flagKey];
  }
}
