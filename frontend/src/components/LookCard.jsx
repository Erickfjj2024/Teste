import { motion } from 'framer-motion';

export function LookCard({ look }) {
  return (
    <motion.article layout initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="rounded-2xl border border-slate-700 bg-slate-900 p-4 space-y-2">
      <h3 className="text-xl font-semibold">{look.lookName}</h3>
      <p className="text-slate-300 text-sm">{look.styleDescription}</p>
      <p><strong>Peças:</strong> {look.pieces.top}, {look.pieces.bottom}, {look.pieces.shoes}, {look.pieces.extraLayer}.</p>
      <p><strong>Faixa:</strong> {look.priceRange} ({look.budgetLevel})</p>
      <p><strong>Ocasião ideal:</strong> {look.idealOccasion}</p>
      <p><strong>Cores:</strong> {look.colorCombination}</p>
      <p><strong>Acessórios:</strong> {look.accessoriesTip}</p>
      <p className="text-xs text-slate-400">{look.rationale}</p>
    </motion.article>
  );
}
