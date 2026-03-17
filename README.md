# Lifestyle Look Builder

Aplicativo full-stack para recomendação de looks personalizados com base no estilo de vida, clima, ocasiões e orçamento.

## Stack
- **Frontend:** React + Vite + Tailwind CSS + Framer Motion
- **Backend:** Node.js + Express
- **Banco:** SQLite em WASM via `sql.js` (sem dependências nativas pesadas)

## Funcionalidades
- Landing premium com visual fashion.
- Onboarding completo com dados de estilo de vida.
- Geração inteligente de looks personalizados.
- Três faixas por look: barato, meio termo e caro.
- Filtro por ocasião e orçamento.
- Recalcular usando “meu estilo de vida”.
- Cards de resultado com peças, cor, acessório e explicação.
- Dados mockados para teste rápido.

## Estrutura
```
.
├── backend
│   ├── src
│   │   ├── data            # Catálogo de looks e perfis mockados
│   │   ├── db              # Inicialização e persistência SQLite
│   │   ├── routes          # Endpoints REST
│   │   └── services        # Lógica de perfil e motor de recomendação
├── frontend
│   ├── src
│   │   ├── components      # Hero, formulário, filtros e cards
│   │   ├── lib             # Cliente API
│   │   └── styles          # Tailwind e estilos globais
└── ARCHITECTURE.md
```

## Variáveis de ambiente
Backend (`backend/.env`):
- `PORT=4000`
- `CORS_ORIGIN=http://localhost:5173`

Frontend (`frontend/.env`):
- `VITE_API_URL=http://localhost:4000/api`

## Como rodar
```bash
npm install
npm install --prefix backend
npm install --prefix frontend
npm run dev
```

Acesse:
- Frontend: `http://localhost:5173`
- API: `http://localhost:4000/api/health`

## Endpoints
- `GET /api/health`
- `GET /api/mock-profiles`
- `POST /api/profile`
- `GET /api/profile/:id`
- `POST /api/looks/generate`

## Melhorias futuras
1. Login e múltiplos perfis por usuário.
2. Integração com API de clima em tempo real por cidade.
3. Histórico e favoritos de looks.
4. Recomendação com IA generativa e feedback loop.
5. Catálogo conectado a e-commerces reais.
