// k1s0 tier1 Bidi handshake state machine
// TLA+ spec（src/formal/tla/tier1_bidi_handshake.tla）の NoDoubleHandshake 不変条件を Rust で double-bind する

// 外部クレートインポート
use serde::{Deserialize, Serialize};

// Bidi handshake の状態を表す列挙型（TLA+ spec の state 変数と 1:1 対応する）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandshakeState {
    // 初期状態: ハンドシェイクが開始されていない
    Init,
    // Hello 送信済み: クライアントから Hello を送信した
    SentHello,
    // Hello 受信済み: サーバーから Hello を受信した
    ReceivedHello,
    // Ack 送信済み: クライアントから Ack を送信した
    SentAck,
    // 完了状態: ハンドシェイクが正常完了した
    Done,
    // 終了状態: コネクションが閉じられた
    Closed,
}

// Bidi handshake セッションを管理する構造体
// TLA+ spec の send_count / recv_count / state 変数に対応する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiSession {
    // セッション ID: UUID v4 で一意識別する
    pub session_id: String,
    // 現在の handshake 状態
    pub state: HandshakeState,
    // 送信回数カウンタ（TLA+ の send_count に対応、最大 1 を超えてはならない）
    pub send_count: u32,
    // 受信回数カウンタ（TLA+ の recv_count に対応、最大 1 を超えてはならない）
    pub recv_count: u32,
    // 使用する transport adapter（grpc_go / connect_go 等）
    pub adapter: String,
    // 対象 conformance class（v1_bidi_streaming 等）
    pub conformance_class: String,
}

// handshake 操作のエラー型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    // NoDoubleHandshake 違反: send_count が 1 を超えた
    DoubleHandshakeViolation { send_count: u32 },
    // 不正な状態遷移: 現在の状態では許可されない操作
    InvalidStateTransition { from: HandshakeState, operation: &'static str },
    // セッション終了後の操作: Closed 状態での操作試行
    SessionAlreadyClosed,
}

impl std::fmt::Display for HandshakeError {
    // エラーメッセージを人間が読める形式で表示する
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // エラー種別に応じてメッセージをフォーマットする
        match self {
            // NoDoubleHandshake 違反メッセージ
            Self::DoubleHandshakeViolation { send_count } => {
                write!(f, "NoDoubleHandshake violation: send_count={send_count} > 1")
            }
            // 不正遷移メッセージ
            Self::InvalidStateTransition { from, operation } => {
                write!(f, "Invalid state transition from {from:?} via {operation}")
            }
            // セッション終了後操作メッセージ
            Self::SessionAlreadyClosed => write!(f, "Session is already in Closed state"),
        }
    }
}

impl std::error::Error for HandshakeError {}

impl BidiSession {
    // 新しい BidiSession を生成する（状態は Init から始まる）
    pub fn new(session_id: String, adapter: String, conformance_class: String) -> Self {
        // セッションを初期状態で構築する
        Self {
            session_id,
            state: HandshakeState::Init,
            send_count: 0,
            recv_count: 0,
            adapter,
            conformance_class,
        }
    }

    // Hello メッセージを送信する操作（TLA+ の SendHello アクションに対応する）
    pub fn send_hello(&mut self) -> Result<(), HandshakeError> {
        // Closed 状態では操作を拒否する
        if self.state == HandshakeState::Closed {
            return Err(HandshakeError::SessionAlreadyClosed);
        }
        // Init 状態のみ SendHello を許可する（TLA+ spec の前提条件と一致する）
        if self.state != HandshakeState::Init {
            return Err(HandshakeError::InvalidStateTransition {
                from: self.state.clone(),
                operation: "send_hello",
            });
        }
        // send_count をインクリメントして NoDoubleHandshake 違反を検査する
        self.send_count += 1;
        // NoDoubleHandshake 不変条件: send_count は 1 を超えてはならない
        if self.send_count > 1 {
            return Err(HandshakeError::DoubleHandshakeViolation {
                send_count: self.send_count,
            });
        }
        // 状態を SentHello に遷移する
        self.state = HandshakeState::SentHello;
        Ok(())
    }

    // Hello メッセージを受信する操作（TLA+ の ReceiveHello アクションに対応する）
    pub fn receive_hello(&mut self) -> Result<(), HandshakeError> {
        // Closed 状態では操作を拒否する
        if self.state == HandshakeState::Closed {
            return Err(HandshakeError::SessionAlreadyClosed);
        }
        // SentHello 状態のみ ReceiveHello を許可する
        if self.state != HandshakeState::SentHello {
            return Err(HandshakeError::InvalidStateTransition {
                from: self.state.clone(),
                operation: "receive_hello",
            });
        }
        // recv_count をインクリメントする
        self.recv_count += 1;
        // 状態を ReceivedHello に遷移する
        self.state = HandshakeState::ReceivedHello;
        Ok(())
    }

