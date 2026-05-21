// k1s0 tier3 添付ファイル管理クライアントパッケージ
// チャンクアップロード / MIME タイプ検査 / ハッシュチェーン整合性検証を提供する
// tier2 AttachmentService BFF エンドポイントを経由してアップロードする

// --------- 定数 ---------

// 許可する MIME タイプ一覧（ allowlist 方式でセキュリティを確保する）
export const ALLOWED_MIME_TYPES: ReadonlySet<string> = new Set([
  // PDF 文書
  "application/pdf",
  // Microsoft Excel（新形式）
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  // Microsoft Word（新形式）
  "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  // CSV テキスト
  "text/csv",
  // プレーンテキスト
  "text/plain",
  // JPEG 画像
  "image/jpeg",
  // PNG 画像
  "image/png",
]);

// デフォルトのチャンクサイズ（4 MiB）
export const DEFAULT_CHUNK_SIZE_BYTES = 4 * 1024 * 1024;

// --------- 型定義 ---------

// アップロード対象のファイルメタデータ型
export interface AttachmentMeta {
  // ファイル名
  readonly fileName: string;
  // MIME タイプ
  readonly mimeType: string;
  // ファイルサイズ（バイト）
  readonly sizeBytes: number;
  // ファイル全体の SHA-256 ハッシュ（hex 文字列）
  readonly sha256Hex: string;
}

// チャンクアップロードの 1 チャンク型
export interface AttachmentChunk {
  // チャンク番号（0 始まり）
  readonly chunkIndex: number;
  // チャンクのバイナリデータ
  readonly data: ArrayBuffer;
  // このチャンクの SHA-256 ハッシュ（hex 文字列）
  readonly chunkSha256Hex: string;
}

// アップロード結果型
export interface AttachmentUploadResult {
  // 付与された添付ファイル ID（サーバー生成 UUID）
  readonly attachmentId: string;
  // アップロード完了時刻（ISO 8601）
  readonly completedAt: string;
  // サーバー側で計算されたファイル全体ハッシュ（クライアント側と一致する必要がある）
  readonly serverSha256Hex: string;
}

// --------- MIME 検査 ---------

// MIME タイプが許可リストに含まれているか検査する
export function checkMimeType(mimeType: string): boolean {
  // 許可 MIME タイプ一覧に含まれているか確認する
  return ALLOWED_MIME_TYPES.has(mimeType);
}

// --------- ハッシュ計算 ---------

// ArrayBuffer の SHA-256 ハッシュを hex 文字列で計算する
// WebCrypto API（SubtleCrypto）を使用する（ブラウザ標準 API）
export async function computeSha256Hex(data: ArrayBuffer): Promise<string> {
  // SubtleCrypto で SHA-256 ダイジェストを計算する
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  // Uint8Array に変換する
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  // 各バイトを 2 桁 hex 文字列に変換して結合する
  return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
}

// --------- チャンク分割 ---------

// File をチャンクに分割してメタデータ付きで返す
export async function splitIntoChunks(
  file: File,
  chunkSizeBytes: number = DEFAULT_CHUNK_SIZE_BYTES,
): Promise<AttachmentChunk[]> {
  // チャンク一覧を格納する配列
  const chunks: AttachmentChunk[] = [];
  // チャンク番号（0 始まり）
  let chunkIndex = 0;
  // オフセット（処理済みバイト数）
  let offset = 0;
  // ファイル全体を処理し終わるまでループする
  while (offset < file.size) {
    // 次のチャンクの終端バイトを計算する（ファイル末尾を超えないよう clamp する）
    const end = Math.min(offset + chunkSizeBytes, file.size);
    // ファイルのスライスを取得する
    const slice = file.slice(offset, end);
    // ArrayBuffer に変換する
    const data = await slice.arrayBuffer();
    // チャンクの SHA-256 を計算する
    const chunkSha256Hex = await computeSha256Hex(data);
    // チャンクを追加する
    chunks.push({ chunkIndex, data, chunkSha256Hex });
    // オフセットを更新する
    offset = end;
    // チャンク番号をインクリメントする
    chunkIndex++;
  }
  // チャンク一覧を返す
  return chunks;
}

