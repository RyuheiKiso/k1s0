use serde::{Deserialize, Serialize};

/// 進捗イベント。
///
/// ビルドやデプロイなど長時間操作の進捗をストリーミングするための型。
/// Tauri Channel でフロントエンドに送信される。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ProgressEvent {
    /// ステップ開始
    StepStarted {
        step: usize,
        total: usize,
        message: String,
    },
    /// ステップ完了
    StepCompleted {
        step: usize,
        total: usize,
        message: String,
    },
    /// ログメッセージ
    Log { message: String },
    /// 警告
    Warning { message: String },
    /// エラー
    Error { message: String },
    /// 全体完了
    Finished { success: bool, message: String },
}

/// プログレスコールバックの型。
pub type ProgressCallback = Box<dyn Fn(ProgressEvent) + Send + 'static>;

/// プログレスイベントを stdout に出力するデフォルトコールバック。
pub fn print_progress(event: &ProgressEvent) {
    match event {
        ProgressEvent::StepStarted {
            step,
            total,
            message,
        } => {
            println!("[{step}/{total}] {message} ...");
        }
        ProgressEvent::StepCompleted {
            step,
            total,
            message,
        } => {
            println!("[{step}/{total}] \u{2713} {message}");
        }
        ProgressEvent::Log { message } => {
            println!("  {message}");
        }
        ProgressEvent::Warning { message } => {
            println!("  警告: {message}");
        }
        ProgressEvent::Error { message } => {
            eprintln!("  エラー: {message}");
        }
        ProgressEvent::Finished { success, message } => {
            if *success {
                println!("\u{2713} {message}");
            } else {
                eprintln!("\u{2717} {message}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_event_step_started_serde_roundtrip() {
        let event = ProgressEvent::StepStarted {
            step: 1,
            total: 4,
            message: "ビルド中".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
        assert!(json.contains("\"kind\":\"StepStarted\""));
    }

    #[test]
    fn test_progress_event_step_completed_serde_roundtrip() {
        let event = ProgressEvent::StepCompleted {
            step: 2,
            total: 3,
            message: "完了".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_progress_event_log_serde_roundtrip() {
        let event = ProgressEvent::Log {
            message: "処理中...".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_progress_event_warning_serde_roundtrip() {
        let event = ProgressEvent::Warning {
            message: "ディレクトリが見つかりません".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_progress_event_error_serde_roundtrip() {
        let event = ProgressEvent::Error {
            message: "コマンドが失敗しました".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_progress_event_finished_success_serde_roundtrip() {
        let event = ProgressEvent::Finished {
            success: true,
            message: "ビルドが完了しました".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_progress_event_finished_failure_serde_roundtrip() {
        let event = ProgressEvent::Finished {
            success: false,
            message: "ビルドに失敗しました".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_progress_event_tagged_json_format() {
        let event = ProgressEvent::StepStarted {
            step: 1,
            total: 2,
            message: "テスト".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        // Tagged enum format: {"kind":"StepStarted","step":1,"total":2,"message":"テスト"}
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["kind"], "StepStarted");
        assert_eq!(value["step"], 1);
        assert_eq!(value["total"], 2);
        assert_eq!(value["message"], "テスト");
    }

    #[test]
    fn test_print_progress_does_not_panic() {
        // print_progress が各バリアントでパニックしないことを確認
        let events = vec![
            ProgressEvent::StepStarted {
                step: 1,
                total: 3,
                message: "開始".to_string(),
            },
            ProgressEvent::StepCompleted {
                step: 1,
                total: 3,
                message: "完了".to_string(),
            },
            ProgressEvent::Log {
                message: "ログ".to_string(),
            },
            ProgressEvent::Warning {
                message: "警告".to_string(),
            },
            ProgressEvent::Error {
                message: "エラー".to_string(),
            },
            ProgressEvent::Finished {
                success: true,
                message: "成功".to_string(),
            },
            ProgressEvent::Finished {
                success: false,
                message: "失敗".to_string(),
            },
        ];
        for event in &events {
            print_progress(event);
        }
    }

    #[test]
    fn test_progress_callback_type_compiles() {
        // ProgressCallback 型がクロージャから構築できることを確認
        let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let collected_clone = collected.clone();
        let callback: ProgressCallback = Box::new(move |event| {
            collected_clone.lock().unwrap().push(event);
        });
        callback(ProgressEvent::Log {
            message: "test".to_string(),
        });
        assert_eq!(collected.lock().unwrap().len(), 1);
    }
}
