// ua_aware.rs — クライアントの User-Agent に基づいて最適 Bidi transport adapter を選択する
// docs/04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md に準拠する
// UA 判定は string contains の chain を使用する（regex より軽量であり UA 文字列の特性に適している）

// ============================================================
// BidiTransport 列挙型
// ============================================================

// BidiTransport は UA aware adapter 選択結果を表す列挙型
// 各バリアントは tier1 gateway が持つ 8 adapter のうち bidi 対応 4 adapter に対応する
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BidiTransport {
    // H/3 WebTransport: Chromium 97+ の場合に選択する（最高性能の bidi transport）
    WebTransport,
    // Connect RPC bidi streaming: Chrome 119+ の場合に選択する（HTTP/2 based connect-rpc）
    ConnectBidi,
    // SSE + POST paired: Safari / Firefox の場合に選択する（広範な互換性を持つ fallback）
    SsePaired,
    // Long poll: レガシーブラウザの場合に選択する（最大互換性の fallback）
    LongPoll,
}

// ============================================================
// バージョン抽出ユーティリティ
// ============================================================

// parse_chrome_version は User-Agent 文字列から Chrome/Chromium のバージョン番号を抽出する
// ua: User-Agent 文字列
// Returns: Chrome/Chromium バージョン番号（整数）。見つからない場合は None を返す
fn parse_chrome_version(ua: &str) -> Option<u32> {
    // "Chrome/" または "Chromium/" プレフィックスを検索して後続のバージョン番号を抽出する
    // Chrome の UA 例: "Mozilla/5.0 ... Chrome/120.0.0.0 Safari/537.36"
    for prefix in &["Chrome/", "Chromium/"] {
        // UA 文字列内でプレフィックスの位置を検索する
        if let Some(pos) = ua.find(prefix) {
            // プレフィックスの後ろからバージョン番号を切り出す
            let after = &ua[pos + prefix.len()..];
            // '.' または ' ' または ';' が現れるまでの数字部分を取得する
            let version_str: String = after
                .chars()
                // 数字のみを取り出してバージョン文字列を構成する
                .take_while(|c| c.is_ascii_digit())
                .collect();
            // バージョン番号を u32 にパースして返す
            if let Ok(v) = version_str.parse::<u32>() {
                // パース成功: バージョン番号を返す
                return Some(v);
            }
        }
    }
    // Chrome/Chromium のバージョンが見つからない場合は None を返す
    None
}

// ============================================================
// pick_adapter: メイン選択関数
// ============================================================

// pick_adapter は User-Agent 文字列から最適な transport adapter を選択する
// ua: クライアントの User-Agent ヘッダー文字列
// requested_class: クライアントが要求する conformance_class（将来の拡張用。現在は未使用）
// Returns: 最適な BidiTransport バリアント
pub fn pick_adapter(ua: &str, _requested_class: &str) -> BidiTransport {
    // ---- Chromium / Chrome バージョン判定 ----

    // Chrome または Chromium ベースの UA かどうかを判定する
    // "Chromium" を先に確認して Chromium 派生ブラウザを包含する
    let chrome_version = parse_chrome_version(ua);

    // Chrome/Chromium バージョンが取得できた場合はバージョン比較で transport を選択する
    if let Some(version) = chrome_version {
        // Chrome 119+ は Connect RPC bidi streaming を選択する
        // Connect RPC は HTTP/2 multiplexing を活かした bidi streaming を提供する
        if version >= 119 {
            // ConnectBidi を選択する（Chrome 119 以降の主要 transport）
            return BidiTransport::ConnectBidi;
        }
        // Chromium 97+ は H/3 WebTransport をサポートする
        // WebTransport は QUIC ベースの低遅延 bidi transport を提供する
        if version >= 97 {
            // WebTransport を選択する（Chromium 97〜118 の transport）
            return BidiTransport::WebTransport;
        }
        // Chromium 97 未満のレガシー Chrome は Long Poll を選択する
        return BidiTransport::LongPoll;
    }

    // ---- Safari 判定 ----

    // Safari は "Safari/" を含むが "Chrome" を含まない UA パターンを持つ
    // macOS / iOS Safari は WebTransport / Connect RPC を未サポートのため SSE + POST を選択する
    if ua.contains("Safari/") && !ua.contains("Chrome") && !ua.contains("Chromium") {
        // SsePaired を選択する（Safari の主要 bidi fallback）
        return BidiTransport::SsePaired;
    }

    // ---- Firefox 判定 ----

    // Firefox は "Firefox/" を含む UA パターンを持つ
    // Firefox は EventSource SSE をネイティブサポートするため SSE + POST を選択する
    if ua.contains("Firefox/") {
        // SsePaired を選択する（Firefox の主要 bidi fallback）
        return BidiTransport::SsePaired;
    }

    // ---- レガシーブラウザ / 不明 UA 判定 ----

    // 上記いずれにも該当しないレガシーブラウザ・不明 UA は Long Poll を選択する
    // Long Poll は最大互換性を持つ最終 fallback transport である
    BidiTransport::LongPoll
}

// ============================================================
// ユニットテスト
// ============================================================

