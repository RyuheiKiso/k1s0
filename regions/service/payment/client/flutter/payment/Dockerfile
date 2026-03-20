# マルチステージビルド: Flutter Webアプリのビルドとnginxデプロイ

# ステージ1: Flutterビルド環境でWebアプリをビルドする
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
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
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
FROM nginx:alpine

# ビルド成果物をnginxのドキュメントルートにコピーする
COPY --from=builder /app/build/web /usr/share/nginx/html

# SPA対応のnginx設定をコピーする
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
