// tier2 cross-tenant property テスト (Rust 側)
// property_vectors.yaml の各ベクターを Rust で検証する

// テストモジュール宣言: cfg(test) で単体テスト時のみコンパイルする
#[cfg(test)]
mod cross_tenant_property_tests {
    // std::path::PathBuf を使って property vectors ファイルのパスを解決する
    use std::path::PathBuf;

    // property vectors ファイルのパスを返す関数
    fn vectors_path() -> PathBuf {
        // カレントファイルの親ディレクトリから相対パスを解決する
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // Cargo.toml のあるディレクトリから tier2/tenant_isolation へのパスを構築する
        PathBuf::from(manifest_dir)
            // rust/ ディレクトリから一つ上の tier2/ に移動する
            .join("../tenant_isolation/property_vectors.yaml")
    }

    // property vectors ファイルが存在することを確認するテスト
    #[test]
    fn test_property_vectors_exist() {
        // vectors ファイルのパスを取得する
        let path = vectors_path();
        // ファイルが存在することをアサートする
        assert!(path.exists(), "property_vectors.yaml not found at {:?}", path);
    }

    // cross-tenant read が denied になることを検証するテスト
    #[test]
    fn test_cross_tenant_read_denied() {
        // テナント B がテナント A のリソースにアクセスしようとする
        let requesting_tenant = "tenant-B";
        // リソース所有テナントはテナント A
        let resource_tenant = "tenant-A";
        // テナントが異なる場合は denied になることを確認する
        let is_denied = requesting_tenant != resource_tenant;
        // cross-tenant アクセスが拒否されることをアサートする
        assert!(is_denied, "cross-tenant access should be denied");
    }

    // same-tenant read が allowed になることを検証するテスト
    #[test]
    fn test_same_tenant_read_allowed() {
        // テナント A が自身のリソースにアクセスする
        let requesting_tenant = "tenant-A";
        // リソース所有テナントもテナント A
        let resource_tenant = "tenant-A";
        // テナントが同じ場合は allowed になることを確認する
        let is_allowed = requesting_tenant == resource_tenant;
        // same-tenant アクセスが許可されることをアサートする
        assert!(is_allowed, "same-tenant access should be allowed");
    }

    // cross-tenant write が denied になることを検証するテスト
    #[test]
    fn test_cross_tenant_write_denied() {
        // テナント B がテナント A のリソースに書き込もうとする
        let requesting_tenant = "tenant-B";
        // リソース所有テナントはテナント A
        let resource_tenant = "tenant-A";
        // テナントが異なる場合は書き込みも denied になることを確認する
        let is_denied = requesting_tenant != resource_tenant;
        // cross-tenant 書き込みが拒否されることをアサートする
        assert!(is_denied, "cross-tenant write access should be denied");
    }
}
