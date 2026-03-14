// AI クライアントエラー: API 通信エラーを表す
export class AiClientError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
  ) {
    super(message);
    this.name = 'AiClientError';
  }
}

// AI との会話メッセージ: role は "user" / "assistant" / "system" のいずれか
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

// テキスト補完リクエストのパラメータ
export interface CompleteRequest {
  model: string;
  messages: ChatMessage[];
  maxTokens?: number;
  temperature?: number;
  stream?: boolean;
}

// トークン使用量
export interface Usage {
  inputTokens: number;
  outputTokens: number;
}

// テキスト補完レスポンス
export interface CompleteResponse {
  id: string;
  model: string;
  content: string;
  usage: Usage;
}

// テキスト埋め込みリクエストのパラメータ
export interface EmbedRequest {
  model: string;
  texts: string[];
}

// テキスト埋め込みレスポンス
export interface EmbedResponse {
  model: string;
  embeddings: number[][];
}

// モデルの基本情報
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
}

// AI ゲートウェイへのアクセスを抽象化するインターフェース
export interface AiClient {
  complete(req: CompleteRequest): Promise<CompleteResponse>;
  embed(req: EmbedRequest): Promise<EmbedResponse>;
  listModels(): Promise<ModelInfo[]>;
}

// インメモリ AI クライアント: テスト用のモック実装
export class InMemoryAiClient implements AiClient {
  private completeFn: (req: CompleteRequest) => Promise<CompleteResponse>;
  private embedFn: (req: EmbedRequest) => Promise<EmbedResponse>;
  private listModelsFn: () => Promise<ModelInfo[]>;

  constructor(opts?: {
    complete?: (req: CompleteRequest) => Promise<CompleteResponse>;
    embed?: (req: EmbedRequest) => Promise<EmbedResponse>;
    listModels?: () => Promise<ModelInfo[]>;
  }) {
    // デフォルトはダミーレスポンスを返す
    this.completeFn = opts?.complete ?? (async (req) => ({
      id: 'mock-id',
      model: req.model,
      content: 'mock response',
      usage: { inputTokens: 0, outputTokens: 0 },
    }));
    this.embedFn = opts?.embed ?? (async (req) => ({
      model: req.model,
      embeddings: req.texts.map(() => []),
    }));
    this.listModelsFn = opts?.listModels ?? (async () => []);
  }

  async complete(req: CompleteRequest): Promise<CompleteResponse> {
    return this.completeFn(req);
  }

  async embed(req: EmbedRequest): Promise<EmbedResponse> {
    return this.embedFn(req);
  }

  async listModels(): Promise<ModelInfo[]> {
    return this.listModelsFn();
  }
}

// HTTP AI クライアント: AI ゲートウェイと HTTP 通信する実装
// Node.js 組み込みの fetch API を使用する（外部依存なし）
export class HttpAiClient implements AiClient {
  private readonly baseUrl: string;
  private readonly apiKey: string;
  private readonly timeoutMs: number;

  constructor(opts: { baseUrl: string; apiKey?: string; timeoutMs?: number }) {
    this.baseUrl = opts.baseUrl.replace(/\/$/, '');
    this.apiKey = opts.apiKey ?? '';
    // デフォルトタイムアウトは 30 秒
    this.timeoutMs = opts.timeoutMs ?? 30_000;
  }

  // AI ゲートウェイの /v1/complete エンドポイントを呼び出す
  async complete(req: CompleteRequest): Promise<CompleteResponse> {
    // Go の JSON タグ (snake_case) に合わせてリクエストを変換する
    const body = {
      model: req.model,
      messages: req.messages,
      max_tokens: req.maxTokens,
      temperature: req.temperature,
      stream: req.stream,
    };
    const raw = await this.post('/v1/complete', body);
    return {
      id: raw.id as string,
      model: raw.model as string,
      content: raw.content as string,
      usage: {
        inputTokens: (raw.usage as Record<string, number>).input_tokens,
        outputTokens: (raw.usage as Record<string, number>).output_tokens,
      },
    };
  }

  // AI ゲートウェイの /v1/embed エンドポイントを呼び出す
  async embed(req: EmbedRequest): Promise<EmbedResponse> {
    const raw = await this.post('/v1/embed', { model: req.model, texts: req.texts });
    return { model: raw.model as string, embeddings: raw.embeddings as number[][] };
  }

  // AI ゲートウェイの /v1/models エンドポイントを呼び出す
  async listModels(): Promise<ModelInfo[]> {
    const raw = await this.get('/v1/models');
    return (raw as Array<{ id: string; name: string; description: string }>).map((m) => ({
      id: m.id,
      name: m.name,
      description: m.description,
    }));
  }

  // POST リクエストを送信してレスポンス JSON を返す
  private async post(path: string, body: unknown): Promise<Record<string, unknown>> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    try {
      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
      if (this.apiKey) {
        headers['Authorization'] = `Bearer ${this.apiKey}`;
      }
      const res = await fetch(this.baseUrl + path, {
        method: 'POST',
        headers,
        body: JSON.stringify(body),
        signal: controller.signal,
      });
      if (!res.ok) {
        throw new AiClientError(`API error: ${res.status} ${res.statusText}`, res.status);
      }
      return res.json() as Promise<Record<string, unknown>>;
    } finally {
      clearTimeout(timer);
    }
  }

  // GET リクエストを送信してレスポンス JSON を返す
  private async get(path: string): Promise<unknown> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    try {
      const headers: Record<string, string> = {};
      if (this.apiKey) {
        headers['Authorization'] = `Bearer ${this.apiKey}`;
      }
      const res = await fetch(this.baseUrl + path, {
        method: 'GET',
        headers,
        signal: controller.signal,
      });
      if (!res.ok) {
        throw new AiClientError(`API error: ${res.status} ${res.statusText}`, res.status);
      }
      return res.json();
    } finally {
      clearTimeout(timer);
    }
  }
}
