export {
  CorrelationId,
  TraceId,
  CorrelationContext,
  newCorrelationContext,
} from './types.js';

export {
  HEADER_CORRELATION_ID,
  HEADER_TRACE_ID,
  toHeaders,
  fromHeaders,
} from './headers.js';
