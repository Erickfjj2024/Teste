import crypto from 'node:crypto';
import { getDb, persistDb } from '../db/database.js';

export async function saveProfile(profile) {
  try {
    const db = await getDb();
    const id = profile.id || crypto.randomUUID();
    const payload = JSON.stringify({ ...profile, id });

    db.run(
      `INSERT INTO user_profiles (id, payload, updated_at)
       VALUES (?, ?, ?)
       ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at;`,
      [id, payload, new Date().toISOString()]
    );

    await persistDb();
    return { ...profile, id };
  } catch (error) {
    console.log('[backend/src/services/profileService.js][saveProfile] Failed to save profile:', error?.message);
    throw error;
  }
}

export async function getProfile(id) {
  try {
    const db = await getDb();
    const result = db.exec('SELECT payload FROM user_profiles WHERE id = ? LIMIT 1;', [id]);
    if (!result.length) return null;
    return JSON.parse(result[0].values[0][0]);
  } catch (error) {
    console.log('[backend/src/services/profileService.js][getProfile] Failed to read profile:', error?.message);
    throw error;
  }
}
