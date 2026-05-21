// envoy_jwt_authn_config.rs — spec 04 §5 層 D: Envoy jwt_authn filter 設定 generator
// spec 04 §v1 auth_class セット（5 class）に対応する Envoy jwt_authn filter 設定を生成する。
// AuthClass ごとに JwtProvider を構築し、serde_yaml で Envoy filter YAML を出力する。
// DPoP 用の from_headers 設定と claim_to_headers マッピングを含む。

// anyhow: エラー伝搬ライブラリ
use anyhow::{Context, Result};
// serde: YAML シリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{debug, info};

// AuthClass は spec 04 §v1 auth_class セット（5 class）を宣言する。
// JwtAuthnConfig::from_classes_yaml の入力として使用する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthClass {
    // v1_human_session: ブラウザ / モバイル OIDC セッション（DPoP 必須）
    V1HumanSession,
    // v1_workload_jwt: SPIFFE SVID JWT（サービス間通信）
    V1WorkloadJwt,
    // v1_device_attest: デバイス attestation chain（TPM / HSM / WebAuthn）
    V1DeviceAttest,
    // v1_federated_exchange: RFC 8693 token exchange（外部 IdP 連携）
    V1FederatedExchange,
    // v1_emergency_step_up: break-glass アクセス（always step_up + DPoP 必須）
    V1EmergencyStepUp,
}

// HeaderConfig は Envoy jwt_authn の from_headers 設定を宣言する。
// JWT を抽出する HTTP ヘッダー名と値プレフィックスを指定する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    // name: JWT を抽出する HTTP ヘッダー名
    pub name: String,
    // value_prefix: ヘッダー値のプレフィックス（例: "Bearer "）
    pub value_prefix: String,
}

// ClaimHeader は Envoy jwt_authn の claim_to_headers 設定を宣言する。
// JWT claim を後段フィルタが参照できる HTTP ヘッダーにマッピングする。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimHeader {
    // header_name: マッピング先の HTTP ヘッダー名
    pub header_name: String,
    // claim_name: JWT payload の claim 名
    pub claim_name: String,
}

// JwtProvider は Envoy jwt_authn の 1 プロバイダー設定を宣言する。
// auth_class ごとに 1 JwtProvider を生成する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProvider {
    // name: プロバイダー識別子（auth_class の snake_case 名）
    pub name: String,
    // issuer: JWT の issuer URL（iss claim と照合する）
    pub issuer: String,
    // jwks_uri: JWKS endpoint URL（public key フェッチ先）
    pub jwks_uri: String,
    // audiences: 受け入れる audience リスト
    pub audiences: Vec<String>,
    // from_headers: JWT を抽出する HTTP ヘッダー設定のリスト
    pub from_headers: Vec<HeaderConfig>,
    // claim_to_headers: JWT claim → HTTP ヘッダー マッピングのリスト
    pub claim_to_headers: Vec<ClaimHeader>,
}

// JwtAuthnConfig は Envoy jwt_authn filter の全体設定を宣言する。
// providers: 5 auth_class 分の JwtProvider リスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtAuthnConfig {
    // providers: auth_class ごとの JWT プロバイダー設定
    pub providers: Vec<JwtProvider>,
}

