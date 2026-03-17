import { useEffect, useMemo, useState } from 'react';
import { Hero } from './components/Hero';
import { OnboardingForm } from './components/OnboardingForm';
import { Filters } from './components/Filters';
import { LookCard } from './components/LookCard';
import { fetchMockProfiles, generateLooks, saveProfile } from './lib/api';

function App() {
  const [profile, setProfile] = useState(null);
  const [looks, setLooks] = useState([]);
  const [loading, setLoading] = useState(false);
  const [filters, setFilters] = useState({ occasion: 'trabalho', budget: '' });

  useEffect(() => {
    fetchMockProfiles().then((profiles) => setProfile(profiles[0])).catch((error) => {
      console.log('[frontend/src/App.jsx][fetchMockProfiles] Failed:', error.message);
    });
  }, []);

  const runGeneration = async (inputProfile = profile, inputFilters = filters) => {
    if (!inputProfile) return;
    setLoading(true);
    try {
      const generated = await generateLooks({ profile: inputProfile, filters: inputFilters });
      setLooks(generated);
    } catch (error) {
      console.log('[frontend/src/App.jsx][runGeneration] Failed:', error.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (profile) runGeneration(profile, filters);
  }, [profile]);

  const title = useMemo(() => profile ? `Looks para ${profile.name}` : 'Preencha seu perfil', [profile]);

  return (
    <main className="max-w-6xl mx-auto p-4 md:p-8 space-y-6">
      <Hero />
      <h2 className="text-2xl font-semibold">Onboarding personalizado</h2>
      <OnboardingForm
        loading={loading}
        onSubmit={async (payload) => {
          setLoading(true);
          try {
            const saved = await saveProfile(payload);
            setProfile(saved);
            await runGeneration(saved, filters);
          } finally {
            setLoading(false);
          }
        }}
      />

      <section className="space-y-4">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h2 className="text-2xl font-semibold">{title}</h2>
        </div>
        <Filters
          filters={filters}
          onChange={(key, value) => {
            if (key === 'reset') {
              setFilters({ occasion: profile?.commonOccasions?.[0] || 'casual', budget: '' });
              runGeneration(profile, { occasion: profile?.commonOccasions?.[0] || 'casual', budget: '' });
              return;
            }
            const next = { ...filters, [key]: value };
            setFilters(next);
            runGeneration(profile, next);
          }}
          onRefresh={() => runGeneration(profile, filters)}
        />

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {looks.map((look) => <LookCard key={look.id} look={look} />)}
        </div>
      </section>
    </main>
  );
}

export default App;
