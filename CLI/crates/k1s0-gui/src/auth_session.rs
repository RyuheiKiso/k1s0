use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(not(test))]
const AUTH_SESSION_SERVICE: &str = "com.k1s0.gui";
#[cfg(not(test))]
const AUTH_SESSION_ACCOUNT: &str = "operator-session";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthSessionSummary {
    pub issuer: String,
    pub authenticated_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub token_type: String,
    pub scope: Option<String>,
    pub can_refresh: bool,
}

#[derive(Debug, Clone)]
pub struct TokenPayload {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAuthSession {
    issuer: String,
    client_id: String,
    scope: Option<String>,
    token_endpoint: String,
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: String,
    authenticated_at_epoch_secs: u64,
    expires_at_epoch_secs: u64,
}

#[derive(Debug, Deserialize)]
struct RawRefreshTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
}

impl StoredAuthSession {
    fn from_token_payload(
        issuer: String,
        client_id: String,
        scope: String,
        token_endpoint: String,
        token_payload: TokenPayload,
        now_epoch_secs: u64,
    ) -> Self {
        Self {
            issuer,
            client_id,
            scope: Some(token_payload.scope.unwrap_or(scope)),
            token_endpoint,
            access_token: token_payload.access_token,
            refresh_token: token_payload.refresh_token,
            id_token: token_payload.id_token,
            token_type: token_payload.token_type,
            authenticated_at_epoch_secs: now_epoch_secs,
            expires_at_epoch_secs: now_epoch_secs.saturating_add(token_payload.expires_in),
        }
    }

    fn is_expired(&self, now_epoch_secs: u64) -> bool {
        self.expires_at_epoch_secs <= now_epoch_secs
    }

    fn to_summary(&self) -> AuthSessionSummary {
        AuthSessionSummary {
            issuer: self.issuer.clone(),
            authenticated_at_epoch_secs: self.authenticated_at_epoch_secs,
            expires_at_epoch_secs: self.expires_at_epoch_secs,
            token_type: self.token_type.clone(),
            scope: self.scope.clone(),
            can_refresh: self.refresh_token.is_some(),
        }
    }
}

pub fn store_auth_session(
    issuer: String,
    client_id: String,
    scope: String,
    token_endpoint: String,
    token_payload: TokenPayload,
) -> Result<AuthSessionSummary, String> {
    let stored = StoredAuthSession::from_token_payload(
        issuer,
        client_id,
        scope,
        token_endpoint,
        token_payload,
        now_epoch_secs(),
    );
    write_session(&stored)?;
    Ok(stored.to_summary())
}

pub fn load_auth_session() -> Result<Option<AuthSessionSummary>, String> {
    let Some(stored) = read_session()? else {
        return Ok(None);
    };

    let Some(active_session) = refresh_if_needed(stored)? else {
        return Ok(None);
    };

    Ok(Some(active_session.to_summary()))
}

pub fn require_auth_session() -> Result<AuthSessionSummary, String> {
    load_auth_session()?.ok_or_else(|| {
        "An active operator session is required. Sign in on the Authentication page.".to_string()
    })
}

pub fn clear_auth_session() -> Result<(), String> {
    delete_session()
}

fn refresh_if_needed(stored: StoredAuthSession) -> Result<Option<StoredAuthSession>, String> {
    if !stored.is_expired(now_epoch_secs()) {
        return Ok(Some(stored));
    }

    let Some(refresh_token) = stored.refresh_token.clone() else {
        delete_session()?;
        return Ok(None);
    };

    let client = http_client()?;
    let response = client
        .post(&stored.token_endpoint)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", stored.client_id.as_str()),
            ("refresh_token", refresh_token.as_str()),
        ])
        .send()
        .map_err(|error| format!("failed to refresh authentication session: {error}"))?;

    if !response.status().is_success() {
        delete_session()?;
        return Ok(None);
    }

    let token_payload = response
        .json::<RawRefreshTokenResponse>()
        .map_err(|error| format!("failed to decode refreshed authentication session: {error}"))?;

    let refreshed = StoredAuthSession {
        issuer: stored.issuer,
        client_id: stored.client_id,
        scope: token_payload.scope.or(stored.scope),
        token_endpoint: stored.token_endpoint,
        access_token: token_payload.access_token,
        refresh_token: token_payload.refresh_token.or(Some(refresh_token)),
        id_token: token_payload.id_token.or(stored.id_token),
        token_type: token_payload.token_type,
        authenticated_at_epoch_secs: now_epoch_secs(),
        expires_at_epoch_secs: now_epoch_secs().saturating_add(token_payload.expires_in),
    };

    write_session(&refreshed)?;
    Ok(Some(refreshed))
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .build()
        .map_err(|error| format!("failed to create HTTP client: {error}"))
}

