use anyhow::{bail, Result};
use tracing::warn;

pub fn allow_insecure_no_auth(environment: &str) -> bool {
    matches!(environment, "dev" | "test")
        && std::env::var("ALLOW_INSECURE_NO_AUTH")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
}

pub fn require_auth_state<T>(
    service_name: &str,
    environment: &str,
    auth_state: Option<T>,
) -> Result<Option<T>> {
    if auth_state.is_some() {
        return Ok(auth_state);
    }

    if allow_insecure_no_auth(environment) {
        warn!(
            environment = %environment,
            service = %service_name,
            "service is running without authentication because ALLOW_INSECURE_NO_AUTH=true"
        );
        return Ok(None);
    }

    bail!(
        "auth configuration is required for {} (environment: {}). \
Set auth.* in the config, or use ALLOW_INSECURE_NO_AUTH=true only for dev/test.",
        service_name,
        environment
    )
}

#[cfg(test)]
mod tests {
    use super::{allow_insecure_no_auth, require_auth_state};

    #[test]
    fn allows_insecure_auth_override_only_for_dev_and_test() {
        std::env::set_var("ALLOW_INSECURE_NO_AUTH", "true");

        assert!(allow_insecure_no_auth("dev"));
        assert!(allow_insecure_no_auth("test"));
        assert!(!allow_insecure_no_auth("staging"));

        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");
    }

    #[test]
    fn rejects_missing_auth_without_override() {
        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");

        let err = require_auth_state::<()>("example-service", "dev", None).unwrap_err();

        assert!(err
            .to_string()
            .contains("auth configuration is required for example-service"));
    }

    #[test]
    fn accepts_missing_auth_when_override_is_enabled() {
        std::env::set_var("ALLOW_INSECURE_NO_AUTH", "true");

        let auth_state = require_auth_state::<()>("example-service", "dev", None).unwrap();

        assert!(auth_state.is_none());

        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");
    }
}
