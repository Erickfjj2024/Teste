import { motion } from 'framer-motion';

export function Hero() {
  return (
    <motion.section
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="gradient-bg rounded-3xl border border-slate-700/50 p-6 md:p-10 shadow-glow"
    >
      <p className="text-xs uppercase tracking-[0.4em] text-slate-400">Lifestyle Look Builder</p>
      <h1 className="mt-4 text-3xl md:text-5xl font-semibold leading-tight">
        Recomendações de looks inteligentes para sua rotina real.
      </h1>
      <p className="mt-4 text-slate-300 max-w-2xl">
        Descubra combinações premium com base no seu estilo de vida, clima, orçamento e ocasiões mais frequentes.
      </p>
    </motion.section>
  );
}
