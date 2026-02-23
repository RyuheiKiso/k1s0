namespace K1s0.System.Idempotency;

public class DuplicateKeyException(string key)
    : Exception($"Duplicate key: {key}")
{
    public string Key { get; } = key;
}
