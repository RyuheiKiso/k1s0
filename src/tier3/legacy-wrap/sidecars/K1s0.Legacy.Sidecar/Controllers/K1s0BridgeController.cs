// 既存 .NET Framework アプリから k1s0 公開 API を呼ぶための HTTP 薄ラッパー Controller。
//
// 既存アプリは本 Controller の HTTP エンドポイント（/api/k1s0bridge/state/{key}）を叩き、
// 内部で Dapr 経由で k1s0 State API に転送する。Dapr / mTLS / 認証は sidecar 側で透過処理される。

using System.Net;
using System.Net.Http;
using System.Threading.Tasks;
using System.Web.Http;
using K1s0.Legacy.Sidecar.Models;
using K1s0.Legacy.Sidecar.Services;

namespace K1s0.Legacy.Sidecar.Controllers
{
    [RoutePrefix("api/k1s0bridge")]
    public class K1s0BridgeController : ApiController
    {
        private readonly DaprClientAdapter _dapr;

        public K1s0BridgeController()
        {
            _dapr = new DaprClientAdapter();
        }

        // GET /api/k1s0bridge/state/{key}
        [HttpGet]
        [Route("state/{key}")]
        public async Task<IHttpActionResult> GetState(string key)
        {
            try
            {
                var value = await _dapr.GetStateAsync(key).ConfigureAwait(false);
                if (!value.Found)
                {
                    return NotFound();
                }
                return Ok(value);
            }
            catch (HttpRequestException)
            {
                return Content(HttpStatusCode.BadGateway, new { error = "dapr request failed" });
            }
        }

        // POST /api/k1s0bridge/state/{key}（body: 文字列値）。
        [HttpPost]
        [Route("state/{key}")]
        public async Task<IHttpActionResult> SaveState(string key, [FromBody] string value)
        {
            try
            {
                await _dapr.SaveStateAsync(key, value).ConfigureAwait(false);
                return Ok();
            }
            catch (HttpRequestException)
            {
                return Content(HttpStatusCode.BadGateway, new { error = "dapr request failed" });
            }
        }

        // GET /api/k1s0bridge/healthz : sidecar 自身の生存確認。
        [HttpGet]
        [Route("healthz")]
        public IHttpActionResult Healthz()
        {
            return Ok(new { status = "ok" });
        }
    }
}
