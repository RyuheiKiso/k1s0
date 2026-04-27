# docs-site の Dockerfile。
#
# build context: docker build -f apps/docs-site/Dockerfile -t ghcr.io/k1s0/t3-docs:dev .

FROM node:20-alpine AS builder
WORKDIR /workspace
RUN npm install -g pnpm@9
COPY pnpm-workspace.yaml pnpm-lock.yaml* package.json ./
COPY apps/docs-site/package.json apps/docs-site/
COPY packages/ packages/
COPY tools/ tools/
RUN pnpm install --frozen-lockfile --filter '@k1s0/docs-site...'

COPY apps/docs-site/ apps/docs-site/
RUN pnpm --filter '@k1s0/docs-site' build

FROM nginx:alpine AS runtime
COPY --from=builder /workspace/apps/docs-site/dist /usr/share/nginx/html
COPY apps/docs-site/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 8080
