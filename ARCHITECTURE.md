# Diário de Bordo da Arquitetura

## Módulos principais
- `backend/src/services/lookEngine.js`: motor de personalização. Interpreta estilo de vida, pontua catálogo por ocasião e preferências e gera três faixas de preço por sugestão.
- `backend/src/services/profileService.js`: salva e recupera perfis no SQLite (sql.js) com upsert.
- `backend/src/db/database.js`: inicializa banco, cria tabela `user_profiles` e persiste em `backend/data/lifestyle.db`.
- `frontend/src/components/OnboardingForm.jsx`: coleta dados completos do usuário e normaliza listas.
- `frontend/src/components/Filters.jsx`: filtros por ocasião/orçamento + ação “meu estilo de vida”.
- `frontend/src/components/LookCard.jsx`: renderiza detalhes de cada recomendação.

## Endpoints atuais
- `GET /api/health`: status da API.
- `GET /api/mock-profiles`: perfil de demonstração.
- `POST /api/profile`: cria/atualiza perfil.
- `GET /api/profile/:id`: retorna perfil salvo.
- `POST /api/looks/generate`: gera looks com payload `{ profile, filters }`.

## Variáveis de ambiente
- `PORT`: porta da API.
- `CORS_ORIGIN`: origem liberada para frontend.
- `VITE_API_URL`: URL base do frontend para API.

## Notas de robustez
- Logs de erro detalhados incluem arquivo e função.
- Blocos `try/catch` aplicados nas integrações de persistência e geração.
- Banco escolhido evita compilação nativa, favorecendo execução em Termux.


## CI/CD (GitHub Actions)
- Workflow: `.github/workflows/ci.yml`.
- Disparos: `push` e `pull_request` para `main`, além de `workflow_dispatch` manual.
- Etapas: instalação de dependências (root/backend/frontend), lint do frontend, build full-stack e upload do artefato `frontend-dist`.
- Objetivo: garantir que o app continue executável e estável antes de merge/deploy.


## Deploy local (produção)
- Script raiz: `npm run deploy:local` (atalho para build + start local) com execução cross-platform, sem `NODE_ENV=...` inline.
- `backend/src/server.js` detecta `frontend/dist` e serve o frontend estático no mesmo processo do backend.
- Rotas `/api/*` seguem no router da API, e demais paths retornam `frontend/dist/index.html` (SPA fallback).
- Porta padrão de acesso único no deploy local: `http://localhost:4000`.
