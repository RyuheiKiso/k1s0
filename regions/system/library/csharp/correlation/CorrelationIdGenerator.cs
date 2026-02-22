using System.Security.Cryptography;

namespace K1s0.System.Correlation;

public static class CorrelationIdGenerator
{
    public static string NewCorrelationId() => Guid.NewGuid().ToString();

    public static string NewTraceId()
    {
        Span<byte> bytes = stackalloc byte[16];
        RandomNumberGenerator.Fill(bytes);
        return Convert.ToHexStringLower(bytes);
    }
}
