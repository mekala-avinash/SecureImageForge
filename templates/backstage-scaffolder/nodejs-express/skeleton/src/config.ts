import { z } from 'zod';

const Schema = z.object({
  PORT: z.string().default('8080').transform(Number),
  LOG_LEVEL: z.enum(['fatal', 'error', 'warn', 'info', 'debug', 'trace']).default('info'),
  OTEL_SERVICE_NAME: z.string().default('{{service_name}}'),
  DRAIN_TIMEOUT_MS: z.string().default('20000').transform(Number),
});

const parsed = Schema.safeParse(process.env);
if (!parsed.success) {
  // eslint-disable-next-line no-console
  console.error('Invalid configuration', parsed.error.flatten().fieldErrors);
  process.exit(2);
}

export const config = {
  port: parsed.data.PORT,
  logLevel: parsed.data.LOG_LEVEL,
  serviceName: parsed.data.OTEL_SERVICE_NAME,
  drainTimeoutMs: parsed.data.DRAIN_TIMEOUT_MS,
};
