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
