pub struct NotificationDomainService;

impl NotificationDomainService {
    pub fn is_supported_channel_type(channel_type: &str) -> bool {
        matches!(channel_type, "email" | "slack" | "webhook" | "sms" | "push")
    }

    pub fn validate_channel_type(channel_type: &str) -> Result<(), String> {
        if Self::is_supported_channel_type(channel_type) {
            Ok(())
        } else {
            Err(format!(
                "invalid channel_type: {} (allowed: email, slack, webhook, sms, push)",
                channel_type
            ))
        }
    }

    pub fn validate_template_body(template: &str) -> Result<(), String> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("template", template)
            .map_err(|e| format!("invalid template syntax: {}", e))
    }

    pub fn is_retryable_status(status: &str) -> bool {
        status != "sent"
    }

    /// CRIT-02 SSRF対策: Webhook URL を検証してサーバーサイドリクエストフォージェリを防止する。
    /// スキーム制限（http/https のみ）、プライベートIPアドレス拒否、クラスタ内部ホスト名拒否を行う。
    pub fn validate_webhook_url(url: &str) -> Result<(), String> {
        // URL 文字列をパースして構造体に変換する
        let parsed = url::Url::parse(url).map_err(|_| "URLの形式が不正です".to_string())?;

        // スキーム制限: http または https のみ許可し、file/ftp/data 等の危険スキームを拒否する
        match parsed.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(format!("許可されていないスキームです: {}", scheme));
            }
        }

        // ホスト名の取得: ホスト名が存在しない URL は拒否する
        let host = parsed
            .host_str()
            .ok_or_else(|| "ホスト名が必要です".to_string())?;

        // IPアドレスの場合はループバック・未指定・プライベートIP範囲を拒否する
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            if ip.is_loopback() || ip.is_unspecified() {
                return Err(
                    "ループバック・未指定IPアドレスへのWebhookは禁止されています".to_string(),
                );
            }
            if is_private_ip(&ip) {
                return Err("プライベートIPアドレスへのWebhookは禁止されています".to_string());
            }
        }

        // クラスタ内部ホスト名の拒否: Kubernetes サービスドメインや localhost へのアクセスを防ぐ
        if host.ends_with(".svc.cluster.local")
            || host.ends_with(".cluster.local")
            || host == "localhost"
            || host == "kubernetes"
            || host == "kubernetes.default"
        {
            return Err("クラスタ内部エンドポイントへのWebhookは禁止されています".to_string());
        }

        Ok(())
    }
}

/// プライベートIPアドレス範囲チェック（RFC1918 および リンクローカル範囲）
/// SSRF 攻撃で内部ネットワークへアクセスされることを防ぐために使用する
fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 10.0.0.0/8 (RFC1918 クラスA プライベートアドレス)
            octets[0] == 10
                // 172.16.0.0/12 (RFC1918 クラスB プライベートアドレス)
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                // 192.168.0.0/16 (RFC1918 クラスC プライベートアドレス)
                || (octets[0] == 192 && octets[1] == 168)
                // 169.254.0.0/16 (リンクローカルアドレス: AWS メタデータエンドポイント 169.254.169.254 等を含む)
                || (octets[0] == 169 && octets[1] == 254)
        }
        std::net::IpAddr::V6(ipv6) => {
            let segments = ipv6.segments();
            // fc00::/7 (IPv6 ユニークローカルアドレス)
            (segments[0] & 0xfe00) == 0xfc00
                // fe80::/10 (IPv6 リンクローカルアドレス)
                || (segments[0] & 0xffc0) == 0xfe80
        }
    }
}
