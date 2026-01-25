/**
 * APIリクエストのテレメトリー情報
 */
export interface RequestTelemetry {
  /** トレースID（trace_idヘッダー/レスポンスから取得） */
  traceId: string;
  /** スパンID（span_idヘッダー/生成） */
  spanId: string;
  /** 親スパンID（存在する場合） */
  parentSpanId?: string;
  /** リクエスト開始時刻 */
  startTime: number;
  /** リクエスト終了時刻 */
  endTime?: number;
  /** HTTPメソッド */
  method: string;
  /** リクエストURL */
  url: string;
  /** HTTPステータスコード */
  statusCode?: number;
  /** エラーの場合のerror_code */
  errorCode?: string;
}

/**
 * テレメトリーイベント種別
 */
export type TelemetryEventType =
  | 'request_start'
  | 'request_end'
  | 'request_error';

/**
 * テレメトリーイベント
 */
export interface TelemetryEvent {
  type: TelemetryEventType;
  telemetry: RequestTelemetry;
  error?: Error;
}

/**
 * テレメトリーリスナー
 */
export type TelemetryListener = (event: TelemetryEvent) => void;
