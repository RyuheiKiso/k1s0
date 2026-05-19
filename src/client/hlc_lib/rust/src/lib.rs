// lib.rs — k1s0-hlc: Hybrid Logical Clock (HLC) の Rust 実装
// Kulkarni et al. (2014) "Logical Physical Clocks" のアルゴリズムを実装する。
// wall-clock 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// TTL / deadline の比較は HLC elapsed で管理し、SystemTime を直接参照しない。
// 本 crate が SystemTime を扱う唯一の許可された場所であり、呼び出し元は本 crate を経由する。

// 標準ライブラリの比較演算子 Ordering をインポートする
use std::cmp::Ordering;
// Mutex: HlcClock のステートをスレッドセーフに保護する
use std::sync::Mutex;
// SystemTime / UNIX_EPOCH: wall-clock の wall_ms 取得（記録目的のみ、TTL 計算禁止）
use std::time::{SystemTime, UNIX_EPOCH};

// serde feature が有効な場合は Serialize / Deserialize を derive する
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// HlcTimestamp: HLC のタイムスタンプ（wall_ms + logical + node_id の 3 tuple）
// 全順序（wall_ms > logical > node_id の辞書順）で比較可能。
// format_compact で "{wall_ms_hex_16}-{logical_04x}-{node_04x}" 形式に変換できる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// serde feature が有効な場合は Serialize / Deserialize を derive する
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HlcTimestamp {
    // wall_ms: UNIX epoch からの経過ミリ秒（wall-clock 部分、記録目的のみ）
    pub wall_ms: u64,
    // logical: 同一 wall_ms 内の単調カウンタ（最大 65535 = u16::MAX）
    pub logical: u16,
    // node_id: ノード識別子（複数インスタンスでの衝突回避、環境変数 HLC_NODE_ID で指定）
    pub node_id: u16,
}

impl HlcTimestamp {
    // EPOCH: wall_ms=0, logical=0, node_id=0 の最小タイムスタンプ（未初期化判定に使用）
    pub const EPOCH: Self = Self {
        wall_ms: 0,
        logical: 0,
        node_id: 0,
    };

    // format_compact は HlcTimestamp を文字列に変換する
    // 形式: "{wall_ms_hex_16}-{logical_04x}-{node_04x}"
    // tier2 cache_layer の generate_hlc_timestamp が生成していた形式と互換性を保つ
    pub fn format_compact(&self) -> String {
        // 16 桁 hex + 4 桁 hex + 4 桁 hex の形式でフォーマットする
        format!("{:016x}-{:04x}-{:04x}", self.wall_ms, self.logical, self.node_id)
    }

    // parse_compact は format_compact が出力した文字列を HlcTimestamp にパースする
    // パースに失敗した場合は None を返す（不正入力を呼び出し元で処理させる）
    pub fn parse_compact(s: &str) -> Option<Self> {
        // ハイフン区切りで 3 パートに分割する（先頭から順に wall_ms / logical / node_id）
        let mut parts = s.splitn(3, '-');
        // wall_ms パート: 16 桁 hex → u64 に変換する
        let wall_ms = u64::from_str_radix(parts.next()?, 16).ok()?;
        // logical パート: 4 桁 hex → u16 に変換する
        let logical = u16::from_str_radix(parts.next()?, 16).ok()?;
        // node_id パート: 4 桁 hex → u16 に変換する
        let node_id = u16::from_str_radix(parts.next()?, 16).ok()?;
        // 3 フィールドから HlcTimestamp を構築して返す
        Some(Self {
            wall_ms,
            logical,
            node_id,
        })
    }

    // add_ms は self に duration_ms を加算した deadline 用 HlcTimestamp を返す
    // wall-clock の直接使用を禁止するため、deadline 表現はこの関数を経由する
    // overflow 時は u64::MAX に飽和する（saturating_add）
    pub fn add_ms(&self, duration_ms: u64) -> Self {
        Self {
            // wall_ms に duration_ms を加算して deadline の wall 部分を計算する（飽和加算）
            wall_ms: self.wall_ms.saturating_add(duration_ms),
            // deadline の先頭イベントを表すため logical を 0 にリセットする
            logical: 0,
            // node_id は引き継ぐ（deadline の発行者を追跡する）
            node_id: self.node_id,
        }
    }

    // elapsed_ms_since は reference から self までの経過ミリ秒を返す
    // self が reference より前の場合は 0 を返す（負の elapsed は表現しない）
    // deadline との差分比較（expired 判定）に使用する
    pub fn elapsed_ms_since(&self, reference: &Self) -> u64 {
        // wall_ms の差分を返す（self が reference 以前なら 0 に飽和する）
        self.wall_ms.saturating_sub(reference.wall_ms)
    }

