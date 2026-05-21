# companion_local_mock Dockerfile
# Bun ランタイムで companion mock WebSocket サーバーを起動する
# lib.rs プロトコル形式 {command, layer, entry, psk_hmac, hlc_timestamp} に準拠する

# Bun の公式 Alpine イメージをベースにする（軽量化のため alpine を使用する）
FROM oven/bun:1-alpine

# 作業ディレクトリを設定する
WORKDIR /app

# package.json と bun.lockb を先にコピーして依存関係をキャッシュする
COPY package.json ./

# ws ライブラリをインストールする（WebSocket プロトコル実装）
RUN bun install --frozen-lockfile 2>/dev/null || bun install

# サーバーソースをコピーする
COPY server.ts ./

# companion mock が待ち受けるポートを公開する（lib.rs のデフォルトポートと一致させる）
EXPOSE 9999

# Bun で TypeScript を直接実行する（コンパイル不要）
CMD ["bun", "run", "server.ts"]
