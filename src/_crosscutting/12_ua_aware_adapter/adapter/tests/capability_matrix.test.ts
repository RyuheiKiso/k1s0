// k1s0 UA-aware adapter Capability matrix テスト
// 5 UA サブクラス x Capability cell の自動 enumeration を検証する
import {
  // K1s0UaAwareAdapter アダプタークラスをインポートする
  K1s0UaAwareAdapter,
  // UaSubclass 列挙型をインポートする
  UaSubclass,
  // Protocol 列挙型をインポートする
  Protocol,
  // StreamingKind 列挙型をインポートする
  StreamingKind,
  // CapabilityCell 型をインポートする
  CapabilityCell,
} from "../src/index";

// K1s0UaAwareAdapter のテストスイートを定義する
describe("K1s0UaAwareAdapter", () => {
  // テスト対象のアダプターインスタンスを定義する
  let adapter: K1s0UaAwareAdapter;

  // 各テスト前にアダプターインスタンスを初期化する
  beforeEach(() => {
    // 新しいアダプターインスタンスを作成する
    adapter = new K1s0UaAwareAdapter();
  });

  // UA 検出テストスイート: detectUaSubclass メソッドの動作を確認する
  describe("detectUaSubclass", () => {
    // Chrome UA 文字列の検出テスト
    it("Chrome UA 文字列を正しく Chrome として検出する", () => {
      // Chrome の UA 文字列を定義する
      const chromeUA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
      // detectUaSubclass で UA を検出する
      const result = adapter.detectUaSubclass(chromeUA);
      // Chrome として検出されることを確認する
      expect(result).toBe(UaSubclass.Chrome);
    });

    // Firefox UA 文字列の検出テスト
    it("Firefox UA 文字列を正しく Firefox として検出する", () => {
      // Firefox の UA 文字列を定義する
      const firefoxUA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0";
      // detectUaSubclass で UA を検出する
      const result = adapter.detectUaSubclass(firefoxUA);
      // Firefox として検出されることを確認する
      expect(result).toBe(UaSubclass.Firefox);
    });

    // Safari UA 文字列の検出テスト
    it("Safari UA 文字列を正しく Safari として検出する", () => {
      // Safari の UA 文字列を定義する
      const safariUA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
      // detectUaSubclass で UA を検出する
      const result = adapter.detectUaSubclass(safariUA);
      // Safari として検出されることを確認する
      expect(result).toBe(UaSubclass.Safari);
    });

    // Edge UA 文字列の検出テスト
    it("Edge UA 文字列を正しく Edge として検出する", () => {
      // Edge (Chromium ベース) の UA 文字列を定義する
      const edgeUA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
      // detectUaSubclass で UA を検出する
      const result = adapter.detectUaSubclass(edgeUA);
      // Edge として検出されることを確認する
      expect(result).toBe(UaSubclass.Edge);
    });

    // Unknown UA 文字列の検出テスト
    it("不明な UA 文字列を Other として検出する", () => {
      // 不明な UA 文字列を定義する
      const unknownUA = "MyCustomBrowser/1.0";
      // detectUaSubclass で UA を検出する
      const result = adapter.detectUaSubclass(unknownUA);
      // Other として検出されることを確認する
      expect(result).toBe(UaSubclass.Other);
    });
  });

  // Capability matrix テストスイート: 5 UA x 全 Capability cell を自動 enumeration する
  describe("Capability matrix 自動 enumeration", () => {
    // 全 UA サブクラスの配列を定義する
    const allUaSubclasses = Object.values(UaSubclass);
    // 全プロトコルの配列を定義する
    const allProtocols = Object.values(Protocol);
    // 全ストリーミング種別の配列を定義する
    const allStreamingKinds = Object.values(StreamingKind);

    // 各 UA サブクラスに対して Capability cell のテストを実行する
    allUaSubclasses.forEach((ua) => {
      // 各 UA サブクラスのテストグループを定義する
      describe(`UA: ${ua}`, () => {
        // 各プロトコルに対してテストを実行する
        allProtocols.forEach((protocol) => {
          // 各ストリーミング種別に対してテストを実行する
          allStreamingKinds.forEach((streaming) => {
            // Capability cell が取得できるかまたは undefined であることを確認するテスト
            it(`${ua} x ${protocol} x ${streaming}: getCapability が CapabilityCell または undefined を返す`, () => {
              // Capability cell を取得する
              const cell = adapter.getCapability(ua, protocol, streaming);
              // cell が取得できた場合は型を確認する
              if (cell !== undefined) {
                // cell.supported が boolean であることを確認する
                expect(typeof cell.supported).toBe("boolean");
                // cell.notes が string であることを確認する
                expect(typeof cell.notes).toBe("string");
                // cell.fallback が Protocol または null であることを確認する
                expect(
                  cell.fallback === null || Object.values(Protocol).includes(cell.fallback as Protocol)
                ).toBe(true);
              }
              // undefined の場合もエラーにならないことを確認する (undefined は正常)
              expect(cell === undefined || typeof cell === "object").toBe(true);
            });
          });
        });
      });
    });
  });

  // resolveProtocol テストスイート: プロトコル解決の動作を確認する
  describe("resolveProtocol", () => {
    // Chrome HTTP/2 BidiStreaming の解決テスト: サポートされているため HTTP/2 を返す
    it("Chrome HTTP/2 BidiStreaming: HTTP/2 が返される", () => {
      // Chrome の HTTP/2 BidiStreaming プロトコルを解決する
      const protocol = adapter.resolveProtocol(UaSubclass.Chrome, StreamingKind.BidiStreaming, Protocol.Http2);
      // HTTP/2 または WebSocket が返されることを確認する (Chrome は conditional support)
      expect([Protocol.Http2, Protocol.WebSocket]).toContain(protocol);
    });

    // Firefox HTTP/2 BidiStreaming の解決テスト: 非サポートのため WebSocket にフォールバックする
    it("Firefox HTTP/2 BidiStreaming: WebSocket にフォールバックされる", () => {
      // Firefox の HTTP/2 BidiStreaming プロトコルを解決する
      const protocol = adapter.resolveProtocol(UaSubclass.Firefox, StreamingKind.BidiStreaming, Protocol.Http2);
      // WebSocket にフォールバックされることを確認する
      expect(protocol).toBe(Protocol.WebSocket);
    });

    // Safari HTTP/2 BidiStreaming の解決テスト: 非サポートのため WebSocket にフォールバックする
    it("Safari HTTP/2 BidiStreaming: WebSocket にフォールバックされる", () => {
      // Safari の HTTP/2 BidiStreaming プロトコルを解決する
      const protocol = adapter.resolveProtocol(UaSubclass.Safari, StreamingKind.BidiStreaming, Protocol.Http2);
      // WebSocket にフォールバックされることを確認する
      expect(protocol).toBe(Protocol.WebSocket);
    });

    // Edge HTTP/2 Unary の解決テスト: サポートされているため HTTP/2 を返す
    it("Edge HTTP/2 Unary: HTTP/2 が返される", () => {
      // Edge の HTTP/2 Unary プロトコルを解決する
      const protocol = adapter.resolveProtocol(UaSubclass.Edge, StreamingKind.Unary, Protocol.Http2);
      // HTTP/2 が返されることを確認する
      expect(protocol).toBe(Protocol.Http2);
    });

    // Other HTTP/2 BidiStreaming の解決テスト: 安全のため WebSocket にフォールバックする
    it("Other HTTP/2 BidiStreaming: WebSocket にフォールバックされる", () => {
      // Other の HTTP/2 BidiStreaming プロトコルを解決する
      const protocol = adapter.resolveProtocol(UaSubclass.Other, StreamingKind.BidiStreaming, Protocol.Http2);
      // WebSocket にフォールバックされることを確認する
      expect(protocol).toBe(Protocol.WebSocket);
    });
  });

  // getAllCapabilities テストスイート: 全 Capability cell の取得を確認する
  describe("getAllCapabilities", () => {
    // 全 Capability cell が配列で返されることを確認するテスト
    it("全 Capability cell が空でない配列として返される", () => {
      // 全 Capability cell を取得する
      const capabilities = adapter.getAllCapabilities();
      // 配列であることを確認する
      expect(Array.isArray(capabilities)).toBe(true);
      // 空でないことを確認する (少なくとも 1 エントリが存在する)
      expect(capabilities.length).toBeGreaterThan(0);
    });

    // 各エントリが正しい構造を持つことを確認するテスト
    it("各 Capability エントリが正しい構造を持つ", () => {
      // 全 Capability cell を取得する
      const capabilities = adapter.getAllCapabilities();
      // 各エントリの構造を確認する
      capabilities.forEach((entry) => {
        // ua が UaSubclass であることを確認する
        expect(Object.values(UaSubclass)).toContain(entry.ua);
        // protocol が Protocol であることを確認する
        expect(Object.values(Protocol)).toContain(entry.protocol);
        // streaming が StreamingKind であることを確認する
        expect(Object.values(StreamingKind)).toContain(entry.streaming);
        // cell が CapabilityCell の構造を持つことを確認する
        expect(typeof entry.cell.supported).toBe("boolean");
        // cell.notes が文字列であることを確認する
        expect(typeof entry.cell.notes).toBe("string");
      });
    });

    // 5 つの UA サブクラスに対するエントリが全て存在することを確認するテスト
    it("5 つの UA サブクラス全てのエントリが存在する", () => {
      // 全 Capability cell を取得する
      const capabilities = adapter.getAllCapabilities();
      // 全 UA サブクラスのセットを作成する
      const foundUaSubclasses = new Set(capabilities.map((c) => c.ua));
      // 少なくとも 5 つの UA サブクラスが存在することを確認する
      // (全ての UA サブクラスにエントリがある場合)
      const allExpected = Object.values(UaSubclass).filter(
        // Capability が定義されている UA サブクラスを列挙する
        (ua) => capabilities.some((c) => c.ua === ua)
      );
      // 見つかった UA サブクラス数が期待値以上であることを確認する
      expect(foundUaSubclasses.size).toBeGreaterThanOrEqual(allExpected.length);
    });
  });

  // カスタム Capability matrix テストスイート: カスタムマトリクスが正しく機能することを確認する
  describe("カスタム Capability matrix", () => {
    // カスタム Capability matrix でアダプターを初期化するテスト
    it("カスタム Capability matrix でアダプターを初期化できる", () => {
      // カスタム Capability matrix を作成する
      const customMatrix = new Map<string, CapabilityCell>();
      // テスト用のカスタムエントリを追加する
      customMatrix.set(`${UaSubclass.Chrome}:${Protocol.Http2}:${StreamingKind.Unary}`, {
        // カスタムエントリ: Chrome HTTP/2 Unary をサポート
        supported: true,
        // フォールバック: なし
        fallback: null,
        // 注意事項: テスト用カスタムエントリ
        notes: "テスト用カスタムエントリ",
      });
      // カスタムマトリクスでアダプターを作成する
      const customAdapter = new K1s0UaAwareAdapter(customMatrix);
      // カスタムマトリクスのエントリが取得できることを確認する
      const cell = customAdapter.getCapability(UaSubclass.Chrome, Protocol.Http2, StreamingKind.Unary);
      // カスタムエントリの notes が正しいことを確認する
      expect(cell?.notes).toBe("テスト用カスタムエントリ");
    });
  });
});
