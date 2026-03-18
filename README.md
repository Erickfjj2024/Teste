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



## Deploy local (produção) no Termux
Agora o backend consegue servir o frontend buildado (`frontend/dist`) no mesmo processo.

Passo a passo:
1. `npm install`
2. `npm install --prefix backend`
3. `npm install --prefix frontend`
4. `npm run deploy:local`

Após isso, abra:
- App + API no mesmo servidor: `http://localhost:4000`
- Health check: `http://localhost:4000/api/health`

Observações:
- `deploy:local` executa build e sobe o backend em modo de deploy local.
- Se `frontend/dist` não existir, o backend loga mensagem orientando rodar build.

## Rodando via GitHub Actions
Foi adicionado workflow em `.github/workflows/ci.yml` com gatilhos em `push`, `pull_request` e execução manual (`workflow_dispatch`).

Pipeline executa:
1. `npm ci`
2. `npm ci --prefix backend`
3. `npm ci --prefix frontend`
4. `npm run lint`
5. `npm run build`
6. upload de artefato `frontend-dist`

Como usar no GitHub:
1. Abra a aba **Actions** no repositório.
2. Clique no workflow **CI - Lifestyle Look Builder**.
3. Clique em **Run workflow** para rodar manualmente.
4. Após sucesso, baixe o artefato `frontend-dist` se quiser inspecionar o build.

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
