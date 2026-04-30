# admin app の Dockerfile。
#
# build context: docker build -f apps/admin/Dockerfile -t ghcr.io/k1s0/t3-admin:dev .

FROM node:20-alpine AS builder
WORKDIR /workspace
RUN npm install -g pnpm@9
COPY pnpm-workspace.yaml pnpm-lock.yaml* package.json tsconfig.base.json ./
COPY apps/admin/package.json apps/admin/
COPY packages/ packages/
COPY tools/ tools/
RUN pnpm install --filter '@k1s0/admin...'

COPY apps/admin/ apps/admin/
RUN pnpm --filter '@k1s0/admin' build

FROM nginx:alpine AS runtime
COPY --from=builder /workspace/apps/admin/dist /usr/share/nginx/html
COPY apps/admin/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 8080
