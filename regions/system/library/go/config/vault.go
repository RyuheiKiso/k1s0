package config

// MergeVaultSecrets は Vault から取得したシークレットで設定値を上書きする。
func (c *Config) MergeVaultSecrets(secrets map[string]string) {
	if v, ok := secrets["database.password"]; ok && c.Database != nil {
		c.Database.Password = v
	}
	if v, ok := secrets["redis.password"]; ok && c.Redis != nil {
		c.Redis.Password = v
	}
	if v, ok := secrets["kafka.sasl.username"]; ok && c.Kafka != nil && c.Kafka.SASL != nil {
		c.Kafka.SASL.Username = v
	}
	if v, ok := secrets["kafka.sasl.password"]; ok && c.Kafka != nil && c.Kafka.SASL != nil {
		c.Kafka.SASL.Password = v
	}
	if v, ok := secrets["redis_session.password"]; ok && c.RedisSession != nil {
		c.RedisSession.Password = v
	}
	if v, ok := secrets["auth.oidc.client_secret"]; ok && c.Auth.OIDC != nil {
		c.Auth.OIDC.ClientSecret = v
	}
}