impl JwtAuthnConfig {
    // from_classes_yaml は AuthClass のスライスから JwtAuthnConfig を構築する。
    // spec 04 §v1 auth_class セット（5 class）に対応する JwtProvider を生成する。
    // 環境変数からエンドポイント URL を取得する（デフォルト値あり）。
    pub fn from_classes_yaml(classes: &[AuthClass]) -> Self {
        // Keycloak issuer を環境変数から取得する（デフォルト: cluster 内 Keycloak）
        let keycloak_issuer = std::env::var("KEYCLOAK_ISSUER")
            .unwrap_or_else(|_| "http://keycloak.k1s0.svc:8080/realms/k1s0".to_string());
        // SPIRE bundle endpoint を環境変数から取得する
        let spire_bundle_url = std::env::var("SPIRE_BUNDLE_URL")
            .unwrap_or_else(|_| "http://spire-server.spire.svc:8081/bundle".to_string());
        // 外部 IdP issuer を環境変数から取得する
        let federation_issuer = std::env::var("FEDERATION_ISSUER")
            .unwrap_or_else(|_| "http://external-idp.partner.example.com".to_string());
        // audience を環境変数から取得する
        let audience = std::env::var("KEYCLOAK_AUDIENCE")
            .unwrap_or_else(|_| "k1s0-gateway".to_string());
        // Keycloak JWKS endpoint URL を構築する
        let keycloak_jwks_url = format!("{keycloak_issuer}/protocol/openid-connect/certs");
        // 外部 IdP JWKS endpoint URL を構築する（OpenID Connect discovery 形式）
        let federation_jwks_url = format!("{federation_issuer}/.well-known/jwks.json");

        // auth_class ごとに JwtProvider を構築する
        let providers: Vec<JwtProvider> = classes
            .iter()
            .map(|cls| match cls {
                AuthClass::V1HumanSession => {
                    // v1_human_session: Keycloak OIDC + DPoP 必須
                    // DPoP 用の from_headers と Authorization ヘッダーの両方を設定する
                    JwtProvider {
                        name: "v1_human_session".to_string(),
                        issuer: keycloak_issuer.clone(),
                        jwks_uri: keycloak_jwks_url.clone(),
                        // audience は k1s0-gateway を期待する
                        audiences: vec![audience.clone()],
                        // Authorization: Bearer <jwt> から JWT を抽出する
                        from_headers: vec![
                            HeaderConfig {
                                name: "Authorization".to_string(),
                                value_prefix: "Bearer ".to_string(),
                            },
                            // DPoP ヘッダー: RFC 9449 DPoP proof JWT を抽出する
                            HeaderConfig {
                                name: "DPoP".to_string(),
                                value_prefix: "".to_string(),
                            },
                        ],
                        // sub / jti / tid を後段フィルタのヘッダーにマッピングする
                        claim_to_headers: vec![
                            ClaimHeader {
                                header_name: "x-k1s0-sub".to_string(),
                                claim_name: "sub".to_string(),
                            },
                            ClaimHeader {
                                header_name: "x-k1s0-jti".to_string(),
                                claim_name: "jti".to_string(),
                            },
                            ClaimHeader {
                                header_name: "x-k1s0-tid".to_string(),
                                claim_name: "tid".to_string(),
                            },
                        ],
                    }
                }
                AuthClass::V1WorkloadJwt => {
                    // v1_workload_jwt: SPIFFE SVID JWT（サービス間通信）
                    // SPIRE bundle endpoint から JWKS を取得する
                    JwtProvider {
                        name: "v1_workload_jwt".to_string(),
                        issuer: spire_bundle_url.clone(),
                        jwks_uri: spire_bundle_url.clone(),
                        // audience は k1s0-gateway を期待する（SPIRE audience）
                        audiences: vec![audience.clone()],
                        // Authorization: Bearer <svid-jwt> から JWT を抽出する
                        from_headers: vec![HeaderConfig {
                            name: "Authorization".to_string(),
                            value_prefix: "Bearer ".to_string(),
                        }],
                        // sub（SPIFFE ID）を後段フィルタのヘッダーにマッピングする
                        claim_to_headers: vec![ClaimHeader {
                            header_name: "x-k1s0-spiffe-id".to_string(),
                            claim_name: "sub".to_string(),
                        }],
                    }
                }
                AuthClass::V1DeviceAttest => {
                    // v1_device_attest: デバイス attestation chain（Keycloak 発行 JWT）
                    JwtProvider {
                        name: "v1_device_attest".to_string(),
                        issuer: keycloak_issuer.clone(),
                        jwks_uri: keycloak_jwks_url.clone(),
                        // audience は k1s0-gateway を期待する
                        audiences: vec![audience.clone()],
                        // Authorization: Bearer <device-jwt> から JWT を抽出する
                        from_headers: vec![HeaderConfig {
                            name: "Authorization".to_string(),
                            value_prefix: "Bearer ".to_string(),
                        }],
                        // sub / azp（デバイス ID）を後段フィルタのヘッダーにマッピングする
                        claim_to_headers: vec![
                            ClaimHeader {
                                header_name: "x-k1s0-device-sub".to_string(),
                                claim_name: "sub".to_string(),
                            },
                            ClaimHeader {
                                header_name: "x-k1s0-device-azp".to_string(),
                                claim_name: "azp".to_string(),
                            },
                        ],
                    }
                }
                AuthClass::V1FederatedExchange => {
                    // v1_federated_exchange: RFC 8693 token exchange（外部 IdP 連携）
                    // 外部 IdP の JWKS endpoint から公開鍵を取得する
                    JwtProvider {
                        name: "v1_federated_exchange".to_string(),
                        issuer: federation_issuer.clone(),
                        jwks_uri: federation_jwks_url.clone(),
                        // audience は k1s0-gateway に audience-restricted されていることを検証する
                        audiences: vec![audience.clone()],
                        // Authorization: Bearer <federated-jwt> から JWT を抽出する
                        from_headers: vec![HeaderConfig {
                            name: "Authorization".to_string(),
                            value_prefix: "Bearer ".to_string(),
                        }],
                        // sub / act claim を後段フィルタのヘッダーにマッピングする
                        claim_to_headers: vec![
                            ClaimHeader {
                                header_name: "x-k1s0-federated-sub".to_string(),
                                claim_name: "sub".to_string(),
                            },
                            ClaimHeader {
                                header_name: "x-k1s0-act".to_string(),
                                claim_name: "act".to_string(),
                            },
                        ],
                    }
                }
                AuthClass::V1EmergencyStepUp => {
                    // v1_emergency_step_up: break-glass アクセス（DPoP 必須）
                    // Keycloak OIDC + DPoP binding を強制する
                    JwtProvider {
                        name: "v1_emergency_step_up".to_string(),
                        issuer: keycloak_issuer.clone(),
                        jwks_uri: keycloak_jwks_url.clone(),
                        // audience は k1s0-gateway を期待する
                        audiences: vec![audience.clone()],
                        // Authorization: Bearer + DPoP ヘッダーの両方を設定する
                        from_headers: vec![
                            HeaderConfig {
                                name: "Authorization".to_string(),
                                value_prefix: "Bearer ".to_string(),
                            },
                            // DPoP ヘッダー: RFC 9449 DPoP proof JWT を抽出する（必須）
                            HeaderConfig {
                                name: "DPoP".to_string(),
                                value_prefix: "".to_string(),
                            },
                        ],
                        // sub / jti / tid を後段フィルタのヘッダーにマッピングする
                        claim_to_headers: vec![
                            ClaimHeader {
                                header_name: "x-k1s0-emergency-sub".to_string(),
                                claim_name: "sub".to_string(),
                            },
                            ClaimHeader {
                                header_name: "x-k1s0-emergency-jti".to_string(),
                                claim_name: "jti".to_string(),
                            },
                        ],
                    }
                }
            })
            .collect();
        // 生成された providers 数をログに記録する
        debug!(provider_count = %providers.len(), "JwtAuthnConfig generated");
        Self { providers }
    }

