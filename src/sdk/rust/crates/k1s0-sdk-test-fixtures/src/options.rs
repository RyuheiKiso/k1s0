//! Options 構造体定義。
//!
//! 4 言語対称 API の field 名を Rust イディオム（snake_case）で実装する。

/// kind cluster に install する k1s0 stack の規模
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Stack {
    /// Dapr + tier1 facade + Keycloak + 1 backend のみ install
    Minimum,
    /// user suite 任意 stack 全部入り（owner 経路ではない）
    Full,
}

impl Default for Stack {
    fn default() -> Self {
        // 既定は Minimum（user suite の最小成立形）
        Stack::Minimum
    }
}

/// Setup の動作を制御するパラメータ。
///
/// 4 言語対称化のため field 名は対応関係を保つ:
/// - Go: `KindNodes` / `Stack` / `AddOns` / `Tenant` / `Namespace`
/// - Rust: `kind_nodes` / `stack` / `add_ons` / `tenant` / `namespace`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    /// kind cluster の node 数（control-plane + worker、既定 2）
    pub kind_nodes: u32,

    /// install する k1s0 stack（既定 Minimum）
    pub stack: Stack,

    /// Setup 時に追加で install する任意 component の名前一覧
    pub add_ons: Vec<String>,

    /// デフォルトの tenant ID（既定 "tenant-a"）
    pub tenant: String,

    /// k1s0 install 先 namespace（既定 "k1s0"）
    pub namespace: String,
}

impl Default for Options {
    fn default() -> Self {
        // 試験で典型的な値（Go の DefaultOptions() と対称）
        Self {
            kind_nodes: 2,
            stack: Stack::Minimum,
            add_ons: Vec::new(),
            tenant: "tenant-a".to_string(),
            namespace: "k1s0".to_string(),
        }
    }
}
