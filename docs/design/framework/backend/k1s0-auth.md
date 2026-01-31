# k1s0-auth

## 目的

JWT/OIDC 検証、ポリシー評価、監査ログの統一化を提供する。

## 設計原則

1. **JWT/OIDC 統一**: JWKS 自動更新、複数キーのローテーション対応
2. **ポリシー柔軟性**: RBAC/ABAC 両対応
3. **監査ログ**: 全認証・認可操作の記録
4. **ミドルウェア統合**: Axum/Tonic 両対応

## 主要な型

### Claims

```rust
pub struct Claims {
    pub sub: String,                // ユーザーID
    pub iss: String,                // 発行者
    pub aud: Option<AudienceClaim>, // 対象者
    pub exp: i64,                   // 有効期限
    pub iat: i64,                   // 発行日時
    pub roles: Vec<String>,         // ロール
    pub permissions: Vec<String>,   // パーミッション
    pub tenant_id: Option<String>,  // テナントID
}
```

### JwtVerifier

```rust
pub struct JwtVerifierConfig {
    issuer: String,
    jwks_uri: String,
    audience: Option<String>,
    jwks_cache_ttl_secs: u64,
}

impl JwtVerifierConfig {
    pub fn new(issuer: impl Into<String>) -> Self;
    pub fn with_jwks_uri(self, uri: impl Into<String>) -> Self;
    pub fn with_audience(self, audience: impl Into<String>) -> Self;
}

pub struct JwtVerifier {
    config: JwtVerifierConfig,
}

impl JwtVerifier {
    pub fn new(config: JwtVerifierConfig) -> Self;
    pub async fn verify(&self, token: &str) -> Result<Claims, AuthError>;
}
```

### PolicyEvaluator

```rust
pub enum PolicyDecision {
    Allow,
    Deny,
    NotApplicable,
}

pub struct PolicyRequest {
    pub subject: PolicySubject,
    pub action: Action,
    pub resource: ResourceContext,
}

pub struct PolicyResult {
    pub decision: PolicyDecision,
    pub reason: Option<String>,
    pub matched_rules: Vec<String>,
}

pub struct PolicyEvaluator {
    rules: Arc<RwLock<Vec<PolicyRule>>>,
}

impl PolicyEvaluator {
    pub fn new() -> Self;
    pub async fn add_rules(&self, rules: Vec<PolicyRule>);
    pub async fn evaluate(&self, request: &PolicyRequest) -> PolicyResult;
}
```

### AuditLogger

```rust
pub enum AuditEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationSuccess,
    AuthorizationFailure,
    DataAccess,
    DataModification,
}

pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: AuditActor,
    pub resource: Option<AuditResource>,
    pub action: String,
    pub result: AuditResult,
}

pub struct AuditLogger {
    service_name: String,
}

impl AuditLogger {
    pub fn with_default_sink(service_name: &str) -> Self;
    pub fn log_authentication_success(&self, actor: AuditActor);
    pub fn log_authentication_failure(&self, actor: AuditActor, reason: &str);
    pub fn log_authorization(&self, request: &PolicyRequest, decision: PolicyDecision);
}
```

### UserInfoClient (OIDC UserInfo)

```rust
/// OIDC UserInfo レスポンス
pub struct UserInfo {
    pub sub: String,                    // Subject Identifier (必須)
    pub name: Option<String>,           // 表示名
    pub given_name: Option<String>,     // 名
    pub family_name: Option<String>,    // 姓
    pub email: Option<String>,          // メールアドレス
    pub email_verified: Option<bool>,   // メール検証済みフラグ
    pub picture: Option<String>,        // プロフィール画像URL
    pub locale: Option<String>,         // ロケール
    pub address: Option<UserInfoAddress>, // 住所
    pub additional_claims: HashMap<String, Value>, // 追加クレーム
}

/// UserInfo クライアント
pub struct UserInfoClient {
    discovery: OidcDiscovery,
    client: reqwest::Client,
}

impl UserInfoClient {
    /// 新しい UserInfo クライアントを作成
    pub fn new(discovery: OidcDiscovery) -> Self;

    /// アクセストークンを使用してユーザー情報を取得
    pub async fn get_userinfo(&self, access_token: &str) -> Result<UserInfo, AuthError>;

    /// エンドポイント直接指定でユーザー情報を取得
    pub async fn get_userinfo_from_endpoint(
        &self,
        endpoint: &str,
        access_token: &str,
    ) -> Result<UserInfo, AuthError>;
}
```

## Go 版（k1s0-auth）