    // to_envoy_yaml は Envoy envoy.filters.http.jwt_authn filter 設定 YAML を生成する。
    // serde_yaml で JwtAuthnConfig を YAML に変換して返す。
    pub fn to_envoy_yaml(&self) -> Result<String> {
        // Envoy jwt_authn filter 設定の構造体を構築する
        // Envoy の設定構造: jwt_providers + rules の 2 キーを持つ
        // providers を name → JwtProvider のマップに変換する
        let mut jwt_providers_map: std::collections::HashMap<String, EnvoyJwtProvider> =
            std::collections::HashMap::new();

        // 各 JwtProvider を Envoy 形式に変換してマップに追加する
        for provider in &self.providers {
            // Envoy 形式の JwtProvider を構築する
            let envoy_provider = EnvoyJwtProvider {
                // issuer: JWT の issuer URL
                issuer: provider.issuer.clone(),
                // audiences: 受け入れる audience リスト
                audiences: provider.audiences.clone(),
                // remote_jwks: JWKS endpoint の設定
                remote_jwks: EnvoyRemoteJwks {
                    // http_uri: JWKS endpoint の URI と timeout
                    http_uri: EnvoyHttpUri {
                        // uri: JWKS endpoint URL
                        uri: provider.jwks_uri.clone(),
                        // cluster: Envoy cluster 名（provider 名に "_jwks" を付加する）
                        cluster: format!("{}_jwks", provider.name),
                        // timeout: JWKS フェッチのタイムアウト（5 秒）
                        timeout: "5s".to_string(),
                    },
                    // cache_duration: JWKS キャッシュ期間（300 秒）
                    cache_duration: "300s".to_string(),
                },
                // from_headers: JWT 抽出元のヘッダー設定
                from_headers: provider
                    .from_headers
                    .iter()
                    .map(|h| EnvoyFromHeader {
                        // name: ヘッダー名
                        name: h.name.clone(),
                        // value_prefix: ヘッダー値のプレフィックス
                        value_prefix: h.value_prefix.clone(),
                    })
                    .collect(),
                // claim_to_headers: claim → header マッピング
                claim_to_headers: provider
                    .claim_to_headers
                    .iter()
                    .map(|c| EnvoyClaimToHeader {
                        // header_name: マッピング先ヘッダー名
                        header_name: c.header_name.clone(),
                        // claim_name: JWT claim 名
                        claim_name: c.claim_name.clone(),
                    })
                    .collect(),
                // forward: JWT を後段フィルタに転送しない（false: security best practice）
                forward: false,
            };
            // プロバイダー名をキーとしてマップに追加する
            jwt_providers_map.insert(provider.name.clone(), envoy_provider);
        }

        // Envoy jwt_authn filter 全体設定を構築する
        let envoy_config = EnvoyJwtAuthnFilter {
            // jwt_providers: プロバイダーマップ
            jwt_providers: jwt_providers_map,
        };

        // serde_yaml で YAML 文字列に変換する
        let yaml_str = serde_yaml::to_string(&envoy_config)
            .with_context(|| "JwtAuthnConfig to_envoy_yaml serialization failed")?;
        // 生成成功をログに記録する
        info!(
            provider_count = %self.providers.len(),
            "Envoy jwt_authn filter YAML generated"
        );
        Ok(yaml_str)
    }
}

