// 本ファイルは K1s0.Sdk.Grpc の公開エントリ（薄い placeholder）。
// 動詞統一 facade（k1s0.State.SaveAsync 等）はロードマップ #8 で追加予定。

// 生成 stub の名前空間（K1s0.Sdk.Generated）への利用パスを示すための namespace
namespace K1s0.Sdk;

// SDK バージョン情報を返す静的クラス
public static class SdkInfo
{
    // SDK 版本（package version と整合）
    public const string Version = "0.1.0";

    // ターゲット tier1 API の正典バージョン（v1）
    public const string Tier1ApiVersion = "v1";
}
