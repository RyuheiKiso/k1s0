//! k1s0 CLI コマンド
//!
//! このモジュールは k1s0 CLI の各サブコマンドを提供します。

pub mod completions;
pub mod docker;
pub mod doctor;
pub mod domain_catalog;
pub mod domain_dependents;
pub mod domain_graph;
pub mod domain_impact;
pub mod domain_list;
pub mod domain_version;
pub mod feature_update_domain;
pub mod init;
pub mod lint;
pub mod new_domain;
pub mod new_feature;
pub mod new_screen;
pub mod registry;
pub mod playground;
pub mod upgrade;
