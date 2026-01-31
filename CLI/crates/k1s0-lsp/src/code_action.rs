//! Code Action 機能
//!
//! 診断情報に対する自動修正アクションを提供する。

use std::path::PathBuf;

use k1s0_generator::lint::RuleId;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CreateFile, Diagnostic,
    DocumentChangeOperation, DocumentChanges, NumberOrString, ResourceOp, Url, WorkspaceEdit,
};

/// 診断コードから RuleId への変換
pub fn code_to_rule_id(code: &str) -> Option<RuleId> {
    match code {
        "K001" => Some(RuleId::ManifestNotFound),
        "K002" => Some(RuleId::ManifestMissingKey),
        "K010" => Some(RuleId::RequiredDirMissing),
        "K011" => Some(RuleId::RequiredFileMissing),
        _ => None,
    }
}

/// 診断情報から Code Action を生成
pub fn get_code_actions(
    uri: &Url,
    diagnostics: &[Diagnostic],
    workspace_root: Option<&PathBuf>,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    let root = match workspace_root {
        Some(r) => r,
        None => return actions,
    };

    for diag in diagnostics {
        let code_str = match &diag.code {
            Some(NumberOrString::String(s)) => s.as_str(),
            _ => continue,
        };

        let rule_id = match code_to_rule_id(code_str) {
            Some(id) => id,
            None => continue,
        };

        match rule_id {
            RuleId::RequiredDirMissing => {
                // K010: ディレクトリ作成
                if let Some(action) = create_dir_action(diag, uri, root) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
            RuleId::RequiredFileMissing => {
                // K011: ファイル作成
                if let Some(action) = create_file_action(diag, uri, root) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
            RuleId::ManifestNotFound => {
                // K001: manifest.json 作成
                if let Some(action) = create_manifest_action(uri, root) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
            _ => {}
        }
    }

    actions
}

/// K010: 必須ディレクトリ作成の Code Action
fn create_dir_action(diag: &Diagnostic, _uri: &Url, root: &std::path::Path) -> Option<CodeAction> {
    // メッセージからパスを抽出（"必須ディレクトリが不足: xxx" 形式を想定）
    let path_str = extract_path_from_message(&diag.message)?;
    let target_path = root.join(path_str);
    // ディレクトリ作成のために .gitkeep ファイルを作成
    let gitkeep_path = target_path.join(".gitkeep");
    let gitkeep_uri = Url::from_file_path(&gitkeep_path).ok()?;

    Some(CodeAction {
        title: format!("修正: 必須ディレクトリ {} を作成 (K010)", path_str),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Operations(vec![
                DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                    uri: gitkeep_uri,
                    options: None,
                    annotation_id: None,
                })),
            ])),
            ..Default::default()
        }),
        is_preferred: Some(true),
        ..Default::default()
    })
}

/// K011: 必須ファイル作成の Code Action
fn create_file_action(diag: &Diagnostic, _uri: &Url, root: &std::path::Path) -> Option<CodeAction> {
    let path_str = extract_path_from_message(&diag.message)?;
    let target_path = root.join(path_str);
    let target_uri = Url::from_file_path(&target_path).ok()?;

    Some(CodeAction {
        title: format!("修正: 必須ファイル {} を作成 (K011)", path_str),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Operations(vec![
                DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                    uri: target_uri,
                    options: None,
                    annotation_id: None,
                })),
            ])),
            ..Default::default()
        }),
        is_preferred: Some(true),
        ..Default::default()
    })
}

/// K001: manifest.json 作成の Code Action
fn create_manifest_action(_uri: &Url, root: &std::path::Path) -> Option<CodeAction> {
    let manifest_path = root.join(".k1s0").join("manifest.json");
    let manifest_uri = Url::from_file_path(&manifest_path).ok()?;

    Some(CodeAction {
        title: "修正: manifest.json を作成 (K001)".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Operations(vec![
                DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                    uri: manifest_uri,
                    options: None,
                    annotation_id: None,
                })),
            ])),
            ..Default::default()
        }),
        is_preferred: Some(true),
        ..Default::default()
    })
}

/// 診断メッセージからパスを抽出
///
/// "必須ディレクトリが不足: src/domain/" や "必須ファイルが不足: config/default.yaml"
/// などのメッセージからパス部分を取得する。
fn extract_path_from_message(message: &str) -> Option<&str> {
    // ": " の後の部分を取得
    message.split(": ").nth(1).map(|s| s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::{DiagnosticSeverity, Position, Range};

    #[test]
    fn test_code_to_rule_id_all_patterns() {
        assert!(matches!(code_to_rule_id("K001"), Some(RuleId::ManifestNotFound)));
        assert!(matches!(code_to_rule_id("K002"), Some(RuleId::ManifestMissingKey)));
        assert!(matches!(code_to_rule_id("K010"), Some(RuleId::RequiredDirMissing)));
        assert!(matches!(code_to_rule_id("K011"), Some(RuleId::RequiredFileMissing)));
        assert!(code_to_rule_id("K020").is_none());
        assert!(code_to_rule_id("K040").is_none());
        assert!(code_to_rule_id("unknown").is_none());
    }

    #[test]
    fn test_extract_path_from_message() {
        assert_eq!(
            extract_path_from_message("必須ディレクトリが不足: src/domain/"),
            Some("src/domain/")
        );
        assert_eq!(
            extract_path_from_message("必須ファイルが不足: config/default.yaml"),
            Some("config/default.yaml")
        );
        assert_eq!(extract_path_from_message("no colon here"), None);
    }

    #[test]
    fn test_get_code_actions_k010_returns_action() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let uri = Url::from_file_path(root.join(".k1s0/manifest.json")).unwrap();

        let diag_k010 = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 100 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("K010".to_string())),
            source: Some("k1s0".to_string()),
            message: "必須ディレクトリが不足: src/domain/".to_string(),
            ..Default::default()
        };

        let actions = get_code_actions(&uri, &[diag_k010], Some(&root));
        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.title.contains("K010"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_get_code_actions_k020_returns_no_action() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let uri = Url::from_file_path(root.join(".k1s0/manifest.json")).unwrap();

        let diag_k020 = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 100 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("K020".to_string())),
            source: Some("k1s0".to_string()),
            message: "環境変数の使用は禁止されています".to_string(),
            ..Default::default()
        };

        let actions = get_code_actions(&uri, &[diag_k020], Some(&root));
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_code_actions_no_workspace() {
        let diag = Diagnostic {
            code: Some(NumberOrString::String("K010".to_string())),
            message: "必須ディレクトリが不足: src/domain/".to_string(),
            ..Default::default()
        };
        let uri = Url::parse("file:///test").unwrap();

        let actions = get_code_actions(&uri, &[diag], None);
        assert!(actions.is_empty());
    }
}
