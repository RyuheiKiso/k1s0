import { z } from 'zod';
export declare const AppConfigSchema: z.ZodObject<{
    name: z.ZodString;
    version: z.ZodString;
    tier: z.ZodEnum<["system", "business", "service"]>;
    environment: z.ZodEnum<["dev", "staging", "prod"]>;
}, "strip", z.ZodTypeAny, {
    name: string;
    version: string;
    tier: "system" | "business" | "service";
    environment: "dev" | "staging" | "prod";
}, {
    name: string;
    version: string;
    tier: "system" | "business" | "service";
    environment: "dev" | "staging" | "prod";
}>;
export declare const ServerConfigSchema: z.ZodObject<{
    host: z.ZodString;
    port: z.ZodNumber;
    read_timeout: z.ZodOptional<z.ZodString>;
    write_timeout: z.ZodOptional<z.ZodString>;
    shutdown_timeout: z.ZodOptional<z.ZodString>;
}, "strip", z.ZodTypeAny, {
    host: string;
    port: number;
    read_timeout?: string | undefined;
    write_timeout?: string | undefined;
    shutdown_timeout?: string | undefined;
}, {
    host: string;
    port: number;
    read_timeout?: string | undefined;
    write_timeout?: string | undefined;
    shutdown_timeout?: string | undefined;
}>;
export declare const GrpcConfigSchema: z.ZodOptional<z.ZodObject<{
    port: z.ZodNumber;
    max_recv_msg_size: z.ZodOptional<z.ZodNumber>;
}, "strip", z.ZodTypeAny, {
    port: number;
    max_recv_msg_size?: number | undefined;
}, {
    port: number;
    max_recv_msg_size?: number | undefined;
}>>;
export declare const DatabaseConfigSchema: z.ZodOptional<z.ZodObject<{
    host: z.ZodString;
    port: z.ZodNumber;
    name: z.ZodString;
    user: z.ZodString;
    password: z.ZodString;
    ssl_mode: z.ZodOptional<z.ZodEnum<["disable", "require", "verify-full"]>>;
    max_open_conns: z.ZodOptional<z.ZodNumber>;
    max_idle_conns: z.ZodOptional<z.ZodNumber>;
    conn_max_lifetime: z.ZodOptional<z.ZodString>;
}, "strip", z.ZodTypeAny, {
    name: string;
    host: string;
    port: number;
    user: string;
    password: string;
    ssl_mode?: "disable" | "require" | "verify-full" | undefined;
    max_open_conns?: number | undefined;
    max_idle_conns?: number | undefined;
    conn_max_lifetime?: string | undefined;
}, {
    name: string;
    host: string;
    port: number;
    user: string;
    password: string;
    ssl_mode?: "disable" | "require" | "verify-full" | undefined;
    max_open_conns?: number | undefined;
    max_idle_conns?: number | undefined;
    conn_max_lifetime?: string | undefined;
}>>;
export declare const KafkaConfigSchema: z.ZodOptional<z.ZodObject<{
    brokers: z.ZodArray<z.ZodString, "many">;
    consumer_group: z.ZodString;
    security_protocol: z.ZodEnum<["PLAINTEXT", "SASL_SSL"]>;
    sasl: z.ZodOptional<z.ZodObject<{
        mechanism: z.ZodEnum<["SCRAM-SHA-512", "PLAIN"]>;
        username: z.ZodString;
        password: z.ZodString;
    }, "strip", z.ZodTypeAny, {
        password: string;
        mechanism: "SCRAM-SHA-512" | "PLAIN";
        username: string;
    }, {
        password: string;
        mechanism: "SCRAM-SHA-512" | "PLAIN";
        username: string;
    }>>;
    tls: z.ZodOptional<z.ZodObject<{
        ca_cert_path: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        ca_cert_path?: string | undefined;
    }, {
        ca_cert_path?: string | undefined;
    }>>;
    topics: z.ZodObject<{
        publish: z.ZodArray<z.ZodString, "many">;
        subscribe: z.ZodArray<z.ZodString, "many">;
    }, "strip", z.ZodTypeAny, {
        publish: string[];
        subscribe: string[];
    }, {
        publish: string[];
        subscribe: string[];
    }>;
}, "strip", z.ZodTypeAny, {
    brokers: string[];
    consumer_group: string;
    security_protocol: "PLAINTEXT" | "SASL_SSL";
    topics: {
        publish: string[];
        subscribe: string[];
    };
    sasl?: {
        password: string;
        mechanism: "SCRAM-SHA-512" | "PLAIN";
        username: string;
    } | undefined;
    tls?: {
        ca_cert_path?: string | undefined;
    } | undefined;
}, {
    brokers: string[];
    consumer_group: string;
    security_protocol: "PLAINTEXT" | "SASL_SSL";
    topics: {
        publish: string[];
        subscribe: string[];
    };
    sasl?: {
        password: string;
        mechanism: "SCRAM-SHA-512" | "PLAIN";
        username: string;
    } | undefined;
    tls?: {
        ca_cert_path?: string | undefined;
    } | undefined;
}>>;
export declare const RedisConfigSchema: z.ZodOptional<z.ZodObject<{
    host: z.ZodString;
    port: z.ZodNumber;
    password: z.ZodOptional<z.ZodString>;
    db: z.ZodOptional<z.ZodNumber>;
    pool_size: z.ZodOptional<z.ZodNumber>;
}, "strip", z.ZodTypeAny, {
    host: string;
    port: number;
    password?: string | undefined;
    db?: number | undefined;
    pool_size?: number | undefined;
}, {
    host: string;
    port: number;
    password?: string | undefined;
    db?: number | undefined;
    pool_size?: number | undefined;
}>>;
export declare const ObservabilityConfigSchema: z.ZodObject<{
    log: z.ZodObject<{
        level: z.ZodEnum<["debug", "info", "warn", "error"]>;
        format: z.ZodEnum<["json", "text"]>;
    }, "strip", z.ZodTypeAny, {
        level: "debug" | "info" | "warn" | "error";
        format: "json" | "text";
    }, {
        level: "debug" | "info" | "warn" | "error";
        format: "json" | "text";
    }>;
    trace: z.ZodObject<{
        enabled: z.ZodBoolean;
        endpoint: z.ZodOptional<z.ZodString>;
        sample_rate: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        enabled: boolean;
        endpoint?: string | undefined;
        sample_rate?: number | undefined;
    }, {
        enabled: boolean;
        endpoint?: string | undefined;
        sample_rate?: number | undefined;
    }>;
    metrics: z.ZodObject<{
        enabled: z.ZodBoolean;
        path: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        enabled: boolean;
        path?: string | undefined;
    }, {
        enabled: boolean;
        path?: string | undefined;
    }>;
}, "strip", z.ZodTypeAny, {
    log: {
        level: "debug" | "info" | "warn" | "error";
        format: "json" | "text";
    };
    trace: {
        enabled: boolean;
        endpoint?: string | undefined;
        sample_rate?: number | undefined;
    };
    metrics: {
        enabled: boolean;
        path?: string | undefined;
    };
}, {
    log: {
        level: "debug" | "info" | "warn" | "error";
        format: "json" | "text";
    };
    trace: {
        enabled: boolean;
        endpoint?: string | undefined;
        sample_rate?: number | undefined;
    };
    metrics: {
        enabled: boolean;
        path?: string | undefined;
    };
}>;
export declare const AuthConfigSchema: z.ZodObject<{
    jwt: z.ZodObject<{
        issuer: z.ZodString;
        audience: z.ZodString;
        public_key_path: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        issuer: string;
        audience: string;
        public_key_path?: string | undefined;
    }, {
        issuer: string;
        audience: string;
        public_key_path?: string | undefined;
    }>;
    oidc: z.ZodOptional<z.ZodObject<{
        discovery_url: z.ZodString;
        client_id: z.ZodString;
        client_secret: z.ZodOptional<z.ZodString>;
        redirect_uri: z.ZodString;
        scopes: z.ZodArray<z.ZodString, "many">;
        jwks_uri: z.ZodString;
        jwks_cache_ttl: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        discovery_url: string;
        client_id: string;
        redirect_uri: string;
        scopes: string[];
        jwks_uri: string;
        client_secret?: string | undefined;
        jwks_cache_ttl?: string | undefined;
    }, {
        discovery_url: string;
        client_id: string;
        redirect_uri: string;
        scopes: string[];
        jwks_uri: string;
        client_secret?: string | undefined;
        jwks_cache_ttl?: string | undefined;
    }>>;
}, "strip", z.ZodTypeAny, {
    jwt: {
        issuer: string;
        audience: string;
        public_key_path?: string | undefined;
    };
    oidc?: {
        discovery_url: string;
        client_id: string;
        redirect_uri: string;
        scopes: string[];
        jwks_uri: string;
        client_secret?: string | undefined;
        jwks_cache_ttl?: string | undefined;
    } | undefined;
}, {
    jwt: {
        issuer: string;
        audience: string;
        public_key_path?: string | undefined;
    };
    oidc?: {
        discovery_url: string;
        client_id: string;
        redirect_uri: string;
        scopes: string[];
        jwks_uri: string;
        client_secret?: string | undefined;
        jwks_cache_ttl?: string | undefined;
    } | undefined;
}>;
export declare const ConfigSchema: z.ZodObject<{
    app: z.ZodObject<{
        name: z.ZodString;
        version: z.ZodString;
        tier: z.ZodEnum<["system", "business", "service"]>;
        environment: z.ZodEnum<["dev", "staging", "prod"]>;
    }, "strip", z.ZodTypeAny, {
        name: string;
        version: string;
        tier: "system" | "business" | "service";
        environment: "dev" | "staging" | "prod";
    }, {
        name: string;
        version: string;
        tier: "system" | "business" | "service";
        environment: "dev" | "staging" | "prod";
    }>;
    server: z.ZodObject<{
        host: z.ZodString;
        port: z.ZodNumber;
        read_timeout: z.ZodOptional<z.ZodString>;
        write_timeout: z.ZodOptional<z.ZodString>;
        shutdown_timeout: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        host: string;
        port: number;
        read_timeout?: string | undefined;
        write_timeout?: string | undefined;
        shutdown_timeout?: string | undefined;
    }, {
        host: string;
        port: number;
        read_timeout?: string | undefined;
        write_timeout?: string | undefined;
        shutdown_timeout?: string | undefined;
    }>;
    grpc: z.ZodOptional<z.ZodObject<{
        port: z.ZodNumber;
        max_recv_msg_size: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        port: number;
        max_recv_msg_size?: number | undefined;
    }, {
        port: number;
        max_recv_msg_size?: number | undefined;
    }>>;
    database: z.ZodOptional<z.ZodObject<{
        host: z.ZodString;
        port: z.ZodNumber;
        name: z.ZodString;
        user: z.ZodString;
        password: z.ZodString;
        ssl_mode: z.ZodOptional<z.ZodEnum<["disable", "require", "verify-full"]>>;
        max_open_conns: z.ZodOptional<z.ZodNumber>;
        max_idle_conns: z.ZodOptional<z.ZodNumber>;
        conn_max_lifetime: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        name: string;
        host: string;
        port: number;
        user: string;
        password: string;
        ssl_mode?: "disable" | "require" | "verify-full" | undefined;
        max_open_conns?: number | undefined;
        max_idle_conns?: number | undefined;
        conn_max_lifetime?: string | undefined;
    }, {
        name: string;
        host: string;
        port: number;
        user: string;
        password: string;
        ssl_mode?: "disable" | "require" | "verify-full" | undefined;
        max_open_conns?: number | undefined;
        max_idle_conns?: number | undefined;
        conn_max_lifetime?: string | undefined;
    }>>;
    kafka: z.ZodOptional<z.ZodObject<{
        brokers: z.ZodArray<z.ZodString, "many">;
        consumer_group: z.ZodString;
        security_protocol: z.ZodEnum<["PLAINTEXT", "SASL_SSL"]>;
        sasl: z.ZodOptional<z.ZodObject<{
            mechanism: z.ZodEnum<["SCRAM-SHA-512", "PLAIN"]>;
            username: z.ZodString;
            password: z.ZodString;
        }, "strip", z.ZodTypeAny, {
            password: string;
            mechanism: "SCRAM-SHA-512" | "PLAIN";
            username: string;
        }, {
            password: string;
            mechanism: "SCRAM-SHA-512" | "PLAIN";
            username: string;
        }>>;
        tls: z.ZodOptional<z.ZodObject<{
            ca_cert_path: z.ZodOptional<z.ZodString>;
        }, "strip", z.ZodTypeAny, {
            ca_cert_path?: string | undefined;
        }, {
            ca_cert_path?: string | undefined;
        }>>;
        topics: z.ZodObject<{
            publish: z.ZodArray<z.ZodString, "many">;
            subscribe: z.ZodArray<z.ZodString, "many">;
        }, "strip", z.ZodTypeAny, {
            publish: string[];
            subscribe: string[];
        }, {
            publish: string[];
            subscribe: string[];
        }>;
    }, "strip", z.ZodTypeAny, {
        brokers: string[];
        consumer_group: string;
        security_protocol: "PLAINTEXT" | "SASL_SSL";
        topics: {
            publish: string[];
            subscribe: string[];
        };
        sasl?: {
            password: string;
            mechanism: "SCRAM-SHA-512" | "PLAIN";
            username: string;
        } | undefined;
        tls?: {
            ca_cert_path?: string | undefined;
        } | undefined;
    }, {
        brokers: string[];
        consumer_group: string;
        security_protocol: "PLAINTEXT" | "SASL_SSL";
        topics: {
            publish: string[];
            subscribe: string[];
        };
        sasl?: {
            password: string;
            mechanism: "SCRAM-SHA-512" | "PLAIN";
            username: string;
        } | undefined;
        tls?: {
            ca_cert_path?: string | undefined;
        } | undefined;
    }>>;
    redis: z.ZodOptional<z.ZodObject<{
        host: z.ZodString;
        port: z.ZodNumber;
        password: z.ZodOptional<z.ZodString>;
        db: z.ZodOptional<z.ZodNumber>;
        pool_size: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    }, {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    }>>;
    redis_session: z.ZodOptional<z.ZodObject<{
        host: z.ZodString;
        port: z.ZodNumber;
        password: z.ZodOptional<z.ZodString>;
        db: z.ZodOptional<z.ZodNumber>;
        pool_size: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    }, {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    }>>;
    observability: z.ZodObject<{
        log: z.ZodObject<{
            level: z.ZodEnum<["debug", "info", "warn", "error"]>;
            format: z.ZodEnum<["json", "text"]>;
        }, "strip", z.ZodTypeAny, {
            level: "debug" | "info" | "warn" | "error";
            format: "json" | "text";
        }, {
            level: "debug" | "info" | "warn" | "error";
            format: "json" | "text";
        }>;
        trace: z.ZodObject<{
            enabled: z.ZodBoolean;
            endpoint: z.ZodOptional<z.ZodString>;
            sample_rate: z.ZodOptional<z.ZodNumber>;
        }, "strip", z.ZodTypeAny, {
            enabled: boolean;
            endpoint?: string | undefined;
            sample_rate?: number | undefined;
        }, {
            enabled: boolean;
            endpoint?: string | undefined;
            sample_rate?: number | undefined;
        }>;
        metrics: z.ZodObject<{
            enabled: z.ZodBoolean;
            path: z.ZodOptional<z.ZodString>;
        }, "strip", z.ZodTypeAny, {
            enabled: boolean;
            path?: string | undefined;
        }, {
            enabled: boolean;
            path?: string | undefined;
        }>;
    }, "strip", z.ZodTypeAny, {
        log: {
            level: "debug" | "info" | "warn" | "error";
            format: "json" | "text";
        };
        trace: {
            enabled: boolean;
            endpoint?: string | undefined;
            sample_rate?: number | undefined;
        };
        metrics: {
            enabled: boolean;
            path?: string | undefined;
        };
    }, {
        log: {
            level: "debug" | "info" | "warn" | "error";
            format: "json" | "text";
        };
        trace: {
            enabled: boolean;
            endpoint?: string | undefined;
            sample_rate?: number | undefined;
        };
        metrics: {
            enabled: boolean;
            path?: string | undefined;
        };
    }>;
    auth: z.ZodObject<{
        jwt: z.ZodObject<{
            issuer: z.ZodString;
            audience: z.ZodString;
            public_key_path: z.ZodOptional<z.ZodString>;
        }, "strip", z.ZodTypeAny, {
            issuer: string;
            audience: string;
            public_key_path?: string | undefined;
        }, {
            issuer: string;
            audience: string;
            public_key_path?: string | undefined;
        }>;
        oidc: z.ZodOptional<z.ZodObject<{
            discovery_url: z.ZodString;
            client_id: z.ZodString;
            client_secret: z.ZodOptional<z.ZodString>;
            redirect_uri: z.ZodString;
            scopes: z.ZodArray<z.ZodString, "many">;
            jwks_uri: z.ZodString;
            jwks_cache_ttl: z.ZodOptional<z.ZodString>;
        }, "strip", z.ZodTypeAny, {
            discovery_url: string;
            client_id: string;
            redirect_uri: string;
            scopes: string[];
            jwks_uri: string;
            client_secret?: string | undefined;
            jwks_cache_ttl?: string | undefined;
        }, {
            discovery_url: string;
            client_id: string;
            redirect_uri: string;
            scopes: string[];
            jwks_uri: string;
            client_secret?: string | undefined;
            jwks_cache_ttl?: string | undefined;
        }>>;
    }, "strip", z.ZodTypeAny, {
        jwt: {
            issuer: string;
            audience: string;
            public_key_path?: string | undefined;
        };
        oidc?: {
            discovery_url: string;
            client_id: string;
            redirect_uri: string;
            scopes: string[];
            jwks_uri: string;
            client_secret?: string | undefined;
            jwks_cache_ttl?: string | undefined;
        } | undefined;
    }, {
        jwt: {
            issuer: string;
            audience: string;
            public_key_path?: string | undefined;
        };
        oidc?: {
            discovery_url: string;
            client_id: string;
            redirect_uri: string;
            scopes: string[];
            jwks_uri: string;
            client_secret?: string | undefined;
            jwks_cache_ttl?: string | undefined;
        } | undefined;
    }>;
}, "strip", z.ZodTypeAny, {
    app: {
        name: string;
        version: string;
        tier: "system" | "business" | "service";
        environment: "dev" | "staging" | "prod";
    };
    server: {
        host: string;
        port: number;
        read_timeout?: string | undefined;
        write_timeout?: string | undefined;
        shutdown_timeout?: string | undefined;
    };
    observability: {
        log: {
            level: "debug" | "info" | "warn" | "error";
            format: "json" | "text";
        };
        trace: {
            enabled: boolean;
            endpoint?: string | undefined;
            sample_rate?: number | undefined;
        };
        metrics: {
            enabled: boolean;
            path?: string | undefined;
        };
    };
    auth: {
        jwt: {
            issuer: string;
            audience: string;
            public_key_path?: string | undefined;
        };
        oidc?: {
            discovery_url: string;
            client_id: string;
            redirect_uri: string;
            scopes: string[];
            jwks_uri: string;
            client_secret?: string | undefined;
            jwks_cache_ttl?: string | undefined;
        } | undefined;
    };
    grpc?: {
        port: number;
        max_recv_msg_size?: number | undefined;
    } | undefined;
    database?: {
        name: string;
        host: string;
        port: number;
        user: string;
        password: string;
        ssl_mode?: "disable" | "require" | "verify-full" | undefined;
        max_open_conns?: number | undefined;
        max_idle_conns?: number | undefined;
        conn_max_lifetime?: string | undefined;
    } | undefined;
    kafka?: {
        brokers: string[];
        consumer_group: string;
        security_protocol: "PLAINTEXT" | "SASL_SSL";
        topics: {
            publish: string[];
            subscribe: string[];
        };
        sasl?: {
            password: string;
            mechanism: "SCRAM-SHA-512" | "PLAIN";
            username: string;
        } | undefined;
        tls?: {
            ca_cert_path?: string | undefined;
        } | undefined;
    } | undefined;
    redis?: {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    } | undefined;
    redis_session?: {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    } | undefined;
}, {
    app: {
        name: string;
        version: string;
        tier: "system" | "business" | "service";
        environment: "dev" | "staging" | "prod";
    };
    server: {
        host: string;
        port: number;
        read_timeout?: string | undefined;
        write_timeout?: string | undefined;
        shutdown_timeout?: string | undefined;
    };
    observability: {
        log: {
            level: "debug" | "info" | "warn" | "error";
            format: "json" | "text";
        };
        trace: {
            enabled: boolean;
            endpoint?: string | undefined;
            sample_rate?: number | undefined;
        };
        metrics: {
            enabled: boolean;
            path?: string | undefined;
        };
    };
    auth: {
        jwt: {
            issuer: string;
            audience: string;
            public_key_path?: string | undefined;
        };
        oidc?: {
            discovery_url: string;
            client_id: string;
            redirect_uri: string;
            scopes: string[];
            jwks_uri: string;
            client_secret?: string | undefined;
            jwks_cache_ttl?: string | undefined;
        } | undefined;
    };
    grpc?: {
        port: number;
        max_recv_msg_size?: number | undefined;
    } | undefined;
    database?: {
        name: string;
        host: string;
        port: number;
        user: string;
        password: string;
        ssl_mode?: "disable" | "require" | "verify-full" | undefined;
        max_open_conns?: number | undefined;
        max_idle_conns?: number | undefined;
        conn_max_lifetime?: string | undefined;
    } | undefined;
    kafka?: {
        brokers: string[];
        consumer_group: string;
        security_protocol: "PLAINTEXT" | "SASL_SSL";
        topics: {
            publish: string[];
            subscribe: string[];
        };
        sasl?: {
            password: string;
            mechanism: "SCRAM-SHA-512" | "PLAIN";
            username: string;
        } | undefined;
        tls?: {
            ca_cert_path?: string | undefined;
        } | undefined;
    } | undefined;
    redis?: {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    } | undefined;
    redis_session?: {
        host: string;
        port: number;
        password?: string | undefined;
        db?: number | undefined;
        pool_size?: number | undefined;
    } | undefined;
}>;
export type Config = z.infer<typeof ConfigSchema>;
export type AppConfig = z.infer<typeof AppConfigSchema>;
export type ServerConfig = z.infer<typeof ServerConfigSchema>;
export type DatabaseConfig = z.infer<typeof DatabaseConfigSchema>;
export type AuthConfig = z.infer<typeof AuthConfigSchema>;
//# sourceMappingURL=config.d.ts.map