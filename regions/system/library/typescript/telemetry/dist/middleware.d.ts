import type { IncomingMessage, ServerResponse } from 'node:http';
import pino from 'pino';
/**
 * Express/Fastify 互換の HTTP ミドルウェア。
 * リクエストごとに構造化ログ（メソッド・パス・ステータスコード・レイテンシ）を出力する。
 */
export declare function httpMiddleware(logger: pino.Logger): (req: IncomingMessage, res: ServerResponse, next: () => void) => void;
