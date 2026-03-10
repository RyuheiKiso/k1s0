use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Platform はアプリがサポートする OS プラットフォームを表す列挙型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
    Macos,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Windows => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
            Platform::Macos => write!(f, "macos"),
        }
    }
}

impl std::str::FromStr for Platform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "windows" => Ok(Platform::Windows),
            "linux" => Ok(Platform::Linux),
            "macos" => Ok(Platform::Macos),
            _ => Err(anyhow::anyhow!("unknown platform: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Windows.to_string(), "windows");
        assert_eq!(Platform::Linux.to_string(), "linux");
        assert_eq!(Platform::Macos.to_string(), "macos");
    }

    #[test]
    fn test_platform_from_str() {
        assert_eq!("windows".parse::<Platform>().unwrap(), Platform::Windows);
        assert_eq!("linux".parse::<Platform>().unwrap(), Platform::Linux);
        assert_eq!("macos".parse::<Platform>().unwrap(), Platform::Macos);
        assert_eq!("WINDOWS".parse::<Platform>().unwrap(), Platform::Windows);
    }

    #[test]
    fn test_platform_from_str_invalid() {
        assert!("android".parse::<Platform>().is_err());
    }

    #[test]
    fn test_platform_serde_roundtrip() {
        let p = Platform::Macos;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "\"macos\"");
        let parsed: Platform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Platform::Macos);
    }
}