// --------- Object Storage メタデータ型 ---------

// AttachmentMetadata: Object Storage に保存された添付ファイルのフルメタデータ型
// spec arch.tier3 §26_添付帳票 UX: signed URL / sandbox iframe / virus scan / hashChain を含む
export interface AttachmentMetadata {
  // サーバー生成の添付ファイル UUID
  readonly id: string;
  // テナント識別子（公開 URL / クエリパラメータに露出しない — tier3 CLAUDE.md §データ保護）
  readonly tenantId: string;
  // MIME タイプ（allowlist で検査済み）
  readonly mimeType: string;
  // ファイルサイズ（バイト）
  readonly sizeBytes: number;
  // ハッシュチェーン（"sha256:<チャンク0>|sha256:<チャンク1>|..." 形式）
  readonly hashChain: string;
  // アップロード完了時刻（ISO 8601 文字列）
  readonly uploadedAt: string;
}

// --------- IAttachmentStore interface ---------

// IAttachmentStore: Object Storage への添付ファイル操作を抽象化する interface
// spec arch.tier3 §26_添付帳票 UX: upload / download / delete / signed URL 生成の 4 操作を要求する
export interface IAttachmentStore {
  // ファイルをアップロードして AttachmentMetadata を返す
  // file: アップロード対象の File オブジェクト
  upload(file: File): Promise<AttachmentMetadata>;

  // 添付ファイルをダウンロードして ArrayBuffer で返す
  // attachmentId: 取得する添付ファイルの UUID
  download(attachmentId: string): Promise<ArrayBuffer>;

  // 添付ファイルを論理削除する
  // attachmentId: 削除する添付ファイルの UUID
  delete(attachmentId: string): Promise<void>;

  // Object Storage の署名付き URL を生成する（sandbox iframe 表示用）
  // attachmentId: 署名付き URL を生成する添付ファイルの UUID
  // expiresInSeconds: URL の有効期限（秒）
  generateSignedUrl(attachmentId: string, expiresInSeconds: number): Promise<string>;
}

// --------- ハッシュチェーン構築ヘルパー ---------

// buildHashChain: チャンクの SHA-256 ハッシュ配列からハッシュチェーン文字列を生成する
// chunkHashes: 各チャンクの SHA-256 hex ハッシュ（順序通り）
// フォーマット: "sha256:<hash0>|sha256:<hash1>|..." — hashChain 連結形式
export function buildHashChain(chunkHashes: readonly string[]): string {
  // 各チャンクハッシュに "sha256:" プレフィックスを付けてパイプ区切りで連結する
  return chunkHashes.map((h) => `sha256:${h}`).join("|");
}

// verifyHashChain: AttachmentMetadata の hashChain が再計算値と一致するか検証する
// metadata: 検証対象の AttachmentMetadata
// computedChunkHashes: クライアント側で再計算したチャンクハッシュ配列
// 戻り値: ハッシュチェーンが一致する場合 true、不一致の場合 false
export function verifyHashChain(
  metadata: AttachmentMetadata,
  computedChunkHashes: readonly string[],
): boolean {
  // 再計算したハッシュチェーンを生成する
  const expected = buildHashChain(computedChunkHashes);
  // metadata のハッシュチェーンと比較する（定数時間比較ではないが整合性確認用途）
  return metadata.hashChain === expected;
}

// --------- sandbox iframe URL 生成 ---------

// createSandboxUrl: 添付ファイル ID から sandbox iframe 用の URL を生成する
// spec arch.tier3 §26_添付帳票 UX: CSP sandbox iframe で表示するため tier2 proxy URL を生成する
// attachmentId: sandbox iframe に表示する添付ファイルの UUID
// bffOrigin: tier2 BFF のオリジン（例: "https://bff.example.com"）
// 戻り値: sandbox iframe に使用する tier2 proxy URL 文字列
export function createSandboxUrl(attachmentId: string, bffOrigin: string): string {
  // tier2 BFF の attachment proxy エンドポイント URL を組み立てる
  // attachmentId を path パラメータとして埋め込む（クエリパラメータでは tenant_id を含めない）
  return `${bffOrigin}/attachments/${encodeURIComponent(attachmentId)}/view`;
}

