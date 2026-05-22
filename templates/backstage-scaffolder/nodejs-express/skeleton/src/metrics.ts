import { Counter, Histogram, Registry, collectDefaultMetrics } from 'prom-client';

export const register = new Registry();
collectDefaultMetrics({ register });

export const httpRequests = new Counter({
  name: 'http_requests_total',
  help: 'Total HTTP requests.',
  labelNames: ['method', 'path', 'code'] as const,
  registers: [register],
});
export const httpLatency = new Histogram({
  name: 'http_request_duration_seconds',
  help: 'HTTP request duration in seconds.',
  labelNames: ['method', 'path'] as const,
  buckets: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
  registers: [register],
});
