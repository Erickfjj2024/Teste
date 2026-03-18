export function Filters({ filters, onChange, onRefresh }) {
  return (
    <div className="grid gap-3 rounded-2xl border border-slate-700 bg-slate-900/70 p-4 md:grid-cols-4">
      <select value={filters.occasion} onChange={(e) => onChange('occasion', e.target.value)} className="rounded-lg bg-slate-950 border border-slate-700 p-2">
        {['trabalho', 'academia', 'passeio', 'encontro', 'festa', 'casual', 'viagem'].map((item) => <option key={item}>{item}</option>)}
      </select>
      <select value={filters.budget} onChange={(e) => onChange('budget', e.target.value)} className="rounded-lg bg-slate-950 border border-slate-700 p-2">
        <option value="">todos orçamentos</option>
        <option value="barato">barato</option>
        <option value="meio_termo">meio termo</option>
        <option value="caro">caro</option>
      </select>
      <button onClick={onRefresh} className="rounded-lg border border-slate-600 hover:border-fuchsia-400 p-2">Gerar novos looks</button>
      <button onClick={() => onChange('reset', '')} className="rounded-lg border border-slate-600 hover:border-cyan-400 p-2">Meu estilo de vida</button>
    </div>
  );
}
