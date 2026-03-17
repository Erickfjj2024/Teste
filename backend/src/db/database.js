import fs from 'node:fs/promises';
import path from 'node:path';
import initSqlJs from 'sql.js';

const dataDir = path.resolve(process.cwd(), 'backend/data');
const dbPath = path.join(dataDir, 'lifestyle.db');
let SQL;
let db;

export async function getDb() {
  try {
    if (db) return db;
    SQL = SQL || (await initSqlJs({}));
    await fs.mkdir(dataDir, { recursive: true });

    try {
      const file = await fs.readFile(dbPath);
      db = new SQL.Database(file);
    } catch {
      db = new SQL.Database();
    }

    db.run(`
      CREATE TABLE IF NOT EXISTS user_profiles (
        id TEXT PRIMARY KEY,
        payload TEXT NOT NULL,
        updated_at TEXT NOT NULL
      );
    `);

    await persistDb();
    return db;
  } catch (error) {
    console.log('[backend/src/db/database.js][getDb] Failed to initialize database:', error?.message);
    throw error;
  }
}

export async function persistDb() {
  try {
    if (!db) return;
    const data = db.export();
    await fs.writeFile(dbPath, Buffer.from(data));
  } catch (error) {
    console.log('[backend/src/db/database.js][persistDb] Failed to persist database:', error?.message);
    throw error;
  }
}