    // is_expired_at は deadline と比較して self が期限切れかどうかを返す
    // current: 現在の HLC タイムスタンプ（HlcClock::now() で取得）
    // true = current が self（deadline）を超えた = 期限切れ
    pub fn is_expired_at(&self, current: &Self) -> bool {
        // current が self より後（または同時）なら期限切れ
        current >= self
    }
}

impl PartialOrd for HlcTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Ord 実装に委譲する（HlcTimestamp は全順序）
        Some(self.cmp(other))
    }
}

impl Ord for HlcTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        // wall_ms が異なれば wall_ms で比較する（因果関係の主軸）
        match self.wall_ms.cmp(&other.wall_ms) {
            // wall_ms が同一なら logical で比較する（同一 ms 内の順序）
            Ordering::Equal => match self.logical.cmp(&other.logical) {
                // logical も同一なら node_id で比較する（完全全順序を保証する）
                Ordering::Equal => self.node_id.cmp(&other.node_id),
                ord => ord,
            },
            ord => ord,
        }
    }
}

// HlcClock: スレッドセーフな HLC クロック
// tick / recv / now の 3 操作でイベント間の因果関係を追跡する
// 複数スレッドから Arc<HlcClock> として共有して使用する
pub struct HlcClock {
    // state: (wall_ms, logical) のペアを Mutex で保護する（スレッドセーフ）
    state: Mutex<(u64, u16)>,
    // node_id: このノード固有の識別子（HLC_NODE_ID 環境変数 or 0）
    node_id: u16,
}

impl HlcClock {
    // new は node_id を受け取って HlcClock を生成する
    // 複数ノード環境では異なる node_id を設定してタイムスタンプの衝突を避ける
    pub fn new(node_id: u16) -> Self {
        Self {
            // 初期ステート: wall_ms=0, logical=0（first tick で物理クロックに更新される）
            state: Mutex::new((0, 0)),
            // node_id を設定する
            node_id,
        }
    }

    // from_env は環境変数 HLC_NODE_ID から node_id を読み込んで HlcClock を生成する
    // HLC_NODE_ID が未設定 / パース失敗時は node_id=0 を使用する
    pub fn from_env() -> Self {
        // HLC_NODE_ID 環境変数を文字列として取得する
        let node_id: u16 = std::env::var("HLC_NODE_ID")
            // 文字列として取得できなければ None に変換する
            .ok()
            // 文字列を u16 にパースする（失敗時は None）
            .and_then(|s| s.parse().ok())
            // None の場合は 0 を使用する（単一ノード環境のデフォルト）
            .unwrap_or(0);
        // node_id を使って HlcClock を初期化する
        Self::new(node_id)
    }

    // wall_ms_now は現在の UNIX epoch からの経過ミリ秒を返す（内部専用）
    // HLC の wall-clock 部分の取得にのみ使用する（TTL/deadline 計算での直接使用禁止）
    fn wall_ms_now() -> u64 {
        // SystemTime::now() を呼び出す（本 crate 内でのみ許可、呼び出し元では禁止）
        SystemTime::now()
            // UNIX_EPOCH からの経過 Duration を取得する（時刻巻き戻りは Err）
            .duration_since(UNIX_EPOCH)
            // Duration をミリ秒単位の u64 に変換する（overflow は 0 にフォールバック）
            .map(|d| d.as_millis() as u64)
            // 時刻巻き戻り（Err）の場合は 0 を返す
            .unwrap_or(0)
    }

    // tick は send/local event のタイムスタンプを生成する（HLC の "send event"）
    // アルゴリズム:
    //   l' = max(state.wall_ms, pt)
    //   if l' == state.wall_ms: c' = state.logical + 1
    //   else: c' = 0
    pub fn tick(&self) -> HlcTimestamp {
        // Mutex lock を取得する（poison の場合は panic で早期終了する）
        let mut state = self.state.lock().expect("HlcClock state Mutex poisoned");
        // pt: 現在の物理クロック（ms）を取得する
        let pt = Self::wall_ms_now();
        // l': max(state.wall_ms, pt) を計算する（単調増加を保証する）
        let new_wall = state.0.max(pt);
        // c': wall_ms が変化したかどうかで logical を更新する
        let new_logical = if new_wall == state.0 {
            // wall_ms が変わらなければ logical を +1 する（overflow 時は panic）
            state.1.checked_add(1).expect("HLC logical counter overflow (max u16::MAX)")
        } else {
            // wall_ms が進んだ場合は logical を 0 にリセットする
            0
        };
        // ステートを更新する（次回の tick/recv の比較基準になる）
        *state = (new_wall, new_logical);
        // 生成した HlcTimestamp を返す
        HlcTimestamp {
            wall_ms: new_wall,
            logical: new_logical,
            node_id: self.node_id,
        }
    }

