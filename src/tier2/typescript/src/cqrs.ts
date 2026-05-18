/**
 * k1s0 tier2 CQRS TypeScript インターフェース定義
 * Rust 実装（cqrs/src/projector.rs）と 4 言語等価強度を持つ TypeScript 版
 * Outbox リレー経由で受信したドメインイベントを読み取りモデルに投影する（設計方針 15）
 */

/**
 * DomainEvent: ドメインイベントの構造体定義（Outbox から受信する共通エンベロープ形式）
 * Rust の DomainEvent 構造体に対応する
 */
// DomainEvent インターフェース定義
export interface DomainEvent {
  // イベント一意識別子（UUID v4）
  readonly id: string;
  // テナント識別子（RLS 述語の基底 / AuthContext からのみ注入する）
  readonly tenantId: string;
  // 集約型名（業界中立語のみ使用可）
  readonly aggregateType: string;
  // イベント種別名
  readonly eventType: string;
  // ペイロード（JSON Value 形式 / PII フィールドは Outbox 書込前に redact 済み）
  readonly payload: unknown;
  // HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
  readonly hlcTimestamp: bigint;
}

/**
 * ReadModelProjector: 読み取りモデル投影器インターフェース
 * Rust の ReadModelProjector トレイトに対応する
 * すべての読み取りモデル投影器が実装する契約
 * project メソッドはドメインイベントを受け取り読み取りモデルを更新する
 */
// ReadModelProjector インターフェース定義
export interface ReadModelProjector {
  /**
   * ドメインイベントを受け取り読み取りモデルを更新する
   * 失敗した場合は Error をスローする（Outbox リレーがリトライする）
   */
  // project メソッド（読み取りモデル投影操作）
  project(event: DomainEvent): Promise<void>;

  /**
   * 投影器が処理対象とするイベント種別の一覧を返す
   * 登録済み投影器のルーティングに使用する
   */
  // handledEventTypes メソッド（処理対象イベント種別一覧）
  handledEventTypes(): readonly string[];
}

/**
 * VectorSearchResult: ベクトル検索結果の構造体
 * Rust の VectorSearchResult 構造体に対応する（pgvector 類似検索）
 */
// VectorSearchResult インターフェース定義（検索ヒット 1 件分のデータを保持する）
export interface VectorSearchResult {
  // 検索ヒットしたドキュメントの識別子
  readonly documentId: string;
  // テナント識別子（RLS で自動フィルタリング済み）
  readonly tenantId: string;
  // コサイン類似度スコア（0.0〜1.0 / 高いほど類似）
  readonly similarityScore: number;
  // ドキュメントメタデータ（JSON 形式）
  readonly metadata: unknown;
}

/**
 * ReadModelRegistry: 複数の ReadModelProjector を集約するクラス
 * イベント種別に基づいて対応する投影器へルーティングする
 * Rust の ReadModelRegistry 構造体に対応する
 */
// ReadModelRegistry クラス定義
export class ReadModelRegistry {
  // 登録済み投影器のリスト（ReadModelProjector インターフェース形式で保持する）
  readonly #projectors: ReadModelProjector[] = [];

  /**
   * 投影器をレジストリに登録する
   */
  // register メソッド（投影器登録操作）
  register(projector: ReadModelProjector): void {
    // 投影器リストに追加する
    this.#projectors.push(projector);
  }

  /**
   * ドメインイベントを対応する投影器へルーティングして投影する
   */
  // dispatch メソッド（イベントルーティングおよび投影操作）
  async dispatch(event: DomainEvent): Promise<void> {
    // 登録済み投影器を順に検索して対象イベント種別を処理する
    for (const projector of this.#projectors) {
      // 投影器がこのイベント種別を処理するか確認する
      if (projector.handledEventTypes().includes(event.eventType)) {
        // 対応投影器に処理を委譲する
        await projector.project(event);
      }
    }
  }
}
