import pino from 'pino';
import { trace } from '@opentelemetry/api';
import { config } from './config.js';

export const logger = pino({
  level: config.logLevel,
  formatters: {
    level: (label) => ({ level: label }),
    log: (obj) => {
      const span = trace.getActiveSpan();
      const ctx  = span?.spanContext();
      if (ctx?.traceId) return { ...obj, trace_id: ctx.traceId, span_id: ctx.spanId };
      return obj;
    },
  },
  base: { service: config.serviceName },
  timestamp: pino.stdTimeFunctions.isoTime,
});
