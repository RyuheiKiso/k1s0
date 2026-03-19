# ビルドステージ: Node.jsでReactアプリをビルド
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull node:22-alpine && docker inspect --format="{{index .RepoDigests 0}}" node:22-alpine
FROM node:22-alpine AS builder
WORKDIR /app
COPY package.json ./
RUN npm install
COPY . .
RUN npm run build

# 配信ステージ: nginxで静的ファイルを配信
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull nginx:alpine && docker inspect --format="{{index .RepoDigests 0}}" nginx:alpine
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
