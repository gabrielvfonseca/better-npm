import { Hono } from 'hono';

const app = new Hono();

app.get('/', (c) => c.json({ message: 'Hello from Dash + Hono!' }));

app.get('/health', (c) => c.json({ status: 'ok', service: 'example-hono', version: '0.0.1' }));

app.get('/api/:path', (c) => {
  const path = c.req.param('path');
  return c.json({ path, ts: Date.now() });
});

export default app;