    // recv は受信メッセージのタイムスタンプを踏まえてローカルクロックを更新する（HLC の "receive event"）
    // アルゴリズム（3-way max）:
    //   l' = max(state.wall_ms, msg.wall_ms, pt)
    //   3 者の最大一致に応じて logical を更新する
    pub fn recv(&self, msg_ts: &HlcTimestamp) -> HlcTimestamp {
        // Mutex lock を取得する
        let mut state = self.state.lock().expect("HlcClock state Mutex poisoned");
        // pt: 現在の物理クロックを取得する
        let pt = Self::wall_ms_now();
        // l': max(state.wall_ms, msg.wall_ms, pt) を計算する（3-way max）
        let new_wall = state.0.max(msg_ts.wall_ms).max(pt);
        // c': 3-way の最大一致パターンに応じて logical を更新する
        let new_logical = if new_wall == state.0 && new_wall == msg_ts.wall_ms {
            // 3 者の wall_ms が同一: max(local_logical, msg_logical) + 1
            state
                .1
                .max(msg_ts.logical)
                .checked_add(1)
                .expect("HLC logical counter overflow")
        } else if new_wall == state.0 {
            // ローカルの wall_ms が最大: local_logical + 1
            state.1.checked_add(1).expect("HLC logical counter overflow")
        } else if new_wall == msg_ts.wall_ms {
            // 受信メッセージの wall_ms が最大: msg_logical + 1
            msg_ts
                .logical
                .checked_add(1)
                .expect("HLC logical counter overflow")
        } else {
            // pt が最大（物理クロックが両者を上回った）: logical を 0 にリセットする
            0
        };
        // ステートを更新する
        *state = (new_wall, new_logical);
        // 更新後の HlcTimestamp を返す
        HlcTimestamp {
            wall_ms: new_wall,
            logical: new_logical,
            node_id: self.node_id,
        }
    }

    // now は現在の HLC タイムスタンプを生成する（tick の alias）
    // キャッシュエントリの cached_at_hlc フィールドへの書き込みに使用する
    pub fn now(&self) -> HlcTimestamp {
        // tick と等価：send event として扱う
        self.tick()
    }

