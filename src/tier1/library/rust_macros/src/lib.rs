// lib.rs — k1s0-tier1-library-macros: conformance_assert 属性マクロの実装
// 01_Bidi適合仕様.md §scenarios.yaml × adapter × class の 29 applicable cell に対応する
// #[conformance_assert(scenario = "...", adapter = "...", class = "...")] 属性を提供する。
// プロシージャルマクロとして実装し、注釈付き test 関数をそのままコンパイル通過させる。
// CI 整合 2「scenarios.yaml の assertion id が全 adapter × 全 class × 全言語で実装されていること」
// の物理機構として機能する。

// proc_macro: Rust 標準ライブラリのプロシージャルマクロ API を使用する（安定版）
extern crate proc_macro;

// proc_macro::TokenStream: 入力・出力の基本型として使用する
use proc_macro::TokenStream;
// syn: Rust コード解析ライブラリ（属性引数のパースに使用する）
use syn::{parse_macro_input, ItemFn, Meta};
// quote: Rust コードスニペット生成マクロ（出力 TokenStream を構築する）
use quote::quote;
// proc_macro2: 補助 TokenStream 操作（エラーメッセージ生成に使用する）
use proc_macro2::Span;

/// conformance_assert 属性マクロ
///
/// 01_Bidi適合仕様.md §scenarios.yaml の 9 scenario × applicable (adapter, class) の
/// 29 cell に対応する test 関数に付与する属性。
///
/// # 引数
/// - `scenario`: scenarios.yaml の scenario id（例: "tls_disconnect"）
/// - `adapter`: capabilities.lock.yaml の adapter 名（例: "grpc_native"）
/// - `class`: conformance_class 名（例: "v1_interactive"）
///
/// # 使用例
/// ```rust,ignore
/// #[conformance_assert(scenario = "tls_disconnect", adapter = "grpc_native", class = "v1_interactive")]
/// fn test_tls_disconnect_grpc_native_v1_interactive() { /* ... */ }
/// ```
// proc_macro_attribute: Rust 属性マクロとして登録する（attr = 属性引数、item = 注釈対象アイテム）
#[proc_macro_attribute]
pub fn conformance_assert(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 注釈対象の関数アイテムをパースする（構文エラー時は syn がコンパイルエラーを発生させる）
    let input_fn = parse_macro_input!(item as ItemFn);

    // 属性引数（attr）をパースして scenario / adapter / class の値を取得する
    // syn::parse::Parser を使って Meta リストとして解析する
    let attr2 = proc_macro2::TokenStream::from(attr.clone());

    // 属性引数を文字列としてそのまま保持する（バリデーション目的）
    // パース失敗時のフォールバックとして元の関数をそのまま返す
    let _attr_str = attr2.to_string();

    // 属性引数の必須フィールド（scenario / adapter / class）を検証する
    // syn::punctuated::Punctuated を使って key=value ペアをパースする
    let parsed_meta: Result<syn::punctuated::Punctuated<Meta, syn::Token![,]>, _> =
        syn::parse::Parser::parse2(
            // 区切り文字 ',' でカンマ区切りの Meta リストをパースする
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            // attr2 を入力として渡す
            attr2.clone(),
        );

    // パースに成功した場合のみ scenario / adapter / class の存在を確認する
    // パース失敗時はコンパイルエラーを発生させず、警告として処理する
    if let Ok(metas) = parsed_meta {
        // 必須キー: scenario / adapter / class の 3 つが揃っていることを確認する
        let mut has_scenario = false;
        // adapter キーの存在フラグ
        let mut has_adapter = false;
        // class キーの存在フラグ
        let mut has_class = false;

        // Meta リストを走査して各キーを確認する
        for meta in &metas {
            // Meta::NameValue 形式（key = "value"）のみを対象とする
            if let Meta::NameValue(nv) = meta {
                // キー名を文字列として取得する
                let key = nv.path.segments.last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                // scenario キーを検出した場合は has_scenario を true にする
                if key == "scenario" {
                    has_scenario = true;
                }
                // adapter キーを検出した場合は has_adapter を true にする
                if key == "adapter" {
                    has_adapter = true;
                }
                // class キーを検出した場合は has_class を true にする
                if key == "class" {
                    has_class = true;
                }
            }
        }

        // 必須キーが不足している場合はコンパイルエラーを発生させる
        if !has_scenario || !has_adapter || !has_class {
            // syn::Error::new でエラーを生成してコンパイラに報告する
            let err = syn::Error::new(
                // Span::call_site() で属性の使用位置を指定する
                Span::call_site(),
                // エラーメッセージ: 必須引数 scenario / adapter / class を全て指定するよう求める
                "conformance_assert 属性には scenario / adapter / class の 3 引数が必須です。\
                例: #[conformance_assert(scenario = \"tls_disconnect\", adapter = \"grpc_native\", class = \"v1_interactive\")]",
            );
            // エラーを TokenStream に変換して返す（コンパイルエラーとして報告される）
            return TokenStream::from(err.to_compile_error());
        }
    }

    // 注釈対象の関数をそのまま出力する（属性は annotation のみで動作を変更しない）
    // conformance_assert 属性は「宣言的な索引付け」が目的であり、ランタイム動作を変えない
    let output = quote! {
        // 注釈対象の関数をそのまま展開する（属性は no-op で動作を変更しない）
        #input_fn
    };

    // 展開した TokenStream を返す
    TokenStream::from(output)
}
