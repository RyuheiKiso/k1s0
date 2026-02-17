/**
 * Express/Fastify 互換の HTTP ミドルウェア。
 * リクエストごとに構造化ログ（メソッド・パス・ステータスコード・レイテンシ）を出力する。
 */
export function httpMiddleware(logger) {
    return (req, res, next) => {
        const start = Date.now();
        res.on('finish', () => {
            const duration = Date.now() - start;
            logger.info({
                method: req.method,
                path: req.url,
                status: res.statusCode,
                duration_ms: duration,
            }, 'Request completed');
        });
        next();
    };
}
