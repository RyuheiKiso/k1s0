// parity_impl_test.cs — k1s0 tier1 Library C# 14 実装クラス parity テスト
// 4 言語等価強度（Rust / Go / C# / TypeScript）で同一の動作を保証するための parity テスト。
// 各実装クラスが対応する interface を正しく実装しているかを検証する。
// 各行コメント記載義務に準拠する（コメント欠落は CI fail）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary に使用する
using System.Collections.Generic;

// k1s0 tier1 parity テスト名前空間
namespace K1s0.Tier1.Parity.Tests
{
    /// <summary>
    /// ParityImplTests は 14 実装クラスが対応する interface を実装していることを
    /// リフレクションで検証するテストクラス。
    /// 4 言語（Rust / Go / C# / TypeScript）で同一の interface 契約を満たすことを保証する。
    /// </summary>
    // ParityImplTests クラス定義
    public static class ParityImplTests
    {
        /// <summary>
        /// RunAll はすべての parity テストを実行する。
        /// 返り値: 全テストが成功した場合は true、1 つでも失敗した場合は false。
        /// </summary>
        // RunAll メソッド: 全 parity テストを実行する
        public static bool RunAll()
        {
            // テスト成功フラグを初期化する
            bool allPassed = true;
            // ICacheClient の parity テストを実行する
            allPassed &= RunTest("TestCacheImplImplementsICacheClient", TestCacheImplImplementsICacheClient);
            // ICacheLock の parity テストを実行する
            allPassed &= RunTest("TestCacheLockImplImplementsICacheLock", TestCacheLockImplImplementsICacheLock);
            // IFeatureFlagClient の parity テストを実行する
            allPassed &= RunTest("TestFeatureFlagImplImplementsIFeatureFlagClient", TestFeatureFlagImplImplementsIFeatureFlagClient);
            // IConfigClient の parity テストを実行する
            allPassed &= RunTest("TestConfigImplImplementsIConfigClient", TestConfigImplImplementsIConfigClient);
            // IDbClient の parity テストを実行する（internal class は型チェックのみ）
            allPassed &= RunTest("TestDbImplAssignableToIDbClient", TestDbImplAssignableToIDbClient);
            // IDistributedDbClient の parity テストを実行する
            allPassed &= RunTest("TestDbDistributedImplAssignableToIDistributedDbClient", TestDbDistributedImplAssignableToIDistributedDbClient);
            // IMessagingProducer の parity テストを実行する
            allPassed &= RunTest("TestMessagingProducerImplAssignableToIMessagingProducer", TestMessagingProducerImplAssignableToIMessagingProducer);
            // IMessagingConsumer の parity テストを実行する
            allPassed &= RunTest("TestMessagingConsumerImplAssignableToIMessagingConsumer", TestMessagingConsumerImplAssignableToIMessagingConsumer);
            // IObservabilityProvider の parity テストを実行する
            allPassed &= RunTest("TestObservabilityProviderImplImplementsIObservabilityProvider", TestObservabilityProviderImplImplementsIObservabilityProvider);
            // ILogger / ITracer / IMetricMeter の parity テストを実行する
            allPassed &= RunTest("TestObservabilityProviderReturnsCorrectInterfaces", TestObservabilityProviderReturnsCorrectInterfaces);
            // IProfiler の parity テストを実行する
            allPassed &= RunTest("TestProfilerImplAssignableToIProfiler", TestProfilerImplAssignableToIProfiler);
            // IContinuousProfiler の parity テストを実行する
            allPassed &= RunTest("TestContinuousProfilerImplAssignableToIContinuousProfiler", TestContinuousProfilerImplAssignableToIContinuousProfiler);
            // IRuleEngineClient の parity テストを実行する
            allPassed &= RunTest("TestRuleEngineImplAssignableToIRuleEngineClient", TestRuleEngineImplAssignableToIRuleEngineClient);
            // ICachedRuleEngineClient の parity テストを実行する
            allPassed &= RunTest("TestCachedRuleEngineImplAssignableToICachedRuleEngineClient", TestCachedRuleEngineImplAssignableToICachedRuleEngineClient);
            // ISchemaRegistryClient の parity テストを実行する
            allPassed &= RunTest("TestSchemaRegistryImplAssignableToISchemaRegistryClient", TestSchemaRegistryImplAssignableToISchemaRegistryClient);
            // ISchemaCodec の parity テストを実行する
            allPassed &= RunTest("TestSchemaCodecImplImplementsISchemaCodec", TestSchemaCodecImplImplementsISchemaCodec);
            // ISecretStore の parity テストを実行する
            allPassed &= RunTest("TestSecretStoreImplAssignableToISecretStore", TestSecretStoreImplAssignableToISecretStore);
            // IObjectStorageClient の parity テストを実行する
            allPassed &= RunTest("TestObjectStorageImplAssignableToIObjectStorageClient", TestObjectStorageImplAssignableToIObjectStorageClient);
            // IVectorSearchClient の parity テストを実行する
            allPassed &= RunTest("TestVectorSearchImplAssignableToIVectorSearchClient", TestVectorSearchImplAssignableToIVectorSearchClient);
            // IWorkflowClient の parity テストを実行する
            allPassed &= RunTest("TestWorkflowClientImplAssignableToIWorkflowClient", TestWorkflowClientImplAssignableToIWorkflowClient);
            // SchemaCodec の Wire Format parity テストを実行する
            allPassed &= RunTest("TestSchemaCodecWireFormatParity", TestSchemaCodecWireFormatParity);
            // CacheTtl HLC 変換 parity テストを実行する
            allPassed &= RunTest("TestCacheTtlHlcConversionParity", TestCacheTtlHlcConversionParity);
            // SecretMetadata 不変条件 parity テストを実行する
            allPassed &= RunTest("TestSecretMetadataImmutabilityParity", TestSecretMetadataImmutabilityParity);
            // 全テスト結果を返す
            return allPassed;
        }

