// hlc.ts — k1s0-hlc: Hybrid Logical Clock (HLC) の TypeScript 実装
// Kulkarni et al. (2014) "Logical Physical Clocks" のアルゴリズムを TypeScript で実装する。
// wall-clock 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// TTL / deadline の比較は HLC elapsed で管理し、Date.now() を直接参照しない。
// 本モジュールが Date.now() を扱う唯一の許可された場所であり、呼び出し元は本モジュールを経由する。
// Rust 実装（src/client/hlc_lib/rust/src/lib.rs）と同等の API を提供する。

// HlcTimestamp: HLC のタイムスタンプ（wall_ms + logical + node_id の 3 tuple）
// 全順序（wall_ms > logical > node_id の辞書順）で比較可能。
// formatCompact で "{wall_ms_hex_16}-{logical_04x}-{node_04x}" 形式に変換できる。
export class HlcTimestamp {
  // wall_ms: UNIX epoch からの経過ミリ秒（wall-clock 部分、記録目的のみ、bigint で u64 相当を表現する）
  readonly wall_ms: bigint;
  // logical: 同一 wall_ms 内の単調カウンタ（最大 65535 = u16 相当）
  readonly logical: number;
  // node_id: ノード識別子（複数インスタンスでの衝突回避、環境変数 HLC_NODE_ID で指定）
  readonly node_id: number;

  // EPOCH_WALL_MS: u64 の最小値として 0n を定数定義する（bigint リテラル）
  private static readonly EPOCH_WALL_MS: bigint = 0n;

  // U16_MAX: u16 の最大値（論理カウンタのオーバーフロー検出に使用する）
  static readonly U16_MAX: number = 65535;

  // U64_MAX: u64 の最大値（bigint でオーバーフロー検出に使用する）
  static readonly U64_MAX: bigint = 18446744073709551615n;

  // EPOCH: wall_ms=0n, logical=0, node_id=0 の最小タイムスタンプ（未初期化判定に使用）
  static readonly EPOCH: HlcTimestamp = new HlcTimestamp(0n, 0, 0);

  // constructor は wall_ms（bigint）・logical・node_id を受け取って HlcTimestamp を生成する
  constructor(wall_ms: bigint, logical: number, node_id: number) {
    // wall_ms を bigint として設定する（u64 相当の精度を保つ）
    this.wall_ms = wall_ms;
    // logical を number として設定する（u16 相当）
    this.logical = logical;
    // node_id を number として設定する（u16 相当）
    this.node_id = node_id;
  }

  // formatCompact は HlcTimestamp を文字列に変換する
  // 形式: "{wall_ms_hex_16}-{logical_04x}-{node_04x}"
  // tier2 cache_layer の generate_hlc_timestamp が生成していた形式と互換性を保つ
  formatCompact(): string {
    // wall_ms を 16 桁 hex 文字列にする（bigint の toString(16) を使う）
    const wallHex = this.wall_ms.toString(16).padStart(16, '0');
    // logical を 4 桁 hex 文字列にする
    const logicalHex = this.logical.toString(16).padStart(4, '0');
    // node_id を 4 桁 hex 文字列にする
    const nodeHex = this.node_id.toString(16).padStart(4, '0');
    // 3 パートをハイフン区切りで結合して返す
    return `${wallHex}-${logicalHex}-${nodeHex}`;
  }

  // parseCompact は formatCompact が出力した文字列を HlcTimestamp にパースする
  // パースに失敗した場合は null を返す（不正入力を呼び出し元で処理させる）
  static parseCompact(s: string): HlcTimestamp | null {
    // ハイフン区切りで最大 3 パートに分割する（wall_ms / logical / node_id）
    const parts = s.split('-');
    // パート数が正確に 3 でなければ null を返す（形式不正）
    if (parts.length !== 3) {
      return null;
    }
    // wall_ms パート: 16 桁 hex → bigint に変換する（パース失敗時は null）
    let wall_ms: bigint;
    try {
      // BigInt() に 0x プレフィックスを付けて hex として解析する
      wall_ms = BigInt('0x' + parts[0]);
    } catch {
      // 不正な hex 文字列の場合は null を返す
      return null;
    }
    // logical パート: 4 桁 hex → number に変換する
    const logical = parseInt(parts[1], 16);
    // node_id パート: 4 桁 hex → number に変換する
    const node_id = parseInt(parts[2], 16);
    // parseInt が NaN を返した場合は null を返す（パース失敗）
    if (isNaN(logical) || isNaN(node_id)) {
      return null;
    }
    // パースに成功した場合は HlcTimestamp を生成して返す
    return new HlcTimestamp(wall_ms, logical, node_id);
  }

