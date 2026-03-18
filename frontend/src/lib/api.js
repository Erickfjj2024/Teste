import axios from 'axios';

const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:4000/api'
});

export async function fetchMockProfiles() {
  const { data } = await api.get('/mock-profiles');
  return data;
}

export async function saveProfile(profile) {
  const { data } = await api.post('/profile', profile);
  return data;
}

export async function generateLooks(payload) {
  const { data } = await api.post('/looks/generate', payload);
  return data.looks;
}
