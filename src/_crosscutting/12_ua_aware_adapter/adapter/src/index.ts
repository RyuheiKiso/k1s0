// k1s0 UA-aware adapter: TypeScript
// Connect-RPC の UA 別 fetch full-duplex 実装差を吸収するアダプターを実装する
// Chrome / Firefox / Safari / Edge x HTTP2 / H3 / WebSocket capability matrix を管理する

// UA サブクラスの列挙型: サポートする User-Agent の種類を定義する
export enum UaSubclass {
  // Chrome ブラウザ
  Chrome = "chrome",
  // Firefox ブラウザ
  Firefox = "firefox",
  // Safari ブラウザ
  Safari = "safari",
  // Edge ブラウザ (Chromium ベース)
  Edge = "edge",
  // その他のブラウザ (Chromium ベース以外)
  Other = "other",
}

// 通信プロトコルの列挙型: サポートする通信プロトコルを定義する
export enum Protocol {
  // HTTP/2 プロトコル
  Http2 = "http2",
  // HTTP/3 プロトコル (QUIC ベース)
  Http3 = "http3",
  // WebSocket プロトコル
  WebSocket = "websocket",
}

// ストリーミング種別の列挙型: Connect-RPC のストリーミング種別を定義する
export enum StreamingKind {
  // Unary (単項呼び出し)
  Unary = "unary",
  // Server Streaming (サーバー側ストリーミング)
  ServerStreaming = "server_streaming",
  // Client Streaming (クライアント側ストリーミング)
  ClientStreaming = "client_streaming",
  // Bidirectional Streaming (双方向ストリーミング)
  BidiStreaming = "bidi_streaming",
}

// UA の Capability cell を表す型: UA が特定の機能をサポートするかを示す
export interface CapabilityCell {
  // サポート状態: true = サポート, false = 非サポート
  supported: boolean;
  // フォールバック: 非サポートの場合のフォールバック種別 (nullable)
  fallback: Protocol | null;
  // 注意事項: 実装上の制約や注意点を記録する
  notes: string;
}

// UA と Protocol の組み合わせによる Capability matrix の型
// Key: "${UaSubclass}:${Protocol}:${StreamingKind}" の形式
export type CapabilityMatrix = Map<string, CapabilityCell>;

// buildCapabilityMatrixKey は Capability matrix のキーを構築する関数
function buildCapabilityMatrixKey(
  // UA サブクラス
  ua: UaSubclass,
  // 通信プロトコル
  protocol: Protocol,
  // ストリーミング種別
  streaming: StreamingKind,
): string {
  // キーを "${ua}:${protocol}:${streaming}" の形式で構築して返す
  return `${ua}:${protocol}:${streaming}`;
}

