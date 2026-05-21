// HlcClock.cs — k1s0-hlc: Hybrid Logical Clock (HLC) の C# クロック実装
// Kulkarni et al. (2014) "Logical Physical Clocks" の clock update アルゴリズムを実装する。
// wall-clock 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// DateTimeOffset.UtcNow は本クラス内部のみで使用し、呼び出し元では直接使用を禁止する。
// Rust 実装（src/client/hlc_lib/rust/src/lib.rs HlcClock）と同等の API を提供する。

// K1s0.HlcLib 名前空間を宣言する
namespace K1s0.HlcLib;

// HlcClock: スレッドセーフな HLC クロック
// lock オブジェクトで Mutex 保護を行う（Rust の Mutex<(u64, u16)> と同等）
// Tick / Recv / Now の 3 操作でイベント間の因果関係を追跡する
// 複数スレッドから lock(this._lock) で安全に使用できる
public sealed class HlcClock
{
    // _lock: Tick / Recv 内で状態更新を排他制御するための lock オブジェクト
    private readonly object _lock = new object();

    // _stateWall: 最後に観測した WallMs（単調増加を保証するための状態）
    private ulong _stateWall;

    // _stateLogical: 最後に使用した Logical カウンタ
    private ushort _stateLogical;

    // _nodeId: このノード固有の識別子（HLC_NODE_ID 環境変数 or 0）
    private readonly ushort _nodeId;

    // FromEnv は環境変数 HLC_NODE_ID から nodeId を読み込んで HlcClock を生成する
    // HLC_NODE_ID が未設定 / パース失敗時は nodeId=0 を使用する
    public static HlcClock FromEnv()
    {
        // HLC_NODE_ID 環境変数を文字列として取得する
        var nodeIdStr = System.Environment.GetEnvironmentVariable("HLC_NODE_ID");
        // 環境変数が未設定 / 空の場合は nodeId=0 を使用する
        if (string.IsNullOrEmpty(nodeIdStr))
        {
            return new HlcClock(0);
        }
        // 文字列を ushort にパースする（失敗時は nodeId=0 を使用する）
        if (!ushort.TryParse(nodeIdStr, out var nodeId))
        {
            return new HlcClock(0);
        }
        // パース成功した nodeId を使って HlcClock を初期化する
        return new HlcClock(nodeId);
    }

    // constructor は nodeId を受け取って HlcClock を生成する
    // 複数ノード環境では異なる nodeId を設定してタイムスタンプの衝突を避ける
    public HlcClock(ushort nodeId)
    {
        // 初期 _stateWall: 0（first Tick で物理クロックに更新される）
        _stateWall = 0UL;
        // 初期 _stateLogical: 0
        _stateLogical = 0;
        // nodeId を設定する
        _nodeId = nodeId;
    }

    // WallMsNow は現在の UNIX epoch からの経過ミリ秒を返す（内部専用）
    // HLC の wall-clock 部分の取得にのみ使用する（TTL/deadline 計算での直接使用禁止）
    private static ulong WallMsNow()
    {
        // DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() を呼び出す
        // （本クラス内でのみ許可、呼び出し元では禁止）
        var ms = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        // 負値（UNIX epoch より前）は 0 として返す（理論上は発生しないが安全策）
        if (ms < 0)
        {
            return 0UL;
        }
        // long を ulong に変換して返す
        return (ulong)ms;
    }

    // Tick は send/local event のタイムスタンプを生成する（HLC の "send event"）
    // アルゴリズム:
    //   l' = max(_stateWall, pt)
    //   if l' == _stateWall: c' = _stateLogical + 1
    //   else: c' = 0
    public HlcTimestamp Tick()
    {
        // lock で状態更新を排他制御する（Rust の Mutex::lock と同等）
        lock (_lock)
        {
            // pt: 現在の物理クロック（ms）を取得する
            var pt = WallMsNow();
            // l': max(_stateWall, pt) を計算する（単調増加を保証する）
            var newWall = _stateWall > pt ? _stateWall : pt;
            // c': wall_ms が変化したかどうかで logical を更新する
            ushort newLogical;
            if (newWall == _stateWall)
            {
                // wall_ms が変わらなければ logical を +1 する
                if (_stateLogical == HlcTimestamp.U16Max)
                {
                    // logical が u16 の最大値に達した場合は例外を投げる（Rust の checked_add と同等）
                    throw new InvalidOperationException("HLC logical counter overflow (max ushort = 65535)");
                }
                newLogical = (ushort)(_stateLogical + 1);
            }
            else
            {
                // wall_ms が進んだ場合は logical を 0 にリセットする
                newLogical = 0;
            }
            // ステートを更新する（次回の Tick/Recv の比較基準になる）
            _stateWall = newWall;
            _stateLogical = newLogical;
            // 生成した HlcTimestamp を返す
            return new HlcTimestamp(newWall, newLogical, _nodeId);
        }
    }

    // Recv は受信メッセージのタイムスタンプを踏まえてローカルクロックを更新する（HLC の "receive event"）
    // アルゴリズム（3-way max）:
    //   l' = max(_stateWall, msgTs.WallMs, pt)
    //   3 者の最大一致に応じて logical を更新する
    public HlcTimestamp Recv(HlcTimestamp msgTs)
    {
        // lock で状態更新を排他制御する
        lock (_lock)
        {
            // pt: 現在の物理クロックを取得する
            var pt = WallMsNow();
            // l': max(_stateWall, msgTs.WallMs, pt) を計算する（3-way max）
            var newWall = _stateWall;
            if (msgTs.WallMs > newWall)
            {
                newWall = msgTs.WallMs;
            }
            if (pt > newWall)
            {
                newWall = pt;
            }
            // c': 3-way の最大一致パターンに応じて logical を更新する
            ushort newLogical;
            if (newWall == _stateWall && newWall == msgTs.WallMs)
            {
                // 3 者の wall_ms が同一: max(local_logical, msg_logical) + 1
                var maxLogical = _stateLogical > msgTs.Logical ? _stateLogical : msgTs.Logical;
                if (maxLogical == HlcTimestamp.U16Max)
                {
                    throw new InvalidOperationException("HLC logical counter overflow");
                }
                newLogical = (ushort)(maxLogical + 1);
            }
            else if (newWall == _stateWall)
            {
                // ローカルの wall_ms が最大: local_logical + 1
                if (_stateLogical == HlcTimestamp.U16Max)
                {
                    throw new InvalidOperationException("HLC logical counter overflow");
                }
                newLogical = (ushort)(_stateLogical + 1);
            }
            else if (newWall == msgTs.WallMs)
            {
                // 受信メッセージの wall_ms が最大: msg_logical + 1
                if (msgTs.Logical == HlcTimestamp.U16Max)
                {
                    throw new InvalidOperationException("HLC logical counter overflow");
                }
                newLogical = (ushort)(msgTs.Logical + 1);
            }
            else
            {
                // pt が最大（物理クロックが両者を上回った）: logical を 0 にリセットする
                newLogical = 0;
            }
            // ステートを更新する
            _stateWall = newWall;
            _stateLogical = newLogical;
            // 更新後の HlcTimestamp を返す
            return new HlcTimestamp(newWall, newLogical, _nodeId);
        }
    }

    // Now は現在の HLC タイムスタンプを生成する（Tick の alias）
    // キャッシュエントリの CachedAtHlc フィールドへの書き込みに使用する
    public HlcTimestamp Now()
    {
        // Tick と等価：send event として扱う
        return Tick();
    }
}
