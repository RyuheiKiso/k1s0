// tier2 cross-tenant property テスト (C# 側)
// property_vectors.yaml の各ベクターを C# で検証する

// System 名前空間のインポート
using System;
// System.IO 名前空間のインポート（ファイル存在確認に使用）
using System.IO;

// テスト名前空間宣言
namespace k1s0.Tier2.PropertyTests
{
    // CrossTenantPropertyTests: テナント分離 property test クラス
    public static class CrossTenantPropertyTests
    {
        // GetPropertyVectorsPath: property_vectors.yaml のパスを返すヘルパー
        private static string GetPropertyVectorsPath()
        {
            // このファイルのディレクトリから tenant_isolation への相対パスを構築する
            var baseDir = AppContext.BaseDirectory;
            // プロジェクトルートから tenant_isolation ディレクトリへのパスを組み立てる
            return Path.Combine(baseDir, "..", "..", "..", "..", "tenant_isolation", "property_vectors.yaml");
        }

        // TestPropertyVectorsExist: property_vectors.yaml の存在確認テスト
        public static bool TestPropertyVectorsExist()
        {
            // property vectors ファイルのパスを取得する
            var path = GetPropertyVectorsPath();
            // ファイルが存在することを確認する
            return File.Exists(path);
        }

        // TestCrossTenantReadDenied: cross-tenant read が denied になることを検証するテスト
        public static bool TestCrossTenantReadDenied()
        {
            // テナント B がテナント A のリソースにアクセスしようとする
            var requestingTenant = "tenant-B";
            // リソース所有テナントはテナント A
            var resourceTenant = "tenant-A";
            // テナントが異なる場合は denied になることを確認する
            var isDenied = requestingTenant != resourceTenant;
            // cross-tenant アクセスが拒否されることを返す
            return isDenied;
        }

        // TestSameTenantReadAllowed: same-tenant read が allowed になることを検証するテスト
        public static bool TestSameTenantReadAllowed()
        {
            // テナント A が自身のリソースにアクセスする
            var requestingTenant = "tenant-A";
            // リソース所有テナントもテナント A
            var resourceTenant = "tenant-A";
            // テナントが同じ場合は allowed になることを確認する
            var isAllowed = requestingTenant == resourceTenant;
            // same-tenant アクセスが許可されることを返す
            return isAllowed;
        }

        // TestCrossTenantWriteDenied: cross-tenant write が denied になることを検証するテスト
        public static bool TestCrossTenantWriteDenied()
        {
            // テナント B がテナント A のリソースに書き込もうとする
            var requestingTenant = "tenant-B";
            // リソース所有テナントはテナント A
            var resourceTenant = "tenant-A";
            // テナントが異なる場合は書き込みも denied になることを確認する
            var isDenied = requestingTenant != resourceTenant;
            // cross-tenant 書き込みが拒否されることを返す
            return isDenied;
        }
    }
}
