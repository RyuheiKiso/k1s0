/**
 * vector.ts — k1s0 tier1 Library TypeScript 実装: Vector Search の L1+ interface
 * 15_ベクトル検索適合仕様.md §VectorSearchClient（pgvector / Qdrant L1+ 深耕）に準拠する。
 * Vector Search の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
 * OSS 型（@qdrant/js-client-rest 等）を公開シグネチャに一切含まない。
 */

/**
 * VectorDistanceMetric はベクトル距離計算方法を宣言する enum。
 * pgvector / Qdrant 等の Distance metric に準拠した Library 独自語彙とする。
 */
// VectorDistanceMetric 列挙型定義
export const enum VectorDistanceMetric {
  // Cosine: コサイン類似度（テキスト埋め込みに推奨する）
  Cosine = "cosine",
  // L2: ユークリッド距離（画像 / 音声埋め込みに推奨する）
  L2 = "l2",
  // Dot: 内積（正規化済みベクトルで使用する）
  Dot = "dot",
  // L1: マンハッタン距離（スパースベクトルに使用する）
  L1 = "l1",
}

/**
 * VectorIndexType はベクトルインデックスの種別を宣言する enum。
 */
// VectorIndexType 列挙型定義
export const enum VectorIndexType {
  // Hnsw: Hierarchical Navigable Small World（高速 ANN: デフォルト推奨）
  Hnsw = "hnsw",
  // IvfFlat: IVF Flat（大規模コレクション向け ANN）
  IvfFlat = "ivfflat",
  // Exact: 完全一致（Brute Force: 小規模コレクション向け）
  Exact = "exact",
}

/**
 * Vector はベクトルデータを宣言する型。
 * Float32Array で表現する（pgvector / Qdrant の標準フォーマット）。
 */
// Vector 型定義
export type Vector = Float32Array;

/**
 * VectorPoint はベクトル検索コレクションに格納する単一ポイントを宣言する型。
 * OSS の PointStruct 等を露出せず Library 独自語彙で表現する。
 */
// VectorPoint 型定義
export interface VectorPoint {
  // id: ポイント識別子（UUID v7 形式を推奨する）
  readonly id: string;
  // tenantId: ポイントの所属テナント識別子（必須: tenant 分離フィルタリングに使用する）
  readonly tenantId: string;
  // vector: 埋め込みベクトル（次元数はコレクション設定と一致させる）
  readonly vector: Vector;
  // payload: ポイントに付与するメタデータ（検索結果に含まれる）
  readonly payload: Readonly<Record<string, unknown>>;
}

/**
 * VectorSearchQuery はベクトル近傍検索クエリを宣言する型。
 */
// VectorSearchQuery 型定義
export interface VectorSearchQuery {
  // queryVector: クエリベクトル（コレクションと同じ次元数）
  readonly queryVector: Vector;
  // topK: 返す近傍ポイント数
  readonly topK: number;
  // filter: メタデータフィルター条件（Payload フィールドに対する AND 条件）
  readonly filter?: Readonly<Record<string, unknown>> | undefined;
  // scoreThreshold: 最小スコア閾値（0.0 〜 1.0: スコアがこの値未満の結果を除外する）
  readonly scoreThreshold?: number | undefined;
  // withPayload: 検索結果に Payload を含めるかどうか（false = ID と Score のみ返す）
  readonly withPayload?: boolean | undefined;
  // withVector: 検索結果にベクトルを含めるかどうか（false = ベクトルを返さない）
  readonly withVector?: boolean | undefined;
}

/**
 * VectorSearchResult はベクトル近傍検索の単一結果を宣言する型。
 */