```go
// OIDCUserInfo holds user info from the OIDC provider.
type OIDCUserInfo struct {
    Subject       string `json:"sub"`
    Name          string `json:"name,omitempty"`
    GivenName     string `json:"given_name,omitempty"`
    FamilyName    string `json:"family_name,omitempty"`
    Email         string `json:"email,omitempty"`
    EmailVerified bool   `json:"email_verified,omitempty"`
    Picture       string `json:"picture,omitempty"`
    Locale        string `json:"locale,omitempty"`
    Address       *OIDCAddress `json:"address,omitempty"`
}

// UserInfo fetches user info from the OIDC provider.
func (v *OIDCValidator) UserInfo(ctx context.Context, accessToken string) (*OIDCUserInfo, error)

// UserInfoWithClient fetches user info using a custom HTTP client.
func (v *OIDCValidator) UserInfoWithClient(ctx context.Context, httpClient *http.Client, accessToken string) (*OIDCUserInfo, error)

// UserInfoClient is a standalone client for fetching OIDC UserInfo.
type UserInfoClient struct {
    httpClient       *http.Client
    userInfoEndpoint string
}

func NewUserInfoClient(userInfoEndpoint string, httpClient *http.Client) *UserInfoClient
func (c *UserInfoClient) GetUserInfo(ctx context.Context, accessToken string) (*OIDCUserInfo, error)
```

## C# 版（K1s0.Auth）

### 主要な型

```csharp
public record Claims(
    string Sub, string Iss, string? Aud, long Exp, long Iat,
    List<string> Roles, List<string> Permissions, string? TenantId);

public class JwtVerifierConfig
{
    public string Issuer { get; }
    public string? JwksUri { get; set; }
    public string? Audience { get; set; }

    public JwtVerifierConfig(string issuer);
}

public class JwtVerifier
{
    public JwtVerifier(JwtVerifierConfig config);
    public Task<Claims> VerifyAsync(string token);
}

public enum PolicyDecision { Allow, Deny, NotApplicable }

public record PolicyRequest(PolicySubject Subject, Action Action, ResourceContext Resource);
public record PolicyResult(PolicyDecision Decision, string? Reason, List<string> MatchedRules);

public class PolicyEvaluator
{
    public Task AddRulesAsync(IEnumerable<PolicyRule> rules);
    public Task<PolicyResult> EvaluateAsync(PolicyRequest request);
}

public class AuditLogger
{
    public AuditLogger(string serviceName);
    public void LogAuthenticationSuccess(AuditActor actor);
    public void LogAuthenticationFailure(AuditActor actor, string reason);
    public void LogAuthorization(PolicyRequest request, PolicyDecision decision);
}

// OIDC UserInfo
public record UserInfo(
    string Sub, string? Name, string? Email,
    bool? EmailVerified, string? Picture, string? Locale);

public class UserInfoClient
{
    public UserInfoClient(OidcDiscovery discovery);
    public Task<UserInfo> GetUserInfoAsync(string accessToken);
}
```

### 使用例

```csharp
using K1s0.Auth;

var config = new JwtVerifierConfig("https://auth.example.com")
{
    JwksUri = "https://auth.example.com/.well-known/jwks.json",
    Audience = "my-api"
};
var verifier = new JwtVerifier(config);
var claims = await verifier.VerifyAsync("eyJ...");

var evaluator = new PolicyEvaluator();
var result = await evaluator.EvaluateAsync(new PolicyRequest(
    new PolicySubject("user123", Roles: new[] { "admin" }),
    new Action("user", "delete"),
    ResourceContext.Default));
```

## Python 版（k1s0-auth）

### 主要な型

```python
@dataclass
class Claims:
    sub: str
    iss: str
    aud: str | None
    exp: int
    iat: int
    roles: list[str]
    permissions: list[str]
    tenant_id: str | None = None

@dataclass
class JwtVerifierConfig:
    issuer: str
    jwks_uri: str | None = None
    audience: str | None = None

class JwtVerifier:
    def __init__(self, config: JwtVerifierConfig) -> None: ...
    async def verify(self, token: str) -> Claims: ...

class PolicyDecision(Enum):
    ALLOW = "allow"
    DENY = "deny"
    NOT_APPLICABLE = "not_applicable"

class PolicyEvaluator:
    async def add_rules(self, rules: list[PolicyRule]) -> None: ...
    async def evaluate(self, request: PolicyRequest) -> PolicyResult: ...

class AuditLogger:
    def __init__(self, service_name: str) -> None: ...
    def log_authentication_success(self, actor: AuditActor) -> None: ...
    def log_authentication_failure(self, actor: AuditActor, reason: str) -> None: ...

@dataclass
class UserInfo:
    sub: str
    name: str | None = None
    email: str | None = None
    email_verified: bool | None = None

class UserInfoClient:
    def __init__(self, discovery: OidcDiscovery) -> None: ...
    async def get_userinfo(self, access_token: str) -> UserInfo: ...
```

