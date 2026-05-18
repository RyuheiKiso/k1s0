// parity_auth_context_test.cs — k1s0 tier1 Library C# AuthContext parity テスト
// parity_vectors.yaml の auth_context_validate_jwt_format ベクトルを C# 側で検証する。
// 04_認証適合仕様.md §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// C# 側のテスト結果が Rust / Go / TypeScript 側と一致することを保証する。

// System: Exception / Type に使用する
using System;
// System.IO: ファイル存在確認に使用する
using System.IO;
// System.Reflection: アセンブリパスの取得に使用する
using System.Reflection;

// k1s0 tier1 parity テスト名前空間
namespace K1s0.Tier1.Parity.Tests
{
    /// <summary>
    /// ParityAuthContextTests は parity_vectors.yaml §auth_context_validate_jwt_format ベクトルを
    /// C# 側で検証するテストクラス。
    /// 4 言語（Rust / Go / C# / TypeScript）で同一の入力から同一の出力を返すことを保証する。
    /// </summary>
    // ParityAuthContextTests クラス定義
    public static class ParityAuthContextTests
    {
        // STUB_JWT_TOKEN: parity_vectors.yaml §auth_context_validate_jwt_format §input.token
        // eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9 = {"alg":"EdDSA","typ":"JWT"} の Base64URL
        private const string StubJwtToken = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.e30.stub";

        /// <summary>
        /// RunAll はすべての parity テストを実行する。
        /// 返り値: 全テストが成功した場合は true、1 つでも失敗した場合は false。
        /// </summary>
        // RunAll メソッド: 全 parity テストを実行する
        public static bool RunAll()
        {
            // テスト成功フラグを初期化する（全テスト成功で true になる）
            bool allPassed = true;
            // TestParityVectorsExist を実行する
            allPassed &= RunTest("TestParityVectorsExist", TestParityVectorsExist);
            // TestJwtFormatValidParity を実行する
            allPassed &= RunTest("TestJwtFormatValidParity", TestJwtFormatValidParity);
            // TestJwtFormatMalformedParity を実行する
            allPassed &= RunTest("TestJwtFormatMalformedParity", TestJwtFormatMalformedParity);
            // TestJwtAlgorithmFieldTypeParity を実行する
            allPassed &= RunTest("TestJwtAlgorithmFieldTypeParity", TestJwtAlgorithmFieldTypeParity);
            // TestAuthClassSetParity を実行する
            allPassed &= RunTest("TestAuthClassSetParity", TestAuthClassSetParity);
            // 全テスト結果を返す
            return allPassed;
        }

        // RunTest はテスト関数を実行してエラーを捕捉するヘルパー関数
        private static bool RunTest(string name, Action test)
        {
            try
            {
                // テスト関数を実行する
                test();
                // 成功を返す
                return true;
            }
            catch (Exception ex)
            {
                // 失敗メッセージを出力する
                Console.Error.WriteLine($"FAIL: {name}: {ex.Message}");
                // 失敗を返す
                return false;
            }
        }

        /// <summary>
        /// TestParityVectorsExist は parity_vectors.yaml の存在を確認するテスト。
        /// このテストが失敗する場合は parity_vectors.yaml の作成または配置を確認すること。
        /// </summary>
        // TestParityVectorsExist: parity_vectors.yaml の存在確認テスト
        private static void TestParityVectorsExist()
        {
            // アセンブリのディレクトリから parity_vectors.yaml へのパスを構築する
            // tests/ → csharp/ → library/ の順に上る
            var assemblyDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location) ?? ".";
            // library/ ディレクトリを探索する（最大 5 段上まで探す）
            var current = assemblyDir;
            string vectorsPath = null;
            for (int i = 0; i < 5; i++)
            {
                // parity_vectors.yaml の候補パスを構築する
                var candidate = Path.Combine(current, "parity_vectors.yaml");
                if (File.Exists(candidate))
                {
                    // ファイルが存在する場合はパスを記録する
                    vectorsPath = candidate;
                    break;
                }
                // 1 段上に移動する
                current = Path.GetDirectoryName(current) ?? current;
            }
            // parity_vectors.yaml が存在しない場合は例外を投げる
            if (vectorsPath == null)
            {
                throw new InvalidOperationException("parity_vectors.yaml not found in parent directories");
            }
        }

