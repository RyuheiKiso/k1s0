# マルチステージビルド: Flutter Webアプリのビルドとnginxデプロイ

# ステージ1: Flutterビルド環境でWebアプリをビルドする
FROM ghcr.io/cirruslabs/flutter:3.24.0 AS builder
WORKDIR /app

# 依存関係のキャッシュ最適化のため、pubspec.yamlを先にコピーする
COPY pubspec.yaml ./
RUN flutter pub get

# ソースコードをコピーしてリリースビルドを実行する
COPY . .
RUN flutter build web --release

# ステージ2: nginx上でビルド成果物を配信する
FROM nginx:alpine

# ビルド成果物をnginxのドキュメントルートにコピーする
COPY --from=builder /app/build/web /usr/share/nginx/html

# SPA対応のnginx設定をコピーする
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
