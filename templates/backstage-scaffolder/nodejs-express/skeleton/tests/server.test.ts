import { describe, expect, it } from 'vitest';
import request from 'supertest';
import { buildApp } from '../src/server.js';

const app = buildApp();

describe('paved-road endpoints', () => {
  it('healthz returns 200', async () => {
    const r = await request(app).get('/healthz');
    expect(r.status).toBe(200);
    expect(r.body.ok).toBe(true);
  });
  it('ready returns 200', async () => {
    const r = await request(app).get('/ready');
    expect(r.status).toBe(200);
  });
  it('metrics exposes prom format', async () => {
    const r = await request(app).get('/metrics');
    expect(r.status).toBe(200);
    expect(r.text).toContain('http_requests_total');
  });
});
