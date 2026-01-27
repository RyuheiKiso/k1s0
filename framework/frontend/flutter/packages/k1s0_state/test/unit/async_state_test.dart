import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_state/src/async/async_state.dart';

void main() {
  group('AsyncState', () {
    group('factories', () {
      test('initial creates AsyncInitial', () {
        const state = AsyncState<int>.initial();

        expect(state, isA<AsyncInitial<int>>());
      });

      test('loading creates AsyncLoading', () {
        const state = AsyncState<int>.loading();

        expect(state, isA<AsyncLoading<int>>());
      });

      test('loading with previousData', () {
        const state = AsyncState<int>.loading(previousData: 42);

        expect(state, isA<AsyncLoading<int>>());
        expect((state as AsyncLoading<int>).previousData, 42);
      });

      test('success creates AsyncSuccess', () {
        const state = AsyncState<int>.success(42);

        expect(state, isA<AsyncSuccess<int>>());
        expect((state as AsyncSuccess<int>).data, 42);
      });

      test('failure creates AsyncFailure', () {
        const state = AsyncState<int>.failure('error');

        expect(state, isA<AsyncFailure<int>>());
        expect((state as AsyncFailure<int>).error, 'error');
      });

      test('failure with stackTrace and previousData', () {
        final stackTrace = StackTrace.current;
        final state = AsyncState<int>.failure(
          'error',
          stackTrace: stackTrace,
          previousData: 42,
        );

        final failure = state as AsyncFailure<int>;
        expect(failure.stackTrace, stackTrace);
        expect(failure.previousData, 42);
      });
    });
  });

  group('AsyncStateExtensions', () {
    group('type checks', () {
      test('isInitial', () {
        expect(const AsyncState<int>.initial().isInitial, true);
        expect(const AsyncState<int>.loading().isInitial, false);
        expect(const AsyncState<int>.success(1).isInitial, false);
        expect(const AsyncState<int>.failure('e').isInitial, false);
      });

      test('isLoading', () {
        expect(const AsyncState<int>.initial().isLoading, false);
        expect(const AsyncState<int>.loading().isLoading, true);
        expect(const AsyncState<int>.success(1).isLoading, false);
        expect(const AsyncState<int>.failure('e').isLoading, false);
      });

      test('isSuccess', () {
        expect(const AsyncState<int>.initial().isSuccess, false);
        expect(const AsyncState<int>.loading().isSuccess, false);
        expect(const AsyncState<int>.success(1).isSuccess, true);
        expect(const AsyncState<int>.failure('e').isSuccess, false);
      });

      test('isFailure', () {
        expect(const AsyncState<int>.initial().isFailure, false);
        expect(const AsyncState<int>.loading().isFailure, false);
        expect(const AsyncState<int>.success(1).isFailure, false);
        expect(const AsyncState<int>.failure('e').isFailure, true);
      });
    });

    group('dataOrNull', () {
      test('returns null for initial', () {
        expect(const AsyncState<int>.initial().dataOrNull, isNull);
      });

      test('returns previousData for loading', () {
        expect(const AsyncState<int>.loading().dataOrNull, isNull);
        expect(
          const AsyncState<int>.loading(previousData: 42).dataOrNull,
          42,
        );
      });

      test('returns data for success', () {
        expect(const AsyncState<int>.success(42).dataOrNull, 42);
      });

      test('returns previousData for failure', () {
        expect(const AsyncState<int>.failure('e').dataOrNull, isNull);
        expect(
          const AsyncState<int>.failure('e', previousData: 42).dataOrNull,
          42,
        );
      });
    });

    test('hasData', () {
      expect(const AsyncState<int>.initial().hasData, false);
      expect(const AsyncState<int>.loading().hasData, false);
      expect(const AsyncState<int>.loading(previousData: 42).hasData, true);
      expect(const AsyncState<int>.success(42).hasData, true);
      expect(const AsyncState<int>.failure('e').hasData, false);
      expect(const AsyncState<int>.failure('e', previousData: 42).hasData, true);
    });

    test('errorOrNull', () {
      expect(const AsyncState<int>.initial().errorOrNull, isNull);
      expect(const AsyncState<int>.loading().errorOrNull, isNull);
      expect(const AsyncState<int>.success(42).errorOrNull, isNull);
      expect(const AsyncState<int>.failure('error').errorOrNull, 'error');
    });

    group('when', () {
      test('calls correct callback for initial', () {
        final result = const AsyncState<int>.initial().when(
          initial: () => 'initial',
          loading: (_) => 'loading',
          success: (_) => 'success',
          failure: (_, __, ___) => 'failure',
        );

        expect(result, 'initial');
      });

      test('calls correct callback for loading', () {
        final result = const AsyncState<int>.loading(previousData: 42).when(
          initial: () => 'initial',
          loading: (prev) => 'loading:$prev',
          success: (_) => 'success',
          failure: (_, __, ___) => 'failure',
        );

        expect(result, 'loading:42');
      });

      test('calls correct callback for success', () {
        final result = const AsyncState<int>.success(42).when(
          initial: () => 'initial',
          loading: (_) => 'loading',
          success: (data) => 'success:$data',
          failure: (_, __, ___) => 'failure',
        );

        expect(result, 'success:42');
      });

      test('calls correct callback for failure', () {
        final result = const AsyncState<int>.failure(
          'error',
          previousData: 42,
        ).when(
          initial: () => 'initial',
          loading: (_) => 'loading',
          success: (_) => 'success',
          failure: (e, st, prev) => 'failure:$e:$prev',
        );

        expect(result, 'failure:error:42');
      });
    });

    group('maybeWhen', () {
      test('calls callback when matches', () {
        final result = const AsyncState<int>.success(42).maybeWhen(
          orElse: () => 'else',
          success: (data) => 'success:$data',
        );

        expect(result, 'success:42');
      });

      test('calls orElse when no match', () {
        final result = const AsyncState<int>.initial().maybeWhen(
          orElse: () => 'else',
          success: (_) => 'success',
        );

        expect(result, 'else');
      });
    });

    group('map', () {
      test('transforms success data', () {
        final result = const AsyncState<int>.success(42).map((d) => d * 2);

        expect(result, isA<AsyncSuccess<int>>());
        expect(result.dataOrNull, 84);
      });

      test('transforms loading previousData', () {
        final result =
            const AsyncState<int>.loading(previousData: 42).map((d) => d * 2);

        expect(result, isA<AsyncLoading<int>>());
        expect(result.dataOrNull, 84);
      });

      test('preserves initial', () {
        final result =
            const AsyncState<int>.initial().map((d) => d.toString());

        expect(result, isA<AsyncInitial<String>>());
      });

      test('transforms failure previousData', () {
        final result = const AsyncState<int>.failure(
          'error',
          previousData: 42,
        ).map((d) => d * 2);

        expect(result, isA<AsyncFailure<int>>());
        expect(result.dataOrNull, 84);
      });
    });

    test('toLoading preserves data', () {
      final result = const AsyncState<int>.success(42).toLoading();

      expect(result, isA<AsyncLoading<int>>());
      expect(result.dataOrNull, 42);
    });

    test('toFailure preserves data', () {
      final result = const AsyncState<int>.success(42).toFailure('error');

      expect(result, isA<AsyncFailure<int>>());
      expect(result.dataOrNull, 42);
      expect(result.errorOrNull, 'error');
    });
  });
}