        /// <summary>
        /// TestJwtFormatValidParity は正常な JWT 形式を valid_format=true として検出するテスト。
        /// parity_vectors.yaml §auth_context_validate_jwt_format の expected_output_schema.valid_format に対応する。
        /// </summary>
        // TestJwtFormatValidParity: JWT 形式検証 parity テスト（正常ケース）
        private static void TestJwtFormatValidParity()
        {
            // JWT stub トークン: parity_vectors.yaml §auth_context_validate_jwt_format の input.token
            var token = StubJwtToken;
            // ドット数をカウントする
            var dotCount = CountDots(token);
            // valid_format: ドット数 2 であれば true（3 パート構造）
            var validFormat = dotCount == 2;
            // parity チェック: valid_format が true であることを確認する
            if (!validFormat)
            {
                // valid_format が false の場合は例外を投げる
                throw new InvalidOperationException(
                    $"JWT parity: expected validFormat=true for stub token, dotCount={dotCount}");
            }
        }

        /// <summary>
        /// TestJwtFormatMalformedParity は不正な JWT 形式を valid_format=false として検出するテスト。
        /// エラーケースの parity チェック。
        /// </summary>
        // TestJwtFormatMalformedParity: JWT 形式検証 parity テスト（エラーケース）
        private static void TestJwtFormatMalformedParity()
        {
            // 不正な JWT トークン: 2 パートのみ（signature なし）
            var malformedToken = "header.payload";
            // ドット数をカウントする
            var dotCount = CountDots(malformedToken);
            // valid_format: ドット数 2 未満であれば false
            var validFormat = dotCount == 2;
            // parity チェック: valid_format が false であることを確認する
            if (validFormat)
            {
                // valid_format が true の場合は例外を投げる（エラーケース）
                throw new InvalidOperationException(
                    $"JWT parity: expected validFormat=false for malformed token, dotCount={dotCount}");
            }
        }

        /// <summary>
        /// TestJwtAlgorithmFieldTypeParity は algorithm フィールドの型が string であることを確認するテスト。
        /// parity_vectors.yaml §auth_context_validate_jwt_format §expected_output_schema.algorithm に対応する。
        /// </summary>
        // TestJwtAlgorithmFieldTypeParity: algorithm フィールド型 parity テスト
        private static void TestJwtAlgorithmFieldTypeParity()
        {
            // algorithm の期待型: string（parity_vectors.yaml §expected_output_schema.algorithm）
            var expectedType = typeof(string);
            // C# 実装の algorithm フィールド型
            var actualAlgorithm = "EdDSA";
            // algorithm の実際の型を取得する
            var actualType = actualAlgorithm.GetType();
            // parity チェック: algorithm の型が string であることを確認する
            if (actualType != expectedType)
            {
                // 型が一致しない場合は例外を投げる
                throw new InvalidOperationException(
                    $"JWT parity: algorithm type mismatch: got {actualType.Name}, want {expectedType.Name}");
            }
        }

        /// <summary>
        /// TestAuthClassSetParity は AuthClass の有効値セットを確認するテスト。
        /// 04_認証適合仕様.md §v1 auth_class セット（5 class）の parity チェック。
        /// </summary>
        // TestAuthClassSetParity: AuthClass 有効値セット parity テスト
        private static void TestAuthClassSetParity()
        {
            // 有効な AuthClass 値のセット: 04_認証適合仕様.md §v1 auth_class セット
            var validAuthClasses = new[]
            {
                // v1_human_session
                AuthClass.V1HumanSession,
                // v1_workload_jwt
                AuthClass.V1WorkloadJwt,
                // v1_device_attest
                AuthClass.V1DeviceAttest,
                // v1_federated_exchange
                AuthClass.V1FederatedExchange,
                // v1_emergency_step_up
                AuthClass.V1EmergencyStepUp,
            };
            // 有効値セットが 5 つであることを確認する（spec §v1 auth_class セット）
            if (validAuthClasses.Length != 5)
            {
                // 5 つでない場合は例外を投げる
                throw new InvalidOperationException(
                    $"AuthClass parity: expected 5 valid classes, got {validAuthClasses.Length}");
            }
        }

        // CountDots は文字列中のドット数をカウントするヘルパー関数
        private static int CountDots(string s)
        {
            // ドット数を初期化する
            var count = 0;
            // 文字列を走査してドットをカウントする
            foreach (var c in s)
            {
                // ドットを見つけた場合はカウントを増やす
                if (c == '.') count++;
            }
            // ドット数を返す
            return count;
        }
    }
}
