// vector.go — k1s0 tier1 Library Go 実装: Vector Search の L1+ interface
// 15_ベクトル検索適合仕様.md §VectorSearchClient（pgvector / Qdrant L1+ 深耕）に準拠する。
// Vector Search の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
// OSS 型（qdrant-go / pgvector 等）を公開シグネチャに一切含まない。

// パッケージ名: keyhandle（tier1 Library の Vector Search API を提供する）
package keyhandle

import (
	// context: context.Context（AuthContext 伝播 + 非同期操作に使用する）
	"context"
)

// VectorDistanceMetric はベクトル距離計算方法を宣言する型。
// pgvector / Qdrant 等の Distance metric に準拠した Library 独自語彙とする。
type VectorDistanceMetric string

const (
	// VectorDistanceCosine: コサイン類似度（テキスト埋め込みに推奨する）
	VectorDistanceCosine VectorDistanceMetric = "cosine"
	// VectorDistanceL2: ユークリッド距離（L2 norm: 画像 / 音声埋め込みに推奨する）
	VectorDistanceL2 VectorDistanceMetric = "l2"
	// VectorDistanceDot: 内積（Dot product: 正規化済みベクトルで使用する）
	VectorDistanceDot VectorDistanceMetric = "dot"
	// VectorDistanceL1: マンハッタン距離（L1 norm: スパースベクトルに使用する）
	VectorDistanceL1 VectorDistanceMetric = "l1"
)

// VectorIndexType はベクトルインデックスの種別を宣言する型。
// 近似最近傍探索（ANN）アルゴリズムを Library 独自語彙で表現する。
type VectorIndexType string

const (
	// VectorIndexHNSW: Hierarchical Navigable Small World（高速 ANN: デフォルト推奨）
	VectorIndexHNSW VectorIndexType = "hnsw"
	// VectorIndexIVFFlat: IVF Flat（大規模コレクション向け ANN）
	VectorIndexIVFFlat VectorIndexType = "ivfflat"
	// VectorIndexExact: 完全一致（Brute Force: 小規模コレクション向け）
	VectorIndexExact VectorIndexType = "exact"
)

// Vector はベクトルデータを宣言する型。
// float32 スライスとして表現する（pgvector / Qdrant の標準フォーマット）。
type Vector []float32

// VectorPoint はベクトル検索コレクションに格納する単一ポイントを宣言する型。
// OSS の qdrant.PointStruct 等を露出せず Library 独自語彙で表現する。
type VectorPoint struct {
	// ID: ポイント識別子（UUID v7 形式を推奨する）
	ID string
	// TenantID: ポイントの所属テナント識別子（必須: tenant 分離フィルタリングに使用する）
	TenantID string
	// Vector: 埋め込みベクトル（次元数はコレクション設定と一致させる）
	Vector Vector
	// Payload: ポイントに付与するメタデータ（検索結果に含まれる）
	Payload map[string]any
}

// VectorSearchQuery はベクトル近傍検索クエリを宣言する型。
type VectorSearchQuery struct {
	// QueryVector: クエリベクトル（コレクションと同じ次元数）
	QueryVector Vector
	// TopK: 返す近傍ポイント数
	TopK int
	// Filter: メタデータフィルター条件（Payload フィールドに対する AND 条件）
	Filter map[string]any
	// ScoreThreshold: 最小スコア閾値（0.0 〜 1.0: スコアがこの値未満の結果を除外する）
	ScoreThreshold float32
	// WithPayload: 検索結果に Payload を含めるかどうか（false = ID と Score のみ返す）
	WithPayload bool
	// WithVector: 検索結果にベクトルを含めるかどうか（false = ベクトルを返さない）
	WithVector bool
}

// VectorSearchResult はベクトル近傍検索の単一結果を宣言する型。
type VectorSearchResult struct {
	// ID: 近傍ポイント識別子
	ID string
	// TenantID: 近傍ポイントの所属テナント識別子
	TenantID string
	// Score: クエリとの類似度スコア（距離計算方法に依存する）
	Score float32
	// Payload: ポイントのメタデータ（WithPayload=true の場合のみ設定される）
	Payload map[string]any
	// Vector: ポイントのベクトル（WithVector=true の場合のみ設定される）
	Vector Vector
}

// VectorCollectionConfig はコレクション設定を宣言する型。
// OSS の CreateCollection パラメーターを Library 独自語彙で表現する。
type VectorCollectionConfig struct {
	// Name: コレクション名
	Name string
	// Dimension: ベクトルの次元数（384 / 768 / 1536 等の固定値）
	Dimension int
	// DistanceMetric: ベクトル距離計算方法（デフォルト: VectorDistanceCosine）
	DistanceMetric VectorDistanceMetric
	// IndexType: インデックス種別（デフォルト: VectorIndexHNSW）
	IndexType VectorIndexType
	// HNSWConfig: HNSW インデックスの詳細設定（nil = デフォルト設定を使用する）
	HNSWConfig *HNSWConfig
}

// HNSWConfig は HNSW インデックスの詳細設定を宣言する型。
type HNSWConfig struct {
	// M: HNSW グラフのリンク数（高いほど精度↑ / 構築時間↑）
	M int
	// EfConstruct: 構築時の探索候補数（高いほど精度↑ / 構築時間↑）
	EfConstruct int
	// EfSearch: 検索時の探索候補数（高いほど精度↑ / 検索時間↑）
	EfSearch int
}

// VectorSearchClient は Vector Search の L1+ 抽象 interface を宣言する。
// pgvector / Qdrant / Weaviate / Pinecone 等を抽象化する。
// OSS 型を引数・戻り値に一切含まない。
// ctx に AuthContext が含まれることを強制する（tenant 分離必須）。
type VectorSearchClient interface {
	// CreateCollection はベクトルコレクションを作成する。
	// コレクションが既に存在する場合はエラーを返さない（idempotent 操作推奨）。
	CreateCollection(ctx context.Context, config VectorCollectionConfig) error

	// DeleteCollection はコレクションを削除する（全ポイントを削除する）。
	// コレクションが存在しない場合はエラーを返さない（idempotent 操作）。
	DeleteCollection(ctx context.Context, collectionName string) error

	// Upsert はポイントをコレクションに追加 / 更新する（ID 一致時は上書きする）。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	// points の TenantID は AuthContext.TenantID と一致する必要がある。
	Upsert(ctx context.Context, collectionName string, points []VectorPoint) error

	// Delete はポイントを ID リストで削除する。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	// 実装側は TenantID フィルタリングで tenant 境界を保証する。
	Delete(ctx context.Context, collectionName string, tenantID string, ids []string) error

	// Search は近傍ベクトル検索を実行して結果を返す。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	// 実装側は TenantID フィルタリングを強制する（他テナントの結果を返さない）。
	Search(ctx context.Context, collectionName string, tenantID string, query VectorSearchQuery) ([]VectorSearchResult, error)

	// GetByID は ID でポイントを取得する（ID + TenantID でフィルタリングする）。
	// ポイントが存在しない場合は nil, nil を返す（エラーと区別する）。
	GetByID(ctx context.Context, collectionName string, tenantID string, id string) (*VectorPoint, error)

	// Count はコレクション内の TenantID に属するポイント数を返す。
	Count(ctx context.Context, collectionName string, tenantID string) (int64, error)
}
