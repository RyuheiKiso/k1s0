// ASP.NET Application エントリ。Web API ルーティング登録 + Dapr Client 初期化を行う。

using System.Web.Http;
using K1s0.Legacy.Sidecar.App_Start;

namespace K1s0.Legacy.Sidecar
{
    public class WebApiApplication : System.Web.HttpApplication
    {
        protected void Application_Start()
        {
            // Web API ルートを登録する。
            GlobalConfiguration.Configure(WebApiConfig.Register);
            // Dapr Client 初期化を行う（HttpClient 共有 + endpoint 設定）。
            DaprConfig.Initialize();
        }
    }
}