// --------- TierAttachmentStore（骨格 in-memory 実装） ---------

// InMemoryAttachmentRecord: in-memory ストア内部の添付ファイルレコード型
interface InMemoryAttachmentRecord {
  // 添付ファイルのメタデータ
  metadata: AttachmentMetadata;
  // 添付ファイルのバイナリデータ
  data: ArrayBuffer;
}

// TierAttachmentStore: IAttachmentStore の骨格 in-memory 実装（テスト / モック用途）
// 本番環境では Object Storage (S3 互換) を呼び出す実装に差し替える
export class TierAttachmentStore implements IAttachmentStore {
  // in-memory ストレージ（attachmentId → InMemoryAttachmentRecord のマップ）
  private readonly store = new Map<string, InMemoryAttachmentRecord>();

  // テナント識別子（アップロード時のメタデータに使用する）
  private readonly tenantId: string;

  // コンストラクタ: テナント識別子を受け取る
  public constructor(tenantId: string) {
    // テナント識別子を設定する（公開 URL に露出しない）
    this.tenantId = tenantId;
  }

  // upload: ファイルをチャンク分割してハッシュを計算し、in-memory ストアに保存する
  public async upload(file: File): Promise<AttachmentMetadata> {
    // MIME タイプを検査する（allowlist 方式）
    if (!checkMimeType(file.type)) {
      // 許可されていない MIME タイプはエラーとする
      throw new Error(`許可されていない MIME タイプです: ${file.type}`);
    }
    // チャンクに分割してハッシュを計算する
    const chunks = await splitIntoChunks(file);
    // チャンクハッシュ一覧を取得する
    const chunkHashes = chunks.map((c) => c.chunkSha256Hex);
    // ハッシュチェーン文字列を生成する
    const hashChain = buildHashChain(chunkHashes);
    // 添付ファイル UUID を生成する（crypto.randomUUID を使用する）
    const id = crypto.randomUUID();
    // アップロード完了時刻を ISO 8601 文字列で記録する（表示用途のみ、TTL 計算に使用しない）
    const uploadedAt = new Date().toISOString();
    // ファイル全体の ArrayBuffer を取得する
    const fileBuffer = await file.arrayBuffer();
    // メタデータを構築する
    const metadata: AttachmentMetadata = {
      // 生成した UUID を設定する
      id,
      // テナント識別子を設定する
      tenantId: this.tenantId,
      // MIME タイプを設定する
      mimeType: file.type,
      // ファイルサイズを設定する
      sizeBytes: file.size,
      // ハッシュチェーンを設定する
      hashChain,
      // アップロード完了時刻を設定する
      uploadedAt,
    };
    // in-memory ストアに保存する
    this.store.set(id, { metadata, data: fileBuffer });
    // AttachmentMetadata を返す
    return metadata;
  }

  // download: 添付ファイルのバイナリデータを ArrayBuffer で返す
  public async download(attachmentId: string): Promise<ArrayBuffer> {
    // ストアからレコードを取得する
    const record = this.store.get(attachmentId);
    // レコードが存在しない場合はエラーを投げる
    if (!record) {
      // 存在しない添付ファイル ID はエラーとする
      throw new Error(`添付ファイルが見つかりません: ${attachmentId}`);
    }
    // バイナリデータを返す（Promise でラップする）
    return Promise.resolve(record.data);
  }

  // delete: 添付ファイルを in-memory ストアから論理削除する
  public async delete(attachmentId: string): Promise<void> {
    // ストアからレコードを削除する
    this.store.delete(attachmentId);
    // void を返す（Promise でラップする）
    return Promise.resolve();
  }

  // generateSignedUrl: in-memory 実装ではモック URL を返す（本番は Object Storage SDK 呼び出し）
  public async generateSignedUrl(
    attachmentId: string,
    expiresInSeconds: number,
  ): Promise<string> {
    // ストアにレコードが存在するか確認する
    if (!this.store.has(attachmentId)) {
      // 存在しない添付ファイル ID はエラーとする
      throw new Error(`添付ファイルが見つかりません: ${attachmentId}`);
    }
    // モック signed URL を生成する（本番は S3 互換 SDK の presign を使用する）
    // expiresInSeconds を URL パラメータとして含める（テスト検証用）
    return Promise.resolve(
      `https://mock-storage.example.com/attachments/${encodeURIComponent(attachmentId)}?expires=${expiresInSeconds}`,
    );
  }
}

