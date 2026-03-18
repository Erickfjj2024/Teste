import dotenv from 'dotenv';
import express from 'express';
import cors from 'cors';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { lookRouter } from './routes/lookRoutes.js';

dotenv.config();

const app = express();
const port = Number(process.env.PORT || 4000);
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const frontendDistPath = path.resolve(__dirname, '../../frontend/dist');

app.use(cors({ origin: process.env.CORS_ORIGIN || '*' }));
app.use(express.json({ limit: '1mb' }));
app.use('/api', lookRouter);

try {
  if (fs.existsSync(frontendDistPath)) {
    app.use(express.static(frontendDistPath));
    app.get('*', (req, res, next) => {
      if (req.path.startsWith('/api')) return next();
      return res.sendFile(path.join(frontendDistPath, 'index.html'));
    });
    console.log('[backend/src/server.js][static] Frontend build found. Serving frontend/dist in local deploy mode.');
  } else {
    console.log('[backend/src/server.js][static] frontend/dist not found. Run `npm run build` for local deploy.');
  }
} catch (error) {
  console.log('[backend/src/server.js][static] Failed to configure static hosting:', error?.message);
}

app.listen(port, () => {
  console.log(`[backend/src/server.js][listen] API running at http://localhost:${port}`);
});
