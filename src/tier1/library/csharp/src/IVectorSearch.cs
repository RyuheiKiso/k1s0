// IVectorSearch.cs — k1s0 tier1 Library C# 実装: Vector Search の L1+ interface
// 15_ベクトル検索適合仕様.md §IVectorSearchClient（pgvector / Qdrant L1+ 深耕）に準拠する。
// Vector Search の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
// OSS 型（Qdrant.Client 等）を公開シグネチャに一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList / IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// VectorDistanceMetric はベクトル距離計算方法を宣言する enum。
/// pgvector / Qdrant 等の Distance metric に準拠した Library 独自語彙とする。
/// </summary>
// VectorDistanceMetric 列挙型定義
public enum VectorDistanceMetric
{
    /// <summary>Cosine: コサイン類似度（テキスト埋め込みに推奨する）</summary>
    Cosine,
    /// <summary>L2: ユークリッド距離（画像 / 音声埋め込みに推奨する）</summary>
    L2,
    /// <summary>Dot: 内積（正規化済みベクトルで使用する）</summary>
    Dot,
    /// <summary>L1: マンハッタン距離（スパースベクトルに使用する）</summary>
    L1,
}

/// <summary>
/// VectorIndexType はベクトルインデックスの種別を宣言する enum。
/// </summary>
// VectorIndexType 列挙型定義
public enum VectorIndexType
{
    /// <summary>Hnsw: Hierarchical Navigable Small World（高速 ANN: デフォルト推奨）</summary>
    Hnsw,
    /// <summary>IvfFlat: IVF Flat（大規模コレクション向け ANN）</summary>
    IvfFlat,
    /// <summary>Exact: 完全一致（Brute Force: 小規模コレクション向け）</summary>
    Exact,
}

/// <summary>
/// VectorPoint はベクトル検索コレクションに格納する単一ポイントを宣言する型。
/// OSS の PointStruct 等を露出せず Library 独自語彙で表現する。
/// </summary>
// VectorPoint クラス定義
public sealed class VectorPoint
{
    /// <summary>Id: ポイント識別子（UUID v7 形式を推奨する）</summary>
    // Id プロパティ（必須）
    public required string Id { get; init; }

    /// <summary>TenantId: ポイントの所属テナント識別子（必須: tenant 分離フィルタリングに使用する）</summary>
    // TenantId プロパティ（必須）
    public required string TenantId { get; init; }

    /// <summary>Vector: 埋め込みベクトル（次元数はコレクション設定と一致させる）</summary>
    // Vector プロパティ（必須）
    public required float[] Vector { get; init; }

    /// <summary>Payload: ポイントに付与するメタデータ（検索結果に含まれる）</summary>
    // Payload プロパティ（必須）
    public required IReadOnlyDictionary<string, object?> Payload { get; init; }
}

/// <summary>
/// VectorSearchQuery はベクトル近傍検索クエリを宣言する型。
/// </summary>
// VectorSearchQuery クラス定義
public sealed class VectorSearchQuery
{
    /// <summary>QueryVector: クエリベクトル（コレクションと同じ次元数）</summary>
    // QueryVector プロパティ（必須）
    public required float[] QueryVector { get; init; }

    /// <summary>TopK: 返す近傍ポイント数</summary>
    // TopK プロパティ
    public int TopK { get; init; } = 10;

    /// <summary>Filter: メタデータフィルター条件（Payload フィールドに対する AND 条件）</summary>
    // Filter プロパティ
    public IReadOnlyDictionary<string, object?>? Filter { get; init; }

    /// <summary>ScoreThreshold: 最小スコア閾値（0.0 〜 1.0）</summary>
    // ScoreThreshold プロパティ
    public float ScoreThreshold { get; init; }

    /// <summary>WithPayload: 検索結果に Payload を含めるかどうか</summary>
    // WithPayload プロパティ
    public bool WithPayload { get; init; } = true;

    /// <summary>WithVector: 検索結果にベクトルを含めるかどうか</summary>
    // WithVector プロパティ
    public bool WithVector { get; init; }
}

/// <summary>
/// VectorSearchResult はベクトル近傍検索の単一結果を宣言する型。
/// </summary>
// VectorSearchResult クラス定義
public sealed class VectorSearchResult
{
    /// <summary>Id: 近傍ポイント識別子</summary>
    // Id プロパティ（必須）
    public required string Id { get; init; }

    /// <summary>TenantId: 近傍ポイントの所属テナント識別子</summary>
    // TenantId プロパティ（必須）
    public required string TenantId { get; init; }

    /// <summary>Score: クエリとの類似度スコア</summary>
    // Score プロパティ
    public float Score { get; init; }

    /// <summary>Payload: ポイントのメタデータ（WithPayload=true の場合のみ設定される）</summary>
    // Payload プロパティ
    public IReadOnlyDictionary<string, object?>? Payload { get; init; }