fn now_epoch_secs() -> u64 {
    #[cfg(test)]
    if let Some(now) = test_now_override()
        .lock()
        .expect("test clock mutex poisoned")
        .as_ref()
        .copied()
    {
        return now;
    }

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(not(test))]
fn read_session() -> Result<Option<StoredAuthSession>, String> {
    let entry = keyring::Entry::new(AUTH_SESSION_SERVICE, AUTH_SESSION_ACCOUNT)
        .map_err(|error| format!("failed to access secure session storage: {error}"))?;

    match entry.get_password() {
        Ok(json) => serde_json::from_str(&json)
            .map(Some)
            .map_err(|error| format!("failed to decode secure session storage: {error}")),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("failed to read secure session storage: {error}")),
    }
}

#[cfg(not(test))]
fn write_session(session: &StoredAuthSession) -> Result<(), String> {
    let entry = keyring::Entry::new(AUTH_SESSION_SERVICE, AUTH_SESSION_ACCOUNT)
        .map_err(|error| format!("failed to access secure session storage: {error}"))?;
    let json = serde_json::to_string(session)
        .map_err(|error| format!("failed to serialize auth session: {error}"))?;
    entry
        .set_password(&json)
        .map_err(|error| format!("failed to write secure session storage: {error}"))
}

#[cfg(not(test))]
fn delete_session() -> Result<(), String> {
    let entry = keyring::Entry::new(AUTH_SESSION_SERVICE, AUTH_SESSION_ACCOUNT)
        .map_err(|error| format!("failed to access secure session storage: {error}"))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("failed to clear secure session storage: {error}")),
    }
}

#[cfg(test)]
fn session_blob() -> &'static Mutex<Option<String>> {
    static SESSION_BLOB: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SESSION_BLOB.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn test_now_override() -> &'static Mutex<Option<u64>> {
    static TEST_NOW: OnceLock<Mutex<Option<u64>>> = OnceLock::new();
    TEST_NOW.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn read_session() -> Result<Option<StoredAuthSession>, String> {
    let guard = session_blob()
        .lock()
        .map_err(|_| "failed to lock test session storage".to_string())?;
    guard
        .as_ref()
        .map(|json| {
            serde_json::from_str(json)
                .map_err(|error| format!("failed to decode test session storage: {error}"))
        })
        .transpose()
}

#[cfg(test)]
fn write_session(session: &StoredAuthSession) -> Result<(), String> {
    let json = serde_json::to_string(session)
        .map_err(|error| format!("failed to serialize auth session: {error}"))?;
    let mut guard = session_blob()
        .lock()
        .map_err(|_| "failed to lock test session storage".to_string())?;
    *guard = Some(json);
    Ok(())
}

#[cfg(test)]
fn delete_session() -> Result<(), String> {
    let mut guard = session_blob()
        .lock()
        .map_err(|_| "failed to lock test session storage".to_string())?;
    *guard = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_execution_lock() -> &'static Mutex<()> {
        static TEST_EXECUTION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        TEST_EXECUTION_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn lock_test_execution() -> std::sync::MutexGuard<'static, ()> {
        test_execution_lock()
            .lock()
            .expect("test execution mutex poisoned")
    }

    fn reset_test_state() {
        *session_blob()
            .lock()
            .expect("test session storage mutex poisoned") = None;
        *test_now_override()
            .lock()
            .expect("test clock mutex poisoned") = None;
    }

    #[test]
    fn store_and_load_session_from_secure_storage() {
        let _guard = lock_test_execution();
        reset_test_state();

        let session = store_auth_session(
            "https://issuer.example.com".to_string(),
            "client-id".to_string(),
            "openid profile".to_string(),
            "https://issuer.example.com/token".to_string(),
            TokenPayload {
                access_token: "access".to_string(),
                refresh_token: Some("refresh".to_string()),
                id_token: Some("id".to_string()),
                token_type: "Bearer".to_string(),
                expires_in: 600,
                scope: None,
            },
        )
        .expect("session should be stored");

        let loaded = load_auth_session().expect("session should load");

        assert_eq!(loaded, Some(session));
    }

    #[test]
    fn expired_session_without_refresh_is_cleared() {
        let _guard = lock_test_execution();
        reset_test_state();

        let summary = store_auth_session(
            "https://issuer.example.com".to_string(),
            "client-id".to_string(),
            "openid".to_string(),
            "https://issuer.example.com/token".to_string(),
            TokenPayload {
                access_token: "access".to_string(),
                refresh_token: None,
                id_token: None,
                token_type: "Bearer".to_string(),
                expires_in: 10,
                scope: None,
            },
        )
        .expect("session should be stored");

        *test_now_override()
            .lock()
            .expect("test clock mutex poisoned") = Some(summary.expires_at_epoch_secs + 1);

        assert!(load_auth_session()
            .expect("session load should succeed")
            .is_none());
        assert!(read_session()
            .expect("test storage should be readable")
            .is_none());
    }

    #[test]
    fn require_auth_session_returns_error_when_missing() {
        let _guard = lock_test_execution();
        reset_test_state();

        let error = require_auth_session().expect_err("session should be required");
        assert!(error.contains("Sign in"));
    }
}
