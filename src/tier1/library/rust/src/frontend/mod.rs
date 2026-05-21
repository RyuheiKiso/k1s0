// mod.rs — k1s0 tier1 Library frontend: frontend 向けモジュール群
// auth / config / rpc の 3 サブモジュールを提供する。
// frontend モジュールは backend モジュールの thin wrapper として実装する。
// backend 専用の機能（messaging / db / vector 等）は含まない。
// 公開 API は backend モジュールと整合した型を使用する。

// auth モジュール: Authentication/Authorization L3（frontend 向け thin wrapper）
// FrontendAuthClient trait / FrontendTokenBundle を提供する
// core::auth の AuthClass / AuthContext / ScopeRequirement も re-export する
pub mod auth;

// config モジュール: Configuration/Feature Flag L2*（frontend 向け thin wrapper）
// FrontendConfigClient trait / FrontendConfigSnapshot を提供する
// backend::config の ConfigValue / FeatureFlag / EvaluationContext も re-export する
pub mod config;

// rpc モジュール: RPC/Gateway L3（frontend 向け thin wrapper）
// FrontendRpcClient trait / FrontendRpcOptions / FrontendRpcResponse を提供する
// backend::rpc の RpcRequest / RpcResponse / RpcStatus も re-export する
pub mod rpc;

// transport_negotiation モジュール: Companion 役割 B Transport Negotiation Runtime
// 8 adapter の chosen_transport capability negotiation を担うクライアント実装
// BidiChannel / TransportNegotiationClient / ClientCapabilities 等を提供する
pub mod transport_negotiation;

// 頻繁に使用する型を frontend 名前空間から直接参照できるように re-export する
// auth 関連の主要型を re-export する
pub use auth::{AuthClass, AuthContext, FrontendAuthClient, FrontendTokenBundle, ScopeRequirement};
// config 関連の主要型を re-export する
pub use config::{
    // クライアント trait
    FrontendConfigClient,
    // スナップショット型
    FrontendConfigSnapshot,
    // 共有型
    ConfigValue,
    EvaluationContext,
    FeatureFlag,
};
// transport_negotiation 関連の主要型を re-export する
pub use transport_negotiation::{
    // adapter 種別 enum
    TransportKind,
    // クライアント capability 宣言
    ClientCapabilities,
    // 双方向メッセージ型
    BidiMessage,
    // 双方向チャンネル抽象 trait
    BidiChannel,
    // negotiation facade trait
    TransportNegotiationClient,
    // negotiation 結果型
    NegotiationResult,
    // SSE adapter 設定型
    SsePairedChannelOptions,
    // LongPoll adapter 設定型
    LongPollChannelOptions,
};

// rpc 関連の主要型を re-export する
pub use rpc::{
    // クライアント trait
    FrontendRpcClient,
    // 設定型
    CredentialsMode,
    FrontendRpcOptions,
    // レスポンス型
    FrontendRpcResponse,
    // 共有型
    RpcRequest,
    RpcResponse,
    RpcStatus,
};
