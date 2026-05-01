// 本ファイルは PII 検出 / マスキングの中核実装。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-009（t1-pii: 純関数ステートレス、parser / masker / rule_fetcher / api_server の 4 module）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 役割:
//   テキストに対して複数の PII 検出ルール（regex）を適用し、
//   `findings`（位置 + 種別 + 信頼度）を返す。マスキング時は findings を
//   後ろから順に置換することで位置ズレを発生させない。
//
// 検出種別（リリース時点組込）:
//   EMAIL    : RFC 5322 の素朴な subset
//   PHONE    : 日本固定電話 / 携帯（0X-XXXX-XXXX 形式）
//   MYNUMBER : マイナンバー（連続 12 桁、緩い検出）
//   CREDITCARD : 13〜19 桁数字（- や半角空白の区切り許容、緩い検出）
//   IPV4     : ドット区切り 4 オクテット
//   ADDRESS  : 日本の住所（郵便番号 〒NNN-NNNN または 都道府県 + 後続文字列）
//   NAME     : 検出が困難なため保守的に「英字 First Last」のみ拾う（信頼度 0.4）
//
// 信頼度の指針:
//   完全マッチ系（メール / マイナンバー / IP）: 0.9
//   ヒューリスティック（電話 / クレカ）: 0.7
//   推測（NAME）: 0.4
//
// 設計上の決定:
//   - regex は once_cell::Lazy で静的コンパイル（プロセス起動時に 1 回だけコンパイル）
//   - 純関数（&self も &mut self も持たない、Masker は副作用なし）
//   - 検出位置は **char index** ではなく **byte offset** で返す（regex 標準に整合）。
//     proto は char index 仕様だが char/byte 変換は呼び出し層で実施する想定。

// 必要な依存。
use once_cell::sync::Lazy;
use regex::Regex;

/// PII 種別（proto の string 値と直接対応する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiiKind {
    /// 氏名（英字 First Last の素朴検出のみ）
    Name,
    /// メールアドレス
    Email,
    /// 電話番号
    Phone,
    /// マイナンバー（12 桁）
    MyNumber,
    /// クレジットカード番号
    CreditCard,
    /// IPv4 アドレス
    IPv4,
    /// 住所（郵便番号 + 都道府県以降のヒューリスティック）
    Address,
}

impl PiiKind {
    /// proto findings.type に詰める文字列表現。
    pub fn as_str(self) -> &'static str {
        match self {
            PiiKind::Name => "NAME",
            PiiKind::Email => "EMAIL",
            PiiKind::Phone => "PHONE",
            PiiKind::MyNumber => "MYNUMBER",
            PiiKind::CreditCard => "CREDITCARD",
            PiiKind::IPv4 => "IPV4",
            PiiKind::Address => "ADDRESS",
        }
    }
    /// マスク時の置換トークン。
    pub fn mask_token(self) -> &'static str {
        match self {
            PiiKind::Name => "[NAME]",
            PiiKind::Email => "[EMAIL]",
            PiiKind::Phone => "[PHONE]",
            PiiKind::MyNumber => "[MYNUMBER]",
            PiiKind::CreditCard => "[CREDITCARD]",
            PiiKind::IPv4 => "[IPV4]",
            PiiKind::Address => "[ADDRESS]",
        }
    }
}

/// 1 件の検出結果（byte offset 付き）。
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// 検出種別。
    pub kind: PiiKind,
    /// テキスト内の開始 byte offset（0 始まり、UTF-8 byte 単位）。
    pub start: usize,
    /// テキスト内の終了 byte offset（exclusive）。
    pub end: usize,
    /// 信頼度 0.0〜1.0。
    pub confidence: f64,
}

/// rule の登録形式（種別 + 正規表現 + 信頼度）。
#[derive(Debug)]
struct Rule {
    kind: PiiKind,
    re: Regex,
    confidence: f64,
}

