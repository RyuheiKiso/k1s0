//! Code Action 機能
//!
//! 診断情報に対する自動修正アクションを提供する。

use std::path::PathBuf;

use k1s0_generator::lint::RuleId;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CreateFile, Diagnostic,
    DocumentChangeOperation, DocumentChanges, NumberOrString, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};

/// 診断コードから RuleId への変換
pub fn code_to_rule_id(code: &str) -> Option<RuleId> {
    match code {
        "K001" => Some(RuleId::ManifestNotFound),
        "K002" => Some(RuleId::ManifestMissingKey),
        "K010" => Some(RuleId::RequiredDirMissing),
        "K011" => Some(RuleId::RequiredFileMissing),
        "K020" => Some(RuleId::EnvVarUsage),
        "K021" => Some(RuleId::SecretInConfig),
        "K022" => Some(RuleId::DependencyDirection),
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
            RuleId::EnvVarUsage => {
                // K020: 環境変数使用 — TODO コメント挿入
                if let Some(action) = insert_comment_action(
                    diag,
                    uri,
                    "// TODO: k1s0-config を使用してください",
                    "修正: 環境変数の使用を k1s0-config に置換 (K020)",
                ) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
            RuleId::SecretInConfig => {
                // K021: 機密情報直書き — キーを _file に変換
                if let Some(action) = secret_to_file_ref_action(diag, uri) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
            RuleId::DependencyDirection => {
                // K022: Clean Architecture 違反 — TODO コメント挿入
                if let Some(action) = insert_comment_action(
                    diag,
                    uri,
                    "// TODO: Clean Architecture 違反 - 依存方向を修正してください",
                    "修正: Clean Architecture 依存方向違反 (K022)",
                ) {
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

/// K020/K022: 違反行の上に TODO コメントを挿入する Code Action
fn insert_comment_action(
    diag: &Diagnostic,
    uri: &Url,
    comment: &str,
    title: &str,
) -> Option<CodeAction> {
    let line = diag.range.start.line;
    let new_text = format!("{comment}\n");
    let edit = TextEdit {
        range: Range {
            start: Position {
                line,
                character: 0,
            },
            end: Position {
                line,
                character: 0,
            },
        },
        new_text,
    };

    Some(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: None,
                },
                edits: vec![OneOf::Left(edit)],
            }])),
            ..Default::default()
        }),
        is_preferred: Some(false),
        ..Default::default()
    })
}

/// K021: 機密キーを _file 参照に変換する Code Action
fn secret_to_file_ref_action(diag: &Diagnostic, uri: &Url) -> Option<CodeAction> {
    // メッセージから機密キー名を抽出
    // 形式例: "機密キー 'password' に値が直接設定されています" や
    //         "config YAML に機密情報が直接書かれています: password"
    let key = extract_secret_key_from_message(&diag.message)?;
    let file_key = format!("{key}_file");
    let file_value = format!("/var/run/secrets/k1s0/{key}");

    // 違反行全体を置換
    let line = diag.range.start.line;
    let edit = TextEdit {
        range: Range {
            start: Position {
                line,
                character: 0,
            },
            end: Position {
                line,
                character: u32::MAX,
            },
        },
        new_text: format!("  {file_key}: {file_value}"),
    };

    Some(CodeAction {
        title: format!("修正: '{key}' を '{file_key}' に変換 (K021)"),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: None,
                },
                edits: vec![OneOf::Left(edit)],
            }])),
            ..Default::default()
        }),
        is_preferred: Some(true),
        ..Default::default()
    })
}

/// K021 の診断メッセージから機密キー名を抽出
///
/// 以下の形式に対応:
/// - "機密キー 'xxx' に値が直接設定されています"
/// - "config YAML に機密情報が直接書かれています: xxx: ..."
fn extract_secret_key_from_message(message: &str) -> Option<&str> {
    // 'key' 形式
    if let Some(start) = message.find('\'') {
        let rest = &message[start + 1..];
        if let Some(end) = rest.find('\'') {
            return Some(&rest[..end]);
        }
    }
    // ": key: " or ": key" 形式
    if let Some(idx) = message.rfind(": ") {
        let rest = message[idx + 2..].trim();
        // "key: value" の場合は key 部分だけ取得
        let key = rest.split(':').next()?.trim();
        if !key.is_empty() {
            return Some(key);
        }
    }
    None
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
        assert!(matches!(code_to_rule_id("K020"), Some(RuleId::EnvVarUsage)));
        assert!(matches!(code_to_rule_id("K021"), Some(RuleId::SecretInConfig)));
        assert!(matches!(code_to_rule_id("K022"), Some(RuleId::DependencyDirection)));
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
    fn test_get_code_actions_k020_returns_comment_action() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let uri = Url::from_file_path(root.join("src/main.rs")).unwrap();

        let diag_k020 = Diagnostic {
            range: Range {
                start: Position { line: 5, character: 0 },
                end: Position { line: 5, character: 100 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("K020".to_string())),
            source: Some("k1s0".to_string()),
            message: "環境変数の使用は禁止されています".to_string(),
            ..Default::default()
        };

        let actions = get_code_actions(&uri, &[diag_k020], Some(&root));
        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.title.contains("K020"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_get_code_actions_k021_returns_file_ref_action() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let uri = Url::from_file_path(root.join("config/default.yaml")).unwrap();

        let diag_k021 = Diagnostic {
            range: Range {
                start: Position { line: 3, character: 0 },
                end: Position { line: 3, character: 100 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("K021".to_string())),
            source: Some("k1s0".to_string()),
            message: "機密キー 'password' に値が直接設定されています".to_string(),
            ..Default::default()
        };

        let actions = get_code_actions(&uri, &[diag_k021], Some(&root));
        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.title.contains("K021"));
            assert!(action.title.contains("password_file"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_get_code_actions_k022_returns_comment_action() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let uri = Url::from_file_path(root.join("src/domain/mod.rs")).unwrap();

        let diag_k022 = Diagnostic {
            range: Range {
                start: Position { line: 2, character: 0 },
                end: Position { line: 2, character: 100 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("K022".to_string())),
            source: Some("k1s0".to_string()),
            message: "Clean Architecture の依存方向に違反しています".to_string(),
            ..Default::default()
        };

        let actions = get_code_actions(&uri, &[diag_k022], Some(&root));
        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.title.contains("K022"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_extract_secret_key_from_message() {
        assert_eq!(
            extract_secret_key_from_message("機密キー 'password' に値が直接設定されています"),
            Some("password")
        );
        assert_eq!(
            extract_secret_key_from_message("機密キー 'api_key' に値が直接設定されています"),
            Some("api_key")
        );
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