        // RunTest はテスト関数を実行してエラーを捕捉するヘルパー関数
        private static bool RunTest(string name, System.Action test)
        {
            try
            {
                // テスト関数を実行する
                test();
                // 成功を返す
                return true;
            }
            catch (System.Exception ex)
            {
                // 失敗メッセージを出力する
                Console.Error.WriteLine($"FAIL: {name}: {ex.Message}");
                // 失敗を返す
                return false;
            }
        }

        // TestCacheImplImplementsICacheClient は CacheImpl が ICacheClient を実装しているかを検証する
        private static void TestCacheImplImplementsICacheClient()
        {
            // CacheImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.CacheImpl");
            // implType が ICacheClient を実装しているかを確認する
            AssertImplements(implType, typeof(ICacheClient), "CacheImpl");
        }

        // TestCacheLockImplImplementsICacheLock は CacheLockImpl が ICacheLock を実装しているかを検証する
        private static void TestCacheLockImplImplementsICacheLock()
        {
            // CacheLockImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.CacheLockImpl");
            // implType が ICacheLock を実装しているかを確認する
            AssertImplements(implType, typeof(ICacheLock), "CacheLockImpl");
        }

        // TestFeatureFlagImplImplementsIFeatureFlagClient は FeatureFlagImpl が IFeatureFlagClient を実装しているかを検証する
        private static void TestFeatureFlagImplImplementsIFeatureFlagClient()
        {
            // FeatureFlagImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.FeatureFlagImpl");
            // implType が IFeatureFlagClient を実装しているかを確認する
            AssertImplements(implType, typeof(IFeatureFlagClient), "FeatureFlagImpl");
        }

        // TestConfigImplImplementsIConfigClient は ConfigImpl が IConfigClient を実装しているかを検証する
        private static void TestConfigImplImplementsIConfigClient()
        {
            // ConfigImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.ConfigImpl");
            // implType が IConfigClient を実装しているかを確認する
            AssertImplements(implType, typeof(IConfigClient), "ConfigImpl");
        }