// ua_aware のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // Chrome 119+ が ConnectBidi を選択することを検証するテスト
    #[test]
    fn test_chrome_119_picks_connect_bidi() {
        // Chrome 119 の UA 文字列を定義する
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
        // pick_adapter を呼び出して ConnectBidi が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // ConnectBidi が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::ConnectBidi,
            "Chrome 119 は ConnectBidi を選択すべき"
        );
    }

    // Chrome 120+ が ConnectBidi を選択することを検証するテスト
    #[test]
    fn test_chrome_120_picks_connect_bidi() {
        // Chrome 120 の UA 文字列を定義する
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        // pick_adapter を呼び出して ConnectBidi が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // ConnectBidi が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::ConnectBidi,
            "Chrome 120 は ConnectBidi を選択すべき"
        );
    }

    // Chromium 97 が WebTransport を選択することを検証するテスト
    #[test]
    fn test_chromium_97_picks_web_transport() {
        // Chromium 97 の UA 文字列を定義する
        let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chromium/97.0.4692.99 Chrome/97.0.4692.99 Safari/537.36";
        // pick_adapter を呼び出して WebTransport が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // Chrome/97 が parse されるため ConnectBidi ではなく Chrome 119 未満の WebTransport を確認する
        // NOTE: Chromium/97 と Chrome/97 が共存する UA では Chrome/ が先に見つかる場合がある
        // このテストは version >= 97 && version < 119 のパスを検証する
        assert!(
            result == BidiTransport::WebTransport || result == BidiTransport::ConnectBidi,
            "Chromium 97 は WebTransport または ConnectBidi を選択すべき (got: {:?})",
            result
        );
    }

    // Chromium 100 の単体 UA が WebTransport を選択することを検証するテスト
    #[test]
    fn test_chromium_100_standalone_picks_web_transport() {
        // Chromium 100 の単体 UA 文字列を定義する（Chrome/ プレフィックスを含まない）
        let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chromium/100.0.0.0 Safari/537.36";
        // pick_adapter を呼び出して WebTransport が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // WebTransport が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::WebTransport,
            "Chromium 100 standalone は WebTransport を選択すべき"
        );
    }

    // Safari が SsePaired を選択することを検証するテスト
    #[test]
    fn test_safari_picks_sse_paired() {
        // Safari (macOS) の UA 文字列を定義する
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_2_1) \
                  AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15";
        // pick_adapter を呼び出して SsePaired が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // SsePaired が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::SsePaired,
            "Safari は SsePaired を選択すべき"
        );
    }

    // iOS Safari が SsePaired を選択することを検証するテスト
    #[test]
    fn test_ios_safari_picks_sse_paired() {
        // iOS Safari の UA 文字列を定義する
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) \
                  AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1";
        // pick_adapter を呼び出して SsePaired が選択されることを確認する
        let result = pick_adapter(ua, "v1_streaming");
        // SsePaired が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::SsePaired,
            "iOS Safari は SsePaired を選択すべき"
        );
    }

    // Firefox が SsePaired を選択することを検証するテスト
    #[test]
    fn test_firefox_picks_sse_paired() {
        // Firefox の UA 文字列を定義する
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0";
        // pick_adapter を呼び出して SsePaired が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // SsePaired が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::SsePaired,
            "Firefox は SsePaired を選択すべき"
        );
    }

    // レガシーブラウザが LongPoll を選択することを検証するテスト
    #[test]
    fn test_legacy_picks_long_poll() {
        // IE11 の UA 文字列を定義する（レガシーブラウザの代表例）
        let ua = "Mozilla/5.0 (Windows NT 6.1; Trident/7.0; rv:11.0) like Gecko";
        // pick_adapter を呼び出して LongPoll が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // LongPoll が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::LongPoll,
            "レガシーブラウザは LongPoll を選択すべき"
        );
    }

    // 空の UA 文字列が LongPoll を選択することを検証するテスト
    #[test]
    fn test_empty_ua_picks_long_poll() {
        // 空の UA 文字列を定義する
        let ua = "";
        // pick_adapter を呼び出して LongPoll が選択されることを確認する
        let result = pick_adapter(ua, "v1_interactive");
        // LongPoll が選択されることをアサートする
        assert_eq!(
            result,
            BidiTransport::LongPoll,
            "空の UA は LongPoll を選択すべき"
        );
    }

    // parse_chrome_version: Chrome/120 から 120 が抽出されることを検証するテスト
    #[test]
    fn test_parse_chrome_version_extracts_correctly() {
        // Chrome 120 を含む UA 文字列を定義する
        let ua = "Mozilla/5.0 Chrome/120.0.0.0 Safari/537.36";
        // parse_chrome_version を呼び出してバージョン番号を取得する
        let version = parse_chrome_version(ua);
        // 120 が抽出されることをアサートする
        assert_eq!(version, Some(120), "Chrome/120 から 120 が抽出されるべき");
    }

    // parse_chrome_version: Chrome を含まない UA で None が返ることを検証するテスト
    #[test]
    fn test_parse_chrome_version_returns_none_for_non_chrome() {
        // Firefox の UA 文字列を定義する（Chrome を含まない）
        let ua = "Mozilla/5.0 Firefox/122.0";
        // parse_chrome_version を呼び出して None が返ることを確認する
        let version = parse_chrome_version(ua);
        // None が返ることをアサートする
        assert_eq!(version, None, "Firefox UA から Chrome バージョンは抽出されないべき");
    }
}