### 使用例

```python
from k1s0_auth import JwtVerifier, JwtVerifierConfig, PolicyEvaluator

config = JwtVerifierConfig(
    issuer="https://auth.example.com",
    jwks_uri="https://auth.example.com/.well-known/jwks.json",
    audience="my-api"
)
verifier = JwtVerifier(config)
claims = await verifier.verify("eyJ...")

evaluator = PolicyEvaluator()
result = await evaluator.evaluate(PolicyRequest(
    subject=PolicySubject("user123", roles=["admin"]),
    action=Action("user", "delete"),
    resource=ResourceContext.default()
))
```

## Kotlin 版（k1s0-auth）

nimbus-jose-jwt ベースの認証・認可ライブラリ。

### 主要な型

```kotlin
data class Claims(
    val sub: String, val iss: String, val aud: String?, val exp: Long, val iat: Long,
    val roles: List<String>, val permissions: List<String>, val tenantId: String? = null
)

data class JwtVerifierConfig(
    val issuer: String,
    val jwksUri: String? = null,
    val audience: String? = null
)

class JwtVerifier(private val config: JwtVerifierConfig) {
    suspend fun verify(token: String): Claims
}

enum class PolicyDecision { Allow, Deny, NotApplicable }

data class PolicyRequest(val subject: PolicySubject, val action: Action, val resource: ResourceContext)
data class PolicyResult(val decision: PolicyDecision, val reason: String?, val matchedRules: List<String>)

class PolicyEvaluator {
    suspend fun addRules(rules: List<PolicyRule>)
    suspend fun evaluate(request: PolicyRequest): PolicyResult
}

class AuditLogger(private val serviceName: String) {
    fun logAuthenticationSuccess(actor: AuditActor)
    fun logAuthenticationFailure(actor: AuditActor, reason: String)
    fun logAuthorization(request: PolicyRequest, decision: PolicyDecision)
}

data class UserInfo(
    val sub: String, val name: String? = null,
    val email: String? = null, val emailVerified: Boolean? = null
)

class UserInfoClient(private val discovery: OidcDiscovery) {
    suspend fun getUserInfo(accessToken: String): UserInfo
}
```

### 使用例

```kotlin
import com.k1s0.auth.*

val config = JwtVerifierConfig(
    issuer = "https://auth.example.com",
    jwksUri = "https://auth.example.com/.well-known/jwks.json",
    audience = "my-api"
)
val verifier = JwtVerifier(config)
val claims = verifier.verify("eyJ...")

val evaluator = PolicyEvaluator()
val result = evaluator.evaluate(PolicyRequest(
    subject = PolicySubject("user123", roles = listOf("admin")),
    action = Action("user", "delete"),
    resource = ResourceContext.default()
))
```

## Features

```toml
[features]
default = []
axum-layer = ["axum", "tower"]
tonic-interceptor = ["tonic"]
redis-cache = ["k1s0-cache/redis"]
postgres-policy = ["k1s0-db/postgres"]
full = ["axum-layer", "tonic-interceptor", "redis-cache", "postgres-policy"]
```

## 使用例

```rust
use k1s0_auth::{JwtVerifier, JwtVerifierConfig, PolicyEvaluator, AuditLogger};
use k1s0_auth::policy::{PolicyBuilder, PolicySubject, Action, PolicyRequest};

// JWT検証
let config = JwtVerifierConfig::new("https://auth.example.com")
    .with_jwks_uri("https://auth.example.com/.well-known/jwks.json")
    .with_audience("my-api");

let verifier = JwtVerifier::new(config);
let claims = verifier.verify("eyJ...").await?;

// ポリシー評価
let evaluator = PolicyEvaluator::new();
let rules = PolicyBuilder::new()
    .admin_rule("admin")
    .read_rule("user_read", "user", vec!["user"], 10)
    .build();

evaluator.add_rules(rules).await;

let subject = PolicySubject::new("user123").with_role("admin");
let action = Action::new("user", "delete");
let request = PolicyRequest {
    subject,
    action,
    resource: ResourceContext::default(),
};
let result = evaluator.evaluate(&request).await;

// 監査ログ
let logger = AuditLogger::with_default_sink("my-service");
logger.log_authentication_success(AuditActor::new("user123"));
```
