// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-swift-libraries",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0AuditClient", targets: ["K1s0AuditClient"]),
        .library(name: "K1s0Auth", targets: ["K1s0Auth"]),
        .library(name: "K1s0Cache", targets: ["K1s0Cache"]),
        .library(name: "K1s0CircuitBreaker", targets: ["K1s0CircuitBreaker"]),
        .library(name: "K1s0Config", targets: ["K1s0Config"]),
        .library(name: "K1s0Correlation", targets: ["K1s0Correlation"]),
        .library(name: "K1s0DistributedLock", targets: ["K1s0DistributedLock"]),
        .library(name: "K1s0Dlq", targets: ["K1s0Dlq"]),
        .library(name: "K1s0Encryption", targets: ["K1s0Encryption"]),
        .library(name: "K1s0EventBus", targets: ["K1s0EventBus"]),
        .library(name: "K1s0EventStore", targets: ["K1s0EventStore"]),
        .library(name: "K1s0FeatureFlag", targets: ["K1s0FeatureFlag"]),
        .library(name: "K1s0FileClient", targets: ["K1s0FileClient"]),
        .library(name: "K1s0GraphQlClient", targets: ["K1s0GraphQlClient"]),
        .library(name: "K1s0Health", targets: ["K1s0Health"]),
        .library(name: "K1s0Idempotency", targets: ["K1s0Idempotency"]),
        .library(name: "K1s0Kafka", targets: ["K1s0Kafka"]),
        .library(name: "K1s0Messaging", targets: ["K1s0Messaging"]),
        .library(name: "K1s0Migration", targets: ["K1s0Migration"]),
        .library(name: "K1s0NotificationClient", targets: ["K1s0NotificationClient"]),
        .library(name: "K1s0Outbox", targets: ["K1s0Outbox"]),
        .library(name: "K1s0Pagination", targets: ["K1s0Pagination"]),
        .library(name: "K1s0QuotaClient", targets: ["K1s0QuotaClient"]),
        .library(name: "K1s0RateLimitClient", targets: ["K1s0RateLimitClient"]),
        .library(name: "K1s0Resiliency", targets: ["K1s0Resiliency"]),
        .library(name: "K1s0Retry", targets: ["K1s0Retry"]),
        .library(name: "K1s0Saga", targets: ["K1s0Saga"]),
        .library(name: "K1s0SchedulerClient", targets: ["K1s0SchedulerClient"]),
        .library(name: "K1s0SchemaRegistry", targets: ["K1s0SchemaRegistry"]),
        .library(name: "K1s0SearchClient", targets: ["K1s0SearchClient"]),
        .library(name: "K1s0ServiceAuth", targets: ["K1s0ServiceAuth"]),
        .library(name: "K1s0SessionClient", targets: ["K1s0SessionClient"]),
        .library(name: "K1s0Telemetry", targets: ["K1s0Telemetry"]),
        .library(name: "K1s0TenantClient", targets: ["K1s0TenantClient"]),
        .library(name: "K1s0TestHelper", targets: ["K1s0TestHelper"]),
        .library(name: "K1s0Tracing", targets: ["K1s0Tracing"]),
        .library(name: "K1s0Validation", targets: ["K1s0Validation"]),
        .library(name: "K1s0VaultClient", targets: ["K1s0VaultClient"]),
        .library(name: "K1s0WebhookClient", targets: ["K1s0WebhookClient"]),
        .library(name: "K1s0WebSocket", targets: ["K1s0WebSocket"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-crypto.git", from: "3.0.0"),
        .package(url: "https://github.com/tmthecoder/Argon2Swift.git", from: "1.0.0"),
    ],
    targets: [
        // audit-client
        .target(
            name: "K1s0AuditClient",
            path: "audit-client/Sources/K1s0AuditClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0AuditClientTests",
            dependencies: ["K1s0AuditClient"],
            path: "audit-client/Tests/K1s0AuditClientTests"
        ),
        // auth
        .target(
            name: "K1s0Auth",
            dependencies: [
                .product(name: "Crypto", package: "swift-crypto"),
                .product(name: "_CryptoExtras", package: "swift-crypto"),
            ],
            path: "auth/Sources/K1s0Auth",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0AuthTests",
            dependencies: ["K1s0Auth"],
            path: "auth/Tests/K1s0AuthTests"
        ),
        // cache
        .target(
            name: "K1s0Cache",
            path: "cache/Sources/K1s0Cache",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0CacheTests",
            dependencies: ["K1s0Cache"],
            path: "cache/Tests/K1s0CacheTests"
        ),
        // circuit-breaker
        .target(
            name: "K1s0CircuitBreaker",
            path: "circuit-breaker/Sources/K1s0CircuitBreaker",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0CircuitBreakerTests",
            dependencies: ["K1s0CircuitBreaker"],
            path: "circuit-breaker/Tests/K1s0CircuitBreakerTests"
        ),
        // config
        .target(
            name: "K1s0Config",
            path: "config/Sources/K1s0Config",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ConfigTests",
            dependencies: ["K1s0Config"],
            path: "config/Tests/K1s0ConfigTests"
        ),
        // correlation
        .target(
            name: "K1s0Correlation",
            path: "correlation/Sources/K1s0Correlation",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0CorrelationTests",
            dependencies: ["K1s0Correlation"],
            path: "correlation/Tests/K1s0CorrelationTests"
        ),
        // distributed-lock
        .target(
            name: "K1s0DistributedLock",
            path: "distributed-lock/Sources/K1s0DistributedLock",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0DistributedLockTests",
            dependencies: ["K1s0DistributedLock"],
            path: "distributed-lock/Tests/K1s0DistributedLockTests"
        ),
        // dlq
        .target(
            name: "K1s0Dlq",
            path: "dlq/Sources/K1s0Dlq",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0DlqTests",
            dependencies: ["K1s0Dlq"],
            path: "dlq/Tests/K1s0DlqTests"
        ),
        // encryption
        .target(
            name: "K1s0Encryption",
            dependencies: [
                .product(name: "Crypto", package: "swift-crypto"),
                .product(name: "Argon2Swift", package: "Argon2Swift"),
            ],
            path: "encryption/Sources/K1s0Encryption",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0EncryptionTests",
            dependencies: [
                "K1s0Encryption",
                .product(name: "Crypto", package: "swift-crypto"),
            ],
            path: "encryption/Tests/K1s0EncryptionTests"
        ),
        // event-bus
        .target(
            name: "K1s0EventBus",
            path: "event-bus/Sources/K1s0EventBus",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0EventBusTests",
            dependencies: ["K1s0EventBus"],
            path: "event-bus/Tests/K1s0EventBusTests"
        ),
        // eventstore
        .target(
            name: "K1s0EventStore",
            path: "eventstore/Sources/K1s0EventStore",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0EventStoreTests",
            dependencies: ["K1s0EventStore"],
            path: "eventstore/Tests/K1s0EventStoreTests"
        ),
        // featureflag
        .target(
            name: "K1s0FeatureFlag",
            path: "featureflag/Sources/K1s0FeatureFlag",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0FeatureFlagTests",
            dependencies: ["K1s0FeatureFlag"],
            path: "featureflag/Tests/K1s0FeatureFlagTests"
        ),
        // file-client
        .target(
            name: "K1s0FileClient",
            path: "file-client/Sources/K1s0FileClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0FileClientTests",
            dependencies: ["K1s0FileClient"],
            path: "file-client/Tests/K1s0FileClientTests"
        ),
        // graphql-client
        .target(
            name: "K1s0GraphQlClient",
            path: "graphql-client/Sources/K1s0GraphQlClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0GraphQlClientTests",
            dependencies: ["K1s0GraphQlClient"],
            path: "graphql-client/Tests/K1s0GraphQlClientTests"
        ),
        // health
        .target(
            name: "K1s0Health",
            path: "health/Sources/K1s0Health",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0HealthTests",
            dependencies: ["K1s0Health"],
            path: "health/Tests/K1s0HealthTests"
        ),
        // idempotency
        .target(
            name: "K1s0Idempotency",
            path: "idempotency/Sources/K1s0Idempotency",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0IdempotencyTests",
            dependencies: ["K1s0Idempotency"],
            path: "idempotency/Tests/K1s0IdempotencyTests"
        ),
        // kafka
        .target(
            name: "K1s0Kafka",
            path: "kafka/Sources/K1s0Kafka",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0KafkaTests",
            dependencies: ["K1s0Kafka"],
            path: "kafka/Tests/K1s0KafkaTests"
        ),
        // messaging
        .target(
            name: "K1s0Messaging",
            path: "messaging/Sources/K1s0Messaging",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0MessagingTests",
            dependencies: ["K1s0Messaging"],
            path: "messaging/Tests/K1s0MessagingTests"
        ),
        // migration
        .target(
            name: "K1s0Migration",
            path: "migration/Sources/K1s0Migration",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0MigrationTests",
            dependencies: ["K1s0Migration"],
            path: "migration/Tests/K1s0MigrationTests"
        ),
        // notification-client
        .target(
            name: "K1s0NotificationClient",
            path: "notification-client/Sources/K1s0NotificationClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0NotificationClientTests",
            dependencies: ["K1s0NotificationClient"],
            path: "notification-client/Tests/K1s0NotificationClientTests"
        ),
        // outbox
        .target(
            name: "K1s0Outbox",
            path: "outbox/Sources/K1s0Outbox",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0OutboxTests",
            dependencies: ["K1s0Outbox"],
            path: "outbox/Tests/K1s0OutboxTests"
        ),
        // pagination
        .target(
            name: "K1s0Pagination",
            path: "pagination/Sources/K1s0Pagination",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0PaginationTests",
            dependencies: ["K1s0Pagination"],
            path: "pagination/Tests/K1s0PaginationTests"
        ),
        // quota-client
        .target(
            name: "K1s0QuotaClient",
            path: "quota-client/Sources/K1s0QuotaClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0QuotaClientTests",
            dependencies: ["K1s0QuotaClient"],
            path: "quota-client/Tests/K1s0QuotaClientTests"
        ),
        // ratelimit-client
        .target(
            name: "K1s0RateLimitClient",
            path: "ratelimit-client/Sources/K1s0RateLimitClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0RateLimitClientTests",
            dependencies: ["K1s0RateLimitClient"],
            path: "ratelimit-client/Tests/K1s0RateLimitClientTests"
        ),
        // resiliency
        .target(
            name: "K1s0Resiliency",
            path: "resiliency/Sources/K1s0Resiliency",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ResiliencyTests",
            dependencies: ["K1s0Resiliency"],
            path: "resiliency/Tests/K1s0ResiliencyTests"
        ),
        // retry
        .target(
            name: "K1s0Retry",
            path: "retry/Sources/K1s0Retry",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0RetryTests",
            dependencies: ["K1s0Retry"],
            path: "retry/Tests/K1s0RetryTests"
        ),
        // saga
        .target(
            name: "K1s0Saga",
            path: "saga/Sources/K1s0Saga",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SagaTests",
            dependencies: ["K1s0Saga"],
            path: "saga/Tests/K1s0SagaTests"
        ),
        // scheduler-client
        .target(
            name: "K1s0SchedulerClient",
            path: "scheduler-client/Sources/K1s0SchedulerClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SchedulerClientTests",
            dependencies: ["K1s0SchedulerClient"],
            path: "scheduler-client/Tests/K1s0SchedulerClientTests"
        ),
        // schemaregistry
        .target(
            name: "K1s0SchemaRegistry",
            path: "schemaregistry/Sources/K1s0SchemaRegistry",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SchemaRegistryTests",
            dependencies: ["K1s0SchemaRegistry"],
            path: "schemaregistry/Tests/K1s0SchemaRegistryTests"
        ),
        // search-client
        .target(
            name: "K1s0SearchClient",
            path: "search-client/Sources/K1s0SearchClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SearchClientTests",
            dependencies: ["K1s0SearchClient"],
            path: "search-client/Tests/K1s0SearchClientTests"
        ),
        // serviceauth
        .target(
            name: "K1s0ServiceAuth",
            path: "serviceauth/Sources/K1s0ServiceAuth",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ServiceAuthTests",
            dependencies: ["K1s0ServiceAuth"],
            path: "serviceauth/Tests/K1s0ServiceAuthTests"
        ),
        // session-client
        .target(
            name: "K1s0SessionClient",
            path: "session-client/Sources/K1s0SessionClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SessionClientTests",
            dependencies: ["K1s0SessionClient"],
            path: "session-client/Tests/K1s0SessionClientTests"
        ),
        // telemetry
        .target(
            name: "K1s0Telemetry",
            path: "telemetry/Sources/K1s0Telemetry",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TelemetryTests",
            dependencies: ["K1s0Telemetry"],
            path: "telemetry/Tests/K1s0TelemetryTests"
        ),
        // tenant-client
        .target(
            name: "K1s0TenantClient",
            path: "tenant-client/Sources/K1s0TenantClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TenantClientTests",
            dependencies: ["K1s0TenantClient"],
            path: "tenant-client/Tests/K1s0TenantClientTests"
        ),
        // test-helper
        .target(
            name: "K1s0TestHelper",
            path: "test-helper/Sources/K1s0TestHelper",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TestHelperTests",
            dependencies: ["K1s0TestHelper"],
            path: "test-helper/Tests/K1s0TestHelperTests"
        ),
        // tracing
        .target(
            name: "K1s0Tracing",
            path: "tracing/Sources/K1s0Tracing",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TracingTests",
            dependencies: ["K1s0Tracing"],
            path: "tracing/Tests/K1s0TracingTests"
        ),
        // validation
        .target(
            name: "K1s0Validation",
            path: "validation/Sources/K1s0Validation",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ValidationTests",
            dependencies: ["K1s0Validation"],
            path: "validation/Tests/K1s0ValidationTests"
        ),
        // vault-client
        .target(
            name: "K1s0VaultClient",
            path: "vault-client/Sources/K1s0VaultClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0VaultClientTests",
            dependencies: ["K1s0VaultClient"],
            path: "vault-client/Tests/K1s0VaultClientTests"
        ),
        // webhook-client
        .target(
            name: "K1s0WebhookClient",
            dependencies: [
                .product(name: "Crypto", package: "swift-crypto"),
            ],
            path: "webhook-client/Sources/K1s0WebhookClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0WebhookClientTests",
            dependencies: ["K1s0WebhookClient"],
            path: "webhook-client/Tests/K1s0WebhookClientTests"
        ),
        // websocket
        .target(
            name: "K1s0WebSocket",
            path: "websocket/Sources/K1s0WebSocket",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0WebSocketTests",
            dependencies: ["K1s0WebSocket"],
            path: "websocket/Tests/K1s0WebSocketTests"
        ),
    ]
)