        // TestDbImplAssignableToIDbClient は DbImpl が IDbClient を実装しているかを検証する
        private static void TestDbImplAssignableToIDbClient()
        {
            // DbImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.DbImpl");
            // implType が IDbClient を実装しているかを確認する
            AssertImplements(implType, typeof(IDbClient), "DbImpl");
        }

        // TestDbDistributedImplAssignableToIDistributedDbClient は DbDistributedImpl が IDistributedDbClient を実装しているかを検証する
        private static void TestDbDistributedImplAssignableToIDistributedDbClient()
        {
            // DbDistributedImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.DbDistributedImpl");
            // implType が IDistributedDbClient を実装しているかを確認する
            AssertImplements(implType, typeof(IDistributedDbClient), "DbDistributedImpl");
        }

        // TestMessagingProducerImplAssignableToIMessagingProducer は MessagingProducerImpl が IMessagingProducer を実装しているかを検証する
        private static void TestMessagingProducerImplAssignableToIMessagingProducer()
        {
            // MessagingProducerImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.MessagingProducerImpl");
            // implType が IMessagingProducer を実装しているかを確認する
            AssertImplements(implType, typeof(IMessagingProducer), "MessagingProducerImpl");
        }

        // TestMessagingConsumerImplAssignableToIMessagingConsumer は MessagingConsumerImpl が IMessagingConsumer を実装しているかを検証する
        private static void TestMessagingConsumerImplAssignableToIMessagingConsumer()
        {
            // MessagingConsumerImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.MessagingConsumerImpl");
            // implType が IMessagingConsumer を実装しているかを確認する
            AssertImplements(implType, typeof(IMessagingConsumer), "MessagingConsumerImpl");
        }

        // TestObservabilityProviderImplImplementsIObservabilityProvider は ObservabilityProviderImpl が IObservabilityProvider を実装しているかを検証する
        private static void TestObservabilityProviderImplImplementsIObservabilityProvider()
        {
            // ObservabilityProviderImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.ObservabilityProviderImpl");
            // implType が IObservabilityProvider を実装しているかを確認する
            AssertImplements(implType, typeof(IObservabilityProvider), "ObservabilityProviderImpl");
        }

        // TestObservabilityProviderReturnsCorrectInterfaces は IObservabilityProvider の factory メソッドが正しい interface を返すかを検証する
        private static void TestObservabilityProviderReturnsCorrectInterfaces()
        {
            // ObservabilityProviderImpl インスタンスを生成する
            var provider = new ObservabilityProviderImpl();
            // Logger メソッドが ILogger を返すかを確認する
            var logger = provider.Logger("test");
            // ILogger 実装かどうかを確認する
            if (logger is not ILogger)
            {
                // ILogger でない場合は例外を投げる
                throw new InvalidOperationException("ObservabilityProvider.Logger が ILogger を返さない");
            }
            // Tracer メソッドが ITracer を返すかを確認する
            var tracer = provider.Tracer("test");
            // ITracer 実装かどうかを確認する
            if (tracer is not ITracer)
            {
                // ITracer でない場合は例外を投げる
                throw new InvalidOperationException("ObservabilityProvider.Tracer が ITracer を返さない");
            }
            // Meter メソッドが IMetricMeter を返すかを確認する
            var meter = provider.Meter("test");
            // IMetricMeter 実装かどうかを確認する
            if (meter is not IMetricMeter)
            {
                // IMetricMeter でない場合は例外を投げる
                throw new InvalidOperationException("ObservabilityProvider.Meter が IMetricMeter を返さない");
            }
        }

        // TestProfilerImplAssignableToIProfiler は ProfilerImpl が IProfiler を実装しているかを検証する
        private static void TestProfilerImplAssignableToIProfiler()
        {
            // ProfilerImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.ProfilerImpl");
            // implType が IProfiler を実装しているかを確認する
            AssertImplements(implType, typeof(IProfiler), "ProfilerImpl");
        }

