// VectorSearchImpl.cs — k1s0 tier1 Library C# 実装: IVectorSearchClient の Npgsql + pgvector facade 実装
// 15_ベクトル検索適合仕様.md §IVectorSearchClient（pgvector / Qdrant L1+ 深耕）に準拠する。
// Npgsql と pgvector extension を L1+ ラップして公開 API に Npgsql 型を露出しない。
// TenantId フィルタリングを全クエリで強制して tenant 分離を保証する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList / IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.Linq: LINQ 拡張メソッドに使用する
using System.Linq;
// System.Text: Encoding に使用する
using System.Text;
// System.Text.Json: JSON シリアライズ/デシリアライズに使用する
using System.Text.Json;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Npgsql: PostgreSQL / pgvector クライアント（内部のみ使用する）
using Npgsql;
// Dapper: 軽量 ORM（クエリマッピングに使用する）
using Dapper;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// VectorSearchImpl は IVectorSearchClient の Npgsql + pgvector facade 実装クラス。
/// PostgreSQL の pgvector extension を L1+ ラップして公開 API に Npgsql 型を露出しない。
/// TenantId フィルタリングを全クエリで強制して tenant 分離を保証する。
/// </summary>
// VectorSearchImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class VectorSearchImpl : IVectorSearchClient
{
    // _dataSource: Npgsql データソース（pgvector 接続に使用する）
    private readonly NpgsqlDataSource _dataSource;

    /// <summary>
    /// コンストラクタ: NpgsqlDataSource を注入する。
    /// NpgsqlDataSource は pgvector extension が有効な PostgreSQL への接続に使用する。
    /// </summary>
    // コンストラクタ: NpgsqlDataSource を依存注入する
    public VectorSearchImpl(NpgsqlDataSource dataSource)
    {
        // null チェック: dataSource が null の場合は例外を投げる
        _dataSource = dataSource ?? throw new ArgumentNullException(nameof(dataSource));
    }

    // ToDistanceOp は VectorDistanceMetric を pgvector の演算子文字列に変換する
    private static string ToDistanceOp(VectorDistanceMetric metric)
    {
        // 各距離計算方法を pgvector の演算子にマッピングする
        return metric switch
        {
            // Cosine: <=> 演算子（コサイン距離）
            VectorDistanceMetric.Cosine => "<=>",
            // L2: <-> 演算子（ユークリッド距離）
            VectorDistanceMetric.L2 => "<->",
            // Dot: <#> 演算子（内積の負の値）
            VectorDistanceMetric.Dot => "<#>",
            // L1: <+> 演算子（マンハッタン距離）
            VectorDistanceMetric.L1 => "<+>",
            // デフォルトはコサイン距離
            _ => "<=>",
        };
    }

    // ToIndexType は VectorIndexType を pgvector のインデックス種別文字列に変換する
    private static string ToIndexType(VectorIndexType indexType)
    {
        // 各インデックス種別を pgvector のインデックス種別にマッピングする
        return indexType switch
        {
            // Hnsw: hnsw インデックス（高速 ANN）
            VectorIndexType.Hnsw => "hnsw",
            // IvfFlat: ivfflat インデックス（大規模 ANN）
            VectorIndexType.IvfFlat => "ivfflat",
            // Exact: インデックスなし（Brute Force）
            _ => "none",
        };
    }

    // VectorToSql は float[] を pgvector の SQL 文字列に変換する
    private static string VectorToSql(float[] vector)
    {
        // "[1.0, 2.0, 3.0]" 形式の pgvector SQL 文字列に変換する
        return $"[{string.Join(",", vector.Select(v => v.ToString("F6")))}]";
    }

    /// <summary>
    /// CreateCollectionAsync はベクトルコレクションを作成する（idempotent 操作推奨）。
    /// PostgreSQL テーブルとして pgvector 列を持つコレクションを作成する。
    /// </summary>
    // CreateCollectionAsync メソッド実装: CREATE TABLE IF NOT EXISTS で pgvector テーブルを作成する
    public async Task CreateCollectionAsync(VectorCollectionConfig config, CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // pgvector extension を有効化する（冪等: 既に有効な場合はスキップする）
        await conn.ExecuteAsync("CREATE EXTENSION IF NOT EXISTS vector").ConfigureAwait(false);
        // テーブルを作成する（idempotent: 既に存在する場合はスキップする）
        var createTableSql = $@"
CREATE TABLE IF NOT EXISTS ""{config.Name}"" (
    id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    vector vector({config.Dimension}),
    payload JSONB NOT NULL DEFAULT '{{}}',
    PRIMARY KEY (id, tenant_id)
)";
        // CREATE TABLE を実行する
        await conn.ExecuteAsync(createTableSql).ConfigureAwait(false);
        // インデックス種別が none でない場合はインデックスを作成する
        if (config.IndexType != VectorIndexType.Exact)
        {
            // インデックス種別を取得する
            var indexType = ToIndexType(config.IndexType);
            // 距離計算方法に対応する pgvector ops を取得する
            var ops = config.DistanceMetric switch
            {
                // Cosine: vector_cosine_ops
                VectorDistanceMetric.Cosine => "vector_cosine_ops",
                // L2: vector_l2_ops
                VectorDistanceMetric.L2 => "vector_l2_ops",
                // Dot: vector_ip_ops（内積）
                VectorDistanceMetric.Dot => "vector_ip_ops",
                // L1: vector_l1_ops
                _ => "vector_l1_ops",
            };
            // インデックスを作成する
            var createIndexSql = $@"
CREATE INDEX IF NOT EXISTS ""{config.Name}_vector_idx""
ON ""{config.Name}"" USING {indexType} (vector {ops})";
            // CREATE INDEX を実行する
            await conn.ExecuteAsync(createIndexSql).ConfigureAwait(false);
        }
    }

    /// <summary>
    /// DeleteCollectionAsync はコレクションを削除する（idempotent 操作）。
    /// PostgreSQL テーブルを DROP する。
    /// </summary>
    // DeleteCollectionAsync メソッド実装: DROP TABLE IF EXISTS を実行する
    public async Task DeleteCollectionAsync(string collectionName, CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // DROP TABLE IF EXISTS を実行する（idempotent）
        await conn.ExecuteAsync($@"DROP TABLE IF EXISTS ""{collectionName}""").ConfigureAwait(false);
    }

    /// <summary>
    /// UpsertAsync はポイントをコレクションに追加 / 更新する。
    /// points の TenantId は AuthContext.TenantId と一致する必要がある。
    /// PostgreSQL の INSERT ON CONFLICT DO UPDATE を使用して idempotent に処理する。
    /// </summary>
    // UpsertAsync メソッド実装: INSERT ON CONFLICT DO UPDATE で pgvector ポイントをアップサートする
    public async Task UpsertAsync(string collectionName, IReadOnlyList<VectorPoint> points, CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // 各ポイントを UPSERT する
        foreach (var point in points)
        {
            // ベクトルを SQL 文字列に変換する
            var vectorSql = VectorToSql(point.Vector);
            // Payload を JSON にシリアライズする
            var payloadJson = JsonSerializer.Serialize(point.Payload);
            // INSERT ON CONFLICT DO UPDATE を実行する
            var sql = $@"
INSERT INTO ""{collectionName}"" (id, tenant_id, vector, payload)
VALUES (@Id, @TenantId, @Vector::vector, @Payload::jsonb)
ON CONFLICT (id, tenant_id) DO UPDATE SET
    vector = EXCLUDED.vector,
    payload = EXCLUDED.payload";
            // パラメータを設定して実行する
            await conn.ExecuteAsync(sql, new
            {
                // ID を設定する
                Id = point.Id,
                // TenantId を設定する
                TenantId = point.TenantId,
                // ベクトル SQL 文字列を設定する
                Vector = vectorSql,
                // Payload JSON を設定する
                Payload = payloadJson,
            }).ConfigureAwait(false);
        }
    }

    /// <summary>
    /// DeleteAsync はポイントを ID リストで削除する。
    /// TenantId フィルタリングで tenant 境界を保証する。
    /// </summary>
    // DeleteAsync メソッド実装: DELETE WHERE id IN (...) AND tenant_id = @TenantId を実行する
    public async Task DeleteAsync(string collectionName, string tenantId, IReadOnlyList<string> ids, CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // DELETE WHERE id IN (...) AND tenant_id = @TenantId を実行する
        var sql = $@"DELETE FROM ""{collectionName}"" WHERE id = ANY(@Ids) AND tenant_id = @TenantId";
        // パラメータを設定して実行する
        await conn.ExecuteAsync(sql, new
        {
            // ID リストを設定する
            Ids = ids.ToArray(),
            // TenantId を設定する（tenant 分離強制）
            TenantId = tenantId,
        }).ConfigureAwait(false);
    }

    /// <summary>
    /// SearchAsync は近傍ベクトル検索を実行して結果を返す。
    /// TenantId フィルタリングを強制する（tenant 分離必須）。
    /// </summary>
    // SearchAsync メソッド実装: pgvector の距離演算子で近傍検索を実行する
    public async Task<IReadOnlyList<VectorSearchResult>> SearchAsync(
        string collectionName,
        string tenantId,
        VectorSearchQuery query,
        CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // クエリベクトルを SQL 文字列に変換する
        var queryVectorSql = VectorToSql(query.QueryVector);
        // 距離演算子を取得する（デフォルト: コサイン距離）
        const string distanceOp = "<=>";
        // SQL クエリを構築する（TenantId フィルタリングを強制する）
        var sql = $@"
SELECT
    id,
    tenant_id,
    1 - (vector {distanceOp} @QueryVector::vector) AS score,
    payload
FROM ""{collectionName}""
WHERE tenant_id = @TenantId
ORDER BY vector {distanceOp} @QueryVector::vector
LIMIT @TopK";
        // Dapper でクエリを実行する
        var rows = await conn.QueryAsync(sql, new
        {
            // クエリベクトル SQL 文字列を設定する
            QueryVector = queryVectorSql,
            // TenantId を設定する（tenant 分離強制）
            TenantId = tenantId,
            // TopK を設定する
            TopK = query.TopK,
        }).ConfigureAwait(false);
        // 結果を VectorSearchResult リストに変換する
        var results = new List<VectorSearchResult>();
        // 各行を VectorSearchResult に変換する
        foreach (var row in rows)
        {
            // score のしきい値フィルタリングを適用する
            float score = (float)(double)row.score;
            // スコアがしきい値未満の場合はスキップする
            if (query.ScoreThreshold > 0 && score < query.ScoreThreshold) continue;
            // Payload を IReadOnlyDictionary に変換する（WithPayload が true の場合のみ）
            IReadOnlyDictionary<string, object?>? payload = null;
            // WithPayload が true の場合は Payload を取得する
            if (query.WithPayload)
            {
                // JSON をデシリアライズする
                payload = JsonSerializer.Deserialize<Dictionary<string, object?>>((string)row.payload)
                    as IReadOnlyDictionary<string, object?>;
            }
            // VectorSearchResult を構築して追加する
            results.Add(new VectorSearchResult
            {
                // ID を設定する
                Id = (string)row.id,
                // TenantId を設定する
                TenantId = (string)row.tenant_id,
                // スコアを設定する
                Score = score,
                // Payload を設定する
                Payload = payload,
            });
        }
        // 結果リストを返す
        return results.AsReadOnly();
    }

    /// <summary>
    /// GetByIdAsync は ID でポイントを取得する（ID + TenantId でフィルタリングする）。
    /// ポイントが存在しない場合は null を返す（エラーと区別する）。
    /// </summary>
    // GetByIdAsync メソッド実装: SELECT WHERE id = @Id AND tenant_id = @TenantId を実行する
    public async Task<VectorPoint?> GetByIdAsync(string collectionName, string tenantId, string id, CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // SELECT WHERE id = @Id AND tenant_id = @TenantId を実行する
        var sql = $@"SELECT id, tenant_id, vector::text, payload FROM ""{collectionName}"" WHERE id = @Id AND tenant_id = @TenantId";
        // Dapper でクエリを実行する
        var row = await conn.QuerySingleOrDefaultAsync(sql, new { Id = id, TenantId = tenantId }).ConfigureAwait(false);
        // 行が存在しない場合は null を返す
        if (row is null) return null;
        // ベクトル文字列を float[] に変換する
        var vectorStr = (string)row.vector;
        // "[1.0, 2.0, 3.0]" 形式からパースする
        var vector = vectorStr.Trim('[', ']')
            .Split(',')
            .Select(s => float.Parse(s.Trim()))
            .ToArray();
        // Payload を IReadOnlyDictionary に変換する
        var payload = JsonSerializer.Deserialize<Dictionary<string, object?>>((string)row.payload)
            as IReadOnlyDictionary<string, object?>
            ?? new Dictionary<string, object?>();
        // VectorPoint を構築して返す
        return new VectorPoint
        {
            // ID を設定する
            Id = (string)row.id,
            // TenantId を設定する
            TenantId = (string)row.tenant_id,
            // ベクトルを設定する
            Vector = vector,
            // Payload を設定する
            Payload = payload,
        };
    }

    /// <summary>
    /// CountAsync はコレクション内の TenantId に属するポイント数を返す。
    /// </summary>
    // CountAsync メソッド実装: SELECT COUNT(*) WHERE tenant_id = @TenantId を実行する
    public async Task<long> CountAsync(string collectionName, string tenantId, CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // SELECT COUNT(*) WHERE tenant_id = @TenantId を実行する
        var sql = $@"SELECT COUNT(*) FROM ""{collectionName}"" WHERE tenant_id = @TenantId";
        // Dapper でクエリを実行する
        return await conn.QuerySingleAsync<long>(sql, new { TenantId = tenantId }).ConfigureAwait(false);
    }
}
