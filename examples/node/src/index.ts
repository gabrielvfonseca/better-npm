import express from 'express';

const app = express();
const port = process.env.PORT ?? 3000;

app.get('/', (_req, res) => {
  res.json({ message: 'Hello from Dash + Node!' });
});

app.get('/health', (_req, res) => {
  res.json({ status: 'ok', service: 'example-node' });
});

app.listen(port, () => {
  console.log(`Dash example server running on http://localhost:${port}`);
});
