"""設定型定義（pydantic BaseModel）"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field


class AppSection(BaseModel):
    """アプリケーション基本設定。"""

    name: str
    version: str = "0.1.0"
    tier: str = "system"
    environment: str = "development"


class ServerSection(BaseModel):
    """HTTP サーバー設定。"""

    host: str = "0.0.0.0"
    port: int = Field(default=8080, ge=1, le=65535)
    read_timeout: int = 30
    write_timeout: int = 30
    shutdown_timeout: int = 10


class GrpcSection(BaseModel):
    """gRPC サーバー設定。"""

    port: int = Field(default=50051, ge=1, le=65535)
    max_recv_msg_size: int = 4 * 1024 * 1024  # 4MB


class DatabaseSection(BaseModel):
    """データベース接続設定。"""

    host: str = "localhost"
    port: int = Field(default=5432, ge=1, le=65535)
    name: str
    user: str
    password: str = ""
    ssl_mode: str = "disable"
    pool_min_size: int = 2
    pool_max_size: int = 10
    connect_timeout: int = 10


class KafkaSaslSection(BaseModel):
    """Kafka SASL 設定。"""

    mechanism: str = "PLAIN"
    username: str = ""
    password: str = ""


class KafkaSection(BaseModel):
    """Kafka 接続設定。"""

    brokers: list[str] = Field(default_factory=list)
    consumer_group: str = ""
    security_protocol: str = "PLAINTEXT"
    sasl: KafkaSaslSection = Field(default_factory=KafkaSaslSection)
    topics: list[str] = Field(default_factory=list)


class RedisSection(BaseModel):
    """Redis 接続設定。"""

    host: str = "localhost"
    port: int = Field(default=6379, ge=1, le=65535)
    password: str = ""
    db: int = 0
    pool_size: int = 10


class LogSection(BaseModel):
    """ログ設定。"""

    level: str = "INFO"
    format: Literal["json", "text"] = "json"


class TraceSection(BaseModel):
    """分散トレーシング設定。"""

    enabled: bool = False
    endpoint: str = ""
    sample_rate: float = Field(default=1.0, ge=0.0, le=1.0)


class MetricsSection(BaseModel):
    """メトリクス設定。"""

    enabled: bool = True
    path: str = "/metrics"


class ObservabilitySection(BaseModel):
    """可観測性設定。"""

    log: LogSection = Field(default_factory=LogSection)
    trace: TraceSection = Field(default_factory=TraceSection)
    metrics: MetricsSection = Field(default_factory=MetricsSection)


class JwtSection(BaseModel):
    """JWT 設定。"""

    issuer: str = ""
    audience: str = ""
    public_key_path: str = ""


class OidcSection(BaseModel):
    """OIDC 設定。"""

    jwks_uri: str = ""
    token_endpoint: str = ""
    client_id: str = ""
    client_secret: str = ""


class AuthSection(BaseModel):
    """認証設定。"""

    jwt: JwtSection = Field(default_factory=JwtSection)
    oidc: OidcSection = Field(default_factory=OidcSection)


class AppConfig(BaseModel):
    """アプリケーション設定全体。"""

    app: AppSection
    server: ServerSection = Field(default_factory=ServerSection)
    grpc: GrpcSection | None = None
    database: DatabaseSection | None = None
    kafka: KafkaSection | None = None
    redis: RedisSection | None = None
    observability: ObservabilitySection = Field(default_factory=ObservabilitySection)
    auth: AuthSection | None = None