  // addMs は self に durationMs を加算した deadline 用 HlcTimestamp を返す
  // wall-clock の直接使用を禁止するため、deadline 表現はこの関数を経由する
  // overflow 時は U64_MAX に飽和する（saturating_add 相当）
  addMs(durationMs: bigint): HlcTimestamp {
    // wall_ms に durationMs を加算して deadline の wall 部分を計算する
    const added = this.wall_ms + durationMs;
    // U64_MAX を超えた場合は U64_MAX に飽和させる（Rust の saturating_add と同等）
    const newWall = added > HlcTimestamp.U64_MAX ? HlcTimestamp.U64_MAX : added;
    // deadline の先頭イベントを表すため logical を 0 にリセットする
    // node_id は引き継ぐ（deadline の発行者を追跡する）
    return new HlcTimestamp(newWall, 0, this.node_id);
  }

  // elapsedMsSince は reference から self までの経過ミリ秒を返す
  // self が reference より前の場合は 0n を返す（負の elapsed は表現しない）
  // deadline との差分比較（expired 判定）に使用する
  elapsedMsSince(reference: HlcTimestamp): bigint {
    // wall_ms の差分を返す（self が reference 以前なら 0n に飽和する）
    const diff = this.wall_ms - reference.wall_ms;
    // 差分が負（self が reference より前）なら 0n を返す（飽和減算）
    return diff < 0n ? 0n : diff;
  }

  // isExpiredAt は deadline と比較して self が期限切れかどうかを返す
  // current: 現在の HLC タイムスタンプ（HlcClock.now() で取得）
  // true = current が self（deadline）を超えた = 期限切れ
  isExpiredAt(current: HlcTimestamp): boolean {
    // current が self 以上（後または同時）なら期限切れ
    return HlcTimestamp.compare(current, this) >= 0;
  }

  // compare は 2 つの HlcTimestamp を全順序で比較する静的メソッド
  // 戻り値: -1（a < b）/ 0（a === b）/ 1（a > b）
  // 比較順序: wall_ms → logical → node_id（辞書順）
  static compare(a: HlcTimestamp, b: HlcTimestamp): number {
    // wall_ms を比較する（bigint 比較）
    if (a.wall_ms < b.wall_ms) {
      return -1;
    }
    if (a.wall_ms > b.wall_ms) {
      return 1;
    }
    // wall_ms が同一なら logical を比較する
    if (a.logical < b.logical) {
      return -1;
    }
    if (a.logical > b.logical) {
      return 1;
    }
    // logical も同一なら node_id を比較する（完全全順序を保証する）
    if (a.node_id < b.node_id) {
      return -1;
    }
    if (a.node_id > b.node_id) {
      return 1;
    }
    // 全フィールドが同一なら 0（等値）を返す
    return 0;
  }
}

// HlcClock: シングルスレッド（JavaScript）向け HLC クロック
// tick / recv / now の 3 操作でイベント間の因果関係を追跡する
// JavaScript はシングルスレッドのため Mutex は不要。単純な state 管理で実装する。
export class HlcClock {
  // state_wall: 最後に観測した wall_ms（単調増加を保証するための状態）
  private state_wall: bigint;
  // state_logical: 最後に使用した logical カウンタ
  private state_logical: number;
  // node_id: このノード固有の識別子（HLC_NODE_ID 環境変数 or 0）
  private readonly node_id: number;

  // constructor は node_id を受け取って HlcClock を生成する
  // 複数ノード環境では異なる node_id を設定してタイムスタンプの衝突を避ける
  constructor(nodeId: number) {
    // 初期 wall_ms: 0n（first tick で物理クロックに更新される）
    this.state_wall = 0n;
    // 初期 logical: 0
    this.state_logical = 0;
    // node_id を設定する
    this.node_id = nodeId;
  }

  // fromEnv は環境変数 HLC_NODE_ID から node_id を読み込んで HlcClock を生成する
  // HLC_NODE_ID が未設定 / パース失敗時は node_id=0 を使用する
  // import.meta.env.HLC_NODE_ID（Vite 等のバンドラ環境）から取得する
  static fromEnv(): HlcClock {
    // import.meta.env が存在する場合に HLC_NODE_ID を取得する（Vite / Rollup 環境）
    let nodeIdStr: string | undefined;
    try {
      // import.meta.env は静的解析時には存在しない場合があるため try-catch で保護する
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const env = (import.meta as any).env;
      // env が存在すれば HLC_NODE_ID を取得する
      if (env != null) {
        nodeIdStr = env['HLC_NODE_ID'];
      }
    } catch {
      // import.meta.env が存在しない環境（Node.js 等）では undefined のままにする
    }
    // 環境変数が未設定 / 空の場合は node_id=0 を使用する
    if (nodeIdStr == null || nodeIdStr === '') {
      return new HlcClock(0);
    }
    // 文字列を 10 進整数にパースする
    const parsed = parseInt(nodeIdStr, 10);
    // パース失敗（NaN）または u16 範囲外の場合は 0 を使用する
    if (isNaN(parsed) || parsed < 0 || parsed > HlcTimestamp.U16_MAX) {
      return new HlcClock(0);
    }
    // パース成功した node_id を使って HlcClock を初期化する
    return new HlcClock(parsed);
  }