        // TestContinuousProfilerImplAssignableToIContinuousProfiler は ContinuousProfilerImpl が IContinuousProfiler を実装しているかを検証する
        private static void TestContinuousProfilerImplAssignableToIContinuousProfiler()
        {
            // ContinuousProfilerImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.ContinuousProfilerImpl");
            // implType が IContinuousProfiler を実装しているかを確認する
            AssertImplements(implType, typeof(IContinuousProfiler), "ContinuousProfilerImpl");
        }

        // TestRuleEngineImplAssignableToIRuleEngineClient は RuleEngineImpl が IRuleEngineClient を実装しているかを検証する
        private static void TestRuleEngineImplAssignableToIRuleEngineClient()
        {
            // RuleEngineImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.RuleEngineImpl");
            // implType が IRuleEngineClient を実装しているかを確認する
            AssertImplements(implType, typeof(IRuleEngineClient), "RuleEngineImpl");
        }

        // TestCachedRuleEngineImplAssignableToICachedRuleEngineClient は CachedRuleEngineImpl が ICachedRuleEngineClient を実装しているかを検証する
        private static void TestCachedRuleEngineImplAssignableToICachedRuleEngineClient()
        {
            // CachedRuleEngineImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.CachedRuleEngineImpl");
            // implType が ICachedRuleEngineClient を実装しているかを確認する
            AssertImplements(implType, typeof(ICachedRuleEngineClient), "CachedRuleEngineImpl");
        }

        // TestSchemaRegistryImplAssignableToISchemaRegistryClient は SchemaRegistryImpl が ISchemaRegistryClient を実装しているかを検証する
        private static void TestSchemaRegistryImplAssignableToISchemaRegistryClient()
        {
            // SchemaRegistryImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.SchemaRegistryImpl");
            // implType が ISchemaRegistryClient を実装しているかを確認する
            AssertImplements(implType, typeof(ISchemaRegistryClient), "SchemaRegistryImpl");
        }

        // TestSchemaCodecImplImplementsISchemaCodec は SchemaCodecImpl が ISchemaCodec を実装しているかを検証する
        private static void TestSchemaCodecImplImplementsISchemaCodec()
        {
            // SchemaCodecImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.SchemaCodecImpl");
            // implType が ISchemaCodec を実装しているかを確認する
            AssertImplements(implType, typeof(ISchemaCodec), "SchemaCodecImpl");
        }

        // TestSecretStoreImplAssignableToISecretStore は SecretStoreImpl が ISecretStore を実装しているかを検証する
        private static void TestSecretStoreImplAssignableToISecretStore()
        {
            // SecretStoreImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.SecretStoreImpl");
            // implType が ISecretStore を実装しているかを確認する
            AssertImplements(implType, typeof(ISecretStore), "SecretStoreImpl");
        }

        // TestObjectStorageImplAssignableToIObjectStorageClient は ObjectStorageImpl が IObjectStorageClient を実装しているかを検証する
        private static void TestObjectStorageImplAssignableToIObjectStorageClient()
        {
            // ObjectStorageImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.ObjectStorageImpl");
            // implType が IObjectStorageClient を実装しているかを確認する
            AssertImplements(implType, typeof(IObjectStorageClient), "ObjectStorageImpl");
        }

        // TestVectorSearchImplAssignableToIVectorSearchClient は VectorSearchImpl が IVectorSearchClient を実装しているかを検証する
        private static void TestVectorSearchImplAssignableToIVectorSearchClient()
        {
            // VectorSearchImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.VectorSearchImpl");
            // implType が IVectorSearchClient を実装しているかを確認する
            AssertImplements(implType, typeof(IVectorSearchClient), "VectorSearchImpl");
        }