    // peek は Mutex を保持したまま現在のステートを読む（テスト用）
    // production では now() / tick() を使用する
    #[cfg(test)]
    pub fn peek(&self) -> (u64, u16) {
        // Mutex を lock して現在のステートを返す
        *self.state.lock().expect("HlcClock state Mutex poisoned")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// テスト
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;
    // Arc: HlcClock を複数スレッドで共有するためにインポートする
    use std::sync::Arc;

    // HlcTimestamp の基本プロパティを確認するテスト
    #[test]
    fn test_hlc_timestamp_ordering() {
        // wall_ms が大きければ後のタイムスタンプであることを確認する
        let a = HlcTimestamp { wall_ms: 100, logical: 0, node_id: 0 };
        let b = HlcTimestamp { wall_ms: 200, logical: 0, node_id: 0 };
        assert!(a < b, "wall_ms が大きい方が後であるべき");
        // wall_ms が同じなら logical で比較することを確認する
        let c = HlcTimestamp { wall_ms: 100, logical: 1, node_id: 0 };
        assert!(a < c, "同一 wall_ms では logical が大きい方が後であるべき");
        // logical も同じなら node_id で比較することを確認する
        let d = HlcTimestamp { wall_ms: 100, logical: 0, node_id: 1 };
        assert!(a < d, "同一 wall_ms/logical では node_id が大きい方が後であるべき");
    }

    // format_compact と parse_compact のラウンドトリップを確認するテスト
    #[test]
    fn test_format_and_parse() {
        // 既知の値で HlcTimestamp を生成する
        let ts = HlcTimestamp { wall_ms: 0x0123456789abcdef, logical: 0x00ff, node_id: 0x1234 };
        // format_compact で文字列に変換する
        let formatted = ts.format_compact();
        // 期待値: 16 桁 hex-4 桁 hex-4 桁 hex
        assert_eq!(formatted, "0123456789abcdef-00ff-1234");
        // parse_compact でラウンドトリップが成立することを確認する
        let parsed = HlcTimestamp::parse_compact(&formatted).expect("parse_compact should succeed");
        assert_eq!(ts, parsed, "format_compact -> parse_compact ラウンドトリップ失敗");
    }

    // add_ms が正しく deadline を計算することを確認するテスト
    #[test]
    fn test_add_ms() {
        // 基準タイムスタンプを生成する
        let base = HlcTimestamp { wall_ms: 1_000_000, logical: 5, node_id: 1 };
        // 1000ms 後の deadline を計算する
        let deadline = base.add_ms(1_000);
        // wall_ms が正しく加算されることを確認する
        assert_eq!(deadline.wall_ms, 1_001_000, "add_ms が wall_ms を正しく加算すべき");
        // logical は 0 にリセットされることを確認する
        assert_eq!(deadline.logical, 0, "add_ms 後の logical は 0 であるべき");
        // node_id は引き継がれることを確認する
        assert_eq!(deadline.node_id, 1, "add_ms 後の node_id は引き継ぐべき");
    }

    // elapsed_ms_since と is_expired_at の動作を確認するテスト
    #[test]
    fn test_elapsed_and_expired() {
        // 現在時刻を表すタイムスタンプを生成する
        let now = HlcTimestamp { wall_ms: 2_000_000, logical: 0, node_id: 0 };
        // 1000ms 後の deadline を計算する
        let deadline = now.add_ms(1_000);
        // now では期限切れでないことを確認する
        assert!(!deadline.is_expired_at(&now), "deadline より前では期限切れにならないべき");
        // deadline ちょうどで期限切れになることを確認する
        assert!(deadline.is_expired_at(&deadline), "deadline ちょうどで期限切れになるべき");
        // deadline を過ぎたら期限切れになることを確認する
        let future = HlcTimestamp { wall_ms: 2_001_001, logical: 0, node_id: 0 };
        assert!(deadline.is_expired_at(&future), "deadline を超えたら期限切れになるべき");
        // elapsed_ms_since の値が正しいことを確認する
        assert_eq!(future.elapsed_ms_since(&now), 1_001, "elapsed_ms_since が正しくあるべき");
    }

    // HlcClock::tick が単調増加タイムスタンプを生成することを確認するテスト
    #[test]
    fn test_tick_monotonic() {
        // node_id=0 で HlcClock を生成する
        let clock = HlcClock::new(0);
        // 連続で 100 回 tick して全て単調増加することを確認する
        let mut prev = clock.tick();
        for _ in 0..99 {
            // 次のタイムスタンプを生成する
            let next = clock.tick();
            // prev よりも next が後（または同時）であることを確認する
            assert!(next >= prev, "tick は単調増加を保証すべき: prev={:?}, next={:?}", prev, next);
            // prev を更新する
            prev = next;
        }
    }

    // HlcClock::recv がメッセージの因果関係を正しく反映することを確認するテスト
    #[test]
    fn test_recv_causality() {
        // 送信者クロックを生成する
        let sender = HlcClock::new(1);
        // 受信者クロックを生成する
        let receiver = HlcClock::new(2);
        // 送信者で tick する
        let send_ts = sender.tick();
        // 受信者で recv する（send_ts の後になるはずである）
        let recv_ts = receiver.recv(&send_ts);
        // recv の結果が send_ts より後（または同時）であることを確認する
        assert!(
            recv_ts >= send_ts,
            "recv 後のタイムスタンプは send より後であるべき: send={:?}, recv={:?}",
            send_ts,
            recv_ts
        );
    }

    // 複数スレッドで concurrent tick が単調増加を維持することを確認するテスト
    #[tokio::test]
    async fn test_concurrent_tick() {
        // Arc で HlcClock を共有する
        let clock = Arc::new(HlcClock::new(0));
        // 10 スレッドで並行 tick を実行する
        let mut handles = Vec::new();
        for _ in 0..10 {
            // Arc をクローンしてスレッドに移動する
            let c = Arc::clone(&clock);
            handles.push(tokio::spawn(async move {
                // 各スレッドで 100 回 tick する
                let mut results = Vec::new();
                for _ in 0..100 {
                    results.push(c.tick());
                }
                // 結果を返す
                results
            }));
        }
        // 全スレッドの結果を収集する
        let mut all: Vec<HlcTimestamp> = Vec::new();
        for h in handles {
            all.extend(h.await.expect("task panicked"));
        }
        // 全タイムスタンプをソートして重複がないことを確認する
        all.sort();
        all.dedup();
        // 1000 個（10 スレッド × 100 tick）の unique タイムスタンプが生成されたことを確認する
        assert_eq!(all.len(), 1000, "concurrent tick は 1000 個の unique タイムスタンプを生成すべき");
    }

    // EPOCH 定数が最小タイムスタンプであることを確認するテスト
    #[test]
    fn test_epoch_is_minimum() {
        // 任意の非 EPOCH タイムスタンプを生成する
        let ts = HlcTimestamp { wall_ms: 1, logical: 0, node_id: 0 };
        // EPOCH が ts より前であることを確認する
        assert!(HlcTimestamp::EPOCH < ts, "EPOCH は全タイムスタンプの最小値であるべき");
    }
}