  // wallMsNow は現在の UNIX epoch からの経過ミリ秒を bigint で返す（内部専用）
  // HLC の wall-clock 部分の取得にのみ使用する（TTL/deadline 計算での直接使用禁止）
  private wallMsNow(): bigint {
    // Date.now() を呼び出す（本モジュール内でのみ許可、呼び出し元では禁止）
    return BigInt(Date.now());
  }

  // tick は send/local event のタイムスタンプを生成する（HLC の "send event"）
  // アルゴリズム:
  //   l' = max(state_wall, pt)
  //   if l' == state_wall: c' = state_logical + 1
  //   else: c' = 0
  tick(): HlcTimestamp {
    // pt: 現在の物理クロック（ms）を取得する
    const pt = this.wallMsNow();
    // l': max(state_wall, pt) を計算する（単調増加を保証する）
    const newWall = this.state_wall > pt ? this.state_wall : pt;
    // c': wall_ms が変化したかどうかで logical を更新する
    let newLogical: number;
    if (newWall === this.state_wall) {
      // wall_ms が変わらなければ logical を +1 する
      newLogical = this.state_logical + 1;
      // logical が u16 の最大値を超えた場合は例外を投げる（Rust の checked_add と同等）
      if (newLogical > HlcTimestamp.U16_MAX) {
        throw new Error('HLC logical counter overflow (max u16 = 65535)');
      }
    } else {
      // wall_ms が進んだ場合は logical を 0 にリセットする
      newLogical = 0;
    }
    // ステートを更新する（次回の tick/recv の比較基準になる）
    this.state_wall = newWall;
    this.state_logical = newLogical;
    // 生成した HlcTimestamp を返す
    return new HlcTimestamp(newWall, newLogical, this.node_id);
  }

  // recv は受信メッセージのタイムスタンプを踏まえてローカルクロックを更新する（HLC の "receive event"）
  // アルゴリズム（3-way max）:
  //   l' = max(state_wall, msg.wall_ms, pt)
  //   3 者の最大一致に応じて logical を更新する
  recv(msgTs: HlcTimestamp): HlcTimestamp {
    // pt: 現在の物理クロックを取得する
    const pt = this.wallMsNow();
    // l': max(state_wall, msg.wall_ms, pt) を計算する（3-way max）
    const maxWall = this.state_wall > msgTs.wall_ms ? this.state_wall : msgTs.wall_ms;
    const newWall = maxWall > pt ? maxWall : pt;
    // c': 3-way の最大一致パターンに応じて logical を更新する
    let newLogical: number;
    if (newWall === this.state_wall && newWall === msgTs.wall_ms) {
      // 3 者の wall_ms が同一: max(local_logical, msg_logical) + 1
      const maxLogical = this.state_logical > msgTs.logical ? this.state_logical : msgTs.logical;
      newLogical = maxLogical + 1;
      // logical がオーバーフローした場合は例外を投げる
      if (newLogical > HlcTimestamp.U16_MAX) {
        throw new Error('HLC logical counter overflow');
      }
    } else if (newWall === this.state_wall) {
      // ローカルの wall_ms が最大: local_logical + 1
      newLogical = this.state_logical + 1;
      // logical がオーバーフローした場合は例外を投げる
      if (newLogical > HlcTimestamp.U16_MAX) {
        throw new Error('HLC logical counter overflow');
      }
    } else if (newWall === msgTs.wall_ms) {
      // 受信メッセージの wall_ms が最大: msg_logical + 1
      newLogical = msgTs.logical + 1;
      // logical がオーバーフローした場合は例外を投げる
      if (newLogical > HlcTimestamp.U16_MAX) {
        throw new Error('HLC logical counter overflow');
      }
    } else {
      // pt が最大（物理クロックが両者を上回った）: logical を 0 にリセットする
      newLogical = 0;
    }
    // ステートを更新する
    this.state_wall = newWall;
    this.state_logical = newLogical;
    // 更新後の HlcTimestamp を返す
    return new HlcTimestamp(newWall, newLogical, this.node_id);
  }

  // now は現在の HLC タイムスタンプを生成する（tick の alias）
  // キャッシュエントリの cached_at_hlc フィールドへの書き込みに使用する
  now(): HlcTimestamp {
    // tick と等価：send event として扱う
    return this.tick();
  }
}