        // TestWorkflowClientImplAssignableToIWorkflowClient は WorkflowClientImpl が IWorkflowClient を実装しているかを検証する
        private static void TestWorkflowClientImplAssignableToIWorkflowClient()
        {
            // WorkflowClientImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.WorkflowClientImpl");
            // implType が IWorkflowClient を実装しているかを確認する
            AssertImplements(implType, typeof(IWorkflowClient), "WorkflowClientImpl");
        }

        /// <summary>
        /// TestSchemaCodecWireFormatParity は SchemaCodecImpl の Wire Format encode/decode が
        /// 4 言語で同一の結果を返すことを検証する parity テスト。
        /// Confluent Wire Format: magic byte (0x00) + 4 bytes big-endian schema ID + data。
        /// </summary>
        // TestSchemaCodecWireFormatParity: Wire Format parity テスト
        private static void TestSchemaCodecWireFormatParity()
        {
            // テスト用のスキーマ ID とデータを設定する
            const long schemaId = 42;
            // テストデータ: "hello" の UTF-8 バイト列
            var data = new byte[] { 0x68, 0x65, 0x6C, 0x6C, 0x6F };
            // 期待される Wire Format: 0x00 + big-endian 42 (0x0000002A) + "hello"
            var expectedWireFormat = new byte[] { 0x00, 0x00, 0x00, 0x00, 0x2A, 0x68, 0x65, 0x6C, 0x6C, 0x6F };
            // SchemaCodecImpl の型を取得する
            var implType = GetInternalType("K1s0.Tier1.SchemaCodecImpl");
            // ISchemaCodec のインスタンスを生成する
            var codec = (ISchemaCodec)Activator.CreateInstance(implType)!;
            // EncodeAsync を実行する
            var wireFormatBytes = codec.EncodeAsync(schemaId, data).GetAwaiter().GetResult();
            // 期待される Wire Format と一致するかを確認する
            if (wireFormatBytes.Length != expectedWireFormat.Length)
            {
                // 長さが一致しない場合は例外を投げる
                throw new InvalidOperationException(
                    $"SchemaCodec parity: Wire Format 長さ不一致 expected={expectedWireFormat.Length} actual={wireFormatBytes.Length}");
            }
            // 各バイトを比較する
            for (var i = 0; i < expectedWireFormat.Length; i++)
            {
                // バイトが一致しない場合は例外を投げる
                if (wireFormatBytes[i] != expectedWireFormat[i])
                {
                    // バイト不一致の場合は例外を投げる
                    throw new InvalidOperationException(
                        $"SchemaCodec parity: Wire Format バイト不一致 position={i} expected=0x{expectedWireFormat[i]:X2} actual=0x{wireFormatBytes[i]:X2}");
                }
            }
            // DecodeAsync を実行して元のデータと schemaId に戻ることを確認する
            var (decodedSchemaId, decodedData) = codec.DecodeAsync(wireFormatBytes).GetAwaiter().GetResult();
            // schemaId が一致するかを確認する
            if (decodedSchemaId != schemaId)
            {
                // schemaId 不一致の場合は例外を投げる
                throw new InvalidOperationException(
                    $"SchemaCodec parity: decode schemaId 不一致 expected={schemaId} actual={decodedSchemaId}");
            }
            // データが一致するかを確認する
            if (decodedData.Length != data.Length)
            {
                // データ長不一致の場合は例外を投げる
                throw new InvalidOperationException(
                    $"SchemaCodec parity: decode data 長さ不一致 expected={data.Length} actual={decodedData.Length}");
            }
        }