// defaultCapabilityMatrix はデフォルトの Capability matrix を構築して返す関数
// 各 UA と Protocol の組み合わせによる実装差を定義する
function buildDefaultCapabilityMatrix(): CapabilityMatrix {
  // 空の Capability matrix を作成する
  const matrix: CapabilityMatrix = new Map();

  // Chrome の Capability を定義する
  // Chrome HTTP/2 Unary: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Chrome, Protocol.Http2, StreamingKind.Unary), {
    // HTTP/2 Unary は Chrome で完全サポートされる
    supported: true,
    fallback: null,
    notes: "Chrome HTTP/2 Unary: fetch API で完全サポート",
  });
  // Chrome HTTP/2 Server Streaming: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Chrome, Protocol.Http2, StreamingKind.ServerStreaming), {
    // HTTP/2 Server Streaming は Chrome で fetch API を使用してサポートされる
    supported: true,
    fallback: null,
    notes: "Chrome HTTP/2 Server Streaming: ReadableStream API で完全サポート",
  });
  // Chrome HTTP/2 BidiStreaming: 条件付きサポート (fetch full-duplex は制限あり)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Chrome, Protocol.Http2, StreamingKind.BidiStreaming), {
    // Chrome の fetch full-duplex BidiStreaming は experimental
    supported: true,
    fallback: Protocol.WebSocket,
    notes: "Chrome HTTP/2 BidiStreaming: fetch full-duplex (experimental) または WebSocket にフォールバック",
  });
  // Chrome WebSocket BidiStreaming: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Chrome, Protocol.WebSocket, StreamingKind.BidiStreaming), {
    // WebSocket BidiStreaming は Chrome で完全サポートされる
    supported: true,
    fallback: null,
    notes: "Chrome WebSocket BidiStreaming: WebSocket API で完全サポート",
  });

  // Firefox の Capability を定義する
  // Firefox HTTP/2 Unary: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Firefox, Protocol.Http2, StreamingKind.Unary), {
    // Firefox HTTP/2 Unary は fetch API で完全サポートされる
    supported: true,
    fallback: null,
    notes: "Firefox HTTP/2 Unary: fetch API で完全サポート",
  });
  // Firefox HTTP/2 BidiStreaming: 非サポート (WebSocket にフォールバック)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Firefox, Protocol.Http2, StreamingKind.BidiStreaming), {
    // Firefox は HTTP/2 fetch full-duplex をサポートしない
    supported: false,
    fallback: Protocol.WebSocket,
    notes: "Firefox HTTP/2 BidiStreaming: 非サポート。WebSocket にフォールバックする",
  });
  // Firefox WebSocket BidiStreaming: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Firefox, Protocol.WebSocket, StreamingKind.BidiStreaming), {
    // Firefox WebSocket BidiStreaming は完全サポートされる
    supported: true,
    fallback: null,
    notes: "Firefox WebSocket BidiStreaming: WebSocket API で完全サポート",
  });

  // Safari の Capability を定義する
  // Safari HTTP/2 Unary: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Safari, Protocol.Http2, StreamingKind.Unary), {
    // Safari HTTP/2 Unary は fetch API で完全サポートされる
    supported: true,
    fallback: null,
    notes: "Safari HTTP/2 Unary: fetch API で完全サポート",
  });
  // Safari HTTP/2 BidiStreaming: 非サポート (WebSocket にフォールバック)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Safari, Protocol.Http2, StreamingKind.BidiStreaming), {
    // Safari は HTTP/2 fetch full-duplex をサポートしない
    supported: false,
    fallback: Protocol.WebSocket,
    notes: "Safari HTTP/2 BidiStreaming: 非サポート。WebSocket にフォールバックする",
  });
  // Safari WebSocket BidiStreaming: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Safari, Protocol.WebSocket, StreamingKind.BidiStreaming), {
    // Safari WebSocket BidiStreaming は完全サポートされる
    supported: true,
    fallback: null,
    notes: "Safari WebSocket BidiStreaming: WebSocket API で完全サポート",
  });
  // Safari HTTP/3: 非サポート (2024 年時点では対応中)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Safari, Protocol.Http3, StreamingKind.Unary), {
    // Safari HTTP/3 はサポートが限定的なため HTTP/2 にフォールバックする
    supported: false,
    fallback: Protocol.Http2,
    notes: "Safari HTTP/3: サポートが限定的。HTTP/2 にフォールバックする",
  });

  // Edge の Capability を定義する (Chromium ベースのため Chrome とほぼ同等)
  // Edge HTTP/2 Unary: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Edge, Protocol.Http2, StreamingKind.Unary), {
    // Edge HTTP/2 Unary は Chromium ベースのため Chrome と同等にサポートされる
    supported: true,
    fallback: null,
    notes: "Edge HTTP/2 Unary: Chromium ベースのため Chrome と同等サポート",
  });
  // Edge HTTP/2 BidiStreaming: 条件付きサポート (Chrome と同等)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Edge, Protocol.Http2, StreamingKind.BidiStreaming), {
    // Edge HTTP/2 BidiStreaming は Chrome と同様に experimental
    supported: true,
    fallback: Protocol.WebSocket,
    notes: "Edge HTTP/2 BidiStreaming: Chrome と同様の fetch full-duplex (experimental)",
  });
  // Edge WebSocket BidiStreaming: 完全サポート
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Edge, Protocol.WebSocket, StreamingKind.BidiStreaming), {
    // Edge WebSocket BidiStreaming は完全サポートされる
    supported: true,
    fallback: null,
    notes: "Edge WebSocket BidiStreaming: WebSocket API で完全サポート",
  });

  // Other (その他) の Capability を定義する (保守的な設定)
  // Other HTTP/2 Unary: サポート (fetch API は広く実装されている)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Other, Protocol.Http2, StreamingKind.Unary), {
    // その他のブラウザでも fetch API による HTTP/2 Unary は概ねサポートされる
    supported: true,
    fallback: null,
    notes: "Other HTTP/2 Unary: fetch API は広くサポートされている",
  });
  // Other HTTP/2 BidiStreaming: 非サポート (安全のため WebSocket にフォールバック)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Other, Protocol.Http2, StreamingKind.BidiStreaming), {
    // その他のブラウザでは HTTP/2 BidiStreaming を安全のため非サポートとする
    supported: false,
    fallback: Protocol.WebSocket,
    notes: "Other HTTP/2 BidiStreaming: 安全のため非サポート。WebSocket にフォールバックする",
  });
  // Other WebSocket BidiStreaming: サポート (WebSocket は広く実装されている)
  matrix.set(buildCapabilityMatrixKey(UaSubclass.Other, Protocol.WebSocket, StreamingKind.BidiStreaming), {
    // その他のブラウザでも WebSocket は概ねサポートされる
    supported: true,
    fallback: null,
    notes: "Other WebSocket BidiStreaming: WebSocket は広くサポートされている",
  });

  // 構築した Capability matrix を返す
  return matrix;
}

