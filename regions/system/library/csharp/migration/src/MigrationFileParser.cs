using System.Security.Cryptography;
using System.Text;

namespace K1s0.System.Migration;

public enum MigrationDirection
{
    Up,
    Down,
}

public record ParsedMigrationFile(string Version, string Name, MigrationDirection Direction);

public static class MigrationFileParser
{
    public static ParsedMigrationFile? ParseFilename(string filename)
    {
        if (!filename.EndsWith(".sql", StringComparison.Ordinal))
        {
            return null;
        }

        var stem = filename[..^4];

        MigrationDirection direction;
        string rest;

        if (stem.EndsWith(".up", StringComparison.Ordinal))
        {
            direction = MigrationDirection.Up;
            rest = stem[..^3];
        }
        else if (stem.EndsWith(".down", StringComparison.Ordinal))
        {
            direction = MigrationDirection.Down;
            rest = stem[..^5];
        }
        else
        {
            return null;
        }

        var idx = rest.IndexOf('_');
        if (idx <= 0 || idx >= rest.Length - 1)
        {
            return null;
        }

        var version = rest[..idx];
        var name = rest[(idx + 1)..];

        return new ParsedMigrationFile(version, name, direction);
    }

    public static string ComputeChecksum(string content)
    {
        var bytes = SHA256.HashData(Encoding.UTF8.GetBytes(content));
        return Convert.ToHexStringLower(bytes);
    }
}