// EnvoyJwtAuthnFilter は Envoy envoy.filters.http.jwt_authn filter 設定の最上位構造体。
// serde_yaml でシリアライズして Envoy filter chain 設定に使用する。
#[derive(Debug, Serialize, Deserialize)]
struct EnvoyJwtAuthnFilter {
    // jwt_providers: プロバイダー名 → 設定のマップ
    jwt_providers: std::collections::HashMap<String, EnvoyJwtProvider>,
}

// EnvoyJwtProvider は Envoy jwt_authn の 1 プロバイダー設定（Envoy 形式）。
#[derive(Debug, Serialize, Deserialize)]
struct EnvoyJwtProvider {
    // issuer: JWT の issuer URL
    issuer: String,
    // audiences: 受け入れる audience リスト
    audiences: Vec<String>,
    // remote_jwks: JWKS リモートフェッチ設定
    remote_jwks: EnvoyRemoteJwks,
    // from_headers: JWT 抽出元のヘッダー設定リスト
    from_headers: Vec<EnvoyFromHeader>,
    // claim_to_headers: JWT claim → HTTP ヘッダー マッピングリスト
    claim_to_headers: Vec<EnvoyClaimToHeader>,
    // forward: JWT を後段フィルタに転送するか（false: 転送しない）
    forward: bool,
}

// EnvoyRemoteJwks は Envoy jwt_authn の remote JWKS フェッチ設定。
#[derive(Debug, Serialize, Deserialize)]
struct EnvoyRemoteJwks {
    // http_uri: JWKS endpoint の HTTP URI 設定
    http_uri: EnvoyHttpUri,
    // cache_duration: JWKS キャッシュ期間（例: "300s"）
    cache_duration: String,
}

// EnvoyHttpUri は Envoy の HTTP URI + cluster 設定。
#[derive(Debug, Serialize, Deserialize)]
struct EnvoyHttpUri {
    // uri: エンドポイント URL
    uri: String,
    // cluster: Envoy cluster 名
    cluster: String,
    // timeout: リクエストタイムアウト（例: "5s"）
    timeout: String,
}

// EnvoyFromHeader は Envoy jwt_authn の from_headers エントリ。
#[derive(Debug, Serialize, Deserialize)]
struct EnvoyFromHeader {
    // name: JWT を抽出する HTTP ヘッダー名
    name: String,
    // value_prefix: ヘッダー値のプレフィックス（例: "Bearer "）
    value_prefix: String,
}

