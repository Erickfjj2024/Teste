import { Router } from 'express';
import { generateLooks } from '../services/lookEngine.js';
import { getProfile, saveProfile } from '../services/profileService.js';
import { mockProfiles } from '../data/mockProfiles.js';

export const lookRouter = Router();

lookRouter.get('/health', (_req, res) => {
  res.json({ ok: true, service: 'Lifestyle Look Builder API' });
});

lookRouter.get('/mock-profiles', (_req, res) => {
  res.json(mockProfiles);
});

lookRouter.post('/profile', async (req, res) => {
  try {
    const saved = await saveProfile(req.body);
    res.status(201).json(saved);
  } catch (error) {
    console.log('[backend/src/routes/lookRoutes.js][/profile] Failed to save profile:', error?.message);
    res.status(500).json({ message: 'Erro ao salvar perfil.' });
  }
});

lookRouter.get('/profile/:id', async (req, res) => {
  try {
    const profile = await getProfile(req.params.id);
    if (!profile) return res.status(404).json({ message: 'Perfil não encontrado.' });
    res.json(profile);
  } catch (error) {
    console.log('[backend/src/routes/lookRoutes.js][/profile/:id] Failed to fetch profile:', error?.message);
    res.status(500).json({ message: 'Erro ao carregar perfil.' });
  }
});

lookRouter.post('/looks/generate', async (req, res) => {
  try {
    const { profile, filters } = req.body;
    const looks = generateLooks(profile, filters || {});
    res.json({ looks });
  } catch (error) {
    console.log('[backend/src/routes/lookRoutes.js][/looks/generate] Failed to generate looks:', error?.message);
    res.status(500).json({ message: 'Erro ao gerar looks.' });
  }
});
