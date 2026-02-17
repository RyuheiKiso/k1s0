//! JWT Claims 構造体（認証認可設計.md 準拠）。

use serde::Deserialize;
use std::collections::HashMap;

/// RealmAccess は Keycloak の realm_access Claim を表す。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RealmAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Access はリソースアクセスのロール一覧を表す。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Access {
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Claims は JWT トークンの Claims 構造体（認証認可設計.md 準拠）。
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    /// ユーザーの一意識別子（UUID）
    pub sub: String,

    /// トークン発行者
    pub iss: String,

    /// トークンの対象オーディエンス
    #[serde(default)]
    pub aud: Audience,

    /// トークンの有効期限（Unix タイムスタンプ）
    pub exp: u64,

    /// トークンの発行時刻（Unix タイムスタンプ）
    pub iat: u64,

    /// JWT ID
    #[serde(default)]
    pub jti: Option<String>,

    /// トークン種別
    #[serde(default)]
    pub typ: Option<String>,

    /// Authorized party
    #[serde(default)]
    pub azp: Option<String>,

    /// スコープ
    #[serde(default)]
    pub scope: Option<String>,

    /// ユーザー名
    #[serde(default)]
    pub preferred_username: Option<String>,

    /// メールアドレス
    #[serde(default)]
    pub email: Option<String>,

    /// グローバルロール
    #[serde(default)]
    pub realm_access: Option<RealmAccess>,

    /// サービス固有のロール
    #[serde(default)]
    pub resource_access: Option<HashMap<String, Access>>,

    /// アクセス可能な Tier の一覧
    #[serde(default)]
    pub tier_access: Option<Vec<String>>,
}

/// Audience は JWT の aud Claim を表す。
/// 文字列または文字列配列のどちらも受け付ける。
#[derive(Debug, Clone, Default)]
pub struct Audience(pub Vec<String>);

impl<'de> Deserialize<'de> for Audience {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct AudienceVisitor;

        impl<'de> de::Visitor<'de> for AudienceVisitor {
            type Value = Audience;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or array of strings")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Audience(vec![v.to_string()]))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(v) = seq.next_element::<String>()? {
                    values.push(v);
                }
                Ok(Audience(values))
            }
        }

        deserializer.deserialize_any(AudienceVisitor)
    }
}

impl Claims {
    /// 最初のオーディエンスを返す。
    pub fn audience(&self) -> Option<&str> {
        self.aud.0.first().map(|s| s.as_str())
    }

    /// realm_access のロール一覧を返す。
    pub fn realm_roles(&self) -> &[String] {
        self.realm_access
            .as_ref()
            .map(|ra| ra.roles.as_slice())
            .unwrap_or(&[])
    }

    /// 指定リソースのロール一覧を返す。
    pub fn resource_roles(&self, resource: &str) -> &[String] {
        self.resource_access
            .as_ref()
            .and_then(|ra| ra.get(resource))
            .map(|a| a.roles.as_slice())
            .unwrap_or(&[])
    }

    /// tier_access を返す。
    pub fn tier_access_list(&self) -> &[String] {
        self.tier_access.as_deref().unwrap_or(&[])
    }
}

impl std::fmt::Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Claims{{sub={}, iss={}, aud={:?}, username={:?}, email={:?}}}",
            self.sub,
            self.iss,
            self.audience(),
            self.preferred_username,
            self.email,
        )
    }
}
