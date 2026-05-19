// k1s0-tier2-notary クレートのルートモジュール
// RFC 3161 タイムスタンプ認証（TSA）および Sigstore Rekor 透明性ログ連携を提供する

// tsa_client モジュール: RFC 3161 TSA 呼び出しと Rekor ログエントリ登録を提供する
pub mod tsa_client;

// TsaClient を公開 API として再エクスポートする（クレート利用者が直接インポートできるようにする）
pub use tsa_client::TsaClient;
