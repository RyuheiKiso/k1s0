// Dapr State API を呼ぶ薄いアダプタ。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md

using System.Net;
using System.Net.Http;
using System.Threading.Tasks;
using K1s0.Legacy.Sidecar.App_Start;
using K1s0.Legacy.Sidecar.Models;

namespace K1s0.Legacy.Sidecar.Services
{
    public class DaprClientAdapter
    {
        // Dapr State API GET /v1.0/state/{store-name}/{key} を呼ぶ。
        public async Task<StateValue> GetStateAsync(string key)
        {
            var url = string.Format("/v1.0/state/{0}/{1}", DaprConfig.StateStore, key);
            using (var req = new HttpRequestMessage(HttpMethod.Get, url))
            using (var res = await DaprConfig.Http.SendAsync(req).ConfigureAwait(false))
            {
                // 204 No Content は未存在として扱う。
                if (res.StatusCode == HttpStatusCode.NoContent)
                {
                    return new StateValue { Found = false };
                }
                if (!res.IsSuccessStatusCode)
                {
                    throw new HttpRequestException("Dapr returned " + ((int)res.StatusCode));
                }
                var body = await res.Content.ReadAsStringAsync().ConfigureAwait(false);
                IEnumerable<string> etagValues;
                string etag = res.Headers.TryGetValues("ETag", out etagValues) ? string.Join(",", etagValues) : null;
                return new StateValue { Found = true, Data = body, Etag = etag };
            }
        }

        // Dapr State API POST /v1.0/state/{store-name} で 1 件保存する。
        public async Task SaveStateAsync(string key, string data)
        {
            var url = string.Format("/v1.0/state/{0}", DaprConfig.StateStore);
            // Dapr は配列形式の JSON を要求する: [{"key":"...","value":"..."}]
            var payload = string.Format("[{{\"key\":\"{0}\",\"value\":{1}}}]",
                key, Newtonsoft.Json.JsonConvert.SerializeObject(data));
            using (var content = new StringContent(payload, System.Text.Encoding.UTF8, "application/json"))
            using (var res = await DaprConfig.Http.PostAsync(url, content).ConfigureAwait(false))
            {
                if (!res.IsSuccessStatusCode)
                {
                    throw new HttpRequestException("Dapr returned " + ((int)res.StatusCode));
                }
            }
        }
    }
}
