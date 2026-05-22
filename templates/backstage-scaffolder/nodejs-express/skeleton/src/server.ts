import './otel.js';                      // start OTel before any other module
import express, { Request, Response } from 'express';
import helmet from 'helmet';
import pinoHttp from 'pino-http';
import { logger } from './logger.js';
import { register, httpRequests, httpLatency } from './metrics.js';
import { config } from './config.js';

export function buildApp() {
  const app = express();
  app.disable('x-powered-by');
  app.use(helmet());
  app.use(express.json({ limit: '1mb' }));
  app.use(pinoHttp({ logger }));

  app.use((req, res, next) => {
    const start = process.hrtime.bigint();
    res.on('finish', () => {
      const ns = Number(process.hrtime.bigint() - start);
      const route = (req as any).route?.path ?? req.path;
      httpRequests.labels(req.method, route, String(res.statusCode)).inc();
      httpLatency.labels(req.method, route).observe(ns / 1e9);
    });
    next();
  });

  app.get('/healthz', (_req: Request, res: Response) => res.json({ ok: true }));
  app.get('/ready',   (_req: Request, res: Response) => res.json({ ok: true, deps: { db: 'ok' } }));
  app.get('/metrics', async (_req: Request, res: Response) => {
    res.setHeader('Content-Type', register.contentType);
    res.send(await register.metrics());
  });

  app.get('/api/v1/items', (_req: Request, res: Response) => res.json({ items: [{ id: 1, name: 'example' }] }));

  return app;
}

const app = buildApp();
const server = app.listen(config.port, () => logger.info({ port: config.port, service: config.serviceName }, 'listening'));

// Graceful shutdown — drain SIGTERM within DRAIN_TIMEOUT_MS.
const drain = (signal: string) => {
  logger.info({ signal }, 'draining');
  const t = setTimeout(() => { logger.warn('forced exit'); process.exit(1); }, config.drainTimeoutMs);
  server.close(() => { clearTimeout(t); logger.info('bye'); process.exit(0); });
};
process.on('SIGTERM', () => drain('SIGTERM'));
process.on('SIGINT',  () => drain('SIGINT'));