// --------- アップロードクライアント ---------

// AttachmentUploadClient: tier2 BFF へのチャンクアップロードを管理するクライアント
export class AttachmentUploadClient {
  // BFF のアップロードエンドポイント URL
  private readonly uploadUrl: string;

  // コンストラクタ: アップロード URL を受け取る
  public constructor(uploadUrl: string) {
    // アップロード URL を設定する
    this.uploadUrl = uploadUrl;
  }

  // ファイルをチャンク分割してアップロードする
  public async upload(file: File): Promise<AttachmentUploadResult> {
    // MIME タイプを検査する
    if (!checkMimeType(file.type)) {
      // 許可されていない MIME タイプはエラーとする
      throw new Error(`許可されていない MIME タイプです: ${file.type}`);
    }
    // ファイル全体の SHA-256 を計算する
    const fileBuffer = await file.arrayBuffer();
    // ファイル全体のハッシュを計算する
    const sha256Hex = await computeSha256Hex(fileBuffer);
    // チャンクに分割する
    const chunks = await splitIntoChunks(file);
    // メタデータを構築する
    const meta: AttachmentMeta = {
      // ファイル名を設定する
      fileName: file.name,
      // MIME タイプを設定する
      mimeType: file.type,
      // ファイルサイズを設定する
      sizeBytes: file.size,
      // ファイル全体のハッシュを設定する
      sha256Hex,
    };
    // 開始リクエストを送信する（マルチパートアップロードセッションを開始する）
    const initResp = await fetch(`${this.uploadUrl}/init`, {
      // POST リクエスト
      method: "POST",
      // JSON 形式のコンテンツタイプ
      headers: { "Content-Type": "application/json" },
      // メタデータを JSON 文字列に変換して送信する
      body: JSON.stringify(meta),
    });
    // 開始レスポンスが失敗の場合はエラーを投げる
    if (!initResp.ok) {
      // アップロード開始エラーとしてスローする
      throw new Error(`アップロード開始に失敗しました: ${initResp.status}`);
    }
    // アップロードセッション ID を取得する
    const { uploadId } = (await initResp.json()) as { uploadId: string };
    // 各チャンクを順番にアップロードする
    for (const chunk of chunks) {
      // チャンクを FormData に格納する
      const formData = new FormData();
      // チャンク番号を設定する
      formData.append("chunkIndex", String(chunk.chunkIndex));
      // チャンクハッシュを設定する
      formData.append("chunkSha256Hex", chunk.chunkSha256Hex);
      // チャンクバイナリを設定する
      formData.append("data", new Blob([chunk.data]));
      // チャンクをアップロードする
      const chunkResp = await fetch(`${this.uploadUrl}/chunk/${uploadId}`, {
        // PUT リクエストでチャンクを送信する
        method: "PUT",
        // FormData をそのまま送信する（Content-Type は自動設定される）
        body: formData,
      });
      // チャンクアップロードが失敗した場合はエラーを投げる
      if (!chunkResp.ok) {
        // チャンクアップロードエラーとしてスローする
        throw new Error(
          `チャンク ${chunk.chunkIndex} のアップロードに失敗しました: ${chunkResp.status}`,
        );
      }
    }
    // 全チャンクのアップロード完了を通知する
    const completeResp = await fetch(`${this.uploadUrl}/complete/${uploadId}`, {
      // POST でアップロード完了を通知する
      method: "POST",
    });
    // 完了リクエストが失敗した場合はエラーを投げる
    if (!completeResp.ok) {
      // アップロード完了エラーとしてスローする
      throw new Error(`アップロード完了通知に失敗しました: ${completeResp.status}`);
    }
    // アップロード結果を返す
    return (await completeResp.json()) as AttachmentUploadResult;
  }
}
