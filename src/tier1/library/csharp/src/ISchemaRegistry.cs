// ISchemaRegistry.cs — k1s0 tier1 Library C# 実装: Schema Registry の L2* interface
// 12_スキーマレジストリ適合仕様.md §ISchemaRegistryClient（族内共通 L2*）に準拠する。
// Confluent Schema Registry / Apicurio 等の OSS API を Library 独自語彙に翻訳する L2* facade を宣言する。
// 公開シグネチャに OSS 型を一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// SchemaFormat はスキーマフォーマットを宣言する enum。
/// Confluent Schema Registry の schema type に準拠した Library 独自語彙とする。
/// </summary>
// SchemaFormat 列挙型定義
public enum SchemaFormat
{
    /// <summary>Avro: Apache Avro フォーマット</summary>
    Avro,
    /// <summary>Json: JSON Schema フォーマット</summary>
    Json,
    /// <summary>Protobuf: Protocol Buffers フォーマット</summary>
    Protobuf,
}

/// <summary>
/// CompatibilityMode はスキーマ互換性モードを宣言する enum。
/// Confluent Schema Registry の compatibility 設定に準拠した Library 独自語彙とする。
/// </summary>
// CompatibilityMode 列挙型定義
public enum CompatibilityMode
{
    /// <summary>Backward: 後方互換（新スキーマで旧データが読める）</summary>
    Backward,
    /// <summary>Forward: 前方互換（旧スキーマで新データが読める）</summary>
    Forward,
    /// <summary>Full: 双方向互換（Backward + Forward）</summary>
    Full,
    /// <summary>None: 互換性チェックなし（開発環境のみ使用可）</summary>
    None,
}

/// <summary>
/// SchemaReference は別のスキーマを参照する定義を宣言する型。
/// Confluent Schema Registry の SchemaReference に準拠した Library 独自語彙とする。
/// </summary>
// SchemaReference レコード定義
public sealed record SchemaReference(
    string Name,
    string Subject,
    int Version
);

/// <summary>
/// SchemaInfo は Schema Registry に登録されたスキーマ情報を宣言する型。
/// OSS の SchemaInfo レスポンス型を露出せず Library 独自語彙で表現する。
/// </summary>
// SchemaInfo クラス定義
public sealed class SchemaInfo
{
    /// <summary>Id: Schema Registry が発行したスキーマ ID（グローバルユニーク）</summary>
    // Id プロパティ（必須）
    public required long Id { get; init; }

    /// <summary>Subject: スキーマが属する Subject 名</summary>
    // Subject プロパティ（必須）
    public required string Subject { get; init; }

    /// <summary>Version: スキーマバージョン（1 始まりの単調増加整数）</summary>
    // Version プロパティ
    public int Version { get; init; }

    /// <summary>Format: スキーマフォーマット（Avro / JSON / Protobuf）</summary>
    // Format プロパティ（必須）
    public required SchemaFormat Format { get; init; }

    /// <summary>Schema: スキーマ定義文字列（Avro JSON / JSON Schema / .proto 等）</summary>
    // Schema プロパティ（必須）
    public required string Schema { get; init; }

    /// <summary>References: 参照するスキーマの一覧（Protobuf import 等）</summary>
    // References プロパティ（必須）
    public required IReadOnlyList<SchemaReference> References { get; init; }
}

/// <summary>
/// ISchemaRegistryClient は Schema Registry の L2* 抽象 interface を宣言する。
/// Confluent Schema Registry / Apicurio / AWS Glue Schema Registry 等を抽象化する。
/// OSS 型を引数・戻り値に一切含まない。
/// </summary>
// ISchemaRegistryClient インターフェース定義
public interface ISchemaRegistryClient
{
    /// <summary>
    /// RegisterAsync は Subject にスキーマを登録して ID を返す（idempotent 操作）。
    /// </summary>
    // RegisterAsync メソッド: スキーマを登録する
    Task<long> RegisterAsync(string subject, string schema, SchemaFormat format, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetByIdAsync は ID に対応するスキーマ情報を取得する。
    /// スキーマが存在しない場合は null を返す（エラーと区別する）。
    /// </summary>
    // GetByIdAsync メソッド: ID でスキーマを取得する
    Task<SchemaInfo?> GetByIdAsync(long id, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetLatestAsync は Subject の最新バージョンのスキーマ情報を取得する。
    /// </summary>
    // GetLatestAsync メソッド: Subject の最新スキーマを取得する
    Task<SchemaInfo?> GetLatestAsync(string subject, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetByVersionAsync は Subject の指定バージョンのスキーマ情報を取得する。
    /// </summary>
    // GetByVersionAsync メソッド: Subject のバージョン指定でスキーマを取得する
    Task<SchemaInfo?> GetByVersionAsync(string subject, int version, CancellationToken cancellationToken = default);

    /// <summary>
    /// ListSubjectsAsync は登録されている Subject 名の一覧を返す。
    /// </summary>
    // ListSubjectsAsync メソッド: Subject 名の一覧を取得する
    Task<IReadOnlyList<string>> ListSubjectsAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// ListVersionsAsync は Subject のバージョン番号一覧を返す。
    /// </summary>
    // ListVersionsAsync メソッド: Subject のバージョン番号一覧を取得する
    Task<IReadOnlyList<int>> ListVersionsAsync(string subject, CancellationToken cancellationToken = default);

    /// <summary>
    /// CheckCompatibilityAsync は新しいスキーマが Subject の既存スキーマと互換性があるかチェックする。
    /// </summary>
    // CheckCompatibilityAsync メソッド: スキーマ互換性をチェックする
    Task<bool> CheckCompatibilityAsync(string subject, string schema, SchemaFormat format, CancellationToken cancellationToken = default);

    /// <summary>
    /// SetCompatibilityModeAsync は Subject のスキーマ互換性モードを設定する。
    /// </summary>
    // SetCompatibilityModeAsync メソッド: スキーマ互換性モードを設定する
    Task SetCompatibilityModeAsync(string subject, CompatibilityMode mode, CancellationToken cancellationToken = default);

    /// <summary>
    /// DeleteSubjectAsync は Subject を全バージョン削除する。
    /// permanent=true の場合は hard delete（復元不可）、false は soft delete。
    /// </summary>
    // DeleteSubjectAsync メソッド: Subject を削除する
    Task DeleteSubjectAsync(string subject, bool permanent, CancellationToken cancellationToken = default);
}

/// <summary>
/// ISchemaCodec はスキーマを使ってメッセージをエンコード / デコードする L2* interface を宣言する。
/// Confluent Wire Format（magic byte + schema ID）を Library 独自語彙で抽象化する。
/// </summary>
// ISchemaCodec インターフェース定義
public interface ISchemaCodec
{
    /// <summary>
    /// EncodeAsync はスキーマ ID とデータバイト列から Confluent Wire Format のバイト列を生成する。
    /// </summary>
    // EncodeAsync メソッド: Confluent Wire Format にエンコードする
    Task<byte[]> EncodeAsync(long schemaId, byte[] data, CancellationToken cancellationToken = default);

    /// <summary>
    /// DecodeAsync は Confluent Wire Format のバイト列からスキーマ ID とデータバイト列を取り出す。
    /// </summary>
    // DecodeAsync メソッド: Confluent Wire Format からデコードする
    Task<(long SchemaId, byte[] Data)> DecodeAsync(byte[] wireFormatBytes, CancellationToken cancellationToken = default);
}
