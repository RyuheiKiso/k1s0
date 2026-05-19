// SchemaRegistryImpl.cs — k1s0 tier1 Library C# 実装: ISchemaRegistryClient / ISchemaCodec の Apicurio REST API facade 実装
// 12_スキーマレジストリ適合仕様.md §ISchemaRegistryClient（族内共通 L2*）に準拠する。
// Apicurio Schema Registry の REST API を HttpClient でラップして公開 API に OSS SDK 型を露出しない。
// Confluent Wire Format（magic byte 0x00 + 4 bytes schema ID）の encode / decode を実装する。

// System: 基本型に使用する
using System;
// System.Buffers.Binary: BinaryPrimitives（big-endian 変換に使用する）
using System.Buffers.Binary;
// System.Collections.Generic: IReadOnlyList に使用する
using System.Collections.Generic;
// System.Linq: LINQ 拡張メソッドに使用する
using System.Linq;
// System.Net.Http: HttpClient に使用する
using System.Net.Http;
// System.Text: Encoding に使用する
using System.Text;
// System.Text.Json: JSON シリアライズ/デシリアライズに使用する
using System.Text.Json;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// SchemaRegistryImpl は ISchemaRegistryClient の Apicurio REST API facade 実装クラス。
/// Apicurio Schema Registry の REST API を HttpClient でラップして公開 API に OSS SDK 型を露出しない。
/// Confluent Schema Registry 互換 API を使用する（/apis/ccompat/v7 エンドポイント）。
/// </summary>
// SchemaRegistryImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class SchemaRegistryImpl : ISchemaRegistryClient
{
    // _http: Apicurio REST API への HttpClient（内部に隠蔽する）
    private readonly HttpClient _http;
    // _baseUrl: Apicurio Schema Registry のベース URL（例: "http://schema-registry.k1s0.svc:8080"）
    private readonly string _baseUrl;
    // _compatApiBase: Confluent 互換 API のベース URL
    private readonly string _compatApiBase;

    /// <summary>
    /// コンストラクタ: HttpClient と baseUrl を受け取る。
    /// </summary>
    // コンストラクタ: HttpClient と baseUrl を依存注入する
    public SchemaRegistryImpl(HttpClient http, string baseUrl)
    {
        // null チェック: http が null の場合は例外を投げる
        _http = http ?? throw new ArgumentNullException(nameof(http));
        // null チェック: baseUrl が null の場合は例外を投げる
        _baseUrl = baseUrl ?? throw new ArgumentNullException(nameof(baseUrl));
        // Confluent 互換 API のベース URL を構築する
        _compatApiBase = $"{_baseUrl}/apis/ccompat/v7";
    }

    // ToSchemaFormat は SchemaFormat を Apicurio の文字列に変換する
    private static string ToSchemaTypeString(SchemaFormat format)
    {
        // 各 SchemaFormat を Apicurio の schema type 文字列にマッピングする
        return format switch
        {
            // Avro: "AVRO"
            SchemaFormat.Avro => "AVRO",
            // Json: "JSON"
            SchemaFormat.Json => "JSON",
            // Protobuf: "PROTOBUF"
            SchemaFormat.Protobuf => "PROTOBUF",
            // 未知の場合は "AVRO" にフォールバックする
            _ => "AVRO",
        };
    }

    // ParseSchemaInfo は Confluent 互換 API のレスポンス JSON を SchemaInfo に変換する
    private static SchemaInfo? ParseSchemaInfo(string json, string subject)
    {
        // JSON をデシリアライズする
        using var doc = JsonDocument.Parse(json);
        // id フィールドを取得する
        if (!doc.RootElement.TryGetProperty("id", out var idElem)) return null;
        // schema フィールドを取得する
        if (!doc.RootElement.TryGetProperty("schema", out var schemaElem)) return null;
        // SchemaInfo を構築して返す
        var id = idElem.GetInt64();
        // version フィールドを取得する（存在しない場合は 0 を使用する）
        var version = doc.RootElement.TryGetProperty("version", out var versionElem)
            ? versionElem.GetInt32()
            : 0;
        // schemaType フィールドを取得する（存在しない場合は "AVRO" を使用する）
        var schemaType = doc.RootElement.TryGetProperty("schemaType", out var typeElem)
            ? typeElem.GetString() ?? "AVRO"
            : "AVRO";
        // schemaType を SchemaFormat に変換する
        var format = schemaType switch
        {
            // "AVRO" → SchemaFormat.Avro
            "AVRO" => SchemaFormat.Avro,
            // "JSON" → SchemaFormat.Json
            "JSON" => SchemaFormat.Json,
            // "PROTOBUF" → SchemaFormat.Protobuf
            "PROTOBUF" => SchemaFormat.Protobuf,
            // デフォルトは Avro
            _ => SchemaFormat.Avro,
        };
        // SchemaInfo を返す
        return new SchemaInfo
        {
            // スキーマ ID を設定する
            Id = id,
            // Subject を設定する
            Subject = subject,
            // バージョンを設定する
            Version = version,
            // フォーマットを設定する
            Format = format,
            // スキーマ定義を設定する
            Schema = schemaElem.GetString() ?? string.Empty,
            // 参照リストは空で設定する（参照の実装は省略する）
            References = Array.Empty<SchemaReference>(),
        };
    }

    /// <summary>
    /// RegisterAsync は Subject にスキーマを登録して ID を返す（idempotent 操作）。
    /// Confluent 互換 API POST /subjects/{subject}/versions を使用する。
    /// </summary>
    // RegisterAsync メソッド実装: Confluent 互換 API でスキーマを登録する
    public async Task<long> RegisterAsync(string subject, string schema, SchemaFormat format, CancellationToken cancellationToken = default)
    {
        // リクエスト JSON を構築する（Confluent 互換 API 形式）
        var body = JsonSerializer.Serialize(new
        {
            // スキーマ定義を設定する
            schema,
            // スキーマタイプを設定する
            schemaType = ToSchemaTypeString(format),
        });
        // HTTP POST リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Post, $"{_compatApiBase}/subjects/{Uri.EscapeDataString(subject)}/versions")
        {
            // リクエストボディを設定する
            Content = new StringContent(body, Encoding.UTF8, "application/json"),
        };
        // Confluent 互換 API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // id フィールドを取得して返す
        using var doc = JsonDocument.Parse(respJson);
        // id フィールドを返す
        return doc.RootElement.GetProperty("id").GetInt64();
    }

    /// <summary>
    /// GetByIdAsync は ID に対応するスキーマ情報を取得する。
    /// Confluent 互換 API GET /schemas/ids/{id} を使用する。
    /// </summary>
    // GetByIdAsync メソッド実装: Confluent 互換 API でスキーマを ID 検索する
    public async Task<SchemaInfo?> GetByIdAsync(long id, CancellationToken cancellationToken = default)
    {
        // GET /schemas/ids/{id} を呼び出す
        var response = await _http.GetAsync($"{_compatApiBase}/schemas/ids/{id}", cancellationToken).ConfigureAwait(false);
        // 404 の場合は null を返す
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound) return null;
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // SchemaInfo に変換して返す（subject は ID では取得できないため空文字列とする）
        return ParseSchemaInfo($"{{\"id\":{id},{respJson[1..]}", string.Empty);
    }

    /// <summary>
    /// GetLatestAsync は Subject の最新バージョンのスキーマ情報を取得する。
    /// Confluent 互換 API GET /subjects/{subject}/versions/latest を使用する。
    /// </summary>
    // GetLatestAsync メソッド実装: Confluent 互換 API で最新スキーマを取得する
    public async Task<SchemaInfo?> GetLatestAsync(string subject, CancellationToken cancellationToken = default)
    {
        // GET /subjects/{subject}/versions/latest を呼び出す
        var response = await _http.GetAsync($"{_compatApiBase}/subjects/{Uri.EscapeDataString(subject)}/versions/latest", cancellationToken).ConfigureAwait(false);
        // 404 の場合は null を返す
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound) return null;
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // SchemaInfo に変換して返す
        return ParseSchemaInfo(respJson, subject);
    }

    /// <summary>
    /// GetByVersionAsync は Subject の指定バージョンのスキーマ情報を取得する。
    /// </summary>
    // GetByVersionAsync メソッド実装: Confluent 互換 API でバージョン指定スキーマを取得する
    public async Task<SchemaInfo?> GetByVersionAsync(string subject, int version, CancellationToken cancellationToken = default)
    {
        // GET /subjects/{subject}/versions/{version} を呼び出す
        var response = await _http.GetAsync($"{_compatApiBase}/subjects/{Uri.EscapeDataString(subject)}/versions/{version}", cancellationToken).ConfigureAwait(false);
        // 404 の場合は null を返す
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound) return null;
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // SchemaInfo に変換して返す
        return ParseSchemaInfo(respJson, subject);
    }

    /// <summary>
    /// ListSubjectsAsync は登録されている Subject 名の一覧を返す。
    /// </summary>
    // ListSubjectsAsync メソッド実装: Confluent 互換 API で Subject 一覧を取得する
    public async Task<IReadOnlyList<string>> ListSubjectsAsync(CancellationToken cancellationToken = default)
    {
        // GET /subjects を呼び出す
        var response = await _http.GetAsync($"{_compatApiBase}/subjects", cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // JSON 配列をデシリアライズして返す
        return JsonSerializer.Deserialize<List<string>>(respJson) ?? new List<string>();
    }

    /// <summary>
    /// ListVersionsAsync は Subject のバージョン番号一覧を返す。
    /// </summary>
    // ListVersionsAsync メソッド実装: Confluent 互換 API でバージョン一覧を取得する
    public async Task<IReadOnlyList<int>> ListVersionsAsync(string subject, CancellationToken cancellationToken = default)
    {
        // GET /subjects/{subject}/versions を呼び出す
        var response = await _http.GetAsync($"{_compatApiBase}/subjects/{Uri.EscapeDataString(subject)}/versions", cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // JSON 配列をデシリアライズして返す
        return JsonSerializer.Deserialize<List<int>>(respJson) ?? new List<int>();
    }

    /// <summary>
    /// CheckCompatibilityAsync は新しいスキーマが Subject の既存スキーマと互換性があるかチェックする。
    /// </summary>
    // CheckCompatibilityAsync メソッド実装: Confluent 互換 API で互換性をチェックする
    public async Task<bool> CheckCompatibilityAsync(string subject, string schema, SchemaFormat format, CancellationToken cancellationToken = default)
    {
        // リクエスト JSON を構築する
        var body = JsonSerializer.Serialize(new { schema, schemaType = ToSchemaTypeString(format) });
        // HTTP POST リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Post, $"{_compatApiBase}/compatibility/subjects/{Uri.EscapeDataString(subject)}/versions/latest")
        {
            // リクエストボディを設定する
            Content = new StringContent(body, Encoding.UTF8, "application/json"),
        };
        // Confluent 互換 API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // is_compatible フィールドを取得して返す
        using var doc = JsonDocument.Parse(respJson);
        // is_compatible フィールドを返す（存在しない場合は false を返す）
        return doc.RootElement.TryGetProperty("is_compatible", out var compElem) && compElem.GetBoolean();
    }

    /// <summary>
    /// SetCompatibilityModeAsync は Subject のスキーマ互換性モードを設定する。
    /// </summary>
    // SetCompatibilityModeAsync メソッド実装: Confluent 互換 API で互換性モードを設定する
    public async Task SetCompatibilityModeAsync(string subject, CompatibilityMode mode, CancellationToken cancellationToken = default)
    {
        // 互換性モードを Confluent 形式の文字列に変換する
        var compatibility = mode switch
        {
            // Backward: "BACKWARD"
            CompatibilityMode.Backward => "BACKWARD",
            // Forward: "FORWARD"
            CompatibilityMode.Forward => "FORWARD",
            // Full: "FULL"
            CompatibilityMode.Full => "FULL",
            // None: "NONE"
            _ => "NONE",
        };
        // リクエスト JSON を構築する
        var body = JsonSerializer.Serialize(new { compatibility });
        // HTTP PUT リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Put, $"{_compatApiBase}/config/{Uri.EscapeDataString(subject)}")
        {
            // リクエストボディを設定する
            Content = new StringContent(body, Encoding.UTF8, "application/json"),
        };
        // Confluent 互換 API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
    }

    /// <summary>
    /// DeleteSubjectAsync は Subject を全バージョン削除する。
    /// permanent=true の場合は hard delete、false は soft delete。
    /// </summary>
    // DeleteSubjectAsync メソッド実装: Confluent 互換 API で Subject を削除する
    public async Task DeleteSubjectAsync(string subject, bool permanent, CancellationToken cancellationToken = default)
    {
        // URL を構築する（permanent の場合は ?permanent=true クエリを追加する）
        var url = permanent
            ? $"{_compatApiBase}/subjects/{Uri.EscapeDataString(subject)}?permanent=true"
            : $"{_compatApiBase}/subjects/{Uri.EscapeDataString(subject)}";
        // HTTP DELETE リクエストを送信する
        var response = await _http.DeleteAsync(url, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する（404 は idempotent なので無視する）
        if (response.StatusCode != System.Net.HttpStatusCode.NotFound)
        {
            // エラーがある場合は例外を投げる
            response.EnsureSuccessStatusCode();
        }
    }
}

/// <summary>
/// SchemaCodecImpl は ISchemaCodec の Confluent Wire Format 実装クラス。
/// Confluent Wire Format（magic byte 0x00 + 4 bytes big-endian schema ID）を実装する。
/// </summary>
// SchemaCodecImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class SchemaCodecImpl : ISchemaCodec
{
    // _magicByte: Confluent Wire Format の magic byte（常に 0x00）
    private const byte MagicByte = 0x00;
    // _headerSize: Confluent Wire Format のヘッダーサイズ（magic byte 1 + schema ID 4 = 5 bytes）
    private const int HeaderSize = 5;

    /// <summary>
    /// EncodeAsync はスキーマ ID とデータバイト列から Confluent Wire Format のバイト列を生成する。
    /// magic byte (0x00) + 4 bytes big-endian schema ID + data の形式で返す。
    /// </summary>
    // EncodeAsync メソッド実装: Confluent Wire Format にエンコードする
    public Task<byte[]> EncodeAsync(long schemaId, byte[] data, CancellationToken cancellationToken = default)
    {
        // 出力バイト列: magic byte (1) + schema ID (4) + data
        var output = new byte[HeaderSize + data.Length];
        // magic byte を設定する（0x00）
        output[0] = MagicByte;
        // schema ID を big-endian 4 bytes として書き込む
        BinaryPrimitives.WriteInt32BigEndian(output.AsSpan(1, 4), (int)schemaId);
        // データをコピーする
        Array.Copy(data, 0, output, HeaderSize, data.Length);
        // エンコード結果を返す
        return Task.FromResult(output);
    }

    /// <summary>
    /// DecodeAsync は Confluent Wire Format のバイト列からスキーマ ID とデータバイト列を取り出す。
    /// </summary>
    // DecodeAsync メソッド実装: Confluent Wire Format からデコードする
    public Task<(long SchemaId, byte[] Data)> DecodeAsync(byte[] wireFormatBytes, CancellationToken cancellationToken = default)
    {
        // バイト列の長さが最小ヘッダーサイズ以上かどうかを確認する
        if (wireFormatBytes.Length < HeaderSize)
        {
            // 不正なバイト列の場合は例外を投げる
            throw new ArgumentException($"Confluent Wire Format: バイト列が短すぎる（最小 {HeaderSize} bytes 必要）", nameof(wireFormatBytes));
        }
        // magic byte を確認する（0x00 以外は不正）
        if (wireFormatBytes[0] != MagicByte)
        {
            // magic byte 不一致の場合は例外を投げる
            throw new ArgumentException($"Confluent Wire Format: magic byte 不一致（expected 0x00, got 0x{wireFormatBytes[0]:X2}）", nameof(wireFormatBytes));
        }
        // schema ID を big-endian 4 bytes から読み取る
        var schemaId = (long)BinaryPrimitives.ReadInt32BigEndian(wireFormatBytes.AsSpan(1, 4));
        // データ部分をコピーする
        var data = new byte[wireFormatBytes.Length - HeaderSize];
        // データをコピーする
        Array.Copy(wireFormatBytes, HeaderSize, data, 0, data.Length);
        // スキーマ ID とデータを返す
        return Task.FromResult((schemaId, data));
    }
}
