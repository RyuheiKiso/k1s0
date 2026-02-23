class MigrationConfig {
  final String migrationsDir;
  final String databaseUrl;
  final String tableName;

  const MigrationConfig({
    required this.migrationsDir,
    required this.databaseUrl,
    this.tableName = '_migrations',
  });
}