/// グローバル ruleset（lazy static）。プロセス内で 1 回だけ初期化される。
static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // メールアドレス: RFC 5322 の素朴版。@ 周辺で広めに拾う。
        Rule {
            kind: PiiKind::Email,
            re: Regex::new(r"\b[\w.+\-]+@[\w\-]+(?:\.[\w\-]+)+\b").unwrap(),
            confidence: 0.9,
        },
        // マイナンバー: 連続 12 桁。誤検出避けに word boundary を要求。
        Rule {
            kind: PiiKind::MyNumber,
            re: Regex::new(r"\b\d{12}\b").unwrap(),
            confidence: 0.9,
        },
        // クレジットカード: 13〜19 桁の数字、- や半角空白での区切り許容（緩い）。
        // word boundary は付けない（区切り文字込みで括るため）。
        Rule {
            kind: PiiKind::CreditCard,
            re: Regex::new(r"(?:\d[ \-]?){12,18}\d").unwrap(),
            confidence: 0.7,
        },
        // 日本国内電話: 先頭 0、続いて 1〜4 桁、ハイフン任意で終端 4 桁を 2 ブロック。
        Rule {
            kind: PiiKind::Phone,
            re: Regex::new(r"\b0\d{1,4}-?\d{1,4}-?\d{4}\b").unwrap(),
            confidence: 0.7,
        },
        // IPv4: 4 オクテット。範囲チェックは regex で厳密化（0-255 の各 octet）。
        Rule {
            kind: PiiKind::IPv4,
            re: Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d?\d)\b").unwrap(),
            confidence: 0.9,
        },
        // 氏名: 英字 First Last の素朴検出のみ（信頼度低）。
        Rule {
            kind: PiiKind::Name,
            re: Regex::new(r"\b[A-Z][a-z]+ [A-Z][a-z]+\b").unwrap(),
            confidence: 0.4,
        },
        // 住所 (1): 日本の郵便番号「〒NNN-NNNN」または 7 桁通常表記。
        // 続く文字列（都道府県 / 市区町村 / 番地）も含めて 80 文字までを拾う。
        // 都道府県名で始まる住所も同 rule で拾うため、市町村のみの短縮表記は対象外（false negative 受容）。
        Rule {
            kind: PiiKind::Address,
            re: Regex::new(
                r"(?:〒\s*\d{3}-?\d{4}|\b\d{3}-\d{4}\b)[^\n,。]{0,80}",
            )
            .unwrap(),
            confidence: 0.7,
        },
        // 住所 (2): 都道府県（47 + 1）で始まる「県/府/都/道 + 任意 + 市区町村 + 番地」。
        // 47 prefecture を network of OR で列挙する。先頭が都道府県名なら「住所」として
        // 抽出（false positive あり: 文中の都道府県言及を過拾。confidence 0.5 で運用閾値で抑制）。
        Rule {
            kind: PiiKind::Address,
            re: Regex::new(
                r"(?:北海道|青森県|岩手県|宮城県|秋田県|山形県|福島県|茨城県|栃木県|群馬県|埼玉県|千葉県|東京都|神奈川県|新潟県|富山県|石川県|福井県|山梨県|長野県|岐阜県|静岡県|愛知県|三重県|滋賀県|京都府|大阪府|兵庫県|奈良県|和歌山県|鳥取県|島根県|岡山県|広島県|山口県|徳島県|香川県|愛媛県|高知県|福岡県|佐賀県|長崎県|熊本県|大分県|宮崎県|鹿児島県|沖縄県)[^\s,。\n]{2,60}",
            )
            .unwrap(),
            confidence: 0.5,
        },
    ]
});

/// PII 検出器。Lazy 初期化された RULES を参照するだけのステートレス struct。
#[derive(Debug, Default, Clone, Copy)]
pub struct Masker;

impl Masker {
    /// 新規 Masker を生成（実態は zero-sized type）。
    pub fn new() -> Self {
        Masker
    }

    /// テキストに対して全 rule を適用し、検出 findings を位置順で返す。
    ///
    /// 重複範囲は **後勝ち** で除外しない（重なって検出された場合は両方残す）。
    /// マスク時は呼び出し側で重複処理を行う想定。
    pub fn classify(&self, text: &str) -> Vec<Finding> {
        let mut findings: Vec<Finding> = Vec::new();
        for rule in RULES.iter() {
            for m in rule.re.find_iter(text) {
                findings.push(Finding {
                    kind: rule.kind,
                    start: m.start(),
                    end: m.end(),
                    confidence: rule.confidence,
                });
            }
        }
        // 開始位置で sort（同位置の場合は終了位置の遅い方を優先 = 長い match を残す）。
        findings.sort_by_key(|f| (f.start, std::cmp::Reverse(f.end)));
        findings
    }

