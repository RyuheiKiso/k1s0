namespace K1s0.System.ServiceAuth;

public sealed record SpiffeId(string TrustDomain, string Namespace, string ServiceAccount)
{
    private const string Prefix = "spiffe://";

    public static SpiffeId Parse(string uri)
    {
        if (string.IsNullOrWhiteSpace(uri))
        {
            throw new ServiceAuthException("InvalidSpiffeId", "SPIFFE URI must not be empty.");
        }

        if (!uri.StartsWith(Prefix, StringComparison.Ordinal))
        {
            throw new ServiceAuthException("InvalidSpiffeId", $"Invalid SPIFFE URI scheme: {uri}");
        }

        var path = uri[Prefix.Length..];
        var segments = path.Split('/');

        // Expected: <trust-domain>/ns/<namespace>/sa/<service-account>
        if (segments.Length != 5 ||
            segments[1] != "ns" ||
            segments[3] != "sa")
        {
            throw new ServiceAuthException(
                "InvalidSpiffeId",
                $"Invalid SPIFFE URI format. Expected spiffe://<trust-domain>/ns/<ns>/sa/<sa>, got: {uri}");
        }

        var trustDomain = segments[0];
        var ns = segments[2];
        var sa = segments[4];

        if (string.IsNullOrEmpty(trustDomain) || string.IsNullOrEmpty(ns) || string.IsNullOrEmpty(sa))
        {
            throw new ServiceAuthException(
                "InvalidSpiffeId",
                $"SPIFFE URI contains empty segments: {uri}");
        }

        return new SpiffeId(trustDomain, ns, sa);
    }

    public override string ToString() => $"spiffe://{TrustDomain}/ns/{Namespace}/sa/{ServiceAccount}";
}