// VectorSearchResult 型定義
export interface VectorSearchResult {
  // id: 近傍ポイント識別子
  readonly id: string;
  // tenantId: 近傍ポイントの所属テナント識別子
  readonly tenantId: string;
  // score: クエリとの類似度スコア（距離計算方法に依存する）
  readonly score: number;
  // payload: ポイントのメタデータ（withPayload=true の場合のみ設定される）
  readonly payload?: Readonly<Record<string, unknown>> | undefined;
  // vector: ポイントのベクトル（withVector=true の場合のみ設定される）
  readonly vector?: Vector | undefined;
}

/**
 * HnswConfig は HNSW インデックスの詳細設定を宣言する型。
 */
// HnswConfig 型定義
export interface HnswConfig {
  // m: HNSW グラフのリンク数（高いほど精度↑ / 構築時間↑）
  readonly m?: number | undefined;
  // efConstruct: 構築時の探索候補数
  readonly efConstruct?: number | undefined;
  // efSearch: 検索時の探索候補数
  readonly efSearch?: number | undefined;
}

/**
 * VectorCollectionConfig はコレクション設定を宣言する型。
 */
// VectorCollectionConfig 型定義
export interface VectorCollectionConfig {
  // name: コレクション名
  readonly name: string;
  // dimension: ベクトルの次元数（384 / 768 / 1536 等の固定値）
  readonly dimension: number;
  // distanceMetric: ベクトル距離計算方法（未指定 = Cosine）
  readonly distanceMetric?: VectorDistanceMetric | undefined;
  // indexType: インデックス種別（未指定 = Hnsw）
  readonly indexType?: VectorIndexType | undefined;
  // hnswConfig: HNSW インデックスの詳細設定（未指定 = デフォルト設定を使用する）
  readonly hnswConfig?: HnswConfig | undefined;
}

/**
 * VectorSearchClient は Vector Search の L1+ 抽象 interface を宣言する。
 * pgvector / Qdrant / Weaviate / Pinecone 等を抽象化する。
 * OSS 型を一切含まない。
 * tenantId を必須引数として受け取る（tenant 分離必須）。
 */
// VectorSearchClient インターフェース定義
export interface VectorSearchClient {
  /**
   * createCollection はベクトルコレクションを作成する。
   * コレクションが既に存在する場合はエラーを投げない（idempotent 操作推奨）。
   */
  // createCollection メソッド: コレクションを作成する
  createCollection(config: VectorCollectionConfig): Promise<void>;

  /**
   * deleteCollection はコレクションを削除する（全ポイントを削除する）。
   */
  // deleteCollection メソッド: コレクションを削除する
  deleteCollection(collectionName: string): Promise<void>;

  /**
   * upsert はポイントをコレクションに追加 / 更新する（ID 一致時は上書きする）。
   * points の tenantId は AuthContext.tenantId と一致する必要がある。
   */
  // upsert メソッド: ポイントを追加 / 更新する
  upsert(collectionName: string, points: readonly VectorPoint[]): Promise<void>;

  /**
   * delete はポイントを ID リストで削除する。
   * 実装側は tenantId フィルタリングで tenant 境界を保証する。
   */
  // delete メソッド: ポイントを削除する
  delete(collectionName: string, tenantId: string, ids: readonly string[]): Promise<void>;

  /**
   * search は近傍ベクトル検索を実行して結果を返す。
   * 実装側は tenantId フィルタリングを強制する（他テナントの結果を返さない）。
   */
  // search メソッド: 近傍ベクトル検索を実行する
  search(
    collectionName: string,
    tenantId: string,
    query: VectorSearchQuery,
  ): Promise<readonly VectorSearchResult[]>;

  /**
   * getById は ID でポイントを取得する（ID + tenantId でフィルタリングする）。
   * ポイントが存在しない場合は null を返す（エラーと区別する）。
   */
  // getById メソッド: ID でポイントを取得する
  getById(
    collectionName: string,
    tenantId: string,
    id: string,
  ): Promise<VectorPoint | null>;

  /**
   * count はコレクション内の tenantId に属するポイント数を返す。
   */
  // count メソッド: ポイント数を返す
  count(collectionName: string, tenantId: string): Promise<number>;
}
