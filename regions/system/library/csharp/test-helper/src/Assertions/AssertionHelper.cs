using System.Text.Json;

namespace K1s0.System.TestHelper.Assertions;

/// <summary>テスト用アサーションヘルパー。</summary>
public static class AssertionHelper
{
    /// <summary>JSON 部分一致アサーション。</summary>
    public static void AssertJsonContains(string actual, string expected)
    {
        var actualDoc = JsonDocument.Parse(actual);
        var expectedDoc = JsonDocument.Parse(expected);
        if (!JsonContains(actualDoc.RootElement, expectedDoc.RootElement))
        {
            throw new InvalidOperationException(
                $"JSON partial match failed.\nActual: {actual}\nExpected: {expected}");
        }
    }

    /// <summary>イベント一覧に指定タイプのイベントが含まれるか検証する。</summary>
    public static void AssertEventEmitted(IEnumerable<JsonElement> events, string eventType)
    {
        var found = events.Any(e =>
            e.TryGetProperty("type", out var t) && t.GetString() == eventType);
        if (!found)
        {
            throw new InvalidOperationException($"Event '{eventType}' not found in events");
        }
    }

    private static bool JsonContains(JsonElement actual, JsonElement expected)
    {
        if (expected.ValueKind == JsonValueKind.Object)
        {
            if (actual.ValueKind != JsonValueKind.Object)
            {
                return false;
            }

            foreach (var prop in expected.EnumerateObject())
            {
                if (!actual.TryGetProperty(prop.Name, out var actualProp))
                {
                    return false;
                }

                if (!JsonContains(actualProp, prop.Value))
                {
                    return false;
                }
            }

            return true;
        }

        if (expected.ValueKind == JsonValueKind.Array)
        {
            if (actual.ValueKind != JsonValueKind.Array)
            {
                return false;
            }

            foreach (var expectedItem in expected.EnumerateArray())
            {
                var found = false;
                foreach (var actualItem in actual.EnumerateArray())
                {
                    if (JsonContains(actualItem, expectedItem))
                    {
                        found = true;
                        break;
                    }
                }

                if (!found)
                {
                    return false;
                }
            }

            return true;
        }

        return actual.ToString() == expected.ToString();
    }
}
