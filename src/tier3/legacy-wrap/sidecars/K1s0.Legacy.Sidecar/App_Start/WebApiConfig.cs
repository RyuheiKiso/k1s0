// ASP.NET Web API 設定（ルート登録）。

using System.Web.Http;

namespace K1s0.Legacy.Sidecar.App_Start
{
    public static class WebApiConfig
    {
        public static void Register(HttpConfiguration config)
        {
            // 属性ルーティングを有効化する。
            config.MapHttpAttributeRoutes();

            // 既定のルート（/api/{controller}/{id}）を登録する。
            config.Routes.MapHttpRoute(
                name: "DefaultApi",
                routeTemplate: "api/{controller}/{id}",
                defaults: new { id = RouteParameter.Optional });
        }
    }
}