    /// <summary>Vector: ポイントのベクトル（WithVector=true の場合のみ設定される）</summary>
    // Vector プロパティ
    public float[]? Vector { get; init; }
}

/// <summary>
/// HnswConfig は HNSW インデックスの詳細設定を宣言する型。
/// </summary>
// HnswConfig クラス定義
public sealed class HnswConfig
{
    /// <summary>M: HNSW グラフのリンク数</summary>
    // M プロパティ
    public int? M { get; init; }

    /// <summary>EfConstruct: 構築時の探索候補数</summary>
    // EfConstruct プロパティ
    public int? EfConstruct { get; init; }

    /// <summary>EfSearch: 検索時の探索候補数</summary>
    // EfSearch プロパティ
    public int? EfSearch { get; init; }
}

/// <summary>
/// VectorCollectionConfig はコレクション設定を宣言する型。
/// </summary>
// VectorCollectionConfig クラス定義
public sealed class VectorCollectionConfig
{
    /// <summary>Name: コレクション名</summary>
    // Name プロパティ（必須）
    public required string Name { get; init; }

    /// <summary>Dimension: ベクトルの次元数</summary>
    // Dimension プロパティ
    public int Dimension { get; init; }

    /// <summary>DistanceMetric: ベクトル距離計算方法（デフォルト: Cosine）</summary>
    // DistanceMetric プロパティ
    public VectorDistanceMetric DistanceMetric { get; init; } = VectorDistanceMetric.Cosine;

    /// <summary>IndexType: インデックス種別（デフォルト: Hnsw）</summary>
    // IndexType プロパティ
    public VectorIndexType IndexType { get; init; } = VectorIndexType.Hnsw;

    /// <summary>HnswConfig: HNSW インデックスの詳細設定（null = デフォルト設定を使用する）</summary>
    // HnswConfig プロパティ
    public HnswConfig? HnswConfig { get; init; }
}

/// <summary>
/// IVectorSearchClient は Vector Search の L1+ 抽象 interface を宣言する。
/// pgvector / Qdrant / Weaviate / Pinecone 等を抽象化する。
/// OSS 型を一切含まない。
/// TenantId を必須引数として受け取る（tenant 分離必須）。
/// </summary>
// IVectorSearchClient インターフェース定義
public interface IVectorSearchClient
{
    /// <summary>
    /// CreateCollectionAsync はベクトルコレクションを作成する（idempotent 操作推奨）。
    /// </summary>
    // CreateCollectionAsync メソッド: コレクションを作成する
    Task CreateCollectionAsync(VectorCollectionConfig config, CancellationToken cancellationToken = default);

    /// <summary>
    /// DeleteCollectionAsync はコレクションを削除する（idempotent 操作）。
    /// </summary>
    // DeleteCollectionAsync メソッド: コレクションを削除する
    Task DeleteCollectionAsync(string collectionName, CancellationToken cancellationToken = default);

    /// <summary>
    /// UpsertAsync はポイントをコレクションに追加 / 更新する。
    /// points の TenantId は AuthContext.TenantId と一致する必要がある。
    /// </summary>
    // UpsertAsync メソッド: ポイントを追加 / 更新する
    Task UpsertAsync(string collectionName, IReadOnlyList<VectorPoint> points, CancellationToken cancellationToken = default);

    /// <summary>
    /// DeleteAsync はポイントを ID リストで削除する。
    /// 実装側は TenantId フィルタリングで tenant 境界を保証する。
    /// </summary>
    // DeleteAsync メソッド: ポイントを削除する
    Task DeleteAsync(string collectionName, string tenantId, IReadOnlyList<string> ids, CancellationToken cancellationToken = default);

    /// <summary>
    /// SearchAsync は近傍ベクトル検索を実行して結果を返す。
    /// 実装側は TenantId フィルタリングを強制する。
    /// </summary>
    // SearchAsync メソッド: 近傍ベクトル検索を実行する
    Task<IReadOnlyList<VectorSearchResult>> SearchAsync(string collectionName, string tenantId, VectorSearchQuery query, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetByIdAsync は ID でポイントを取得する（ID + TenantId でフィルタリングする）。
    /// ポイントが存在しない場合は null を返す（エラーと区別する）。
    /// </summary>
    // GetByIdAsync メソッド: ID でポイントを取得する
    Task<VectorPoint?> GetByIdAsync(string collectionName, string tenantId, string id, CancellationToken cancellationToken = default);

    /// <summary>
    /// CountAsync はコレクション内の TenantId に属するポイント数を返す。
    /// </summary>
    // CountAsync メソッド: ポイント数を返す
    Task<long> CountAsync(string collectionName, string tenantId, CancellationToken cancellationToken = default);
}
