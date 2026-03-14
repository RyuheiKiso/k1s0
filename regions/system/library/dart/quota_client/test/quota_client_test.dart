import 'package:test/test.dart';
import 'package:k1s0_quota_client/quota_client.dart';

void main() {
  late InMemoryQuotaClient client;

  setUp(() {
    client = InMemoryQuotaClient();
  });

  group('check（使用量チェック）', () {
    test('制限以内のリクエストで許可が返ること', () async {
      final status = await client.check('q1', 100);
      expect(status.allowed, isTrue);
      expect(status.remaining, equals(1000));
      expect(status.limit, equals(1000));
    });

    test('クォータ超過時に不許可が返ること', () async {
      await client.increment('q1', 900);
      final status = await client.check('q1', 200);
      expect(status.allowed, isFalse);
      expect(status.remaining, equals(100));
    });
  });

  group('increment（使用量加算）', () {
    test('使用量が累積されること', () async {
      await client.increment('q1', 300);
      final usage = await client.increment('q1', 200);
      expect(usage.used, equals(500));
      expect(usage.limit, equals(1000));
    });
  });

  group('getUsage（使用量取得）', () {
    test('現在の使用量が返ること', () async {
      await client.increment('q1', 100);
      final usage = await client.getUsage('q1');
      expect(usage.used, equals(100));
      expect(usage.quotaId, equals('q1'));
    });
  });

  group('getPolicy（ポリシー取得）', () {
    test('デフォルトポリシーが返ること', () async {
      final policy = await client.getPolicy('q1');
      expect(policy.quotaId, equals('q1'));
      expect(policy.limit, equals(1000));
      expect(policy.period, equals(QuotaPeriod.daily));
    });

    test('カスタムポリシーが返ること', () async {
      client.setPolicy(
        'q1',
        const QuotaPolicy(
          quotaId: 'q1',
          limit: 5000,
          period: QuotaPeriod.monthly,
          resetStrategy: 'sliding',
        ),
      );
      final policy = await client.getPolicy('q1');
      expect(policy.limit, equals(5000));
      expect(policy.period, equals(QuotaPeriod.monthly));
    });
  });

  group('CachedQuotaClient（キャッシュ付きクライアント）', () {
    test('checkを委譲すること', () async {
      final cached = CachedQuotaClient(client, const Duration(minutes: 1));
      final status = await cached.check('q1', 100);
      expect(status.allowed, isTrue);
    });

    test('incrementを委譲すること', () async {
      final cached = CachedQuotaClient(client, const Duration(minutes: 1));
      final usage = await cached.increment('q1', 100);
      expect(usage.used, equals(100));
    });

    test('getUsageを委譲すること', () async {
      final cached = CachedQuotaClient(client, const Duration(minutes: 1));
      await cached.increment('q1', 50);
      final usage = await cached.getUsage('q1');
      expect(usage.used, equals(50));
    });

    test('ポリシーをキャッシュすること', () async {
      final cached = CachedQuotaClient(client, const Duration(minutes: 1));
      final p1 = await cached.getPolicy('q1');
      client.setPolicy(
        'q1',
        const QuotaPolicy(
          quotaId: 'q1',
          limit: 9999,
          period: QuotaPeriod.hourly,
          resetStrategy: 'fixed',
        ),
      );
      final p2 = await cached.getPolicy('q1');
      expect(p2.limit, equals(p1.limit));
    });
  });

  group('QuotaExceededError（クォータ超過エラー）', () {
    test('quotaIdとremainingを保持すること', () {
      final error = QuotaExceededError('q1', 0);
      expect(error.quotaId, equals('q1'));
      expect(error.remaining, equals(0));
      expect(error.toString(), contains('Quota exceeded'));
    });
  });

  group('QuotaStatus（クォータ状態）', () {
    test('全フィールドを保持すること', () {
      final now = DateTime.now();
      final status = QuotaStatus(
        allowed: true,
        remaining: 500,
        limit: 1000,
        resetAt: now,
      );
      expect(status.allowed, isTrue);
      expect(status.remaining, equals(500));
    });
  });
}
