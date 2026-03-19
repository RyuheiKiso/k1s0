# マルチステージビルド: Flutter Webアプリのビルドとnginxデプロイ

# ステージ1: Flutterビルド環境でWebアプリをビルドする
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull ghcr.io/cirruslabs/flutter:3.24.0 && docker inspect --format="{{index .RepoDigests 0}}" ghcr.io/cirruslabs/flutter:3.24.0
FROM ghcr.io/cirruslabs/flutter:3.24.0 AS builder
WORKDIR /app

# 依存関係のキャッシュ最適化のため、pubspec.yamlを先にコピーする
COPY pubspec.yaml ./
RUN flutter pub get

# ソースコードをコピーしてリリースビルドを実行する
COPY . .
RUN flutter build web --release

# ステージ2: nginx上でビルド成果物を配信する
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull nginx:alpine && docker inspect --format="{{index .RepoDigests 0}}" nginx:alpine
FROM nginx:alpine

# 非rootユーザーでnginxを実行するためのセキュリティ設定
# コンテナ内でroot権限を使わないことで、攻撃者による権限昇格リスクを低減する
RUN sed -i 's/listen\s*80;/listen 8080;/g' /etc/nginx/conf.d/default.conf && \
    sed -i '/user\s*nginx;/d' /etc/nginx/nginx.conf && \
    chown -R nginx:nginx /var/cache/nginx /var/log/nginx /etc/nginx/conf.d

# ビルド成果物をnginxのドキュメントルートにコピーする
COPY --from=builder /app/build/web /usr/share/nginx/html

# SPA対応のnginx設定をコピーする
COPY nginx.conf /etc/nginx/conf.d/default.conf

# nginxユーザーに切り替えて非特権ポートで配信する
USER nginx
EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
