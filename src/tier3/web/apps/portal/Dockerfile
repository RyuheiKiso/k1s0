# portal app の Dockerfile（multi-stage）。
#
# docs 正典:
#   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_web_pnpm_workspace配置.md
#
# build context: docker build -f apps/portal/Dockerfile -t ghcr.io/k1s0/t3-portal:dev .
#   （src/tier3/web/ をルートとして実行する前提）

# stage 1: build。
FROM node:20-alpine AS builder
WORKDIR /workspace
RUN npm install -g pnpm@9
# workspace 全体をコピー（filter で portal だけ install / build する）。
COPY pnpm-workspace.yaml pnpm-lock.yaml* package.json ./
COPY apps/portal/package.json apps/portal/
COPY packages/ packages/
COPY tools/ tools/
RUN pnpm install --frozen-lockfile --filter '@k1s0/portal...'

COPY apps/portal/ apps/portal/
RUN pnpm --filter '@k1s0/portal' build

# stage 2: runtime（nginx で静的配信、Node ランタイム不要）。
FROM nginx:alpine AS runtime
COPY --from=builder /workspace/apps/portal/dist /usr/share/nginx/html
COPY apps/portal/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 8080