    // Ack メッセージを送信する操作（TLA+ の SendAck アクションに対応する）
    pub fn send_ack(&mut self) -> Result<(), HandshakeError> {
        // Closed 状態では操作を拒否する
        if self.state == HandshakeState::Closed {
            return Err(HandshakeError::SessionAlreadyClosed);
        }
        // ReceivedHello 状態のみ SendAck を許可する
        if self.state != HandshakeState::ReceivedHello {
            return Err(HandshakeError::InvalidStateTransition {
                from: self.state.clone(),
                operation: "send_ack",
            });
        }
        // 状態を SentAck に遷移する
        self.state = HandshakeState::SentAck;
        Ok(())
    }

    // ハンドシェイクを完了する操作（TLA+ の Close アクションに対応する）
    pub fn complete(&mut self) -> Result<(), HandshakeError> {
        // Closed 状態では操作を拒否する
        if self.state == HandshakeState::Closed {
            return Err(HandshakeError::SessionAlreadyClosed);
        }
        // SentAck 状態のみ完了を許可する
        if self.state != HandshakeState::SentAck {
            return Err(HandshakeError::InvalidStateTransition {
                from: self.state.clone(),
                operation: "complete",
            });
        }
        // 状態を Done に遷移する
        self.state = HandshakeState::Done;
        Ok(())
    }

    // セッションを閉じる操作（任意の状態から Closed に遷移できる）
    pub fn close(&mut self) {
        // 状態を Closed に強制遷移する
        self.state = HandshakeState::Closed;
    }

    // NoDoubleHandshake 不変条件を検査する（TLA+ の TypeInvariant に対応する）
    pub fn check_invariant(&self) -> bool {
        // send_count は 1 以下でなければならない
        self.send_count <= 1
            // recv_count は 1 以下でなければならない
            && self.recv_count <= 1
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // 正常ハンドシェイクフローが完走することを検証する
    fn test_normal_handshake_flow() {
        // セッションを Init 状態で生成する
        let mut session = BidiSession::new(
            "test-session-001".to_string(),
            "grpc_go".to_string(),
            "v1_bidi_streaming".to_string(),
        );
        // send_hello → receive_hello → send_ack → complete の正常フローを実行する
        assert!(session.send_hello().is_ok());
        assert!(session.receive_hello().is_ok());
        assert!(session.send_ack().is_ok());
        assert!(session.complete().is_ok());
        // 完了後の状態を検証する
        assert_eq!(session.state, HandshakeState::Done);
        // 不変条件が保持されていることを確認する
        assert!(session.check_invariant());
    }

    #[test]
    // NoDoubleHandshake 違反（二重 Hello 送信）を検出することを検証する
    fn test_double_handshake_violation() {
        // セッションを Init 状態で生成する
        let mut session = BidiSession::new(
            "test-session-002".to_string(),
            "connect_go".to_string(),
            "v1_unary_rpc".to_string(),
        );
        // 最初の Hello 送信は成功する
        assert!(session.send_hello().is_ok());
        // 二重送信を強制的に試みるため、状態を Init に戻して再送を試みる
        session.state = HandshakeState::Init;
        // 二番目の send_hello 呼び出しは send_count=2 で NoDoubleHandshake 違反となる
        let result = session.send_hello();
        // DoubleHandshakeViolation エラーが返されることを確認する
        assert!(matches!(
            result,
            Err(HandshakeError::DoubleHandshakeViolation { send_count: 2 })
        ));
    }

    #[test]
    // 不変条件チェックが正常に機能することを検証する
    fn test_invariant_holds_after_complete() {
        // セッションを生成して完了まで実行する
        let mut session = BidiSession::new(
            "test-session-003".to_string(),
            "grpc_node".to_string(),
            "v1_server_streaming".to_string(),
        );
        // 全ステップを実行する
        session.send_hello().unwrap();
        session.receive_hello().unwrap();
        session.send_ack().unwrap();
        session.complete().unwrap();
        // 完了後も不変条件が保持されていることを確認する
        assert!(session.check_invariant());
        // send_count と recv_count が正確に 1 であることを確認する
        assert_eq!(session.send_count, 1);
        assert_eq!(session.recv_count, 1);
    }
}
