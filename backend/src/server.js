import dotenv from 'dotenv';
import express from 'express';
import cors from 'cors';
import { lookRouter } from './routes/lookRoutes.js';

dotenv.config();

const app = express();
const port = Number(process.env.PORT || 4000);

app.use(cors({ origin: process.env.CORS_ORIGIN || '*' }));
app.use(express.json({ limit: '1mb' }));
app.use('/api', lookRouter);

app.listen(port, () => {
  console.log(`[backend/src/server.js][listen] API running at http://localhost:${port}`);
});
