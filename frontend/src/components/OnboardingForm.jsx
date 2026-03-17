const defaultData = {
  name: '',
  age: 22,
  gender: '',
  city: '',
  lifestyle: 'casual',
  clothingPreferences: '',
  favoriteColors: 'preto, branco',
  preferredPieces: 'camiseta, tênis',
  dislikedPieces: 'neon',
  cityClimate: 'ameno',
  commonOccasions: 'trabalho, casual'
};

export function OnboardingForm({ onSubmit, loading }) {
  const submit = (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const payload = Object.fromEntries(form.entries());
    onSubmit({
      ...payload,
      age: Number(payload.age),
      favoriteColors: payload.favoriteColors.split(',').map((item) => item.trim()),
      preferredPieces: payload.preferredPieces.split(',').map((item) => item.trim()),
      dislikedPieces: payload.dislikedPieces.split(',').map((item) => item.trim()),
      commonOccasions: payload.commonOccasions.split(',').map((item) => item.trim())
    });
  };

  return (
    <form onSubmit={submit} className="grid gap-4 rounded-3xl border border-slate-700 bg-slate-900/80 p-6 md:grid-cols-2">
      {Object.entries(defaultData).map(([key, value]) => (
        <label key={key} className="flex flex-col gap-2 text-sm text-slate-300">
          <span className="capitalize">{key}</span>
          <input
            name={key}
            required={key !== 'gender'}
            defaultValue={value}
            className="rounded-xl border border-slate-700 bg-slate-950 px-3 py-2 outline-none focus:border-fuchsia-400"
          />
        </label>
      ))}
      <button disabled={loading} className="md:col-span-2 rounded-xl bg-fuchsia-500 py-3 font-semibold hover:bg-fuchsia-400 transition disabled:opacity-60">
        {loading ? 'Gerando looks...' : 'Salvar perfil e gerar looks'}
      </button>
    </form>
  );
}
