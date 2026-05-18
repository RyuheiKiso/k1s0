// k1s0 tier3 帳票テンプレートラッパーパッケージ
// tier2 ReportService BFF に対して帳票生成リクエストを発行し、結果をダウンロードする
// PDF / Excel 両出力フォーマットをサポートする

// --------- 型定義 ---------

// 帳票出力フォーマット
export type ReportOutputFormat = "pdf" | "xlsx";

// 帳票生成リクエスト型
export interface ReportGenerateRequest {
  // テンプレート ID（tier2 側で管理する帳票テンプレート）
  readonly templateId: string;
  // 帳票パラメータ（テンプレート変数に渡す値の map）
  readonly params: Readonly<Record<string, unknown>>;
  // 出力フォーマット（pdf / xlsx）
  readonly outputFormat: ReportOutputFormat;
  // 言語ロケール（省略時は ja-JP）
  readonly locale?: string;
}

// 帳票生成結果型
export interface ReportGenerateResult {
  // 生成された帳票のダウンロード URL（一時 URL、TTL はサーバー側管理）
  readonly downloadUrl: string;
  // ファイル名（Content-Disposition ヘッダ相当）
  readonly fileName: string;
  // MIME タイプ
  readonly mimeType: string;
  // ファイルサイズ（バイト、0 は不明）
  readonly sizeBytes: number;
}

// --------- MIME タイプマップ ---------

// 出力フォーマットから MIME タイプを返す
export function getMimeType(format: ReportOutputFormat): string {
  // フォーマットに応じた MIME タイプを返す
  switch (format) {
    case "pdf":
      // PDF の MIME タイプ
      return "application/pdf";
    case "xlsx":
      // Excel 新形式の MIME タイプ
      return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
    default: {
      // 網羅性チェック（新フォーマット追加時はコンパイルエラー）
      const _exhaustive: never = format;
      throw new Error(`Unknown format: ${String(_exhaustive)}`);
    }
  }
}

// --------- 帳票クライアント ---------

// ReportClient: tier2 BFF への帳票生成リクエストを管理するクライアント
export class ReportClient {
  // BFF の帳票生成エンドポイント URL
  private readonly reportUrl: string;

  // コンストラクタ: 帳票サービス URL を受け取る
  public constructor(reportUrl: string) {
    // 帳票サービス URL を設定する
    this.reportUrl = reportUrl;
  }

  // 帳票生成リクエストを送信して結果を返す
  public async generate(
    request: ReportGenerateRequest,
  ): Promise<ReportGenerateResult> {
    // リクエストを JSON 形式で BFF に送信する
    const resp = await fetch(`${this.reportUrl}/generate`, {
      // POST リクエスト
      method: "POST",
      // JSON 形式のコンテンツタイプ
      headers: { "Content-Type": "application/json" },
      // リクエストボディを JSON に変換して送信する
      body: JSON.stringify(request),
    });
    // レスポンスが失敗の場合はエラーを投げる
    if (!resp.ok) {
      // 帳票生成エラーとしてスローする
      throw new Error(`帳票生成に失敗しました: ${resp.status}`);
    }
    // 生成結果を JSON として返す
    return (await resp.json()) as ReportGenerateResult;
  }

  // 生成された帳票をブラウザでダウンロードする
  public downloadReport(result: ReportGenerateResult): void {
    // a 要素を生成してクリックすることでダウンロードをトリガーする
    const anchor = document.createElement("a");
    // ダウンロード URL を設定する
    anchor.href = result.downloadUrl;
    // ファイル名を設定する（Content-Disposition 相当）
    anchor.download = result.fileName;
    // DOM に追加してクリックする（一部ブラウザで必要）
    document.body.appendChild(anchor);
    // クリックしてダウンロードを開始する
    anchor.click();
    // クリック後に a 要素を DOM から除去する
    document.body.removeChild(anchor);
  }

  // 帳票を生成してそのままダウンロードする（generate + downloadReport の合成）
  public async generateAndDownload(
    request: ReportGenerateRequest,
  ): Promise<void> {
    // 帳票生成リクエストを送信する
    const result = await this.generate(request);
    // 生成された帳票をダウンロードする
    this.downloadReport(result);
  }
}
