import crypto from 'node:crypto';
import { lookCatalog, priceBands } from '../data/lookCatalog.js';

const fallbackStyle = 'casual';

function normalize(text = '') {
  return text.toLowerCase().normalize('NFD').replace(/\p{Diacritic}/gu, '').trim();
}

function resolveLifestyle(profile) {
  const normalized = normalize(profile.lifestyle);
  const map = {
    'trabalho formal': 'formal',
    formal: 'formal',
    fitness: 'fitness',
    universitario: 'universitario',
    universitário: 'universitario',
    social: 'social',
    criativo: 'criativo',
    urbano: 'urbano',
    religioso: 'minimalista',
    minimalista: 'minimalista',
    casual: 'casual'
  };
  return map[normalized] || fallbackStyle;
}

function scoreLook(look, profile, filters) {
  let score = 0;
  const occasion = normalize(filters.occasion || profile.commonOccasions?.[0] || 'casual');
  const budget = filters.budget || 'meio_termo';

  if (look.occasions.some((item) => normalize(item) === occasion)) score += 5;
  if (profile.favoriteColors.some((color) => normalize(look.colorCombo).includes(normalize(color)))) score += 3;
  if (profile.preferredPieces.some((piece) => normalize(look.top + look.bottom + look.shoes + look.extraLayer).includes(normalize(piece)))) score += 2;
  if (profile.dislikedPieces.some((piece) => normalize(look.top + look.bottom + look.shoes + look.extraLayer).includes(normalize(piece)))) score -= 4;
  if (budget === 'caro') score += 1;

  return score;
}

function buildLookOptions(baseLook, profile) {
  return Object.entries(priceBands).map(([key, value]) => ({
    id: crypto.randomUUID(),
    budgetLevel: key,
    priceRange: value.range,
    budgetStrategy: value.strategy,
    lookName: `${baseLook.name} · ${key.replace('_', ' ')}`,
    styleDescription: baseLook.styleDescription,
    pieces: {
      top: baseLook.top,
      bottom: baseLook.bottom,
      shoes: baseLook.shoes,
      accessories: profile.preferredPieces.slice(0, 2).join(', ') || baseLook.accessoriesTip,
      extraLayer: baseLook.extraLayer
    },
    idealOccasion: baseLook.occasions[0],
    colorCombination: baseLook.colorCombo,
    accessoriesTip: baseLook.accessoriesTip,
    rationale: `Selecionado para ${profile.name}, considerando rotina ${profile.lifestyle}, clima ${profile.cityClimate} e preferência por ${profile.favoriteColors.slice(0, 2).join(' e ') || 'tons neutros'}.`
  }));
}

export function generateLooks(profile, filters = {}) {
  try {
    const occasion = normalize(filters.occasion || profile.commonOccasions?.[0] || 'casual');
    const lifestyleKey = resolveLifestyle(profile);
    const occasionCollection = lookCatalog[occasion] || lookCatalog.casual;
    const lifestyleCollection = occasionCollection[lifestyleKey] || Object.values(occasionCollection).flat();

    const ranked = [...lifestyleCollection]
      .map((look) => ({ look, score: scoreLook(look, profile, filters) }))
      .sort((a, b) => b.score - a.score)
      .slice(0, 3);

    return ranked.flatMap(({ look }) => buildLookOptions(look, profile))
      .filter((item) => !filters.budget || item.budgetLevel === filters.budget);
  } catch (error) {
    console.log('[backend/src/services/lookEngine.js][generateLooks] Failed to generate looks:', error?.message);
    return [];
  }
}
