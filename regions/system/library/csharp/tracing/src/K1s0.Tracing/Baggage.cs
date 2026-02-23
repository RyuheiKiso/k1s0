namespace K1s0.Tracing;

public class Baggage
{
    private readonly Dictionary<string, string> _entries = new();

    public void Set(string key, string value) => _entries[key] = value;

    public string? Get(string key) => _entries.TryGetValue(key, out var v) ? v : null;

    public IReadOnlyDictionary<string, string> Entries => _entries;

    public string ToHeader() => string.Join(",", _entries.Select(e => $"{e.Key}={e.Value}"));

    public static Baggage FromHeader(string s)
    {
        var baggage = new Baggage();
        if (string.IsNullOrEmpty(s))
        {
            return baggage;
        }

        foreach (var pair in s.Split(','))
        {
            var idx = pair.IndexOf('=');
            if (idx > 0)
            {
                baggage.Set(pair[..idx].Trim(), pair[(idx + 1)..].Trim());
            }
        }

        return baggage;
    }
}