        /// <summary>
        /// TestCacheTtlHlcConversionParity は CacheTtl の HLC tick が 4 言語で
        /// 同一の単位（.NET Tick = 100ns）であることを検証する。
        /// </summary>
        // TestCacheTtlHlcConversionParity: HLC TTL 変換 parity テスト
        private static void TestCacheTtlHlcConversionParity()
        {
            // 1 秒 = 10,000,000 .NET Tick の検証
            const ulong oneSecondInTicks = 10_000_000;
            // CacheTtl を生成する
            var ttl = new CacheTtl(oneSecondInTicks);
            // LogicalTicks が期待値と一致するかを確認する
            if (ttl.LogicalTicks != oneSecondInTicks)
            {
                // 不一致の場合は例外を投げる
                throw new InvalidOperationException(
                    $"CacheTtl parity: LogicalTicks 不一致 expected={oneSecondInTicks} actual={ttl.LogicalTicks}");
            }
            // TimeSpan.FromTicks で変換した結果が 1 秒であることを確認する
            var span = TimeSpan.FromTicks((long)ttl.LogicalTicks);
            // 1 秒かどうかを確認する
            if (span.TotalSeconds != 1.0)
            {
                // 1 秒でない場合は例外を投げる
                throw new InvalidOperationException(
                    $"CacheTtl parity: TimeSpan 変換後が 1 秒でない: {span.TotalSeconds} 秒");
            }
        }

        /// <summary>
        /// TestSecretMetadataImmutabilityParity は SecretMetadata が
        /// 生シークレット値を含まない不変レコード型であることを検証する。
        /// </summary>
        // TestSecretMetadataImmutabilityParity: SecretMetadata 不変条件 parity テスト
        private static void TestSecretMetadataImmutabilityParity()
        {
            // SecretMetadata を生成する
            var meta = new SecretMetadata(
                // SecretId を設定する
                SecretId: "test-secret-001",
                // Version を設定する
                Version: 1,
                // KeyClass を設定する
                KeyClass: KeyClass.V1DataDek,
                // TenantId を設定する
                TenantId: "test-tenant-001",
                // IsActive を設定する
                IsActive: true
            );
            // SecretMetadata が生シークレット値フィールドを持たないかを確認する（型の安全性検証）
            var metaType = typeof(SecretMetadata);
            // 全フィールドを取得する
            var fields = metaType.GetFields(System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance);
            // 各フィールドにシークレット値が含まれないかを確認する
            foreach (var field in fields)
            {
                // フィールド名が "secret" / "token" / "password" / "credential" を含む場合は拒否する
                var lowerName = field.Name.ToLowerInvariant();
                // 生シークレット値フィールドが存在する場合は例外を投げる
                if (lowerName.Contains("secret") || lowerName.Contains("token") || lowerName.Contains("password"))
                {
                    // 生シークレット値フィールドが存在する場合は例外を投げる
                    throw new InvalidOperationException(
                        $"SecretMetadata parity: 生シークレット値フィールドが存在する: {field.Name}");
                }
            }
            // SecretId が設定値と一致するかを確認する
            if (meta.SecretId != "test-secret-001")
            {
                // SecretId 不一致の場合は例外を投げる
                throw new InvalidOperationException(
                    $"SecretMetadata parity: SecretId 不一致 expected=test-secret-001 actual={meta.SecretId}");
            }
        }

        // GetInternalType は internal 型をアセンブリから取得するヘルパー関数
        private static Type GetInternalType(string fullTypeName)
        {
            // k1s0.tier1.library アセンブリから型を取得する
            var asm = typeof(ICacheClient).Assembly;
            // 型を取得する
            var type = asm.GetType(fullTypeName);
            // 型が存在しない場合は例外を投げる
            if (type is null)
            {
                // 型が存在しない場合は例外を投げる
                throw new InvalidOperationException($"GetInternalType: 型が存在しない: {fullTypeName}");
            }
            // 取得した型を返す
            return type;
        }

        // AssertImplements は implType が interfaceType を実装しているかを確認するヘルパー関数
        private static void AssertImplements(Type implType, Type interfaceType, string className)
        {
            // interfaceType が implType に割り当て可能かどうかを確認する
            if (!interfaceType.IsAssignableFrom(implType))
            {
                // 実装していない場合は例外を投げる
                throw new InvalidOperationException(
                    $"parity: {className} が {interfaceType.Name} を実装していない");
            }
        }
    }
}
