// Dapr State API の取得結果モデル。

namespace K1s0.Legacy.Sidecar.Models
{
    public class StateValue
    {
        // 値（JSON 文字列の生）。
        public string Data { get; set; }

        // ETag（Dapr が返す optimistic concurrency 用）。
        public string Etag { get; set; }

        // 取得できたか（false なら未存在）。
        public bool Found { get; set; }
    }
}
