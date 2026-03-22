# ビルドステージ: Node.jsでReactアプリをビルド
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
FROM node:22-alpine AS builder
WORKDIR /app
COPY package.json ./
RUN npm install
COPY . .
RUN npm run build

# 配信ステージ: nginxで静的ファイルを配信
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
FROM nginx:alpine

# 非rootユーザーでnginxを実行するためのセキュリティ設定
# コンテナ内でroot権限を使わないことで、攻撃者による権限昇格リスクを低減する
RUN sed -i 's/listen\s*80;/listen 8080;/g' /etc/nginx/conf.d/default.conf && \
    sed -i '/user\s*nginx;/d' /etc/nginx/nginx.conf && \
    chown -R nginx:nginx /var/cache/nginx /var/log/nginx /etc/nginx/conf.d

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

# nginxユーザーに切り替えて非特権ポートで配信する
USER nginx
EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
