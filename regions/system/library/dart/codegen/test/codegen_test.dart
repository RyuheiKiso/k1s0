import 'package:k1s0_codegen/codegen.dart';
import 'package:test/test.dart';

void main() {
  group('ScaffoldConfig', () {
    // 有効な設定で validate が例外をスローしないことを確認する。
    test('有効な設定でバリデーションが成功すること', () {
      const config = ScaffoldConfig(
        name: 'my-server',
        tier: Tier.system,
        apiStyle: ApiStyle.rest,
        database: DatabaseType.none,
        description: 'テスト用サーバー',
      );
      expect(() => config.validate(), returnsNormally);
    });

    // 空の名前で ConfigError がスローされることを確認する。
    test('空の名前でバリデーションが失敗すること', () {
      const config = ScaffoldConfig(
        name: '',
        tier: Tier.system,
        apiStyle: ApiStyle.rest,
        database: DatabaseType.none,
        description: 'テスト',
      );
      expect(() => config.validate(), throwsA(isA<ConfigError>()));
    });

    // 不正なケバブケースで ConfigError がスローされることを確認する。
    test('大文字を含む名前でバリデーションが失敗すること', () {
      const config = ScaffoldConfig(
        name: 'MyServer',
        tier: Tier.system,
        apiStyle: ApiStyle.rest,
        database: DatabaseType.none,
        description: 'テスト',
      );
      expect(() => config.validate(), throwsA(isA<ConfigError>()));
    });

    // 先頭ハイフンで ConfigError がスローされることを確認する。
    test('先頭にハイフンがある名前でバリデーションが失敗すること', () {
      const config = ScaffoldConfig(
        name: '-server',
        tier: Tier.system,
        apiStyle: ApiStyle.rest,
        database: DatabaseType.none,
        description: 'テスト',
      );
      expect(() => config.validate(), throwsA(isA<ConfigError>()));
    });

    // 連続ハイフンで ConfigError がスローされることを確認する。
    test('連続ハイフンがある名前でバリデーションが失敗すること', () {
      const config = ScaffoldConfig(
        name: 'my--server',
        tier: Tier.system,
        apiStyle: ApiStyle.rest,
        database: DatabaseType.none,
        description: 'テスト',
      );
      expect(() => config.validate(), throwsA(isA<ConfigError>()));
    });

    // hasGrpc / hasRest / hasDatabase の便利ゲッターが正しく動作することを確認する。
    test('便利ゲッターが正しい値を返すこと', () {
      const config = ScaffoldConfig(
        name: 'my-server',
        tier: Tier.system,
        apiStyle: ApiStyle.both,
        database: DatabaseType.postgres,
        description: 'テスト',
      );
      expect(config.hasGrpc(), isTrue);
      expect(config.hasRest(), isTrue);
      expect(config.hasDatabase(), isTrue);
    });

    // REST のみの場合に hasGrpc が false であることを確認する。
    test('RESTのみの場合にhasGrpcがfalseを返すこと', () {
      const config = ScaffoldConfig(
        name: 'my-server',
        tier: Tier.system,
        apiStyle: ApiStyle.rest,
        database: DatabaseType.none,
        description: 'テスト',
      );
      expect(config.hasGrpc(), isFalse);
      expect(config.hasRest(), isTrue);
      expect(config.hasDatabase(), isFalse);
    });
  });

  group('Tier', () {
    // displayName が正しい値を返すことを確認する。
    test('displayNameが正しい文字列を返すこと', () {
      expect(Tier.system.displayName, 'system');
      expect(Tier.business.displayName, 'business');
      expect(Tier.service.displayName, 'service');
    });
  });

  group('ApiStyle', () {
    // hasGrpc / hasRest が各値で正しく動作することを確認する。
    test('hasGrpcとhasRestが正しい値を返すこと', () {
      expect(ApiStyle.rest.hasGrpc, isFalse);
      expect(ApiStyle.rest.hasRest, isTrue);
      expect(ApiStyle.grpc.hasGrpc, isTrue);
      expect(ApiStyle.grpc.hasRest, isFalse);
      expect(ApiStyle.both.hasGrpc, isTrue);
      expect(ApiStyle.both.hasRest, isTrue);
    });
  });

  group('DatabaseType', () {
    // hasDatabase が各値で正しく動作することを確認する。
    test('hasDatabaseが正しい値を返すこと', () {
      expect(DatabaseType.postgres.hasDatabase, isTrue);
      expect(DatabaseType.none.hasDatabase, isFalse);
    });
  });

  group('Naming', () {
    // toSnakeCase がケバブケースをスネークケースに変換することを確認する。
    test('toSnakeCaseがケバブケースをスネークケースに変換すること', () {
      expect(toSnakeCase('auth-server'), 'auth_server');
      expect(toSnakeCase('my-cool-service'), 'my_cool_service');
      expect(toSnakeCase('single'), 'single');
    });

    // toPascalCase がケバブケースをパスカルケースに変換することを確認する。
    test('toPascalCaseがケバブケースをパスカルケースに変換すること', () {
      expect(toPascalCase('auth-server'), 'AuthServer');
      expect(toPascalCase('my-cool-service'), 'MyCoolService');
      expect(toPascalCase('single'), 'Single');
    });

    // toKebabCase がアンダースコア区切りをケバブケースに変換することを確認する。
    test('toKebabCaseがアンダースコア区切りをケバブケースに変換すること', () {
      expect(toKebabCase('auth_server'), 'auth-server');
      expect(toKebabCase('my_cool_service'), 'my-cool-service');
      expect(toKebabCase('single'), 'single');
    });

    // toCamelCase がケバブケースをキャメルケースに変換することを確認する。
    test('toCamelCaseがケバブケースをキャメルケースに変換すること', () {
      expect(toCamelCase('auth-server'), 'authServer');
      expect(toCamelCase('my-cool-service'), 'myCoolService');
      expect(toCamelCase('single'), 'single');
    });
  });

  group('GenerateResult', () {
    // created と skipped のリストが正しく保持されることを確認する。
    test('生成結果が正しく作成されること', () {
      const result = GenerateResult(
        created: ['file1.dart', 'file2.dart'],
        skipped: ['file3.dart'],
      );
      expect(result.created, hasLength(2));
      expect(result.skipped, hasLength(1));
      expect(result.createdCount, 2);
      expect(result.skippedCount, 1);
    });

    // 空の結果が正しく作成されることを確認する。
    test('空の生成結果が正しく作成されること', () {
      const result = GenerateResult(created: [], skipped: []);
      expect(result.createdCount, 0);
      expect(result.skippedCount, 0);
    });
  });

  group('CodegenError', () {
    // CodegenError のサブクラスが正しいコードを持つことを確認する。
    test('エラーサブクラスが正しいコードを持つこと', () {
      const configErr = ConfigError('bad config');
      expect(configErr.code, 'CONFIG_ERROR');
      expect(configErr.message, 'bad config');

      const templateErr = TemplateError('bad template');
      expect(templateErr.code, 'TEMPLATE_ERROR');

      const ioErr = IoError('file not found');
      expect(ioErr.code, 'IO_ERROR');
    });

    // CodegenError が Exception を実装していることを確認する。
    test('CodegenErrorがExceptionを実装していること', () {
      const err = CodegenError('test error');
      expect(err, isA<Exception>());
    });
  });
}
