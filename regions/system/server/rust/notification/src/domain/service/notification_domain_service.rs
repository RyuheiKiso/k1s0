pub struct NotificationDomainService;

impl NotificationDomainService {
    #[must_use] 
    pub fn is_supported_channel_type(channel_type: &str) -> bool {
        matches!(channel_type, "email" | "slack" | "webhook" | "sms" | "push")
    }

    pub fn validate_channel_type(channel_type: &str) -> Result<(), String> {
        if Self::is_supported_channel_type(channel_type) {
            Ok(())
        } else {
            Err(format!(
                "invalid channel_type: {channel_type} (allowed: email, slack, webhook, sms, push)"
            ))
        }
    }

    pub fn validate_template_body(template: &str) -> Result<(), String> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("template", template)
            .map_err(|e| format!("invalid template syntax: {e}"))
    }

    #[must_use] 
    pub fn is_retryable_status(status: &str) -> bool {
        status != "sent"
    }

    /// CRIT-02 / HIGH-03 SSRF対策: Webhook URL を検証してサーバーサイドリクエストフォージェリを防止する。
    /// スキーム制限（http/https のみ）、プライベートIPアドレス拒否、クラスタ内部ホスト名拒否に加え、
    /// HIGH-03 DNS リバインド攻撃対策として DNS 解決後の IP アドレスも検証する。
    /// 攻撃者が DNS TTL を悪用して検証後にプライベート IP に切り替える攻撃を防ぐ。
    pub async fn validate_webhook_url(url: &str) -> Result<(), String> {
        // URL 文字列をパースして構造体に変換する
        let parsed = url::Url::parse(url).map_err(|_| "URLの形式が不正です".to_string())?;

        // スキーム制限: http または https のみ許可し、file/ftp/data 等の危険スキームを拒否する
        match parsed.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(format!("許可されていないスキームです: {scheme}"));
            }
        }

        // ホスト名の取得: ホスト名が存在しない URL は拒否する
        let host = parsed
            .host_str()
            .ok_or_else(|| "ホスト名が必要です".to_string())?;

        // IPアドレスリテラルの場合はループバック・未指定・プライベートIP範囲を即座に拒否する
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            if ip.is_loopback() || ip.is_unspecified() {
                return Err(
                    "ループバック・未指定IPアドレスへのWebhookは禁止されています".to_string(),
                );
            }
            if is_private_ip(&ip) {
                return Err("プライベートIPアドレスへのWebhookは禁止されています".to_string());
            }
            // IP リテラルの場合は DNS 解決不要なのでここで正常終了する
            return Ok(());
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

        // HIGH-03 DNS リバインド攻撃対策: ホスト名を DNS 解決し、解決後の全 IP アドレスを検証する
        // DNS 解決にはポート番号が必要なため :80 を付与してルックアップを行う
        let lookup_target = format!("{host}:80");
        let addrs = tokio::net::lookup_host(lookup_target)
            .await
            .map_err(|e| format!("DNS解決に失敗しました: {e}"))?;

        for addr in addrs {
            let ip = addr.ip();
            // 解決後 IP がループバック・未指定の場合は拒否する
            if ip.is_loopback() || ip.is_unspecified() {
                return Err(
                    "ループバック・未指定IPアドレスに解決されるホスト名へのWebhookは禁止されています"
                        .to_string(),
                );
            }
            // 解決後 IP がプライベートアドレス範囲の場合は DNS リバインド攻撃として拒否する
            if is_private_ip(&ip) {
                return Err(
                    "プライベートIPアドレスに解決されるホスト名へのWebhookは禁止されています"
                        .to_string(),
                );
            }
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// HIGH-03 DNS リバインド攻撃対策: 127.0.0.1 (ループバック IP リテラル) は拒否されることを確認する
    /// 実際の DNS 解決を避けるため IP リテラルで検証し、ループバック拒否ロジックをテストする
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_localhost_resolved() {
        let result = NotificationDomainService::validate_webhook_url("http://127.0.0.1/callback").await;
        assert!(result.is_err(), "127.0.0.1 は拒否されるべき");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("ループバック"),
            "エラーメッセージにループバックが含まれること: {msg}"
        );
    }

    /// HIGH-03 DNS リバインド攻撃対策: プライベート IP リテラル 192.168.1.1 は拒否されることを確認する
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_private_ip_literal() {
        let result = NotificationDomainService::validate_webhook_url("https://192.168.1.1/hook").await;
        assert!(result.is_err(), "プライベートIPリテラルは拒否されるべき");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("プライベートIPアドレス"),
            "エラーメッセージにプライベートIPアドレスが含まれること: {msg}"
        );
    }

    /// HIGH-03 DNS リバインド攻撃対策: リンクローカル IP リテラル 169.254.169.254 (AWS メタデータ) は拒否されることを確認する
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_link_local_ip() {
        let result = NotificationDomainService::validate_webhook_url("http://169.254.169.254/latest/meta-data").await;
        assert!(result.is_err(), "リンクローカル IP は拒否されるべき");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("プライベートIPアドレス"),
            "エラーメッセージにプライベートIPアドレスが含まれること: {msg}"
        );
    }

    /// CRIT-02 / HIGH-03: クラスタ内部ホスト名 (kubernetes.default) は DNS 解決前に拒否されることを確認する
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_cluster_hostname() {
        let result = NotificationDomainService::validate_webhook_url("http://kubernetes.default/api").await;
        assert!(result.is_err(), "クラスタ内部ホスト名は拒否されるべき");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("クラスタ内部"),
            "エラーメッセージにクラスタ内部が含まれること: {msg}"
        );
    }

    /// CRIT-02 / HIGH-03: file:// スキームは拒否されることを確認する
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_file_scheme() {
        let result = NotificationDomainService::validate_webhook_url("file:///etc/passwd").await;
        assert!(result.is_err(), "file:// スキームは拒否されるべき");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("許可されていないスキーム"),
            "エラーメッセージにスキームエラーが含まれること: {msg}"
        );
    }

    /// CRIT-02 / HIGH-03: 10.0.0.1 (RFC1918 クラスA) は拒否されることを確認する
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_rfc1918_class_a() {
        let result = NotificationDomainService::validate_webhook_url("http://10.0.0.1/hook").await;
        assert!(result.is_err(), "10.x.x.x アドレスは拒否されるべき");
    }

    /// CRIT-02 / HIGH-03: 172.16.0.1 (RFC1918 クラスB) は拒否されることを確認する
    #[tokio::test]
    async fn test_validate_webhook_url_rejects_rfc1918_class_b() {
        let result = NotificationDomainService::validate_webhook_url("http://172.16.0.1/hook").await;
        assert!(result.is_err(), "172.16.x.x アドレスは拒否されるべき");
    }
}