    /// テキストをマスキングして返す。findings は **マスキング前の位置情報**。
    /// 重複検出は位置の長い方（先勝ち）を採用し、重なる短い検出は drop する。
    pub fn mask(&self, text: &str) -> (String, Vec<Finding>) {
        let mut findings = self.classify(text);
        // 重複範囲を除去（先頭から走査し、前 finding の end を超えていない start は drop）。
        let mut deduped: Vec<Finding> = Vec::with_capacity(findings.len());
        let mut last_end = 0usize;
        for f in findings.drain(..) {
            if f.start >= last_end {
                last_end = f.end;
                deduped.push(f);
            }
            // else: 既存 finding と重なる → drop
        }
        // 後ろから置換することで位置ズレを発生させない。
        let mut masked = text.to_string();
        for f in deduped.iter().rev() {
            // byte offset で safe に slice する（UTF-8 の場合は char boundary に乗っている前提、
            // regex の match offset は char boundary 上にあるため安全）。
            masked.replace_range(f.start..f.end, f.kind.mask_token());
        }
        (masked, deduped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_email() {
        let m = Masker::new();
        let f = m.classify("contact me at alice@example.co.jp now");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, PiiKind::Email);
        assert_eq!(&"contact me at alice@example.co.jp now"[f[0].start..f[0].end], "alice@example.co.jp");
    }

    #[test]
    fn classify_mynumber_and_phone() {
        let m = Masker::new();
        let text = "番号 123456789012、電話 03-1234-5678";
        let f = m.classify(text);
        // 12 桁マイナンバー + 電話 1 件。
        assert!(f.iter().any(|x| x.kind == PiiKind::MyNumber));
        assert!(f.iter().any(|x| x.kind == PiiKind::Phone));
    }

    #[test]
    fn classify_credit_card() {
        let m = Masker::new();
        let f = m.classify("card 4242-4242-4242-4242 ok");
        assert!(f.iter().any(|x| x.kind == PiiKind::CreditCard));
    }

    #[test]
    fn classify_ipv4() {
        let m = Masker::new();
        let f = m.classify("server at 192.168.1.42:8080");
        assert!(f.iter().any(|x| x.kind == PiiKind::IPv4));
    }

    #[test]
    fn classify_address_postal_code() {
        let m = Masker::new();
        let f = m.classify("住所 〒100-0001 東京都千代田区千代田1-1 です");
        assert!(
            f.iter().any(|x| x.kind == PiiKind::Address),
            "postal code address must be detected, got: {:?}",
            f
        );
    }

    #[test]
    fn classify_address_prefecture_only() {
        let m = Masker::new();
        let f = m.classify("配送先は神奈川県横浜市西区みなとみらい3-1");
        assert!(
            f.iter().any(|x| x.kind == PiiKind::Address),
            "prefecture-prefixed address must be detected, got: {:?}",
            f
        );
    }

    #[test]
    fn mask_address_replaces_with_token() {
        let m = Masker::new();
        let (out, findings) = m.mask("住所: 大阪府大阪市北区梅田1-2-3 連絡先");
        assert!(findings.iter().any(|f| f.kind == PiiKind::Address));
        assert!(out.contains("[ADDRESS]"), "address must be masked: {}", out);
        assert!(!out.contains("梅田"), "address payload must be erased: {}", out);
    }

    #[test]
    fn mask_replaces_with_token() {
        let m = Masker::new();
        let (out, findings) = m.mask("ping alice@k1s0.io and 192.168.1.1 down");
        assert_eq!(findings.len(), 2);
        assert!(out.contains("[EMAIL]"));
        assert!(out.contains("[IPV4]"));
        assert!(!out.contains("alice@k1s0.io"));
        assert!(!out.contains("192.168.1.1"));
    }

    #[test]
    fn mask_no_pii_returns_unchanged() {
        let m = Masker::new();
        let (out, f) = m.mask("hello world");
        assert_eq!(out, "hello world");
        assert!(f.is_empty());
    }

    #[test]
    fn mask_overlapping_range_keeps_first() {
        let m = Masker::new();
        // 12 桁数字はマイナンバーともクレカ短(13桁未満ではない、ぎり境界)とも見える可能性。
        // 実装では classify 順序（Email→MyNumber→CC→Phone→IPv4→Name）で先勝ち重複除去。
        let (out, _) = m.mask("xx 123456789012 xx");
        // MyNumber が拾われ、CreditCard 範囲とは重ならない（CC は 13 桁以上）ため
        // [MYNUMBER] のみ。
        assert!(out.contains("[MYNUMBER]"));
        assert!(!out.contains("[CREDITCARD]"));
    }

    #[test]
    fn classify_returns_sorted_by_start() {
        let m = Masker::new();
        let f = m.classify("a@b.com plus c@d.com");
        // sorted によって 2 件が start 順で並ぶ。
        assert_eq!(f.len(), 2);
        assert!(f[0].start < f[1].start);
    }

    #[test]
    fn pii_kind_strings() {
        // proto string 値の整合性回帰テスト。
        assert_eq!(PiiKind::Email.as_str(), "EMAIL");
        assert_eq!(PiiKind::MyNumber.mask_token(), "[MYNUMBER]");
    }
}
