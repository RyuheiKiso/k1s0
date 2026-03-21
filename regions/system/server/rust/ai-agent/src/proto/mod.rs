// proto 生成コードをインクルード。
// prost-build (tonic-build) によって生成されたファイルを使用。

pub mod k1s0 {
    pub mod system {
        pub mod ai_agent {
            pub mod v1 {
                // tonic-buildで生成されたai_agentサービスのコードをインクルード
                // protoのpackage名はk1s0.system.aiagent.v1なのでファイル名もそれに合わせる
                include!("k1s0.system.aiagent.v1.rs");
            }
        }
    }
}