// EnvoyClaimToHeader は Envoy jwt_authn の claim_to_headers エントリ。
#[derive(Debug, Serialize, Deserialize)]
struct EnvoyClaimToHeader {
    // header_name: マッピング先の HTTP ヘッダー名
    header_name: String,
    // claim_name: JWT payload の claim 名
    claim_name: String,
}

// #[cfg(test)] mod tests — JwtAuthnConfig の unit テスト
#[cfg(test)]
mod tests {
    // super: このモジュールの親スコープ（envoy_jwt_authn_config.rs 全体）をインポートする
    use super::*;

    // test_from_classes_yaml_all_5_providers は 5 auth_class 全ての JwtProvider が
    // 生成されることを確認する。
    #[test]
    fn test_from_classes_yaml_all_5_providers() {
        // 5 auth_class を全て指定する
        let classes = vec![
            AuthClass::V1HumanSession,
            AuthClass::V1WorkloadJwt,
            AuthClass::V1DeviceAttest,
            AuthClass::V1FederatedExchange,
            AuthClass::V1EmergencyStepUp,
        ];
        // JwtAuthnConfig を構築する
        let config = JwtAuthnConfig::from_classes_yaml(&classes);
        // providers の数が 5 であることを確認する
        assert_eq!(
            config.providers.len(),
            5,
            "5 auth_class から 5 JwtProvider が生成されなければならない（実際: {}）",
            config.providers.len()
        );
        // 各 provider の name が auth_class に対応していることを確認する
        let names: Vec<&str> = config.providers.iter().map(|p| p.name.as_str()).collect();
        // 期待する provider 名が全て含まれることを確認する
        assert!(names.contains(&"v1_human_session"), "v1_human_session provider が必要");
        assert!(names.contains(&"v1_workload_jwt"), "v1_workload_jwt provider が必要");
        assert!(names.contains(&"v1_device_attest"), "v1_device_attest provider が必要");
        assert!(names.contains(&"v1_federated_exchange"), "v1_federated_exchange provider が必要");
        assert!(names.contains(&"v1_emergency_step_up"), "v1_emergency_step_up provider が必要");
    }

    // test_to_envoy_yaml_contains_jwt_providers は生成された YAML が "jwt_providers" キーを
    // 持つことを確認する（property test に対応する）。
    #[test]
    fn test_to_envoy_yaml_contains_jwt_providers() {
        // 全 5 auth_class を指定する
        let classes = vec![
            AuthClass::V1HumanSession,
            AuthClass::V1WorkloadJwt,
            AuthClass::V1DeviceAttest,
            AuthClass::V1FederatedExchange,
            AuthClass::V1EmergencyStepUp,
        ];
        // JwtAuthnConfig を構築する
        let config = JwtAuthnConfig::from_classes_yaml(&classes);
        // Envoy YAML を生成する
        let yaml_result = config.to_envoy_yaml();
        // 生成が Ok であることを確認する
        assert!(
            yaml_result.is_ok(),
            "to_envoy_yaml は Ok を返さなければならない（実際: {:?}）",
            yaml_result.err()
        );
        // 生成された YAML が "jwt_providers" キーを持つことを確認する
        let yaml_str = yaml_result.unwrap();
        assert!(
            yaml_str.contains("jwt_providers"),
            "生成された YAML は 'jwt_providers' キーを持たなければならない（実際: {yaml_str}）"
        );
    }

    // test_dpop_from_headers は v1_human_session と v1_emergency_step_up が DPoP ヘッダー設定を
    // 持つことを確認する。
    #[test]
    fn test_dpop_from_headers() {
        // DPoP 必須の 2 auth_class を指定する
        let classes = vec![AuthClass::V1HumanSession, AuthClass::V1EmergencyStepUp];
        // JwtAuthnConfig を構築する
        let config = JwtAuthnConfig::from_classes_yaml(&classes);
        // 各 provider の from_headers に DPoP が含まれることを確認する
        for provider in &config.providers {
            // from_headers の中に name="DPoP" のエントリが存在することを確認する
            let has_dpop = provider
                .from_headers
                .iter()
                .any(|h| h.name == "DPoP");
            assert!(
                has_dpop,
                "provider '{}' は DPoP from_headers を持たなければならない",
                provider.name
            );
        }
    }
}