// K1s0UaAwareAdapter は UA-aware adapter の中心クラス
// Connect-RPC の UA 別の実装差を adapter pattern で吸収する
export class K1s0UaAwareAdapter {
  // Capability matrix: UA と Protocol の組み合わせによるサポート状況
  private readonly capabilityMatrix: CapabilityMatrix;

  // コンストラクタ: デフォルトまたはカスタム Capability matrix を受け取る
  constructor(capabilityMatrix?: CapabilityMatrix) {
    // Capability matrix を設定する (デフォルトまたはカスタム)
    this.capabilityMatrix = capabilityMatrix ?? buildDefaultCapabilityMatrix();
  }

  // detectUaSubclass は User-Agent 文字列から UA サブクラスを検出する
  detectUaSubclass(userAgent: string): UaSubclass {
    // User-Agent 文字列を小文字に変換して比較する
    const ua = userAgent.toLowerCase();
    // Edge の検出: Edg または Edge を含む (Chromium ベース Edge)
    if (ua.includes("edg/") || ua.includes("edge/")) {
      // Edge ブラウザとして返す
      return UaSubclass.Edge;
    }
    // Chrome の検出: Chrome を含み Chromium または Edge でない
    if (ua.includes("chrome/") && !ua.includes("chromium/")) {
      // Chrome ブラウザとして返す
      return UaSubclass.Chrome;
    }
    // Firefox の検出: Firefox または Gecko を含む
    if (ua.includes("firefox/") || ua.includes("gecko/")) {
      // Firefox ブラウザとして返す
      return UaSubclass.Firefox;
    }
    // Safari の検出: Safari を含み Chrome でない (Chrome も Safari を含むため除外)
    if (ua.includes("safari/") && !ua.includes("chrome/")) {
      // Safari ブラウザとして返す
      return UaSubclass.Safari;
    }
    // その他のブラウザとして返す
    return UaSubclass.Other;
  }

  // getCapability は UA、Protocol、Streaming の組み合わせから Capability cell を取得する
  getCapability(
    // UA サブクラス
    ua: UaSubclass,
    // 通信プロトコル
    protocol: Protocol,
    // ストリーミング種別
    streaming: StreamingKind,
  ): CapabilityCell | undefined {
    // Capability matrix のキーを構築する
    const key = buildCapabilityMatrixKey(ua, protocol, streaming);
    // Capability matrix から値を取得して返す
    return this.capabilityMatrix.get(key);
  }

  // resolveProtocol は UA とストリーミング種別から最適なプロトコルを解決する
  resolveProtocol(
    // UA サブクラス
    ua: UaSubclass,
    // 希望するストリーミング種別
    streaming: StreamingKind,
    // 優先プロトコル (省略時は HTTP/2 を試みる)
    preferredProtocol: Protocol = Protocol.Http2,
  ): Protocol {
    // 優先プロトコルの Capability を確認する
    const capability = this.getCapability(ua, preferredProtocol, streaming);
    // Capability が存在してサポートされている場合は優先プロトコルを返す
    if (capability?.supported) {
      // 優先プロトコルをそのまま使用する
      return preferredProtocol;
    }
    // フォールバックプロトコルが存在する場合はフォールバックを返す
    if (capability?.fallback != null) {
      // フォールバックプロトコルを使用する
      return capability.fallback;
    }
    // デフォルトのフォールバック: WebSocket を使用する
    return Protocol.WebSocket;
  }

  // getAllCapabilities は全ての Capability matrix エントリを配列で返す
  getAllCapabilities(): Array<{
    ua: UaSubclass;
    protocol: Protocol;
    streaming: StreamingKind;
    cell: CapabilityCell;
  }> {
    // 結果配列を初期化する
    const result: Array<{
      ua: UaSubclass;
      protocol: Protocol;
      streaming: StreamingKind;
      cell: CapabilityCell;
    }> = [];

    // Capability matrix の全エントリを反復する
    for (const [key, cell] of this.capabilityMatrix.entries()) {
      // キーを分割して UA, Protocol, Streaming を取得する
      const [ua, protocol, streaming] = key.split(":") as [UaSubclass, Protocol, StreamingKind];
      // 結果配列にエントリを追加する
      result.push({ ua, protocol, streaming, cell });
    }

    // 全エントリの配列を返す
    return result;
  }
}

// デフォルトエクスポート: K1s0UaAwareAdapter のシングルトンインスタンスを作成して返す
export default new K1s0UaAwareAdapter();
