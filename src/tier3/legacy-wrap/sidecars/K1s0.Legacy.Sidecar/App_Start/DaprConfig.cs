// Dapr Client 初期化。HttpClient を singleton で共有する。

using System;
using System.Configuration;
using System.Net.Http;

namespace K1s0.Legacy.Sidecar.App_Start
{
    public static class DaprConfig
    {
        // Dapr sidecar の HTTP endpoint（Pod 内 localhost）。
        public static string DaprEndpoint { get; private set; }

        // k1s0 State Component 名。
        public static string StateStore { get; private set; }

        // 共有 HttpClient（重い初期化を一度だけ行う）。
        public static HttpClient Http { get; private set; }

        public static void Initialize()
        {
            DaprEndpoint = ConfigurationManager.AppSettings["DaprHttpEndpoint"] ?? "http://localhost:3500";
            StateStore = ConfigurationManager.AppSettings["K1s0StateStore"] ?? "postgres";
            Http = new HttpClient
            {
                BaseAddress = new Uri(DaprEndpoint),
                Timeout = TimeSpan.FromSeconds(10),
            };
        }
    }
}
