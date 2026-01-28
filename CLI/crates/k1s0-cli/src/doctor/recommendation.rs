//! 推奨アクション生成
//!
//! チェック結果から推奨アクションを生成する。

use crate::doctor::checker::{CheckStatus, ToolCheck};

/// 推奨アクションの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendAction {
    /// インストール
    Install,
    /// アップグレード
    Upgrade,
}

impl RecommendAction {
    /// アクション名を取得
    pub fn label(&self) -> &'static str {
        match self {
            RecommendAction::Install => "インストール",
            RecommendAction::Upgrade => "アップグレード",
        }
    }
}

/// 推奨アクション
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// ツール名
    pub tool_name: &'static str,
    /// アクションの種類
    pub action: RecommendAction,
    /// メッセージ
    pub message: String,
    /// URL
    pub url: &'static str,
    /// インストールコマンド
    pub install_command: Option<&'static str>,
    /// 必須かどうか
    pub required: bool,
}

/// チェック結果から推奨アクションを生成
pub fn generate_recommendations(checks: &[ToolCheck]) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();

    for check in checks {
        match &check.status {
            CheckStatus::NotFound => {
                let requirement = check.requirement;
                recommendations.push(Recommendation {
                    tool_name: requirement.name,
                    action: RecommendAction::Install,
                    message: format!(
                        "{} をインストールしてください（{}）",
                        requirement.name, requirement.description
                    ),
                    url: requirement.install_url,
                    install_command: requirement.install_commands.current_platform(),
                    required: requirement.required,
                });
            }
            CheckStatus::VersionMismatch { actual, required } => {
                let requirement = check.requirement;
                recommendations.push(Recommendation {
                    tool_name: requirement.name,
                    action: RecommendAction::Upgrade,
                    message: format!(
                        "{} をアップグレードしてください（現在: {}, 必要: {} 以上）",
                        requirement.name, actual, required
                    ),
                    url: requirement.install_url,
                    install_command: requirement.install_commands.current_platform(),
                    required: requirement.required,
                });
            }
            CheckStatus::Error(msg) if check.requirement.required => {
                let requirement = check.requirement;
                recommendations.push(Recommendation {
                    tool_name: requirement.name,
                    action: RecommendAction::Install,
                    message: format!(
                        "{} のチェックに失敗しました（{}）。再インストールを検討してください",
                        requirement.name, msg
                    ),
                    url: requirement.install_url,
                    install_command: requirement.install_commands.current_platform(),
                    required: requirement.required,
                });
            }
            _ => {}
        }
    }

    // 必須を先に、その後オプションをソート
    recommendations.sort_by(|a, b| {
        match (a.required, b.required) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.tool_name.cmp(b.tool_name),
        }
    });

    recommendations
}

/// 必須ツールの問題があるかどうか
pub fn has_required_problems(checks: &[ToolCheck]) -> bool {
    checks.iter().any(|c| c.requirement.required && c.has_problem())
}

/// オプションツールの問題があるかどうか
pub fn has_optional_problems(checks: &[ToolCheck]) -> bool {
    checks.iter().any(|c| !c.requirement.required && !c.is_ok())
}

/// サマリー情報
#[derive(Debug, Clone)]
pub struct CheckSummary {
    /// 必須ツールで成功した数
    pub required_ok: usize,
    /// 必須ツールで失敗した数
    pub required_failed: usize,
    /// オプションツールで成功した数
    pub optional_ok: usize,
    /// オプションツールで見つからなかった数
    pub optional_missing: usize,
}

/// サマリーを生成
pub fn generate_summary(checks: &[ToolCheck]) -> CheckSummary {
    let mut required_ok = 0;
    let mut required_failed = 0;
    let mut optional_ok = 0;
    let mut optional_missing = 0;

    for check in checks {
        if check.requirement.required {
            if check.is_ok() {
                required_ok += 1;
            } else {
                required_failed += 1;
            }
        } else if check.is_ok() {
            optional_ok += 1;
        } else {
            optional_missing += 1;
        }
    }

    CheckSummary {
        required_ok,
        required_failed,
        optional_ok,
        optional_missing,
    }
}
