# Docker テンプレートファイル (v0.2.3)

← [テンプレート設計書](./)

---

## 追加ファイル

| テンプレート | ファイル | 管理区分 |
|-------------|---------|---------|
| backend-rust | `Dockerfile.tera`, `.dockerignore`, `docker-compose.yml.tera` | Dockerfile: protected, compose: unmanaged |
| backend-go | `Dockerfile.tera`, `.dockerignore`, `docker-compose.yml.tera` | Dockerfile: protected, compose: unmanaged |
| backend-csharp | `Dockerfile.tera`, `.dockerignore`, `docker-compose.yml.tera` | Dockerfile: protected, compose: unmanaged |
| backend-python | `Dockerfile.tera`, `.dockerignore`, `docker-compose.yml.tera` | Dockerfile: protected, compose: unmanaged |
| frontend-react | `Dockerfile.tera`, `.dockerignore`, `docker-compose.yml.tera`, `deploy/nginx.conf.tera` | Dockerfile/nginx: protected, compose: unmanaged |

## Dockerfile 共通仕様

- マルチステージビルド（build + runtime）
- `ARG HTTP_PROXY` / `HTTPS_PROXY` / `NO_PROXY` 宣言
- 非 root 実行（appuser）
- HEALTHCHECK 命令（`curl -f http://localhost:8080/healthz`）
- ENTRYPOINT + CMD 分離（CMD: `--env default --config /app/config`）
- `{% if with_grpc %}` で 50051 ポート条件分岐

## docker-compose.yml 共通仕様

- app サービス: ビルド、ポート 8080、config ボリューム、command `--env dev`
- `{% if with_db %}`: PostgreSQL 16 サービス（healthcheck, secrets, volumes）
- `{% if with_cache %}`: Redis 7 サービス（healthcheck）
- `depends_on: condition: service_healthy` で起動順序保証
- secrets/db_password.txt によるシークレット管理
